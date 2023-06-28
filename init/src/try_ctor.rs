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
) -> impl TryCtorArgs<T, Error = E> {
    TryCtorFn(f, PhantomData)
}

/// A helper type which converts any Ctor implementation to a `TryCtorArgs` implementation
#[derive(Debug)]
pub struct OfCtor<Args, Err = core::convert::Infallible>(Args, PhantomData<fn() -> Err>);

impl<Args: Copy, Err> Copy for OfCtor<Args, Err> {}
impl<Args: Clone, Err> Clone for OfCtor<Args, Err> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T: ?Sized + Ctor<Args>, Args, Err> TryCtorArgs<T> for OfCtor<Args, Err> {
    type Error = Err;

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

impl<T: ?Sized + crate::layout_provider::HasLayoutProvider<Args>, Args, Err>
    crate::layout_provider::HasLayoutProvider<OfCtor<Args, Err>> for T
{
    type LayoutProvider = OfCtorLayoutProvider<T::LayoutProvider>;
}

/// The layout provider for `OfCtor`
pub struct OfCtorLayoutProvider<L>(L);

// SAFETY: guaranteed by T::LayoutProvider
unsafe impl<T: ?Sized, Args, Err, L: crate::layout_provider::LayoutProvider<T, Args>>
    crate::layout_provider::LayoutProvider<T, OfCtor<Args, Err>> for OfCtorLayoutProvider<L>
{
    fn layout_of(OfCtor(args, _): &OfCtor<Args, Err>) -> Option<core::alloc::Layout> {
        L::layout_of(args)
    }

    unsafe fn cast(
        ptr: core::ptr::NonNull<u8>,
        OfCtor(args, _): &OfCtor<Args, Err>,
    ) -> core::ptr::NonNull<T> {
        // SAFETY: guaranteed by caller
        unsafe { L::cast(ptr, args) }
    }

    fn is_zeroed(OfCtor(args, _): &OfCtor<Args, Err>) -> bool {
        L::is_zeroed(args)
    }
}

/// Converts an argument of `Ctor` to one of `TryCtor`
pub fn of_ctor<Args>(args: Args) -> OfCtor<Args> {
    OfCtor(args, PhantomData)
}

/// Converts an argument of `Ctor` to one of `TryCtor`
pub fn of_ctor_any_err<Args, Err>(args: Args) -> OfCtor<Args, Err> {
    OfCtor(args, PhantomData)
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
    type LayoutProvider = ToCtorLayoutProvider<T::LayoutProvider>;
}

/// The layout provider for `ToCtor`
pub struct ToCtorLayoutProvider<L>(L);

// SAFETY: guaranteed by T::LayoutProvider
unsafe impl<T: ?Sized, Args, L: crate::layout_provider::LayoutProvider<T, Args>>
    crate::layout_provider::LayoutProvider<T, ToCtor<Args>> for ToCtorLayoutProvider<L>
{
    fn layout_of(ToCtor(args): &ToCtor<Args>) -> Option<core::alloc::Layout> {
        L::layout_of(args)
    }

    unsafe fn cast(
        ptr: core::ptr::NonNull<u8>,
        ToCtor(args): &ToCtor<Args>,
    ) -> core::ptr::NonNull<T> {
        // SAFETY: guaranteed by caller
        unsafe { L::cast(ptr, args) }
    }

    fn is_zeroed(ToCtor(args): &ToCtor<Args>) -> bool {
        L::is_zeroed(args)
    }
}

/// Converts an argument of `Ctor` to one of `TryCtor`
pub fn to_ctor<Args>(args: Args) -> ToCtor<Args> {
    ToCtor(args)
}
