//! A thin vector implementation that stores the length and capacity on the heap

use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

use alloc::alloc::handle_alloc_error;
use init::{
    interface::MoveCtor,
    layout_provider::{HasLayoutProvider, LayoutProvider},
    Ctor,
};

use crate::{
    boxed::ThinBox,
    ptr::{PushHeader, RawThinPtr, WithHeader, WithHeaderLayoutProvider},
};

/// A thin vector which stores the length and capacity on the heap
pub struct ThinVec<T> {
    ptr: RawThinPtr<VecData<T>, usize>,
}

#[repr(C)]
struct VecDataInner<T: ?Sized> {
    len: usize,
    data: T,
}
type VecData<T> = VecDataInner<[MaybeUninit<T>]>;
type VecDataSized<T, const N: usize> = VecDataInner<[MaybeUninit<T>; N]>;

type AllocTy<T> = WithHeader<VecData<T>>;

#[repr(C)]
struct VecDataHeader<T> {
    capacity: usize,
    len: usize,
    data: [T; 0],
}

fn _verify_covariant<'a: 'b, 'b, T>(t: ThinVec<&'a T>) -> ThinVec<&'b T> {
    t
}

struct RawThinVec {
    ptr: *mut (),
    layout: Layout,
}

impl Drop for RawThinVec {
    fn drop(&mut self) {
        unsafe { alloc::alloc::dealloc(self.ptr.cast(), self.layout) }
    }
}

impl<T> Drop for ThinVec<T> {
    fn drop(&mut self) {
        if self.capacity() == 0 {
            return;
        }

        let ptr = unsafe { self.ptr.as_mut_with_header_ptr() };
        let _alloc = RawThinVec {
            ptr: self.ptr.as_erased_mut_ptr(),
            layout: unsafe { Layout::for_value(&*ptr) },
        };

        if !core::mem::needs_drop::<T>() {
            return;
        }

        unsafe {
            let len = (*ptr).value.len;
            let data = core::ptr::addr_of_mut!((*ptr).value.data);
            let data = core::ptr::slice_from_raw_parts_mut(data.cast::<T>(), len);
            data.drop_in_place();
        }
    }
}

impl<T> ThinVec<T> {
    const EMPTY_DATA: WithHeader<VecDataSized<T, 0>, usize> = WithHeader {
        metadata: 0,
        value: VecDataSized { len: 0, data: [] },
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
        }
    }

    /// Create a new thin vector with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::new();
        }

        let bx = ThinBox::<VecData<T>>::new(WithCapacity(capacity));

        let ptr = ThinBox::into_raw(bx);

        Self { ptr }
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
            unsafe { (*self.as_header_ptr()).capacity }
        }
    }

    pub fn len(&self) -> usize {
        unsafe { (*self.as_header_ptr()).len }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    pub fn as_ptr(&self) -> *const T {
        let header = self.as_header_ptr();
        unsafe { core::ptr::addr_of!((*header).data).cast() }
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        let header = self.as_header_mut_ptr();
        unsafe { core::ptr::addr_of_mut!((*header).data).cast() }
    }
}

fn new_capacity(capacity: usize, additional: usize) -> Option<usize> {
    let expected_capacity = capacity.checked_add(additional)?;
    let new_capacity = capacity.wrapping_mul(2);
    let min_capacity = 4;
    Some(expected_capacity.max(new_capacity).max(min_capacity))
}

fn new_layout<T>(capacity: usize, additional: usize) -> Option<(Layout, Layout, usize)> {
    let new_capacity = new_capacity(capacity, additional)?;

    let layout =
        init::layout_provider::layout_of::<AllocTy<T>, _>(&PushHeader(WithCapacity(capacity)));
    let new_layout =
        init::layout_provider::layout_of::<AllocTy<T>, _>(&PushHeader(WithCapacity(new_capacity)))?;

    let layout = unsafe { layout.unwrap_unchecked() };

    Some((layout, new_layout, new_capacity))
}

impl<T: MoveCtor> ThinVec<T> {
    pub fn reserve(&mut self, additional: usize) {
        let remaining_capacity = self.capacity() - self.len();

        if remaining_capacity < additional {
            self.reserve_inner(additional)
        }
    }

    #[cold]
    #[inline(never)]
    fn reserve_inner(&mut self, additional: usize) {
        if core::mem::size_of::<T>() == 0 {
            panic!("Cannot reserve more than usize::MAX elements for Zero Sized Types")
        } else if self.capacity() == 0 {
            self.reserve_first(additional)
        } else if T::IS_MOVE_TRIVIAL.get() {
            self.reserve_inner_realloc(additional)
        } else {
            self.reserve_inner_move(additional)
        }
    }

    #[cold]
    fn reserve_first(&mut self, additional: usize) {
        crate::core_ext::write(self, Self::with_capacity(additional))
    }

    fn reserve_inner_realloc(&mut self, additional: usize) {
        let (layout, new_layout, new_capacity) =
            new_layout::<T>(self.capacity(), additional).expect("Could not calculate new layout");

        let ptr = unsafe {
            alloc::alloc::realloc(
                self.ptr.as_erased_mut_ptr().cast(),
                layout,
                new_layout.size(),
            )
        };

        let Some(ptr) = NonNull::new(ptr) else {
            handle_alloc_error(new_layout);
        };

        // SAFETY: WithCapacityLayoutProvider::cast is always safe to call
        let ptr = unsafe {
            init::layout_provider::cast::<AllocTy<T>, _>(
                ptr,
                &PushHeader(WithCapacity(new_capacity)),
            )
        };

        // SAFETY: this pointer is safe to write to, and needs to be written to in order to update the capacity
        unsafe { (*ptr.as_ptr()).metadata = new_capacity }

        self.ptr = RawThinPtr::from_raw(ptr);
    }

    fn reserve_inner_move(&mut self, additional: usize) {
        todo!()
    }
}

struct WithCapacity(usize);

struct WithCapacityLayoutProvider;

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

#[test]
fn test() {
    let mut v = ThinVec::<i32>::new();

    v.reserve(10);
    v.reserve(90);

    // panic!()
}
