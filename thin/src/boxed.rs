//! A thin pointer to a single heap allocation

use core::{
    alloc::Layout,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use init::{layout_provider::LayoutProvider, Ctor};

use crate::ptr::{PushHeader, RawThinPtr, WithHeader};

/// A type that's like a `Box` mut guaranteed to be the same representation as a `*mut ()`
#[repr(transparent)]
pub struct ThinBox<T: ?Sized> {
    ptr: RawThinPtr<T>,
    ty: PhantomData<T>,
}

impl<T: ?Sized> Drop for ThinBox<T> {
    fn drop(&mut self) {
        // SAFETY: the pointer is valid, allocated, and initialized
        let (ptr, layout) = unsafe {
            let ptr = self.ptr.as_mut_with_header_ptr();
            let layout = Layout::for_value(&*ptr);
            ptr.drop_in_place();
            (ptr, layout)
        };
        // SAFETY: the pointer is valid and allocated by the global allocator
        unsafe { alloc::alloc::dealloc(ptr.cast(), layout) }
    }
}

impl<T: ?Sized> ThinBox<T> {
    /// Construct a new ThinBox
    pub fn new<Args>(args: Args) -> Self
    where
        T: Ctor<Args>,
        T::LayoutProvider: LayoutProvider<T, Args>,
    {
        let bx = init::boxed::boxed::<WithHeader<T>, _>(PushHeader(args));

        let bx = alloc::boxed::Box::into_raw(bx);

        Self {
            // SAFETY: This pointer came from a box, which is non-null
            ptr: RawThinPtr::from_raw(unsafe { NonNull::new_unchecked(bx) }),
            ty: PhantomData,
        }
    }
}

impl<T> ThinBox<[T]> {
    /// Get the length of the slice
    pub fn len(&self) -> usize {
        // SAFETY: This pointer is valid, allocated, and initialized
        unsafe { self.ptr.metadata() }
    }

    /// Get the length of the slice
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: ?Sized> Deref for ThinBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: This pointer is valid, allocated, and initialized
        unsafe { &*self.ptr.as_ptr() }
    }
}

impl<T: ?Sized> DerefMut for ThinBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: This pointer is valid, allocated, and initialized
        unsafe { &mut *self.ptr.as_mut_ptr() }
    }
}

#[test]
fn test_u8() {
    let bx = ThinBox::<u8>::new(());
    assert_eq!(*bx, 0);
}

#[test]
fn test_slice() {
    let bx = ThinBox::<[u8]>::new(init::slice::CopyArgsLen(10, ()));
    assert_eq!(*bx, [0; 10]);
}

#[test]
fn test_slice_nonzero() {
    let bx = ThinBox::<[u8]>::new(init::slice::CopyArgsLen(10, 100));
    assert_eq!(*bx, [100; 10]);
}
