//! Constructors for slices

use core::mem::MaybeUninit;

use crate::{
    array::ArrayLayoutProvider, layout_provider::HasLayoutProvider, try_slice::*, TryCtor,
};

/// An adapter to convert a slice initializer to an array initializer
pub struct ArrayAdapter<A>(A);

impl<const N: usize, T, A> TryCtor<ArrayAdapter<A>> for [T; N]
where
    [T]: TryCtor<A>,
{
    type Error = <[T] as TryCtor<A>>::Error;

    fn try_init(
        uninit: crate::Uninit<'_, Self>,
        args: ArrayAdapter<A>,
    ) -> Result<crate::Init<'_, Self>, Self::Error> {
        let init = uninit.as_slice().try_init(args.0)?;
        // SAFETY: this init is the same array as `uninit`, so it has the right length
        Ok(unsafe { init.into_array_unchecked() })
    }

    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        <[T] as TryCtor<A>>::__is_args_clone_cheap()
    }
}

impl<const N: usize, T: TryCtor> TryCtor for [T; N] {
    type Error = T::Error;

    fn try_init(
        uninit: crate::Uninit<'_, Self>,
        (): (),
    ) -> Result<crate::Init<'_, Self>, Self::Error> {
        uninit.try_init(CopyArgs(()))
    }
}

impl<const N: usize, T> HasLayoutProvider<UninitSliceLen> for [MaybeUninit<T>; N] {
    type LayoutProvider = ArrayLayoutProvider<SliceLenLayoutProvider>;
}

impl<const N: usize, T> TryCtor<UninitSliceLen> for [MaybeUninit<T>; N] {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_init(
        uninit: crate::Uninit<'_, Self>,
        _: UninitSliceLen,
    ) -> Result<crate::Init<'_, Self>, Self::Error> {
        Ok(uninit.init(()))
    }
}

impl<const N: usize, T: TryCtor<Args>, Args: Copy> TryCtor<CopyArgs<Args>> for [T; N] {
    type Error = T::Error;

    fn try_init(
        uninit: crate::Uninit<'_, Self>,
        args: CopyArgs<Args>,
    ) -> Result<crate::Init<'_, Self>, Self::Error> {
        uninit.try_init(ArrayAdapter(args))
    }
}

impl<const N: usize, T: TryCtor<Args>, Args: Clone> TryCtor<CloneArgs<Args>> for [T; N] {
    type Error = T::Error;

    fn try_init(
        uninit: crate::Uninit<'_, Self>,
        args: CloneArgs<Args>,
    ) -> Result<crate::Init<'_, Self>, Self::Error> {
        uninit.try_init(ArrayAdapter(args))
    }
}

impl<const N: usize, T: TryCtor<Args>, Args: Copy> HasLayoutProvider<CopyArgsLen<Args>> for [T; N]
where
    T: HasLayoutProvider<Args>,
{
    type LayoutProvider = ArrayLayoutProvider<SliceLenLayoutProvider>;
}

impl<const N: usize, T: TryCtor<Args>, Args: Copy> TryCtor<CopyArgsLen<Args>> for [T; N] {
    type Error = T::Error;

    fn try_init(
        uninit: crate::Uninit<'_, Self>,
        CopyArgsLen(_, args): CopyArgsLen<Args>,
    ) -> Result<crate::Init<'_, Self>, Self::Error> {
        uninit.try_init(CopyArgs(args))
    }
}

impl<const N: usize, T, Args: Clone> HasLayoutProvider<CloneArgsLen<Args>> for [T; N]
where
    T: HasLayoutProvider<Args>,
{
    type LayoutProvider = ArrayLayoutProvider<SliceLenLayoutProvider>;
}

impl<const N: usize, T: TryCtor<Args>, Args: Clone> TryCtor<CloneArgsLen<Args>> for [T; N] {
    type Error = T::Error;

    fn try_init(
        uninit: crate::Uninit<'_, Self>,
        CloneArgsLen(_, args): CloneArgsLen<Args>,
    ) -> Result<crate::Init<'_, Self>, Self::Error> {
        uninit.try_init(CloneArgs(args))
    }
}

impl<const N: usize, T: TryCtor<I::Item>, I: Iterator> TryCtor<IterInit<I>> for [T; N] {
    type Error = IterInitError<T::Error>;

    fn try_init(
        uninit: crate::Uninit<'_, Self>,
        args: IterInit<I>,
    ) -> Result<crate::Init<'_, Self>, Self::Error> {
        uninit.try_init(ArrayAdapter(args))
    }
}

impl<const N: usize, T: TryCtor<I::Item>, I: Iterator> TryCtor<IterLenInit<I>> for [T; N] {
    type Error = IterInitError<T::Error>;

    fn try_init(
        uninit: crate::Uninit<'_, Self>,
        IterLenInit(_, args): IterLenInit<I>,
    ) -> Result<crate::Init<'_, Self>, Self::Error> {
        uninit.try_init(IterInit(args))
    }
}
