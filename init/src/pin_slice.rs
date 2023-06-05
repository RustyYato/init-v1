//! constructors for slices

use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

use crate::{
    layout_provider::{HasLayoutProvider, LayoutProvider, MaybeLayoutProvider, NoLayoutProvider},
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
unsafe impl<T> MaybeLayoutProvider<[MaybeUninit<T>], UninitSliceLen> for SliceLenLayoutProvider {
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

impl<T: PinCtor<Args>, Args> HasLayoutProvider<CopyArgs<Args>> for [T] {
    type LayoutProvider = NoLayoutProvider;
}

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

impl<T: PinCtor<Args>, Args: Clone> HasLayoutProvider<CloneArgs<Args>> for [T] {
    type LayoutProvider = NoLayoutProvider;
}

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
unsafe impl<T: PinCtor<Args>, Args: Copy> MaybeLayoutProvider<[T], CopyArgsLen<Args>>
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
unsafe impl<T: PinCtor<Args>, Args: Clone> MaybeLayoutProvider<[T], CloneArgsLen<Args>>
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

impl<T, Args> LayoutProvider<[T], Args> for SliceLenLayoutProvider
where
    Self: MaybeLayoutProvider<[T], Args>,
    [T]: PinCtor<Args>,
{
}

// SAFETY: The layout is compatible with cast
unsafe impl<'a, T: PinCtor<&'a T>> MaybeLayoutProvider<[T], &'a [T]> for SliceLenLayoutProvider {
    fn layout_of(args: &&[T]) -> Option<core::alloc::Layout> {
        Layout::array::<T>(args.len()).ok()
    }

    unsafe fn cast(ptr: NonNull<u8>, args: &&[T]) -> NonNull<[T]> {
        NonNull::slice_from_raw_parts(ptr.cast(), args.len())
    }
}

impl<'a, T: PinCtor<&'a T>> PinCtor<&'a [T]> for [T] {
    #[inline]
    fn pin_init<'u>(uninit: crate::Uninit<'u, Self>, source: &'a [T]) -> crate::PinInit<'u, Self> {
        if uninit.len() != source.len() {
            length_error(uninit.len(), source.len())
        }

        let mut writer = PinSliceWriter::new(uninit);

        for source in source {
            // SAFETY: The source and iterator have the same length
            // so if the iterator has more elements, then the writer is
            // also incomplete
            unsafe { writer.pin_init_unchecked(source) };
        }

        // SAFETY: The source and iterator have the same length
        // so if the iterator has no more elements, then the writer
        // is complete
        unsafe { writer.finish_unchecked() }
    }
}

// SAFETY: The layout is compatible with cast
unsafe impl<'a, T: PinCtor<&'a mut T>> MaybeLayoutProvider<[T], &'a mut [T]>
    for SliceLenLayoutProvider
{
    fn layout_of(args: &&mut [T]) -> Option<core::alloc::Layout> {
        Layout::array::<T>(args.len()).ok()
    }

    unsafe fn cast(ptr: NonNull<u8>, args: &&mut [T]) -> NonNull<[T]> {
        NonNull::slice_from_raw_parts(ptr.cast(), args.len())
    }
}

impl<'a, T: PinCtor<&'a mut T>> PinCtor<&'a mut [T]> for [T] {
    #[inline]
    fn pin_init<'u>(
        uninit: crate::Uninit<'u, Self>,
        source: &'a mut [T],
    ) -> crate::PinInit<'u, Self> {
        if uninit.len() != source.len() {
            length_error(uninit.len(), source.len())
        }

        let mut writer = PinSliceWriter::new(uninit);

        for source in source {
            // SAFETY: The source and iterator have the same length
            // so if the iterator has more elements, then the writer is
            // also incomplete
            unsafe { writer.pin_init_unchecked(source) };
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
