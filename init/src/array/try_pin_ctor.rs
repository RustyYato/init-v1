//! Constructors for slices
//
use core::mem::MaybeUninit;

use crate::{
    array::ArrayLayoutProvider, layout_provider::HasLayoutProvider, slice::try_pin_ctor::*,
    TryPinCtor,
};

/// An adapter to convert a slice initializer to an array initializer
pub struct ArrayAdapter<A>(pub A);

impl<const N: usize, T, A> TryPinCtor<ArrayAdapter<A>> for [T; N]
where
    [T]: TryPinCtor<A>,
{
    type Error = <[T] as TryPinCtor<A>>::Error;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        args: ArrayAdapter<A>,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        let init = uninit.as_slice().try_pin_init(args.0)?;
        // SAFETY: this init is the same array as `uninit`, so it has the right length
        Ok(unsafe { init.into_array_unchecked() })
    }
}

impl<const N: usize, T: TryPinCtor> TryPinCtor for [T; N] {
    type Error = T::Error;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        (): (),
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        uninit.try_pin_init(CopyArgs(()))
    }
}

impl<const N: usize, T> HasLayoutProvider<UninitSliceLen> for [MaybeUninit<T>; N] {
    type LayoutProvider = ArrayLayoutProvider<SliceLenLayoutProvider>;
}

impl<const N: usize, T> TryPinCtor<UninitSliceLen> for [MaybeUninit<T>; N] {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        _: UninitSliceLen,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        uninit.try_pin_init(())
    }
}

impl<const N: usize, T: TryPinCtor<Args>, Args: Copy> TryPinCtor<CopyArgs<Args>> for [T; N] {
    type Error = T::Error;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        args: CopyArgs<Args>,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        uninit.try_pin_init(ArrayAdapter(args))
    }
}

impl<const N: usize, T: TryPinCtor<Args>, Args: Clone> TryPinCtor<CloneArgs<Args>> for [T; N] {
    type Error = T::Error;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        args: CloneArgs<Args>,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        uninit.try_pin_init(ArrayAdapter(args))
    }
}

impl<const N: usize, T: TryPinCtor<Args>, Args: Copy> HasLayoutProvider<CopyArgsLen<Args>>
    for [T; N]
where
    T: HasLayoutProvider<Args>,
{
    type LayoutProvider = ArrayLayoutProvider<SliceLenLayoutProvider>;
}

impl<const N: usize, T: TryPinCtor<Args>, Args: Copy> TryPinCtor<CopyArgsLen<Args>> for [T; N] {
    type Error = T::Error;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        CopyArgsLen(_, args): CopyArgsLen<Args>,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        uninit.try_pin_init(CopyArgs(args))
    }
}

impl<const N: usize, T, Args: Clone> HasLayoutProvider<CloneArgsLen<Args>> for [T; N]
where
    T: HasLayoutProvider<Args>,
{
    type LayoutProvider = ArrayLayoutProvider<SliceLenLayoutProvider>;
}

impl<const N: usize, T: TryPinCtor<Args>, Args: Clone> TryPinCtor<CloneArgsLen<Args>> for [T; N] {
    type Error = T::Error;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        CloneArgsLen(_, args): CloneArgsLen<Args>,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        uninit.try_pin_init(CloneArgs(args))
    }
}

impl<const N: usize, T: TryPinCtor<I::Item>, I: Iterator> TryPinCtor<IterInit<I>> for [T; N] {
    type Error = IterInitError<T::Error>;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        args: IterInit<I>,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        uninit.try_pin_init(ArrayAdapter(args))
    }
}

impl<const N: usize, T: TryPinCtor<I::Item>, I: Iterator> TryPinCtor<IterLenInit<I>> for [T; N] {
    type Error = IterInitError<T::Error>;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        args: IterLenInit<I>,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        uninit.try_pin_init(ArrayAdapter(args))
    }
}
