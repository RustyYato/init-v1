//! The core interfaces used to initialize types

use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{Ctor, CtorArgs, Init, Uninit};

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

    #[inline]
    #[doc(hidden)]
    fn __is_clone_cheap() -> bool {
        false
    }
}

impl<T: ?Sized, Args: TryCtorArgs<T>> TryCtor<Args> for T {
    type Error = Args::Error;

    #[inline]
    fn try_init(uninit: Uninit<'_, Self>, args: Args) -> Result<Init<'_, Self>, Self::Error> {
        args.try_init_into(uninit)
    }

    #[inline]
    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        Args::__is_clone_cheap()
    }
}

impl<T> TryCtor for MaybeUninit<T> {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_init(uninit: Uninit<'_, Self>, (): ()) -> Result<Init<'_, Self>, Self::Error> {
        Ok(uninit.uninit())
    }

    #[inline]
    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        true
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

/// A helper type which converts any Ctor implementation to a `TryCtorArgs` implementation
#[derive(Debug, Clone, Copy)]
pub struct OfCtor<Args>(Args);

impl<T: ?Sized + Ctor<Args>, Args> TryCtorArgs<T> for OfCtor<Args> {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_init_into(self, uninit: Uninit<'_, T>) -> Result<Init<'_, T>, Self::Error> {
        Ok(uninit.init(self.0))
    }

    #[inline]
    #[doc(hidden)]
    fn __is_clone_cheap() -> bool {
        T::__is_args_clone_cheap()
    }
}

impl<T: ?Sized + crate::layout_provider::HasLayoutProvider<Args>, Args>
    crate::layout_provider::HasLayoutProvider<OfCtor<Args>> for T
{
    type LayoutProvider = OfCtorLayoutProvider;
}

/// The layout provider for `OfCtor`
pub struct OfCtorLayoutProvider;

// SAFETY: guaranteed by T::LayoutProvider
unsafe impl<T: ?Sized + crate::layout_provider::HasLayoutProvider<Args>, Args>
    crate::layout_provider::LayoutProvider<T, OfCtor<Args>> for OfCtorLayoutProvider
{
    fn layout_of(OfCtor(args): &OfCtor<Args>) -> Option<core::alloc::Layout> {
        crate::layout_provider::layout_of::<T, Args>(args)
    }

    unsafe fn cast(
        ptr: core::ptr::NonNull<u8>,
        OfCtor(args): &OfCtor<Args>,
    ) -> core::ptr::NonNull<T> {
        // SAFETY: guaranteed by caller
        unsafe { crate::layout_provider::cast::<T, Args>(ptr, args) }
    }

    fn is_zeroed(OfCtor(args): &OfCtor<Args>) -> bool {
        crate::layout_provider::is_zeroed::<T, Args>(args)
    }
}

/// Converts an argument of `Ctor` to one of `TryCtor`
pub fn of_ctor<Args>(args: Args) -> OfCtor<Args> {
    OfCtor(args)
}

/// A helper type which converts any `TryCtor<Error = Infallible>` implementation to a `CtorArgs` implementation
#[derive(Debug, Clone, Copy)]
pub struct ToCtor<Args>(Args);

impl<T: ?Sized + TryCtor<Args, Error = core::convert::Infallible>, Args> CtorArgs<T>
    for ToCtor<Args>
{
    #[inline]
    fn init_into(self, uninit: Uninit<'_, T>) -> Init<'_, T> {
        match uninit.try_init(self.0) {
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
    crate::layout_provider::HasLayoutProvider<ToCtor<Args>> for T
{
    type LayoutProvider = ToCtorLayoutProvider;
}

/// The layout provider for `ToCtor`
pub struct ToCtorLayoutProvider;

// SAFETY: guaranteed by T::LayoutProvider
unsafe impl<T: ?Sized + crate::layout_provider::HasLayoutProvider<Args>, Args>
    crate::layout_provider::LayoutProvider<T, ToCtor<Args>> for ToCtorLayoutProvider
{
    fn layout_of(ToCtor(args): &ToCtor<Args>) -> Option<core::alloc::Layout> {
        crate::layout_provider::layout_of::<T, Args>(args)
    }

    unsafe fn cast(
        ptr: core::ptr::NonNull<u8>,
        ToCtor(args): &ToCtor<Args>,
    ) -> core::ptr::NonNull<T> {
        // SAFETY: guaranteed by caller
        unsafe { crate::layout_provider::cast::<T, Args>(ptr, args) }
    }

    fn is_zeroed(ToCtor(args): &ToCtor<Args>) -> bool {
        crate::layout_provider::is_zeroed::<T, Args>(args)
    }
}

/// Converts an argument of `Ctor` to one of `TryCtor`
pub fn to_ctor<Args>(args: Args) -> ToCtor<Args> {
    ToCtor(args)
}
