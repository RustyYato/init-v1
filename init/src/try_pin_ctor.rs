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
) -> impl TryPinCtorArgs<T, Error = E> {
    TryPinCtorFn(f, PhantomData)
}

/// A helper type which converts any Ctor implementation to a `TryCtorArgs` implementation
#[derive(Debug, Clone, Copy)]
pub struct OfPinCtor<Args, Err = core::convert::Infallible>(Args, PhantomData<fn() -> Err>);

impl<T: ?Sized + PinCtor<Args>, Args, Err> TryPinCtorArgs<T> for OfPinCtor<Args, Err> {
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

impl<T: ?Sized + crate::layout_provider::HasLayoutProvider<Args>, Args, Err>
    crate::layout_provider::HasLayoutProvider<OfPinCtor<Args, Err>> for T
{
    type LayoutProvider = OfPinCtorLayoutProvider;
}

/// The layout provider for `OfPinCtor`
pub struct OfPinCtorLayoutProvider;

// SAFETY: guaranteed by T::LayoutProvider
unsafe impl<T: ?Sized + crate::layout_provider::HasLayoutProvider<Args>, Args, Err>
    crate::layout_provider::LayoutProvider<T, OfPinCtor<Args, Err>> for OfPinCtorLayoutProvider
{
    fn layout_of(OfPinCtor(args, _): &OfPinCtor<Args, Err>) -> Option<core::alloc::Layout> {
        crate::layout_provider::layout_of::<T, Args>(args)
    }

    unsafe fn cast(
        ptr: core::ptr::NonNull<u8>,
        OfPinCtor(args, _): &OfPinCtor<Args, Err>,
    ) -> core::ptr::NonNull<T> {
        // SAFETY: guaranteed by caller
        unsafe { crate::layout_provider::cast::<T, Args>(ptr, args) }
    }

    fn is_zeroed(OfPinCtor(args, _): &OfPinCtor<Args, Err>) -> bool {
        crate::layout_provider::is_zeroed::<T, Args>(args)
    }
}

/// Converts an argument of `Ctor` to one of `TryCtor`
pub fn of_pin_ctor<Args>(args: Args) -> OfPinCtor<Args> {
    OfPinCtor(args, PhantomData)
}

/// Converts an argument of `Ctor` to one of `TryCtor`
pub fn of_pin_ctor_any_err<Args, Err>(args: Args) -> OfPinCtor<Args, Err> {
    OfPinCtor(args, PhantomData)
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

impl<T: ?Sized + crate::layout_provider::HasLayoutProvider<Args>, Args>
    crate::layout_provider::HasLayoutProvider<ToPinCtor<Args>> for T
{
    type LayoutProvider = ToPinCtorLayoutProvider;
}

/// The layout provider for `ToPinCtor`
pub struct ToPinCtorLayoutProvider;

// SAFETY: guaranteed by T::LayoutProvider
unsafe impl<T: ?Sized + crate::layout_provider::HasLayoutProvider<Args>, Args>
    crate::layout_provider::LayoutProvider<T, ToPinCtor<Args>> for ToPinCtorLayoutProvider
{
    fn layout_of(ToPinCtor(args): &ToPinCtor<Args>) -> Option<core::alloc::Layout> {
        crate::layout_provider::layout_of::<T, Args>(args)
    }

    unsafe fn cast(
        ptr: core::ptr::NonNull<u8>,
        ToPinCtor(args): &ToPinCtor<Args>,
    ) -> core::ptr::NonNull<T> {
        // SAFETY: guaranteed by caller
        unsafe { crate::layout_provider::cast::<T, Args>(ptr, args) }
    }

    fn is_zeroed(ToPinCtor(args): &ToPinCtor<Args>) -> bool {
        crate::layout_provider::is_zeroed::<T, Args>(args)
    }
}

/// Converts an argument of `Ctor` to one of `TryCtor`
pub fn to_pin_ctor<Args>(args: Args) -> ToPinCtor<Args> {
    ToPinCtor(args)
}
