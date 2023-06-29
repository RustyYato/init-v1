//! A pinned thin vector implementation that stores the length and capacity on the heap
//! This type also doesn't do any invisible trivial moves unless the underlying type allows it
//! and guarantees that the values will be dropped before the underling memory is freed
#![forbid(clippy::undocumented_unsafe_blocks)]

use core::{alloc::Layout, marker::PhantomData, mem::MaybeUninit, pin::Pin, ptr::NonNull};

use alloc::alloc::handle_alloc_error;
use init::{
    ctor::{CloneCtor, MoveCtor, TakeCtor},
    layout_provider::{HasLayoutProvider, LayoutProvider},
    pin_ctor::{PinCloneCtor, PinMoveCtor, PinTakeCtor},
    try_pin_ctor::of_pin_ctor,
    Ctor, PinCtor, TryPinCtor,
};

use crate::{
    boxed::ThinBox,
    ptr::{RawThinPtr, WithHeader},
};

/// A thin vector which stores the length and capacity on the heap
pub struct ThinPinVec<T> {
    ptr: RawThinPtr<VecData<T>, usize>,
    _drop: PhantomData<T>,
}

impl<T> Unpin for ThinPinVec<T> {}

#[repr(C)]
struct VecDataInner<T: ?Sized> {
    len: usize,
    data: T,
}

type VecData<T> = VecDataInner<[MaybeUninit<T>]>;
type VecDataSized<T, const N: usize> = VecDataInner<[MaybeUninit<T>; N]>;

fn _verify_covariant<'a: 'b, 'b, T>(t: ThinPinVec<&'a T>) -> ThinPinVec<&'b T> {
    t
}

#[repr(C)]
struct VecDataHeader<T> {
    capacity: usize,
    len: usize,
    data: [T; 0],
}

struct RawAlloc {
    ptr: *mut (),
    layout: Layout,
}

impl Drop for RawAlloc {
    fn drop(&mut self) {
        // SAFETY: This is the same layout used to allocate the vector
        // and all elements have been dropped
        unsafe { alloc::alloc::dealloc(self.ptr.cast(), self.layout) }
    }
}

impl<T> Drop for ThinPinVec<T> {
    fn drop(&mut self) {
        if self.capacity() == 0 {
            return;
        }

        // SAFETY: this pointer is valid because the ThinPinVec guarantees it
        let ptr = unsafe { self.ptr.as_mut_with_header_ptr() };
        let _alloc = RawAlloc {
            // SAFETY: this pointer is valid because the ThinPinVec guarantees it
            layout: Layout::for_value(unsafe { &*ptr }),
            ptr: self.ptr.as_erased_mut_ptr(),
        };

        if !core::mem::needs_drop::<T>() {
            return;
        }

        // SAFETY: ThinPinVec guarantees that the length represents the number of initialized elements in the vector
        unsafe {
            let len = (*ptr).value.len;
            let data = core::ptr::addr_of_mut!((*ptr).value.data);
            let data = core::ptr::slice_from_raw_parts_mut(data.cast::<T>(), len);
            data.drop_in_place();
        }
    }
}

impl<T> ThinPinVec<T> {
    const EMPTY_DATA: WithHeader<VecDataSized<T, 0>, usize> = WithHeader {
        metadata: 0,
        value: VecDataSized {
            len: if core::mem::size_of::<T>() == 0 {
                usize::MAX
            } else {
                0
            },
            data: [],
        },
    };

    const EMPTY_RAW: *const WithHeader<VecData<T>, usize> = &Self::EMPTY_DATA;
    const EMPTY: NonNull<WithHeader<VecData<T>, usize>> = {
        // SAFETY: This pointer came from a reference, which is non-null
        unsafe { NonNull::new_unchecked(Self::EMPTY_RAW.cast_mut()) }
    };

    /// Create a new thin vector
    pub const fn new() -> Self {
        Self {
            ptr: RawThinPtr::from_raw(Self::EMPTY),
            _drop: PhantomData,
        }
    }

    /// Create a new thin vector with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::new();
        }

        let bx = ThinBox::<VecData<T>>::new(WithCapacity(capacity));

        let ptr = ThinBox::into_raw(bx);

        Self {
            ptr,
            _drop: PhantomData,
        }
    }

    fn as_header_ptr(&self) -> *const VecDataHeader<T> {
        self.ptr.as_erased_ptr().cast()
    }

    fn as_header_mut_ptr(&self) -> *mut VecDataHeader<T> {
        self.ptr.as_erased_mut_ptr().cast()
    }

    pub fn capacity(&self) -> usize {
        if core::mem::size_of::<T>() == 0 {
            usize::MAX
        } else {
            // SAFETY: We're not in the
            unsafe { (*self.as_header_ptr()).capacity }
        }
    }

    pub fn len(&self) -> usize {
        // SAFETY: this pointer is valid because the ThinPinVec guarantees it
        unsafe { (*self.as_header_ptr()).len }
    }

    pub fn set_len(&mut self, len: usize) {
        if self.capacity() != 0 {
            // SAFETY: this pointer is valid because the ThinPinVec guarantees it
            unsafe { (*self.as_header_mut_ptr()).len = len }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    pub fn as_ptr(&self) -> *const T {
        let header = self.as_header_ptr();
        // SAFETY: this pointer is valid because the ThinPinVec guarantees it
        unsafe { core::ptr::addr_of!((*header).data).cast() }
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        let header = self.as_header_mut_ptr();
        // SAFETY: this pointer is valid because the ThinPinVec guarantees it
        unsafe { core::ptr::addr_of_mut!((*header).data).cast() }
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: length represents the number of initialized elements that this vector points to
        unsafe { core::slice::from_raw_parts::<T>(self.as_ptr(), self.len()) }
    }

    pub fn as_pin_slice(&self) -> Pin<&[T]> {
        // SAFETY: All elements in this vector are pinned
        unsafe { Pin::new_unchecked(self.as_slice()) }
    }

    pub fn as_pin_slice_mut(&mut self) -> Pin<&mut [T]> {
        // SAFETY: length represents the number of initialized elements that this vector points to
        let slice = unsafe { core::slice::from_raw_parts_mut::<T>(self.as_mut_ptr(), self.len()) };
        // SAFETY: All elements in this vector are pinned
        unsafe { Pin::new_unchecked(slice) }
    }

    /// # Safety
    ///
    /// You must drop all elements or use a `[Try]PinCtor` to move them to another location
    pub unsafe fn take_items(&mut self) -> init::PinInit<'_, [T]> {
        let len = self.len();
        self.set_len(0);

        let slice = core::ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), len);

        // SAFETY: The slice only contains initialized elements and isn't aliased
        // and all elements of a `ThinPinVec` are pinned
        unsafe { init::PinInit::from_raw(slice) }
    }

    /// Construct and push a value in place
    ///
    /// # Safety
    ///
    /// The length must not be equal to the capacity
    pub unsafe fn try_emplace_unchecked<Args>(&mut self, args: Args) -> Result<(), T::Error>
    where
        T: TryPinCtor<Args>,
    {
        let ptr = self.as_mut_ptr();
        let len = self.len();

        // SAFETY: this pointer is
        // * aligned
        // * non-null
        // * dereferencable (for reads and writes, but reads may yield uninitialized memory)
        // * not aliased by any unrelated pointers
        // by the guarantees of `ThinPinVec`
        let uninit = unsafe { init::Uninit::from_raw(ptr.add(len)) };
        let init = uninit.try_pin_init(args)?;

        // the vector will take ownership of the value
        init.take_ownership();

        // SAFETY: the len'th item has been initialized, so we can increment the counter now
        unsafe { (*self.as_header_mut_ptr()).len += 1 }

        Ok(())
    }

    /// Construct and push a value in place
    ///
    /// # Safety
    ///
    /// The length must not be equal to the capacity
    pub unsafe fn emplace_unchecked<Args>(&mut self, args: Args)
    where
        T: PinCtor<Args>,
    {
        // SAFETY: guaranteed by caller
        match unsafe { self.try_emplace_unchecked(of_pin_ctor(args)) } {
            Ok(()) => (),
            Err(inf) => match inf {},
        }
    }

    /// Remove the last element from the vector
    ///
    /// # Safety
    ///
    /// The vector shouldn't be empty
    pub unsafe fn pop_unchecked(&mut self) -> init::Init<'_, T> {
        let ptr = self.as_header_mut_ptr();

        // SAFETY: the length is not zero, because the vector isn't empty so
        // the decrement can't wrap
        // len - 1 is the last initialized element
        unsafe {
            (*ptr).len -= 1;
            let len = (*ptr).len;

            let ptr = self.as_mut_ptr().add(len);

            init::Init::from_raw(ptr)
        }
    }

    /// Remove the last element from the vector
    pub fn pop(&mut self) -> Option<init::Init<'_, T>> {
        if self.is_empty() {
            return None;
        }

        //  SAFETY: The vector isn't empty
        Some(unsafe { self.pop_unchecked() })
    }
}

impl<T: PinMoveCtor> ThinPinVec<T> {
    pub fn reserve(&mut self, additional: usize) {
        let remaining_len = self.capacity() - self.len();

        if remaining_len < additional {
            self.reserve_inner(additional)
        }
    }

    #[cold]
    #[inline(never)]
    fn reserve_inner(&mut self, additional: usize) {
        assert_ne!(
            core::mem::size_of::<T>(),
            0,
            "Tried to allocate more than usize::MAX zero-sized elements"
        );

        let new_capacity = self.capacity().wrapping_mul(2).max(4).max(
            self.len()
                .checked_add(additional)
                .expect("Could not calculate new capacity"),
        );

        if self.capacity() == 0 {
            self.reserve_first(new_capacity)
        } else if self.is_empty() || T::IS_MOVE_TRIVIAL.get() {
            self.reserve_realloc(new_capacity)
        } else {
            self.reserve_move(new_capacity)
        }
    }

    fn reserve_first(&mut self, new_capacity: usize) {
        crate::core_ext::write(self, Self::with_capacity(new_capacity))
    }

    fn reserve_realloc(&mut self, new_capacity: usize) {
        let old_layout = Layout::array::<T>(self.capacity()).unwrap();
        let new_layout = Layout::array::<T>(new_capacity).expect("Could not calculate layout");

        let prefix = Layout::new::<[usize; 2]>();

        let old_layout = prefix.extend(old_layout).unwrap().0.pad_to_align();
        let new_layout = prefix.extend(new_layout).unwrap().0.pad_to_align();

        let capacity = (new_layout.size() - prefix.size()) / core::mem::size_of::<T>();
        debug_assert!(new_capacity >= capacity);
        let new_capacity = capacity;

        if old_layout != new_layout {
            let ptr = self.ptr.as_erased_mut_ptr();

            let new_ptr =
                // SAFETY: The old_layout is the same used to allocate this vector
                // and the new_layout has the same alignment and is non-empty
                unsafe { alloc::alloc::realloc(ptr.cast(), old_layout, new_layout.size()) };

            let new_ptr = core::ptr::slice_from_raw_parts_mut(new_ptr, new_capacity) as *mut _;

            let Some(new_ptr) =  NonNull::new(new_ptr) else {
                handle_alloc_error(new_layout)
            };

            self.ptr = RawThinPtr::from_raw(new_ptr);
        }

        // SAFETY: The pointer is guaranteed to be valid be ThinPinVec
        // the capacity is correct and fits the allocation
        unsafe { (*self.ptr.as_mut_with_header_ptr()).metadata = new_capacity }
    }

    fn reserve_move(&mut self, new_capacity: usize) {
        let mut new_vec = ThinPinVec::with_capacity(new_capacity);

        // SAFETY: all elements get moved or dropped
        let items = unsafe { self.take_items() };

        for item in items {
            // SAFETY: the new vector is guaranteed to have more capacity than the current vector
            // so it can store all of it's elements inside
            unsafe { new_vec.emplace_unchecked(item) }
        }

        *self = new_vec
    }

    pub fn try_emplace<Args>(&mut self, args: Args) -> Result<(), T::Error>
    where
        T: TryPinCtor<Args>,
    {
        if self.capacity() == self.len() {
            self.reserve_inner(1);
        }

        // SAFETY: We just reserved enough space if there wasn't enough already
        unsafe { self.try_emplace_unchecked(args) }
    }

    pub fn emplace<Args>(&mut self, args: Args)
    where
        T: PinCtor<Args>,
    {
        match self.try_emplace(of_pin_ctor(args)) {
            Ok(()) => (),
            Err(inf) => match inf {},
        }
    }
}

struct WithCapacity(usize);

struct WithCapacityLayoutProvider;

// SAFETY: the layout is compatible with the cast
unsafe impl<T> LayoutProvider<VecData<T>, WithCapacity> for WithCapacityLayoutProvider {
    fn layout_of(args: &WithCapacity) -> Option<core::alloc::Layout> {
        Some(
            Layout::new::<usize>()
                .extend(Layout::array::<T>(args.0).ok()?)
                .ok()?
                .0,
        )
    }

    unsafe fn cast(ptr: NonNull<u8>, args: &WithCapacity) -> NonNull<VecData<T>> {
        let ptr = NonNull::<[T]>::slice_from_raw_parts(ptr.cast(), args.0);

        // SAFETY: `ptr` is non-null
        unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut VecData<T>) }
    }
}

impl<T> HasLayoutProvider<WithCapacity> for VecData<T> {
    type LayoutProvider = WithCapacityLayoutProvider;
}

impl<T> Ctor<WithCapacity> for VecData<T> {
    fn init(uninit: init::Uninit<'_, Self>, _: WithCapacity) -> init::Init<'_, Self> {
        init::init_struct! {
            uninit => Self {
                len: 0,
                data: ()
            }
        }
    }
}

impl<T> PinMoveCtor for ThinPinVec<T> {
    const IS_MOVE_TRIVIAL: init::config_value::ConfigValue<Self, init::config_value::PinMoveTag> = {
        // SAFETY: The move-ctor just copies the pointer
        unsafe { init::config_value::ConfigValue::yes() }
    };

    fn pin_move_ctor<'this>(
        uninit: init::Uninit<'this, Self>,
        p: init::PinInit<Self>,
    ) -> init::PinInit<'this, Self> {
        uninit.init(init::PinInit::into_inner(p)).pin()
    }
}

impl<T> PinTakeCtor for ThinPinVec<T> {
    fn pin_take_ctor<'this>(
        uninit: init::Uninit<'this, Self>,
        p: core::pin::Pin<&mut Self>,
    ) -> init::PinInit<'this, Self> {
        uninit.init(Pin::into_inner(p)).pin()
    }
}

impl<T: PinCloneCtor> PinCloneCtor for ThinPinVec<T> {
    fn pin_clone_ctor<'this>(
        uninit: init::Uninit<'this, Self>,
        p: Pin<&Self>,
    ) -> init::PinInit<'this, Self> {
        uninit.init(Pin::into_inner(p)).pin()
    }
}

impl<T> MoveCtor for ThinPinVec<T> {
    const IS_MOVE_TRIVIAL: init::config_value::ConfigValue<Self, init::config_value::MoveTag> = {
        // SAFETY: The move-ctor just copies the pointer
        unsafe { init::config_value::ConfigValue::yes() }
    };

    fn move_ctor<'this>(
        uninit: init::Uninit<'this, Self>,
        p: init::Init<Self>,
    ) -> init::Init<'this, Self> {
        uninit.write(p.into_inner())
    }
}

impl<T> TakeCtor for ThinPinVec<T> {
    fn take_ctor<'this>(
        uninit: init::Uninit<'this, Self>,
        p: &mut Self,
    ) -> init::Init<'this, Self> {
        let this = core::mem::replace(p, Self::new());
        uninit.write(this)
    }
}

impl<T: PinCloneCtor> CloneCtor for ThinPinVec<T> {
    fn clone_ctor<'this>(uninit: init::Uninit<'this, Self>, p: &Self) -> init::Init<'this, Self> {
        let slice = p.as_pin_slice();
        let mut vec = Self::with_capacity(slice.len());

        // SAFETY: the slice and all elements are pinned
        let slice = unsafe { Pin::into_inner_unchecked(slice) };

        for item in slice {
            // SAFETY: the slice and all elements are pinned
            let item = unsafe { Pin::new_unchecked(item) };
            // SAFETY: the vector has enough capacity to hold the entire slice
            unsafe { vec.emplace_unchecked(item) }
        }

        uninit.write(vec)
    }
}

#[test]
fn test_pin_vec() {
    let mut vec = ThinPinVec::<u8>::new();

    for i in 0..100 {
        vec.emplace(i);
    }

    for (i, &x) in vec.as_slice().iter().enumerate() {
        assert_eq!(i, x as usize);
    }
}
