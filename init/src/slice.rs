//! Constructors for slices

use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

use crate::{
    layout_provider::{HasLayoutProvider, LayoutProvider, MaybeLayoutProvider, NoLayoutProvider},
    slice_writer::SliceWriter,
    Ctor,
};

impl<T> HasLayoutProvider for [T] {
    type LayoutProvider = NoLayoutProvider;
}

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
unsafe impl<T> MaybeLayoutProvider<[MaybeUninit<T>], UninitSliceLen> for SliceLenLayoutProvider {
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

impl<T: Ctor<Args>, Args> HasLayoutProvider<CopyArgs<Args>> for [T] {
    type LayoutProvider = NoLayoutProvider;
}

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

impl<T: Ctor<Args>, Args: Clone> HasLayoutProvider<CloneArgs<Args>> for [T] {
    type LayoutProvider = NoLayoutProvider;
}

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

impl<T: Ctor<Args>, Args: Copy> HasLayoutProvider<CopyArgsLen<Args>> for [T] {
    type LayoutProvider = SliceLenLayoutProvider;
}

// SAFETY: The layout is compatible with cast
unsafe impl<T: Ctor<Args>, Args: Copy> MaybeLayoutProvider<[T], CopyArgsLen<Args>>
    for SliceLenLayoutProvider
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

impl<T: Ctor<Args>, Args: Clone> HasLayoutProvider<CloneArgsLen<Args>> for [T] {
    type LayoutProvider = SliceLenLayoutProvider;
}

// SAFETY: The layout is compatible with cast
unsafe impl<T: Ctor<Args>, Args: Clone> MaybeLayoutProvider<[T], CloneArgsLen<Args>>
    for SliceLenLayoutProvider
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

impl<T, Args> LayoutProvider<[T], Args> for SliceLenLayoutProvider
where
    Self: MaybeLayoutProvider<[T], Args>,
    [T]: Ctor<Args>,
{
}

impl<T> HasLayoutProvider<&[T]> for [T] {
    type LayoutProvider = SliceLenLayoutProvider;
}

// SAFETY: The layout is compatible with cast
unsafe impl<'a, T> MaybeLayoutProvider<[T], &'a [T]> for SliceLenLayoutProvider {
    fn layout_of(args: &&[T]) -> Option<core::alloc::Layout> {
        Layout::array::<T>(args.len()).ok()
    }

    unsafe fn cast(ptr: NonNull<u8>, args: &&[T]) -> NonNull<[T]> {
        NonNull::slice_from_raw_parts(ptr.cast(), args.len())
    }
}

impl<'a, T: Ctor<&'a T>> Ctor<&'a [T]> for [T] {
    #[inline]
    fn init<'u>(uninit: crate::Uninit<'u, Self>, source: &'a [T]) -> crate::Init<'u, Self> {
        if uninit.len() != source.len() {
            length_error(uninit.len(), source.len())
        }

        let mut writer = SliceWriter::new(uninit);

        for source in source {
            // SAFETY: The source and iterator have the same length
            // so if the iterator has more elements, then the writer is
            // also incomplete
            unsafe { writer.init_unchecked(source) };
        }

        // SAFETY: The source and iterator have the same length
        // so if the iterator has no more elements, then the writer
        // is complete
        unsafe { writer.finish_unchecked() }
    }
}

impl<T> HasLayoutProvider<&mut [T]> for [T] {
    type LayoutProvider = SliceLenLayoutProvider;
}

// SAFETY: The layout is compatible with cast
unsafe impl<'a, T> MaybeLayoutProvider<[T], &'a mut [T]> for SliceLenLayoutProvider {
    fn layout_of(args: &&mut [T]) -> Option<core::alloc::Layout> {
        Layout::array::<T>(args.len()).ok()
    }

    unsafe fn cast(ptr: NonNull<u8>, args: &&mut [T]) -> NonNull<[T]> {
        NonNull::slice_from_raw_parts(ptr.cast(), args.len())
    }
}

impl<'a, T: Ctor<&'a mut T>> Ctor<&'a mut [T]> for [T] {
    #[inline]
    fn init<'u>(uninit: crate::Uninit<'u, Self>, source: &'a mut [T]) -> crate::Init<'u, Self> {
        if uninit.len() != source.len() {
            length_error(uninit.len(), source.len())
        }

        let mut writer = SliceWriter::new(uninit);

        for source in source {
            // SAFETY: The source and iterator have the same length
            // so if the iterator has more elements, then the writer is
            // also incomplete
            unsafe { writer.init_unchecked(source) };
        }

        // SAFETY: The source and iterator have the same length
        // so if the iterator has no more elements, then the writer
        // is complete
        unsafe { writer.finish_unchecked() }
    }
}

fn length_error(expected: usize, found: usize) -> ! {
    panic!("Could not initialize from slice because lengths didn't match, expected length: {expected} but got {found}")
}
