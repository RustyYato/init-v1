//! Constructors for slices

use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

use crate::{
    config_value::{CloneTag, ConfigValue, MoveTag, TakeTag},
    interface::{CloneCtor, MoveCtor, TakeCtor},
    layout_provider::{HasLayoutProvider, LayoutProvider},
    slice_writer::SliceWriter,
    Ctor,
};

impl<T: Ctor> Ctor for [T] {
    #[inline]
    fn init(uninit: crate::Uninit<'_, Self>, (): ()) -> crate::Init<'_, Self> {
        uninit.init(CopyArgs(()))
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

impl<T> Ctor<UninitSliceLen> for [MaybeUninit<T>] {
    fn init(uninit: crate::Uninit<'_, Self>, _: UninitSliceLen) -> crate::Init<'_, Self> {
        uninit.uninit_slice()
    }
}

/// A slice constructor which copies the argument and uses it to construct each element of the slice
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct CopyArgs<Args>(pub Args);

impl<T: Ctor<Args>, Args: Copy> Ctor<CopyArgs<Args>> for [T] {
    #[inline]
    fn init(
        uninit: crate::Uninit<'_, Self>,
        CopyArgs(args): CopyArgs<Args>,
    ) -> crate::Init<'_, Self> {
        let mut writer = SliceWriter::new(uninit);

        while !writer.is_complete() {
            // SAFETY: The write isn't complete
            unsafe { writer.init_unchecked(args) }
        }

        // SAFETY: the writer is complete
        unsafe { writer.finish_unchecked() }
    }
}

/// A slice constructor which clones the argument and uses it to construct each element of the slice
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct CloneArgs<Args>(pub Args);

impl<T: Ctor<Args>, Args: Clone> Ctor<CloneArgs<Args>> for [T] {
    #[inline]
    fn init(
        uninit: crate::Uninit<'_, Self>,
        CloneArgs(args): CloneArgs<Args>,
    ) -> crate::Init<'_, Self> {
        let mut writer = SliceWriter::new(uninit);

        if T::__is_args_clone_cheap() {
            while !writer.is_complete() {
                // SAFETY: The write isn't complete
                unsafe { writer.init_unchecked(args.clone()) }
            }
        } else {
            loop {
                match writer.remaining_len() {
                    0 => break,
                    1 => {
                        writer.init(args);
                        break;
                    }
                    _ => writer.init(args.clone()),
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

impl<T: Ctor<Args>, Args: Copy> HasLayoutProvider<CopyArgsLen<Args>> for [T]
where
    T: HasLayoutProvider<Args>,
{
    type LayoutProvider = SliceLenLayoutProvider;
}

// SAFETY: The layout is compatible with cast
unsafe impl<T: Ctor<Args>, Args: Copy> LayoutProvider<[T], CopyArgsLen<Args>>
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

impl<T: Ctor<Args>, Args: Copy> Ctor<CopyArgsLen<Args>> for [T] {
    #[inline]
    fn init(
        uninit: crate::Uninit<'_, Self>,
        CopyArgsLen(_, args): CopyArgsLen<Args>,
    ) -> crate::Init<'_, Self> {
        uninit.init(CopyArgs(args))
    }
}

/// A slice constructor which clones the argument and uses it to construct each element of the slice
///
/// It also has a `LayoutProvider` which allocates enough spaces for `self.0` items
#[derive(Debug, Clone, Copy)]
pub struct CloneArgsLen<Args>(pub usize, pub Args);

impl<T: Ctor<Args>, Args: Clone> HasLayoutProvider<CloneArgsLen<Args>> for [T]
where
    T: HasLayoutProvider<Args>,
{
    type LayoutProvider = SliceLenLayoutProvider;
}

// SAFETY: The layout is compatible with cast
unsafe impl<T: Ctor<Args>, Args: Clone> LayoutProvider<[T], CloneArgsLen<Args>>
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

impl<T: Ctor<Args>, Args: Clone> Ctor<CloneArgsLen<Args>> for [T] {
    #[inline]
    fn init(
        uninit: crate::Uninit<'_, Self>,
        CloneArgsLen(_, args): CloneArgsLen<Args>,
    ) -> crate::Init<'_, Self> {
        uninit.init(CloneArgs(args))
    }
}

/// A layout provider for slices
pub struct SliceLenLayoutProvider;

impl<T: MoveCtor> MoveCtor for [T] {
    const IS_MOVE_TRIVIAL: ConfigValue<Self, MoveTag> = {
        // SAFETY: if T is trivially movable then [T] is also trivially movable
        unsafe { T::IS_MOVE_TRIVIAL.cast() }
    };

    fn move_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        p: crate::Init<Self>,
    ) -> crate::Init<'this, Self> {
        if uninit.len() != p.get().len() {
            length_error(uninit.len(), p.get().len())
        }

        if T::IS_MOVE_TRIVIAL.get() {
            // SAFETY: the p was leaked
            let init = unsafe { uninit.copy_from_slice_unchecked(p.get()) };
            core::mem::forget(p);
            init
        } else {
            let mut writer = SliceWriter::new(uninit);

            for source in p {
                // SAFETY: p and the slice have the same length
                unsafe { writer.init_unchecked(source) }
            }

            // SAFETY:p and the slice have the same length
            unsafe { writer.finish_unchecked() }
        }
    }
}

impl<T: TakeCtor> TakeCtor for [T] {
    const IS_TAKE_TRIVIAL: ConfigValue<Self, TakeTag> = {
        // SAFETY: if T is trivially takable then [T] is also trivially takable
        unsafe { T::IS_TAKE_TRIVIAL.cast() }
    };

    fn take_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        p: &mut Self,
    ) -> crate::Init<'this, Self> {
        if uninit.len() != p.len() {
            length_error(uninit.len(), p.len())
        }

        if T::IS_TAKE_TRIVIAL.get() {
            // SAFETY: `T::IS_TAKE_TRIVIAL` guarantees that this is safe
            unsafe { uninit.copy_from_slice_unchecked(p) }
        } else {
            let mut writer = SliceWriter::new(uninit);

            for source in p {
                // SAFETY: p and the slice have the same length
                unsafe { writer.init_unchecked(source) }
            }

            // SAFETY:p and the slice have the same length
            unsafe { writer.finish_unchecked() }
        }
    }
}

impl<T: CloneCtor> CloneCtor for [T] {
    const IS_CLONE_TRIVIAL: ConfigValue<Self, CloneTag> = {
        // SAFETY: if T is trivially clone-able then [T] is also trivially clone-able
        unsafe { T::IS_CLONE_TRIVIAL.cast() }
    };

    fn clone_ctor<'this>(uninit: crate::Uninit<'this, Self>, p: &Self) -> crate::Init<'this, Self> {
        if uninit.len() != p.len() {
            length_error(uninit.len(), p.len())
        }

        if T::IS_CLONE_TRIVIAL.get() {
            // SAFETY: `T::IS_CLONE_TRIVIAL` guarantees that this is safe
            unsafe { uninit.copy_from_slice_unchecked(p) }
        } else {
            let mut writer = SliceWriter::new(uninit);

            for source in p {
                // SAFETY: p and the slice have the same length
                unsafe { writer.init_unchecked(source) }
            }

            // SAFETY:p and the slice have the same length
            unsafe { writer.finish_unchecked() }
        }
    }
}

fn length_error(expected: usize, found: usize) -> ! {
    panic!("Could not initialize from slice because lengths didn't match, expected length: {expected} but got {found}")
}
