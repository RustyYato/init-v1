//! Constructors for arrays

use core::mem::MaybeUninit;

use crate::{
    array::ArrayLayoutProvider,
    config_value::{CloneTag, ConfigValue, MoveTag, TakeTag},
    ctor::{CloneCtor, MoveCtor, TakeCtor},
    layout_provider::{HasLayoutProvider, SizedLayoutProvider},
    slice::ctor::*,
    Ctor,
};

/// An adapter to convert a slice initializer to an array initializer
pub struct ArrayAdapter<A>(A);

impl<const N: usize, T, A> Ctor<ArrayAdapter<A>> for [T; N]
where
    [T]: Ctor<A>,
{
    fn init(uninit: crate::Uninit<'_, Self>, args: ArrayAdapter<A>) -> crate::Init<'_, Self> {
        let init = uninit.as_slice().init(args.0);
        // SAFETY: this init is the same array as `uninit`, so it has the right length
        unsafe { init.into_array_unchecked() }
    }

    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        <[T] as Ctor<A>>::__is_args_clone_cheap()
    }
}

impl<const N: usize, T: Ctor> Ctor for [T; N] {
    #[inline]
    fn init(uninit: crate::Uninit<'_, Self>, (): ()) -> crate::Init<'_, Self> {
        uninit.init(CopyArgs(()))
    }
}

impl<const N: usize, T> HasLayoutProvider<UninitSliceLen> for [MaybeUninit<T>; N] {
    type LayoutProvider = SizedLayoutProvider;
}

impl<const N: usize, T> Ctor<UninitSliceLen> for [MaybeUninit<T>; N] {
    fn init(uninit: crate::Uninit<'_, Self>, _: UninitSliceLen) -> crate::Init<'_, Self> {
        uninit.init(())
    }
}

impl<const N: usize, T: Ctor<Args>, Args: Copy> Ctor<CopyArgs<Args>> for [T; N] {
    #[inline]
    fn init(uninit: crate::Uninit<'_, Self>, args: CopyArgs<Args>) -> crate::Init<'_, Self> {
        uninit.init(ArrayAdapter(args))
    }
}

impl<const N: usize, T: Ctor<Args>, Args: Clone> Ctor<CloneArgs<Args>> for [T; N] {
    #[inline]
    fn init(uninit: crate::Uninit<'_, Self>, args: CloneArgs<Args>) -> crate::Init<'_, Self> {
        uninit.init(ArrayAdapter(args))
    }
}

impl<const N: usize, T, Args: Copy> HasLayoutProvider<CopyArgsLen<Args>> for [T; N]
where
    T: HasLayoutProvider<Args>,
{
    type LayoutProvider = ArrayLayoutProvider<SliceLenLayoutProvider>;
}

impl<const N: usize, T: Ctor<Args>, Args: Copy> Ctor<CopyArgsLen<Args>> for [T; N] {
    #[inline]
    fn init(
        uninit: crate::Uninit<'_, Self>,
        CopyArgsLen(_, args): CopyArgsLen<Args>,
    ) -> crate::Init<'_, Self> {
        uninit.init(CopyArgs(args))
    }
}

impl<const N: usize, T, Args: Clone> HasLayoutProvider<CloneArgsLen<Args>> for [T; N]
where
    T: HasLayoutProvider<Args>,
{
    type LayoutProvider = ArrayLayoutProvider<SliceLenLayoutProvider>;
}

impl<const N: usize, T: Ctor<Args>, Args: Clone> Ctor<CloneArgsLen<Args>> for [T; N] {
    #[inline]
    fn init(
        uninit: crate::Uninit<'_, Self>,
        CloneArgsLen(_, args): CloneArgsLen<Args>,
    ) -> crate::Init<'_, Self> {
        uninit.init(CloneArgs(args))
    }
}

impl<const N: usize, T: MoveCtor> MoveCtor for [T; N] {
    const IS_MOVE_TRIVIAL: ConfigValue<Self, MoveTag> = {
        // SAFETY: if T is trivially movable then [T; N] is also trivially movable
        unsafe { T::IS_MOVE_TRIVIAL.cast() }
    };

    fn move_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        p: crate::Init<Self>,
    ) -> crate::Init<'this, Self> {
        uninit.init(ArrayAdapter(p.to_slice()))
    }
}

impl<const N: usize, T: TakeCtor> TakeCtor for [T; N] {
    const IS_TAKE_TRIVIAL: ConfigValue<Self, TakeTag> = {
        // SAFETY: if T is trivially takable then [T; N] is also trivially takable
        unsafe { T::IS_TAKE_TRIVIAL.cast() }
    };

    fn take_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        p: &mut Self,
    ) -> crate::Init<'this, Self> {
        uninit.init(ArrayAdapter(&mut p[..]))
    }
}

impl<const N: usize, T: CloneCtor> CloneCtor for [T; N] {
    const IS_CLONE_TRIVIAL: ConfigValue<Self, CloneTag> = {
        // SAFETY: if T is trivially clone-able then [T; N] is also trivially clone-able
        unsafe { T::IS_CLONE_TRIVIAL.cast() }
    };

    fn clone_ctor<'this>(uninit: crate::Uninit<'this, Self>, p: &Self) -> crate::Init<'this, Self> {
        uninit.init(ArrayAdapter(&p[..]))
    }
}
