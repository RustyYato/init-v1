//! The core interfaces used to initialize types

use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{PinCtor, PinCtorArgs, PinInit, Uninit};

/// A type which is constructable using `Args`
pub trait TryPinCtor<Args = ()> {
    /// The error type of a failed initialization
    type Error;

    /// Initialize a the type `Self` using `args: Args`
    fn try_pin_init(uninit: Uninit<'_, Self>, args: Args)
        -> Result<PinInit<'_, Self>, Self::Error>;

    #[inline]
    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        false
    }
}

/// A type which can construct a `T`
pub trait TryPinCtorArgs<T: ?Sized> {
    /// The error type of a failed initialization
    type Error;

    /// Initialize a the type `T` using `self`
    fn try_pin_init_into(self, uninit: Uninit<'_, T>) -> Result<PinInit<'_, T>, Self::Error>;

    #[inline]
    #[doc(hidden)]
    fn __is_clone_cheap() -> bool {
        false
    }
}

impl<T: ?Sized, Args: TryPinCtorArgs<T>> TryPinCtor<Args> for T {
    type Error = Args::Error;

    #[inline]
    fn try_pin_init(
        uninit: Uninit<'_, Self>,
        args: Args,
    ) -> Result<PinInit<'_, Self>, Self::Error> {
        args.try_pin_init_into(uninit)
    }
}

impl<T> TryPinCtor for MaybeUninit<T> {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_pin_init(uninit: Uninit<'_, Self>, (): ()) -> Result<PinInit<'_, Self>, Self::Error> {
        Ok(uninit.uninit().pin())
    }

    fn __is_args_clone_cheap() -> bool {
        true
    }
}

struct TryPinCtorFn<F, T: ?Sized>(F, PhantomData<T>);

impl<T: ?Sized, F: FnOnce(Uninit<'_, T>) -> Result<PinInit<'_, T>, E>, E> TryPinCtorArgs<T>
    for TryPinCtorFn<F, T>
{
    type Error = E;

    #[inline]
    fn try_pin_init_into(self, uninit: Uninit<'_, T>) -> Result<PinInit<'_, T>, Self::Error> {
        (self.0)(uninit)
    }
}

/// a no-op helper function to guide type inference
///
/// Rust's type inference doesn't understand the indirection from
/// `FnOnce()` to `PinCtorArgs` to `PinCtor`, so use this no-op to guide inference
pub fn try_pin_ctor<T: ?Sized, F: FnOnce(Uninit<T>) -> Result<PinInit<T>, E>, E>(
    f: F,
) -> impl TryPinCtorArgs<T> {
    TryPinCtorFn(f, PhantomData)
}

/// A helper type which converts any Ctor implementation to a `TryCtorArgs` implementation
#[derive(Debug, Clone, Copy)]
pub struct OfPinCtor<Args>(Args);

impl<T: ?Sized + PinCtor<Args>, Args> TryPinCtorArgs<T> for OfPinCtor<Args> {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_pin_init_into(self, uninit: Uninit<'_, T>) -> Result<PinInit<'_, T>, Self::Error> {
        Ok(uninit.pin_init(self.0))
    }

    #[inline]
    #[doc(hidden)]
    fn __is_clone_cheap() -> bool {
        T::__is_args_clone_cheap()
    }
}

/// Converts an argument of `Ctor` to one of `TryCtor`
pub fn of_pin_ctor<Args>(args: Args) -> OfPinCtor<Args> {
    OfPinCtor(args)
}

/// A helper type which converts any `TryCtor<Error = Infallible>` implementation to a `CtorArgs` implementation
#[derive(Debug, Clone, Copy)]
pub struct ToPinCtor<Args>(Args);

impl<T: ?Sized + TryPinCtor<Args, Error = core::convert::Infallible>, Args> PinCtorArgs<T>
    for ToPinCtor<Args>
{
    #[inline]
    fn pin_init_into(self, uninit: Uninit<'_, T>) -> PinInit<'_, T> {
        match uninit.try_pin_init(self.0) {
            Ok(init) => init,
            Err(inf) => match inf {},
        }
    }

    #[inline]
    #[doc(hidden)]
    fn __is_clone_cheap() -> bool {
        T::__is_args_clone_cheap()
    }
}

/// Converts an argument of `Ctor` to one of `TryCtor`
pub fn to_pin_ctor<Args>(args: Args) -> ToPinCtor<Args> {
    ToPinCtor(args)
}
