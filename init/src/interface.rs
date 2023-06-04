//! The core interfaces used to initialize types

use core::mem::MaybeUninit;

use crate::{
    layout_provider::{MaybeLayoutProvider, NoLayoutProvider, SizedLayoutProvider},
    Init, Uninit,
};

/// A type which is constructable using `Args`
pub trait Ctor<Args = ()> {
    /// The layout provider for this constructor
    type LayoutProvider: MaybeLayoutProvider<Self, Args>;

    /// Initialize a the type `Self` using `args: Args`
    fn init(uninit: Uninit<'_, Self>, args: Args) -> Init<'_, Self>;

    #[inline]
    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        false
    }
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
