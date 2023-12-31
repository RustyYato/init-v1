//! The interfaces for generic argument based layout calculations

use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

/// see [`LayoutProvider::layout_of`]
pub fn layout_of<T, Args>(args: &Args) -> Option<Layout>
where
    T: ?Sized + HasLayoutProvider<Args>,
{
    <T::LayoutProvider as LayoutProvider<T, Args>>::layout_of(args)
}

/// see [`LayoutProvider::cast`]
///
/// # Safety
///
/// `layout_of` must return a layout for the given arguments
pub unsafe fn cast<T, Args>(ptr: NonNull<u8>, args: &Args) -> NonNull<T>
where
    T: ?Sized + HasLayoutProvider<Args>,
{
    // SAFETY: guaranteed by caller
    unsafe { <T::LayoutProvider as LayoutProvider<T, Args>>::cast(ptr, args) }
}

/// see [`LayoutProvider::is_zeroed`]
pub fn is_zeroed<T, Args>(args: &Args) -> bool
where
    T: ?Sized + HasLayoutProvider<Args>,
{
    <T::LayoutProvider as LayoutProvider<T, Args>>::is_zeroed(args)
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
pub trait HasLayoutProvider<Args: ?Sized = ()> {
    /// The layout provider for this constructor
    type LayoutProvider: LayoutProvider<Self, Args>;
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
pub unsafe trait LayoutProvider<T: ?Sized, Args: ?Sized = ()>: Sized {
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

/// The layout provider for any sized type
pub struct SizedLayoutProvider;

// SAFETY: The layout of a sized type doesn't depend on the argument type
unsafe impl<T, Args> LayoutProvider<T, Args> for SizedLayoutProvider {
    #[inline]
    fn layout_of(_: &Args) -> Option<Layout> {
        Some(Layout::new::<T>())
    }

    #[inline]
    unsafe fn cast(ptr: NonNull<u8>, _: &Args) -> NonNull<T> {
        ptr.cast()
    }
}

impl<T> HasLayoutProvider for MaybeUninit<T> {
    type LayoutProvider = SizedLayoutProvider;
}
