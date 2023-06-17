//! The core interfaces used to initialize types

use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{Init, Uninit};

/// A type which is constructable using `Args`
pub trait TryCtor<Args = ()> {
    /// The error type of a failed initialization
    type Error;

    /// Initialize a the type `Self` using `args: Args`
    fn try_init(uninit: Uninit<'_, Self>, args: Args) -> Result<Init<'_, Self>, Self::Error>;

    #[inline]
    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        false
    }
}

/// A type which can construct a `T`
pub trait TryCtorArgs<T: ?Sized> {
    /// The error type of a failed initialization
    type Error;

    /// Initialize a the type `T` using `self`
    fn try_init_into(self, uninit: Uninit<'_, T>) -> Result<Init<'_, T>, Self::Error>;
}

impl<T: ?Sized, Args: TryCtorArgs<T>> TryCtor<Args> for T {
    type Error = Args::Error;

    #[inline]
    fn try_init(uninit: Uninit<'_, Self>, args: Args) -> Result<Init<'_, Self>, Self::Error> {
        args.try_init_into(uninit)
    }
}

impl<T> TryCtor for MaybeUninit<T> {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_init(uninit: Uninit<'_, Self>, (): ()) -> Result<Init<'_, Self>, Self::Error> {
        Ok(uninit.uninit())
    }
}

struct TryCtorFn<F, T: ?Sized>(F, PhantomData<T>);

impl<T: ?Sized, F: FnOnce(Uninit<'_, T>) -> Result<Init<'_, T>, E>, E> TryCtorArgs<T>
    for TryCtorFn<F, T>
{
    type Error = E;

    #[inline]
    fn try_init_into(self, uninit: Uninit<'_, T>) -> Result<Init<'_, T>, Self::Error> {
        (self.0)(uninit)
    }
}

/// a no-op helper function to guide type inference
///
/// Rust's type inference doesn't understand the indirection from
/// `FnOnce()` to `CtorArgs` to `Ctor`, so use this no-op to guide inference
pub fn try_ctor<T: ?Sized, F: FnOnce(Uninit<T>) -> Result<Init<T>, E>, E>(
    f: F,
) -> impl TryCtorArgs<T> {
    TryCtorFn(f, PhantomData)
}
