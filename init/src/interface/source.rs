use core::{alloc::Layout, pin::Pin, ptr::NonNull};

use crate::{Init, PinInit};

use crate::layout_provider::{HasLayoutProvider, MaybeLayoutProvider};

/// The layout provider for the `Source<_>` argument
pub struct SourceLayoutProvider;

/// The argument fo
pub struct Source<T: ?Sized>(pub T);

impl<T: ?Sized> HasLayoutProvider<Source<T>> for T {
    type LayoutProvider = SourceLayoutProvider;
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> MaybeLayoutProvider<T, Source<T>> for SourceLayoutProvider {
    fn layout_of(Source(args): &Source<T>) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args))
    }

    unsafe fn cast(ptr: core::ptr::NonNull<u8>, Source(args): &Source<T>) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(args);
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

impl<T: ?Sized> HasLayoutProvider<Source<&T>> for T {
    type LayoutProvider = SourceLayoutProvider;
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> MaybeLayoutProvider<T, Source<&T>> for SourceLayoutProvider {
    fn layout_of(Source(args): &Source<&T>) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args))
    }

    unsafe fn cast(
        ptr: core::ptr::NonNull<u8>,
        Source(args): &Source<&T>,
    ) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(*args);
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

impl<T: ?Sized> HasLayoutProvider<Source<&mut T>> for T {
    type LayoutProvider = SourceLayoutProvider;
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> MaybeLayoutProvider<T, Source<&mut T>> for SourceLayoutProvider {
    fn layout_of(Source(args): &Source<&mut T>) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args))
    }

    unsafe fn cast(
        ptr: core::ptr::NonNull<u8>,
        Source(args): &Source<&mut T>,
    ) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(*args);
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

impl<T: ?Sized> HasLayoutProvider<Source<Init<'_, T>>> for T {
    type LayoutProvider = SourceLayoutProvider;
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> MaybeLayoutProvider<T, Source<Init<'_, T>>> for SourceLayoutProvider {
    fn layout_of(Source(args): &Source<Init<'_, T>>) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args.get()))
    }

    unsafe fn cast(
        ptr: core::ptr::NonNull<u8>,
        Source(args): &Source<Init<'_, T>>,
    ) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(args.get());
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

impl<T: ?Sized> HasLayoutProvider<Source<Pin<&T>>> for T {
    type LayoutProvider = SourceLayoutProvider;
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> MaybeLayoutProvider<T, Source<Pin<&T>>> for SourceLayoutProvider {
    fn layout_of(Source(args): &Source<Pin<&T>>) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args))
    }

    unsafe fn cast(
        ptr: core::ptr::NonNull<u8>,
        Source(args): &Source<Pin<&T>>,
    ) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(&**args);
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

impl<T: ?Sized> HasLayoutProvider<Source<Pin<&mut T>>> for T {
    type LayoutProvider = SourceLayoutProvider;
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> MaybeLayoutProvider<T, Source<Pin<&mut T>>> for SourceLayoutProvider {
    fn layout_of(Source(args): &Source<Pin<&mut T>>) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args))
    }

    unsafe fn cast(
        ptr: core::ptr::NonNull<u8>,
        Source(args): &Source<Pin<&mut T>>,
    ) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(&**args);
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

impl<T: ?Sized> HasLayoutProvider<Source<PinInit<'_, T>>> for T {
    type LayoutProvider = SourceLayoutProvider;
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> MaybeLayoutProvider<T, Source<PinInit<'_, T>>> for SourceLayoutProvider {
    fn layout_of(Source(args): &Source<PinInit<'_, T>>) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args.get()))
    }

    unsafe fn cast(
        ptr: core::ptr::NonNull<u8>,
        Source(args): &Source<PinInit<'_, T>>,
    ) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(args.get());
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}
