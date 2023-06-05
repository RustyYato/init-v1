use core::{alloc::Layout, pin::Pin, ptr::NonNull};

use crate::{CtorArgs, Init, PinInit, Uninit};

use crate::layout_provider::{HasLayoutProvider, MaybeLayoutProvider};

use super::{CloneCtor, MoveCtor, TakeCtor};

/// The layout provider for the `_` argument
pub struct SourceLayoutProvider;

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> MaybeLayoutProvider<T, T> for SourceLayoutProvider {
    fn layout_of(args: &T) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args))
    }

    unsafe fn cast(ptr: core::ptr::NonNull<u8>, args: &T) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(args);
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> MaybeLayoutProvider<T, &T> for SourceLayoutProvider {
    fn layout_of(args: &&T) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args))
    }

    unsafe fn cast(ptr: core::ptr::NonNull<u8>, args: &&T) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(*args);
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> MaybeLayoutProvider<T, &mut T> for SourceLayoutProvider {
    fn layout_of(args: &&mut T) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args))
    }

    unsafe fn cast(ptr: core::ptr::NonNull<u8>, args: &&mut T) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(*args);
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

impl<T: ?Sized> HasLayoutProvider<Init<'_, T>> for T {
    type LayoutProvider = SourceLayoutProvider;
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> MaybeLayoutProvider<T, Init<'_, T>> for SourceLayoutProvider {
    fn layout_of(args: &Init<'_, T>) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args.get()))
    }

    unsafe fn cast(ptr: core::ptr::NonNull<u8>, args: &Init<'_, T>) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(args.get());
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

impl<T: ?Sized> HasLayoutProvider<Pin<&T>> for T {
    type LayoutProvider = SourceLayoutProvider;
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> MaybeLayoutProvider<T, Pin<&T>> for SourceLayoutProvider {
    fn layout_of(args: &Pin<&T>) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args))
    }

    unsafe fn cast(ptr: core::ptr::NonNull<u8>, args: &Pin<&T>) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(&**args);
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

impl<T: ?Sized> HasLayoutProvider<Pin<&mut T>> for T {
    type LayoutProvider = SourceLayoutProvider;
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> MaybeLayoutProvider<T, Pin<&mut T>> for SourceLayoutProvider {
    fn layout_of(args: &Pin<&mut T>) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args))
    }

    unsafe fn cast(ptr: core::ptr::NonNull<u8>, args: &Pin<&mut T>) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(&**args);
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

impl<T: ?Sized> HasLayoutProvider<PinInit<'_, T>> for T {
    type LayoutProvider = SourceLayoutProvider;
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> MaybeLayoutProvider<T, PinInit<'_, T>> for SourceLayoutProvider {
    fn layout_of(args: &PinInit<'_, T>) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args.get()))
    }

    unsafe fn cast(ptr: core::ptr::NonNull<u8>, args: &PinInit<'_, T>) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(args.get());
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}
