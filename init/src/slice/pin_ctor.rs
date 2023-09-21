//! Constructors for slices

use core::{alloc::Layout, mem::MaybeUninit, pin::Pin, ptr::NonNull};

use crate::{
    config_value::{ConfigValue, PinCloneTag, PinMoveTag, PinTakeTag},
    layout_provider::{HasLayoutProvider, LayoutProvider},
    pin_ctor::{PinCloneCtor, PinMoveCtor, PinTakeCtor},
    pin_slice_writer::PinSliceWriter,
    try_pin_ctor::{of_pin_ctor, to_pin_ctor},
    PinCtor,
};

use super::SliceLayoutProvider;

/// An adapter to convert a slice initializer to an array initializer
pub struct ArrayAdapter<A>(A);

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

macro_rules! mk_ctor {
    (
        for<$T:ident $(, $U:ident)*> [$($slice_ty:tt)*]
        with ($Arg:ty)
        $((where $($bounds:tt)*))?
        $((array_where $($array_bounds:tt)*))?
        layout($layout_args:ident) $(is_zeroed($zeroed_args:pat) $zeroed_imp:block)?
        init($uninit:ident, $args:pat) $imp:block
        $(is_arg_cheap $imp_cheap:block)?
    ) => {
        impl<$T $(, $U)*> HasLayoutProvider<$Arg> for [$($slice_ty)*] $(where $($array_bounds)*)? {
            type LayoutProvider = SliceLayoutProvider;
        }

        // SAFETY: The layout is compatible with cast
        unsafe impl<$T $(, $U)*> LayoutProvider<[$($slice_ty)*], $Arg> for SliceLayoutProvider $(where $($array_bounds)*)? {
            fn layout_of($layout_args: &$Arg) -> Option<core::alloc::Layout> {
                    Layout::array::<T>($layout_args.0).ok()
            }

            unsafe fn cast(ptr: NonNull<u8>, $layout_args: &$Arg) -> NonNull<[$($slice_ty)*]> {
                    NonNull::slice_from_raw_parts(ptr.cast(), $layout_args.0)
            }

            $(
                fn is_zeroed($zeroed_args: &$Arg) -> bool $zeroed_imp
            )?
        }

        mk_ctor! {
            for<$T $(, $U)*> [$($slice_ty)*]
            with ($Arg)
            $((where $($bounds)*))?
            $((array_where $($array_bounds)*))?
            $(is_zeroed($zeroed_args) $zeroed_imp)?
            init($uninit, $args) $imp
            $(is_arg_cheap $imp_cheap)?
        }
    };
    (
        for<$T:ident $(, $U:ident)*> [$($slice_ty:tt)*]
        with ($Arg:ty)
        $((where $($bounds:tt)*))?
        $((array_where $($array_bounds:tt)*))?
        $(is_zeroed($zeroed_args:pat) $zeroed_imp:block)?
        init($uninit:ident, $args:pat) $imp:block
        $(is_arg_cheap $imp_cheap:block)?
    ) => {
        impl<$T $(, $U)*> PinCtor<$Arg> for [$($slice_ty)*] $(where $($bounds)*)? {
            fn pin_init($uninit: crate::Uninit<'_, Self>, $args: $Arg) -> crate::PinInit<'_, Self> $imp
            $(#[doc(hidden)] fn __is_args_clone_cheap() -> bool $imp_cheap)?
        }

        impl<$T $(, $U)*, const N: usize> HasLayoutProvider<$Arg> for [$($slice_ty)*; N] $(where $($array_bounds)*)? {
            type LayoutProvider = SliceLayoutProvider;
        }

        // SAFETY: The layout is compatible with cast
        unsafe impl<$T $(, $U)*, const N: usize> LayoutProvider<[$($slice_ty)*; N], $Arg> for SliceLayoutProvider $(where $($array_bounds)*)? {
            fn layout_of(_: &$Arg) -> Option<core::alloc::Layout> {
                Some(core::alloc::Layout::new::<[$($slice_ty)*; N]>())
            }

            unsafe fn cast(ptr: NonNull<u8>, _: &$Arg) -> NonNull<[$($slice_ty)*; N]> {
                ptr.cast()
            }

            $(
                fn is_zeroed($zeroed_args: &$Arg) -> bool $zeroed_imp
            )?
        }

        impl<$T $(, $U)*, const N: usize> PinCtor<$Arg> for [$($slice_ty)*; N] $(where $($bounds)*)? {
            fn pin_init(uninit: crate::Uninit<'_, Self>, args: $Arg) -> crate::PinInit<'_, Self> {
                uninit.pin_init(ArrayAdapter(args))
            }

            $(#[doc(hidden)] fn __is_args_clone_cheap() -> bool $imp_cheap)?
        }
    };
}

impl<T> PinCtor for [MaybeUninit<T>] {
    fn pin_init(uninit: crate::Uninit<'_, Self>, (): ()) -> crate::PinInit<'_, Self> {
        uninit.uninit_slice().pin()
    }

    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        true
    }
}

impl<T, const N: usize> PinCtor for [MaybeUninit<T>; N] {
    fn pin_init(uninit: crate::Uninit<'_, Self>, (): ()) -> crate::PinInit<'_, Self> {
        uninit.pin_init(())
    }

    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        true
    }
}

/// A slice constructor which clones the argument and uses it to construct each element of the slice
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct UninitSliceLen(pub usize);

mk_ctor! {
    for<T> [MaybeUninit<T>] with (UninitSliceLen)

    layout(args)

    init(uninit, _) {
        uninit.uninit_slice().pin()
    }

    is_arg_cheap {
        true
    }
}

/// A slice constructor which copies the argument and uses it to construct each element of the slice
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct CopyArgs<Args>(pub Args);

/// A slice constructor which copies the argument and uses it to construct each element of the slice
///
/// It also has a `LayoutProvider` which allocates enough spaces for `self.0` items
#[derive(Debug, Clone, Copy)]
pub struct CopyArgsLen<Args>(pub usize, pub Args);

mk_ctor! {
    for<T, Args> [T] with (CopyArgs<Args>)
     (where
        T: PinCtor<Args>,
        Args: Copy)
    (array_where
        T: HasLayoutProvider<Args>)

    is_zeroed(CopyArgs(args)) {
        crate::layout_provider::is_zeroed::<T, Args>(args)
    }

    init(uninit, CopyArgs(args)) {
        uninit.pin_init(to_pin_ctor(super::try_pin_ctor::CopyArgs(of_pin_ctor(args))))
    }

    is_arg_cheap {
        true
    }
}

mk_ctor! {
    for<T, Args> [T] with (CopyArgsLen<Args>)
     (where
        T: PinCtor<Args>,
        Args: Copy,)
    (array_where
        T: HasLayoutProvider<Args>)

    layout(args)

    is_zeroed(CopyArgsLen(_, args)) {
        crate::layout_provider::is_zeroed::<T, Args>(args)
    }

    init(uninit, CopyArgsLen(_, args)) {
        uninit.pin_init(to_pin_ctor(super::try_pin_ctor::CopyArgs(of_pin_ctor(args))))
    }

    is_arg_cheap {
        true
    }
}

/// A slice constructor which clones the argument and uses it to construct each element of the slice
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct CloneArgs<Args>(pub Args);

/// A slice constructor which clones the argument and uses it to construct each element of the slice
///
/// It also has a `LayoutProvider` which allocates enough spaces for `self.0` items
#[derive(Debug, Clone, Copy)]
pub struct CloneArgsLen<Args>(pub usize, pub Args);

mk_ctor! {
    for<T, Args> [T] with (CloneArgs<Args>)
     (where
        T: PinCtor<Args>,
        Args: Clone)
    (array_where
        T: HasLayoutProvider<Args>)

    is_zeroed(CloneArgs(args)) {
        crate::layout_provider::is_zeroed::<T, Args>(args)
    }

    init(uninit, CloneArgs(args)) {
        uninit.pin_init(to_pin_ctor(super::try_pin_ctor::CloneArgs(of_pin_ctor(args))))
    }

    is_arg_cheap {
        T::__is_args_clone_cheap()
    }
}

mk_ctor! {
    for<T, Args> [T] with (CloneArgsLen<Args>)
     (where
        T: PinCtor<Args>,
        Args: Clone)
    (array_where
        T: HasLayoutProvider<Args>)

    layout(args)

    is_zeroed(CloneArgsLen(_, args)) {
        crate::layout_provider::is_zeroed::<T, Args>(args)
    }

    init(uninit, CloneArgsLen(_, args)) {
        uninit.pin_init(to_pin_ctor(super::try_pin_ctor::CloneArgs(of_pin_ctor(args))))
}

    is_arg_cheap {
        T::__is_args_clone_cheap()
    }
}

impl<T: PinMoveCtor> PinMoveCtor for [T] {
    const IS_MOVE_TRIVIAL: ConfigValue<Self, PinMoveTag> = {
        // SAFETY: if T is trivially movable then [T] is also trivially movable
        unsafe { T::IS_MOVE_TRIVIAL.cast() }
    };

    fn pin_move_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        p: crate::PinInit<Self>,
    ) -> crate::PinInit<'this, Self> {
        if uninit.len() != p.get().len() {
            length_error(uninit.len(), p.get().len())
        }

        if T::IS_MOVE_TRIVIAL.get() {
            // SAFETY: the p was leaked
            let init = unsafe { uninit.copy_from_slice_unchecked(p.get()) };
            core::mem::forget(p);
            init.pin()
        } else {
            let mut writer = PinSliceWriter::new(uninit);

            for source in p {
                // SAFETY: p and the slice have the same length
                unsafe { writer.pin_init_unchecked(source) }
            }

            // SAFETY:p and the slice have the same length
            unsafe { writer.finish_unchecked() }
        }
    }
}

impl<T: PinTakeCtor> PinTakeCtor for [T] {
    const IS_TAKE_TRIVIAL: ConfigValue<Self, PinTakeTag> = {
        // SAFETY: if T is trivially takable then [T] is also trivially takable
        unsafe { T::IS_TAKE_TRIVIAL.cast() }
    };

    fn pin_take_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        p: Pin<&mut Self>,
    ) -> crate::PinInit<'this, Self> {
        if uninit.len() != p.len() {
            length_error(uninit.len(), p.len())
        }

        if T::IS_TAKE_TRIVIAL.get() {
            // SAFETY: `T::IS_TAKE_TRIVIAL` guarantees that this is safe
            unsafe { uninit.copy_from_slice_unchecked(&p) }.pin()
        } else {
            let mut writer = PinSliceWriter::new(uninit);

            // SAFETY: we don't move the values behind the pointer
            let p = unsafe { Pin::into_inner_unchecked(p) };

            for source in p {
                // SAFETY: p and the slice have the same length
                unsafe { writer.pin_init_unchecked(Pin::new_unchecked(source)) }
            }

            // SAFETY:p and the slice have the same length
            unsafe { writer.finish_unchecked() }
        }
    }
}

impl<T: PinCloneCtor> PinCloneCtor for [T] {
    const IS_CLONE_TRIVIAL: ConfigValue<Self, PinCloneTag> = {
        // SAFETY: if T is trivially clone-able then [T] is also trivially clone-able
        unsafe { T::IS_CLONE_TRIVIAL.cast() }
    };

    fn pin_clone_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        p: Pin<&Self>,
    ) -> crate::PinInit<'this, Self> {
        if uninit.len() != p.len() {
            length_error(uninit.len(), p.len())
        }

        if T::IS_CLONE_TRIVIAL.get() {
            // SAFETY: `T::IS_TAKE_TRIVIAL` guarantees that this is safe
            unsafe { uninit.copy_from_slice_unchecked(&p) }.pin()
        } else {
            let mut writer = PinSliceWriter::new(uninit);

            // SAFETY: we don't move the values behind the pointer
            let p = unsafe { Pin::into_inner_unchecked(p) };

            for source in p {
                // SAFETY: p and the slice have the same length
                unsafe { writer.pin_init_unchecked(Pin::new_unchecked(source)) }
            }

            // SAFETY:p and the slice have the same length
            unsafe { writer.finish_unchecked() }
        }
    }
}

fn length_error(expected: usize, found: usize) -> ! {
    panic!("Could not initialize from slice because lengths didn't match, expected length: {expected} but got {found}")
}
