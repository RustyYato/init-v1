//! constructors for slices

use core::{alloc::Layout, mem::MaybeUninit, pin::Pin, ptr::NonNull};

use crate::{
    interface::{PinCloneCtor, PinMoveCtor, PinTakeCtor},
    layout_provider::{HasLayoutProvider, LayoutProvider},
    pin_slice_writer::PinSliceWriter,
    PinCtor,
};

impl<T: PinCtor> PinCtor for [T] {
    #[inline]
    fn pin_init(uninit: crate::Uninit<'_, Self>, (): ()) -> crate::PinInit<'_, Self> {
        uninit.pin_init(CopyArgs(()))
    }
}

/// A slice constructor which clones the argument and uses it to construct each element of the slice
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct UninitSliceLen(pub usize);

impl<T> HasLayoutProvider<UninitSliceLen> for [MaybeUninit<T>] {
    type LayoutProvider = SliceLenLayoutProvider;
}

// SAFETY: The layout is compatible with cast
unsafe impl<T> LayoutProvider<[MaybeUninit<T>], UninitSliceLen> for SliceLenLayoutProvider {
    fn layout_of(args: &UninitSliceLen) -> Option<core::alloc::Layout> {
        Layout::array::<T>(args.0).ok()
    }

    unsafe fn cast(ptr: NonNull<u8>, args: &UninitSliceLen) -> NonNull<[MaybeUninit<T>]> {
        NonNull::slice_from_raw_parts(ptr.cast(), args.0)
    }
}

impl<T> PinCtor<UninitSliceLen> for [MaybeUninit<T>] {
    fn pin_init(uninit: crate::Uninit<'_, Self>, _: UninitSliceLen) -> crate::PinInit<'_, Self> {
        uninit.uninit_slice().pin()
    }
}

/// A slice constructor which copies the argument and uses it to construct each element of the slice
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct CopyArgs<Args>(pub Args);

impl<T: PinCtor<Args>, Args: Copy> PinCtor<CopyArgs<Args>> for [T] {
    #[inline]
    fn pin_init(
        uninit: crate::Uninit<'_, Self>,
        CopyArgs(args): CopyArgs<Args>,
    ) -> crate::PinInit<'_, Self> {
        let mut writer = PinSliceWriter::new(uninit);

        while !writer.is_complete() {
            // SAFETY: The write isn't complete
            unsafe { writer.pin_init_unchecked(args) }
        }

        // SAFETY: the writer is complete
        unsafe { writer.finish_unchecked() }
    }
}

/// A slice constructor which clones the argument and uses it to construct each element of the slice
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct CloneArgs<Args>(pub Args);

impl<T: PinCtor<Args>, Args: Clone> PinCtor<CloneArgs<Args>> for [T] {
    #[inline]
    fn pin_init(
        uninit: crate::Uninit<'_, Self>,
        CloneArgs(args): CloneArgs<Args>,
    ) -> crate::PinInit<'_, Self> {
        let mut writer = PinSliceWriter::new(uninit);

        if T::__is_args_clone_cheap() {
            while !writer.is_complete() {
                // SAFETY: The write isn't complete
                unsafe { writer.pin_init_unchecked(args.clone()) }
            }
        } else {
            loop {
                match writer.remaining_len() {
                    0 => break,
                    1 => {
                        writer.pin_init(args);
                        break;
                    }
                    _ => writer.pin_init(args.clone()),
                }
            }
        }

        writer.finish()
    }
}

/// A slice constructor which copies the argument and uses it to construct each element of the slice
///
/// It also has a `LayoutProvider` which allocates enough spaces for `self.0` items
#[derive(Debug, Clone, Copy)]
pub struct CopyArgsLen<Args>(pub usize, pub Args);

impl<T: PinCtor<Args>, Args: Copy> HasLayoutProvider<CopyArgsLen<Args>> for [T]
where
    T: HasLayoutProvider<Args>,
{
    type LayoutProvider = SliceLenLayoutProvider;
}

// SAFETY: The layout is compatible with cast
unsafe impl<T: PinCtor<Args>, Args: Copy> LayoutProvider<[T], CopyArgsLen<Args>>
    for SliceLenLayoutProvider
where
    T: HasLayoutProvider<Args>,
{
    fn layout_of(args: &CopyArgsLen<Args>) -> Option<Layout> {
        Layout::array::<T>(args.0).ok()
    }

    unsafe fn cast(ptr: NonNull<u8>, args: &CopyArgsLen<Args>) -> NonNull<[T]> {
        NonNull::slice_from_raw_parts(ptr.cast(), args.0)
    }

    fn is_zeroed(CopyArgsLen(_, args): &CopyArgsLen<Args>) -> bool {
        crate::layout_provider::is_zeroed::<T, Args>(args)
    }
}

impl<T: PinCtor<Args>, Args: Copy> PinCtor<CopyArgsLen<Args>> for [T] {
    #[inline]
    fn pin_init(
        uninit: crate::Uninit<'_, Self>,
        CopyArgsLen(_, args): CopyArgsLen<Args>,
    ) -> crate::PinInit<'_, Self> {
        uninit.pin_init(CopyArgs(args))
    }
}

/// A slice constructor which clones the argument and uses it to construct each element of the slice
///
/// It also has a `LayoutProvider` which allocates enough spaces for `self.0` items
#[derive(Debug, Clone, Copy)]
pub struct CloneArgsLen<Args>(pub usize, pub Args);

impl<T: PinCtor<Args>, Args: Clone> HasLayoutProvider<CloneArgsLen<Args>> for [T]
where
    T: HasLayoutProvider<Args>,
{
    type LayoutProvider = SliceLenLayoutProvider;
}

// SAFETY: The layout is compatible with cast
unsafe impl<T: PinCtor<Args>, Args: Clone> LayoutProvider<[T], CloneArgsLen<Args>>
    for SliceLenLayoutProvider
where
    T: HasLayoutProvider<Args>,
{
    fn layout_of(args: &CloneArgsLen<Args>) -> Option<Layout> {
        Layout::array::<T>(args.0).ok()
    }

    unsafe fn cast(ptr: NonNull<u8>, args: &CloneArgsLen<Args>) -> NonNull<[T]> {
        NonNull::slice_from_raw_parts(ptr.cast(), args.0)
    }

    fn is_zeroed(CloneArgsLen(_, args): &CloneArgsLen<Args>) -> bool {
        crate::layout_provider::is_zeroed::<T, Args>(args)
    }
}

impl<T: PinCtor<Args>, Args: Clone> PinCtor<CloneArgsLen<Args>> for [T] {
    #[inline]
    fn pin_init(
        uninit: crate::Uninit<'_, Self>,
        CloneArgsLen(_, args): CloneArgsLen<Args>,
    ) -> crate::PinInit<'_, Self> {
        uninit.pin_init(CloneArgs(args))
    }
}

/// A layout provider for slices
pub struct SliceLenLayoutProvider;

impl<T: PinMoveCtor> PinMoveCtor for [T] {
    const IS_MOVE_TRIVIAL: crate::interface::ConfigValue<Self, crate::interface::PinMoveTag> = {
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
    const IS_TAKE_TRIVIAL: crate::interface::ConfigValue<Self, crate::interface::PinTakeTag> = {
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
    const IS_CLONE_TRIVIAL: crate::interface::ConfigValue<Self, crate::interface::PinCloneTag> = {
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

fn length_error(expected: usize, found: usize) -> ! {
    panic!("Could not initialize from slice because lengths didn't match, expected length: {expected} but got {found}")
}
