//! The core interfaces used to initialize types

use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

use crate::{Init, Uninit};

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
pub unsafe trait MaybeLayoutProvider<T: ?Sized + Ctor<Args>, Args> {
    /// The layout of the type for the given arguments
    fn layout_of(args: &Args) -> Option<Layout>;

    ///  # Safety
    ///
    /// `Self::layout(args)` must return a layout
    unsafe fn cast(ptr: NonNull<u8>, args: &Args) -> NonNull<T>;
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

/// A type which is constructable using `Args`
pub trait Ctor<Args = ()> {
    /// The layout provider for this constructor
    type LayoutProvider: MaybeLayoutProvider<Self, Args>;

    /// Initialize a the type `Self` using `args: Args`
    fn init(uninit: Uninit<'_, Self>, args: Args) -> Init<'_, Self>;
}

/// A type which can construct a `T`
pub trait CtorArgs<T: ?Sized>: Sized {
    /// The layout provider for this constructor
    type LayoutProvider: MaybeLayoutProvider<T, Self>;

    /// Initialize a the type `T` using `self`
    fn init_with(self, uninit: Uninit<'_, T>) -> Init<'_, T>;
}

impl<T: ?Sized, Args: CtorArgs<T>> Ctor<Args> for T {
    type LayoutProvider = Args::LayoutProvider;

    #[inline]
    fn init(uninit: Uninit<'_, Self>, args: Args) -> Init<'_, Self> {
        args.init_with(uninit)
    }
}

impl<T> Ctor for MaybeUninit<T> {
    type LayoutProvider = SizedLayoutProvider;

    #[inline]
    fn init(uninit: Uninit<'_, Self>, (): ()) -> Init<'_, Self> {
        uninit.uninit()
    }
}

impl<T: ?Sized, F: FnOnce(Uninit<'_, T>) -> Init<'_, T>> CtorArgs<T> for F {
    type LayoutProvider = NoLayoutProvider;

    #[inline]
    fn init_with(self, uninit: Uninit<'_, T>) -> Init<'_, T> {
        self(uninit)
    }
}

/// a no-op helper function to guide type inference
///
/// Rust's type inference doesn't understand the indirection from
/// `FnOnce()` to `CtorArgs` to `Ctor`, so use this no-op to guide inference
pub fn ctor<T: ?Sized, F: FnOnce(Uninit<T>) -> Init<T>>(f: F) -> F {
    f
}
