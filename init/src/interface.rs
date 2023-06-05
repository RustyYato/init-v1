//! The core interfaces used to initialize types

mod source;

use core::{marker::PhantomData, mem::MaybeUninit, pin::Pin};

use crate::{
    layout_provider::{HasLayoutProvider, NoLayoutProvider, SizedLayoutProvider},
    Init, PinInit, Uninit,
};

pub use source::SourceLayoutProvider;

/// A type which is constructable using `Args`
pub trait Ctor<Args = ()> {
    /// Initialize a the type `Self` using `args: Args`
    fn init(uninit: Uninit<'_, Self>, args: Args) -> Init<'_, Self>;

    #[inline]
    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        false
    }
}

/// A type which is constructable using `Args`
pub trait PinCtor<Args = ()> {
    /// Initialize a the type `Self` using `args: Args`
    fn pin_init(uninit: Uninit<'_, Self>, args: Args) -> PinInit<'_, Self>;

    #[inline]
    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        false
    }
}

/// A type which can construct a `T`
pub trait CtorArgs<T: ?Sized>: Sized {
    /// Initialize a the type `T` using `self`
    fn init_with(self, uninit: Uninit<'_, T>) -> Init<'_, T>;
}

/// A type which can construct a `T`
pub trait PinCtorArgs<T: ?Sized>: Sized {
    /// Initialize a the type `T` using `self`
    fn pin_init_with(self, uninit: Uninit<'_, T>) -> PinInit<'_, T>;
}

impl<T: ?Sized + HasLayoutProvider<Args>, Args: CtorArgs<T>> Ctor<Args> for T {
    #[inline]
    fn init(uninit: Uninit<'_, Self>, args: Args) -> Init<'_, Self> {
        args.init_with(uninit)
    }
}

impl<T: ?Sized + HasLayoutProvider<Args>, Args: PinCtorArgs<T>> PinCtor<Args> for T {
    #[inline]
    fn pin_init(uninit: Uninit<'_, Self>, args: Args) -> PinInit<'_, Self> {
        args.pin_init_with(uninit)
    }
}

impl<T> HasLayoutProvider for MaybeUninit<T> {
    type LayoutProvider = SizedLayoutProvider;
}

impl<T> Ctor for MaybeUninit<T> {
    #[inline]
    fn init(uninit: Uninit<'_, Self>, (): ()) -> Init<'_, Self> {
        uninit.uninit()
    }
}

struct CtorFn<F, T: ?Sized>(F, PhantomData<T>);

impl<T: ?Sized, F> HasLayoutProvider<CtorFn<F, T>> for T {
    type LayoutProvider = NoLayoutProvider;
}

impl<T: ?Sized, F: FnOnce(Uninit<'_, T>) -> Init<'_, T>> CtorArgs<T> for CtorFn<F, T> {
    #[inline]
    fn init_with(self, uninit: Uninit<'_, T>) -> Init<'_, T> {
        (self.0)(uninit)
    }
}

impl<T: ?Sized, F: FnOnce(Uninit<'_, T>) -> PinInit<'_, T>> PinCtorArgs<T> for CtorFn<F, T> {
    #[inline]
    fn pin_init_with(self, uninit: Uninit<'_, T>) -> PinInit<'_, T> {
        (self.0)(uninit)
    }
}

/// a no-op helper function to guide type inference
///
/// Rust's type inference doesn't understand the indirection from
/// `FnOnce()` to `CtorArgs` to `Ctor`, so use this no-op to guide inference
pub fn ctor<T: ?Sized, F: FnOnce(Uninit<T>) -> Init<T>>(f: F) -> impl CtorArgs<T> {
    CtorFn(f, PhantomData)
}

/// a no-op helper function to guide type inference
///
/// Rust's type inference doesn't understand the indirection from
/// `FnOnce()` to `CtorArgs` to `Ctor`, so use this no-op to guide inference
pub fn pin_ctor<T: ?Sized, F: FnOnce(Uninit<T>) -> PinInit<T>>(f: F) -> impl PinCtorArgs<T> {
    CtorFn(f, PhantomData)
}
pub fn pin_ctor<T: ?Sized, F: FnOnce(Uninit<T>) -> PinInit<T>>(f: F) -> F {
    f
}
