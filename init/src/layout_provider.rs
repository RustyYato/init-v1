//! The interfaces for generic argument based layout calculations

use core::{alloc::Layout, ptr::NonNull};

use crate::Ctor;

/// see [`MaybeLayoutProvider::layout_of`]
pub fn layout_of<T, Args>(args: &Args) -> Option<Layout>
where
    T: ?Sized + Ctor<Args>,
    T::LayoutProvider: LayoutProvider,
{
    <T::LayoutProvider as MaybeLayoutProvider<T, Args>>::layout_of(args)
}

/// see [`MaybeLayoutProvider::cast`]
///
/// # Safety
///
/// `layout_of` must return a layout for the given arguments
pub unsafe fn cast<T, Args>(ptr: NonNull<u8>, args: &Args) -> NonNull<T>
where
    T: ?Sized + Ctor<Args>,
    T::LayoutProvider: LayoutProvider,
{
    // SAFETY: guaranteed by caller
    unsafe { <T::LayoutProvider as MaybeLayoutProvider<T, Args>>::cast(ptr, args) }
}

/// see [`MaybeLayoutProvider::is_zeroed`]
pub fn is_zeroed<T, Args>(args: &Args) -> bool
where
    T: ?Sized + Ctor<Args>,
{
    <T::LayoutProvider as MaybeLayoutProvider<T, Args>>::is_zeroed(args)
}

/// A type which provides the layout information for a given type/ctor argument pair
///
/// # Safety
///
/// You, the implementor of this trait, must ensure
///
/// ```rs
/// Self::cast(ptr, _).cast::<u8>() == ptr
/// ```
///
/// and that if `Self::layout(ptr, &args)` returns a `Layout`, that
/// the layout fits the pointer returned by `Self::cast(ptr, &args)`
/// with the same args
pub unsafe trait MaybeLayoutProvider<T: ?Sized + Ctor<Args>, Args = ()> {
    /// The layout of the type for the given arguments
    fn layout_of(args: &Args) -> Option<Layout>;

    ///  # Safety
    ///
    /// `Self::layout(args)` must return a layout
    unsafe fn cast(ptr: NonNull<u8>, args: &Args) -> NonNull<T>;

    /// If the arguments is guaranteed to zero out data and have no other side effects
    /// then this returns true
    ///
    /// If this function returns true, it is safe to just write zeros to all bytes
    /// `0..layout.size()` and skip the calling `Ctor::init` or `CtorArgs::init_with`
    fn is_zeroed(_: &Args) -> bool {
        false
    }
}

/// A type where `MaybeLayoutProvider::layout_of` returns `Some` for some arguments
pub trait LayoutProvider {}

/// The layout provider for any sized type
pub struct SizedLayoutProvider;

impl LayoutProvider for SizedLayoutProvider {}
// SAFETY: The layout of a sized type doesn't depend on the argument type
unsafe impl<T: Ctor<Args>, Args> MaybeLayoutProvider<T, Args> for SizedLayoutProvider {
    #[inline]
    fn layout_of(_: &Args) -> Option<Layout> {
        Some(Layout::new::<T>())
    }

    #[inline]
    unsafe fn cast(ptr: NonNull<u8>, _: &Args) -> NonNull<T> {
        ptr.cast()
    }
}

/// The layout provider for any type (which doesn't actually provide a layout)
pub struct NoLayoutProvider;

// SAFETY: The layout of a sized type doesn't depend on the argument type
unsafe impl<T: ?Sized + Ctor<Args>, Args> MaybeLayoutProvider<T, Args> for NoLayoutProvider {
    #[inline]
    fn layout_of(_: &Args) -> Option<Layout> {
        None
    }

    #[inline]
    unsafe fn cast(_: NonNull<u8>, _: &Args) -> NonNull<T> {
        // SAFETY: layout_of never returns Some, so this function can't be safely called
        unsafe { core::hint::unreachable_unchecked() }
    }
}
