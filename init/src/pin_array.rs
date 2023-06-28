//! constructors for arrays

use core::{mem::MaybeUninit, pin::Pin};

use crate::{
    array::ArrayLayoutProvider,
    config_value::{ConfigValue, PinCloneTag, PinMoveTag, PinTakeTag},
    layout_provider::HasLayoutProvider,
    pin_ctor::{PinCloneCtor, PinMoveCtor, PinTakeCtor},
    pin_slice::*,
    PinCtor,
};

struct ArrayAdapter<A>(pub A);

impl<const N: usize, T, A> PinCtor<ArrayAdapter<A>> for [T; N]
where
    [T]: PinCtor<A>,
{
    fn pin_init(
        uninit: crate::Uninit<'_, Self>,
        args: ArrayAdapter<A>,
    ) -> crate::PinInit<'_, Self> {
        let init = uninit.as_slice().pin_init(args.0);
        // SAFETY: this init is the same array as `uninit`, so it has the right length
        unsafe { init.into_array_unchecked() }
    }

    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        <[T] as PinCtor<A>>::__is_args_clone_cheap()
    }
}

impl<const N: usize, T: PinCtor> PinCtor for [T; N] {
    #[inline]
    fn pin_init(uninit: crate::Uninit<'_, Self>, (): ()) -> crate::PinInit<'_, Self> {
        uninit.pin_init(CopyArgs(()))
    }
}

impl<const N: usize, T> HasLayoutProvider<UninitSliceLen> for [MaybeUninit<T>; N] {
    type LayoutProvider = ArrayLayoutProvider<SliceLenLayoutProvider>;
}

impl<const N: usize, T> PinCtor<UninitSliceLen> for [MaybeUninit<T>; N] {
    fn pin_init(uninit: crate::Uninit<'_, Self>, _: UninitSliceLen) -> crate::PinInit<'_, Self> {
        uninit.pin_init(())
    }
}

impl<const N: usize, T: PinCtor<Args>, Args: Copy> PinCtor<CopyArgs<Args>> for [T; N] {
    #[inline]
    fn pin_init(uninit: crate::Uninit<'_, Self>, args: CopyArgs<Args>) -> crate::PinInit<'_, Self> {
        uninit.pin_init(ArrayAdapter(args))
    }
}

impl<const N: usize, T: PinCtor<Args>, Args: Clone> PinCtor<CloneArgs<Args>> for [T; N] {
    #[inline]
    fn pin_init(
        uninit: crate::Uninit<'_, Self>,
        args: CloneArgs<Args>,
    ) -> crate::PinInit<'_, Self> {
        uninit.pin_init(ArrayAdapter(args))
    }
}

impl<const N: usize, T: PinCtor<Args>, Args: Copy> HasLayoutProvider<CopyArgsLen<Args>> for [T; N]
where
    T: HasLayoutProvider<Args>,
{
    type LayoutProvider = ArrayLayoutProvider<SliceLenLayoutProvider>;
}

impl<const N: usize, T: PinCtor<Args>, Args: Copy> PinCtor<CopyArgsLen<Args>> for [T; N] {
    #[inline]
    fn pin_init(
        uninit: crate::Uninit<'_, Self>,
        CopyArgsLen(_, args): CopyArgsLen<Args>,
    ) -> crate::PinInit<'_, Self> {
        uninit.pin_init(CopyArgs(args))
    }
}

impl<const N: usize, T: PinCtor<Args>, Args: Clone> HasLayoutProvider<CloneArgsLen<Args>> for [T; N]
where
    T: HasLayoutProvider<Args>,
{
    type LayoutProvider = ArrayLayoutProvider<SliceLenLayoutProvider>;
}

impl<const N: usize, T: PinCtor<Args>, Args: Clone> PinCtor<CloneArgsLen<Args>> for [T; N] {
    #[inline]
    fn pin_init(
        uninit: crate::Uninit<'_, Self>,
        CloneArgsLen(_, args): CloneArgsLen<Args>,
    ) -> crate::PinInit<'_, Self> {
        uninit.pin_init(CloneArgs(args))
    }
}

impl<const N: usize, T: PinMoveCtor> PinMoveCtor for [T; N] {
    const IS_MOVE_TRIVIAL: ConfigValue<Self, PinMoveTag> = {
        // SAFETY: if T is trivially movable then [T; N] is also trivially movable
        unsafe { T::IS_MOVE_TRIVIAL.cast() }
    };

    fn pin_move_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        p: crate::PinInit<Self>,
    ) -> crate::PinInit<'this, Self> {
        uninit.pin_init(ArrayAdapter(p.to_slice()))
    }
}

impl<const N: usize, T: PinTakeCtor> PinTakeCtor for [T; N] {
    const IS_TAKE_TRIVIAL: ConfigValue<Self, PinTakeTag> = {
        // SAFETY: if T is trivially takable then [T; N] is also trivially takable
        unsafe { T::IS_TAKE_TRIVIAL.cast() }
    };

    fn pin_take_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        p: Pin<&mut Self>,
    ) -> crate::PinInit<'this, Self> {
        uninit.pin_init(ArrayAdapter(p as Pin<&mut [_]>))
    }
}

impl<const N: usize, T: PinCloneCtor> PinCloneCtor for [T; N] {
    const IS_CLONE_TRIVIAL: ConfigValue<Self, PinCloneTag> = {
        // SAFETY: if T is trivially clone-able then [T; N] is also trivially clone-able
        unsafe { T::IS_CLONE_TRIVIAL.cast() }
    };

    fn pin_clone_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        p: Pin<&Self>,
    ) -> crate::PinInit<'this, Self> {
        uninit.pin_init(ArrayAdapter(p as Pin<&[_]>))
    }
}
