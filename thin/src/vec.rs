//! A thin vector implementation that stores the length and capacity on the heap

use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

use init::{
    layout_provider::{LayoutProvider, MaybeLayoutProvider},
    Ctor,
};

use crate::{
    boxed::ThinBox,
    ptr::{RawThinPtr, WithHeader},
};

#[repr(C)]
struct VecDataInner<T: ?Sized> {
    len: usize,
    data: T,
}
type VecData<T> = VecDataInner<[MaybeUninit<T>]>;
type VecDataSized<T, const N: usize> = VecDataInner<[MaybeUninit<T>; N]>;

#[repr(C)]
struct VecDataHeader<T> {
    capacity: usize,
    len: usize,
    data: [T; 0],
}

/// A thin vector which stores the length and capacity on the heap
pub struct ThinVec<T> {
    ptr: RawThinPtr<VecData<T>, usize>,
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

        let ptr = unsafe { self.ptr.as_mut_ptr() };
        let _alloc = RawThinVec {
            ptr: self.ptr.as_erased_mut_ptr(),
            layout: unsafe { Layout::for_value(&*ptr) },
        };

        if !core::mem::needs_drop::<T>() {
            return;
        }

        unsafe {
            let len = (*ptr).len;
            let data = core::ptr::addr_of_mut!((*ptr).data);
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
        unsafe { (*self.as_header_ptr()).capacity }
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

struct WithCapacity(usize);

struct WithCapacityLayoutProvider;

impl<T> LayoutProvider<VecData<T>, WithCapacity> for WithCapacityLayoutProvider {}
unsafe impl<T> MaybeLayoutProvider<VecData<T>, WithCapacity> for WithCapacityLayoutProvider {
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

impl<T> Ctor<WithCapacity> for VecData<T> {
    type LayoutProvider = WithCapacityLayoutProvider;

    fn init(uninit: init::Uninit<'_, Self>, _: WithCapacity) -> init::Init<'_, Self> {
        init::init_struct! {
            uninit => Self {
                len: 0,
                data: ()
            }
        }
    }
}
