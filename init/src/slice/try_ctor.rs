//! Constructors for slices

use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

use crate::{
    layout_provider::{HasLayoutProvider, LayoutProvider},
    slice_writer::SliceWriter,
    TryCtor,
};

use super::SliceLayoutProvider;

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

macro_rules! mk_ctor {
    (
        for<$T:ident $(, $U:ident)*> [$($slice_ty:tt)*]
        with ($Arg:ty)
        $((where $($bounds:tt)*))?
        $((array_where $($array_bounds:tt)*))?
        type Error = $Error:ty;
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
            type Error = $Error;
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
        type Error = $Error:ty;
        $(is_zeroed($zeroed_args:pat) $zeroed_imp:block)?
        init($uninit:ident, $args:pat) $imp:block
        $(is_arg_cheap $imp_cheap:block)?
    ) => {
        impl<$T $(, $U)*> TryCtor<$Arg> for [$($slice_ty)*] $(where $($bounds)*)? {
            type Error = $Error;

            fn try_init($uninit: crate::Uninit<'_, Self>, $args: $Arg) -> Result<crate::Init<'_, Self>, Self::Error> $imp
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

        impl<$T $(, $U)*, const N: usize> TryCtor<$Arg> for [$($slice_ty)*; N] $(where $($bounds)*)? {
            type Error = $Error;

            fn try_init(uninit: crate::Uninit<'_, Self>, args: $Arg) -> Result<crate::Init<'_, Self>, Self::Error> {
                uninit.try_init(ArrayAdapter(args))
            }

            $(#[doc(hidden)] fn __is_args_clone_cheap() -> bool $imp_cheap)?
        }
    };
}

impl<T: TryCtor> TryCtor for [T] {
    type Error = T::Error;

    fn try_init(
        uninit: crate::Uninit<'_, Self>,
        (): (),
    ) -> Result<crate::Init<'_, Self>, Self::Error> {
        uninit.try_init(CopyArgs(()))
    }

    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        true
    }
}

impl<T: TryCtor, const N: usize> TryCtor for [T; N] {
    type Error = T::Error;

    fn try_init(
        uninit: crate::Uninit<'_, Self>,
        (): (),
    ) -> Result<crate::Init<'_, Self>, Self::Error> {
        uninit.try_init(CopyArgs(()))
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

impl<T> HasLayoutProvider<UninitSliceLen> for [MaybeUninit<T>] {
    type LayoutProvider = SliceLayoutProvider;
}

// SAFETY: The layout is compatible with cast
unsafe impl<T> LayoutProvider<[MaybeUninit<T>], UninitSliceLen> for SliceLayoutProvider {
    fn layout_of(args: &UninitSliceLen) -> Option<core::alloc::Layout> {
        Layout::array::<T>(args.0).ok()
    }

    unsafe fn cast(ptr: NonNull<u8>, args: &UninitSliceLen) -> NonNull<[MaybeUninit<T>]> {
        NonNull::slice_from_raw_parts(ptr.cast(), args.0)
    }
}

impl<T> TryCtor<UninitSliceLen> for [MaybeUninit<T>] {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_init(
        uninit: crate::Uninit<'_, Self>,
        _: UninitSliceLen,
    ) -> Result<crate::Init<'_, Self>, Self::Error> {
        Ok(uninit.uninit_slice())
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
    for<T, A> [T] with (CopyArgs<A>) (where T: TryCtor<A>, A: Copy,) (array_where T: HasLayoutProvider<A>)
    type Error = T::Error;

    is_zeroed(CopyArgs(args)) {
        crate::layout_provider::is_zeroed::<T, A>(args)
    }

    init(uninit, CopyArgs(args)) {
        let mut writer = SliceWriter::new(uninit);

        while !writer.is_complete() {
            // SAFETY: The write isn't complete
            unsafe { writer.try_init_unchecked(args)? }
        }

        // SAFETY: the writer is complete
        Ok(unsafe { writer.finish_unchecked() })
    }
}

mk_ctor! {
    for<T, A> [T] with (CopyArgsLen<A>) (where T: TryCtor<A>, A: Copy,) (array_where T: HasLayoutProvider<A>)
    type Error = T::Error;

    layout(args)

    is_zeroed(CopyArgsLen(_, args)) {
        crate::layout_provider::is_zeroed::<T, A>(args)
    }

    init(uninit, CopyArgsLen(_, args)) {
        uninit.try_init(CopyArgs(args))
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
    for<T, A> [T] with (CloneArgs<A>) (where T: TryCtor<A>, A: Clone,) (array_where T: HasLayoutProvider<A>)
    type Error = T::Error;

    is_zeroed(CloneArgs(args)) {
        crate::layout_provider::is_zeroed::<T, A>(args)
    }

    init(uninit, CloneArgs(args)) {
        let mut writer = SliceWriter::new(uninit);

        if T::__is_args_clone_cheap() {
            while !writer.is_complete() {
                // SAFETY: The write isn't complete
                unsafe { writer.try_init_unchecked(args.clone())? }
            }
        } else {
            loop {
                match writer.remaining_len() {
                    0 => break,
                    1 => {
                        writer.try_init(args)?;
                        break;
                    }
                    _ => writer.try_init(args.clone())?,
                }
            }
        }

        Ok(writer.finish())
    }
}

mk_ctor! {
    for<T, A> [T] with (CloneArgsLen<A>) (where T: TryCtor<A>, A: Clone,) (array_where T: HasLayoutProvider<A>)
    type Error = T::Error;

    layout(args)

    is_zeroed(CloneArgsLen(_, args)) {
        crate::layout_provider::is_zeroed::<T, A>(args)
    }

    init(uninit, CloneArgsLen(_, args)) {
        uninit.try_init(CloneArgs(args))
    }
}

/// An initializer argument to initialize a slice with the items of the iterator
///
/// NOTE: this will take at most enough elements as needed to fill up the slice, and no more
///
/// The initializer will error if not enough elements are passed in to completely fill up the slice
pub struct IterInit<I>(pub I);

/// An initializer argument to initialize a slice with the items of the iterator
///
/// NOTE: this will take at most enough elements as needed to fill up the slice, and no more
///
/// The initializer will error if not enough elements are passed in to completely fill up the slice
pub struct IterLenInit<I>(pub usize, pub I);

impl<I: ExactSizeIterator> IterLenInit<I> {
    /// Create a new `IterLenInit` form an [`ExactSizeIterator`]
    pub fn new(iter: I) -> Self {
        Self(iter.len(), iter)
    }
}

/// An error for the [`IterInit`] type
pub enum IterInitError<E> {
    /// If not enough elements were in the iterator to fill up the slice
    NotEnoughItems,
    /// If any item in the slice failed to initialize
    InitError(E),
}

mk_ctor! {
    for<T, I> [T] with (IterInit<I>) (where I: Iterator, T: TryCtor<I::Item>,)
    type Error = IterInitError<T::Error>;

    init(uninit, IterInit(args)) {
        let mut writer = SliceWriter::new(uninit);

        args.take(writer.remaining_len())
            // SAFETY: we take up to the remaining length of the writer elements, which means if the closure is called
            // there is enough space in the writer to initialize an element
            .try_for_each(|arg| unsafe { writer.try_init_unchecked(arg) })
            .map_err(IterInitError::InitError)?;

        writer.try_finish().ok_or(IterInitError::NotEnoughItems)
    }
}

mk_ctor! {
    for<T, I> [T] with (IterLenInit<I>) (where I: Iterator, T: TryCtor<I::Item>,)
    type Error = IterInitError<T::Error>;

    layout(args)

    init(uninit, IterLenInit(_, args)) {
        uninit.try_init(IterInit(args))
    }
}
