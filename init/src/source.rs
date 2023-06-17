//! A module to define a type which allows getting the layout from some other value as a source

use core::{alloc::Layout, pin::Pin, ptr::NonNull};

use crate::{CtorArgs, Init, PinCtorArgs, PinInit, Uninit};

use crate::layout_provider::{HasLayoutProvider, LayoutProvider};

use crate::ctor::{CloneCtor, MoveCtor, TakeCtor};
use crate::pin_ctor::{PinCloneCtor, PinMoveCtor, PinTakeCtor};

/// The layout provider for the `_` argument
pub struct SourceLayoutProvider;

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> LayoutProvider<T, T> for SourceLayoutProvider {
    fn layout_of(args: &T) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args))
    }

    unsafe fn cast(ptr: core::ptr::NonNull<u8>, args: &T) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(args);
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> LayoutProvider<T, &T> for SourceLayoutProvider {
    fn layout_of(args: &&T) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args))
    }

    unsafe fn cast(ptr: core::ptr::NonNull<u8>, args: &&T) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(*args);
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

// SAFETY: Copying the layout and metadata is always safe
unsafe impl<T: ?Sized> LayoutProvider<T, &mut T> for SourceLayoutProvider {
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
unsafe impl<T: ?Sized> LayoutProvider<T, Init<'_, T>> for SourceLayoutProvider {
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
unsafe impl<T: ?Sized> LayoutProvider<T, Pin<&T>> for SourceLayoutProvider {
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
unsafe impl<T: ?Sized> LayoutProvider<T, Pin<&mut T>> for SourceLayoutProvider {
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
unsafe impl<T: ?Sized> LayoutProvider<T, PinInit<'_, T>> for SourceLayoutProvider {
    fn layout_of(args: &PinInit<'_, T>) -> Option<core::alloc::Layout> {
        Some(Layout::for_value::<T>(args.get()))
    }

    unsafe fn cast(ptr: core::ptr::NonNull<u8>, args: &PinInit<'_, T>) -> core::ptr::NonNull<T> {
        let meta = core::ptr::metadata::<T>(args.get());
        NonNull::from_raw_parts(ptr.cast(), meta)
    }
}

impl<T: ?Sized + MoveCtor> CtorArgs<T> for Init<'_, T> {
    #[inline]
    fn init_into(self, uninit: Uninit<'_, T>) -> Init<'_, T> {
        MoveCtor::move_ctor(uninit, self)
    }
}

impl<T: ?Sized + TakeCtor> CtorArgs<T> for &mut T {
    #[inline]
    fn init_into(self, uninit: Uninit<'_, T>) -> Init<'_, T> {
        TakeCtor::take_ctor(uninit, self)
    }
}

impl<T: ?Sized + CloneCtor> CtorArgs<T> for &T {
    #[inline]
    fn init_into(self, uninit: Uninit<'_, T>) -> Init<'_, T> {
        CloneCtor::clone_ctor(uninit, self)
    }
}

impl<T: ?Sized + PinMoveCtor> PinCtorArgs<T> for PinInit<'_, T> {
    #[inline]
    fn pin_init_into(self, uninit: Uninit<'_, T>) -> PinInit<'_, T> {
        PinMoveCtor::pin_move_ctor(uninit, self)
    }
}

impl<T: ?Sized + PinTakeCtor> PinCtorArgs<T> for Pin<&mut T> {
    #[inline]
    fn pin_init_into(self, uninit: Uninit<'_, T>) -> PinInit<'_, T> {
        PinTakeCtor::pin_take_ctor(uninit, self)
    }
}

impl<T: ?Sized + PinCloneCtor> PinCtorArgs<T> for Pin<&T> {
    #[inline]
    fn pin_init_into(self, uninit: Uninit<'_, T>) -> PinInit<'_, T> {
        PinCloneCtor::pin_clone_ctor(uninit, self)
    }
}
