use core::alloc::Layout;

use crate::{
    interface::{LayoutProvider, MaybeLayoutProvider, NoLayoutProvider},
    slice_writer::SliceWriter,
    Ctor,
};

impl<T: Ctor> Ctor for [T] {
    type LayoutProvider = NoLayoutProvider;

    #[inline]
    fn init(uninit: crate::Uninit<'_, Self>, (): ()) -> crate::Init<'_, Self> {
        uninit.init(CopyArgs(()))
    }
}

#[repr(transparent)]
pub struct CopyArgs<Args>(pub Args);

impl<T: Ctor<Args>, Args: Copy> Ctor<CopyArgs<Args>> for [T] {
    type LayoutProvider = NoLayoutProvider;

    #[inline]
    fn init(
        uninit: crate::Uninit<'_, Self>,
        CopyArgs(args): CopyArgs<Args>,
    ) -> crate::Init<'_, Self> {
        let mut writer = SliceWriter::new(uninit);

        while !writer.is_complete() {
            writer.init(args);
        }

        writer.finish()
    }
}

#[repr(transparent)]
pub struct CloneArgs<Args>(pub Args);

impl<T: Ctor<Args>, Args: Clone> Ctor<CloneArgs<Args>> for [T] {
    type LayoutProvider = NoLayoutProvider;

    #[inline]
    fn init(
        uninit: crate::Uninit<'_, Self>,
        CloneArgs(args): CloneArgs<Args>,
    ) -> crate::Init<'_, Self> {
        let mut writer = SliceWriter::new(uninit);

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

        writer.finish()
    }
}

pub struct SliceLenLayoutProvider;

impl LayoutProvider for SliceLenLayoutProvider {}

// SAFETY: The layout is compatible with cast
unsafe impl<'a, T: Ctor<&'a T>> MaybeLayoutProvider<[T], &'a [T]> for SliceLenLayoutProvider {
    fn layout_of(args: &&[T]) -> Option<core::alloc::Layout> {
        Layout::array::<T>(args.len()).ok()
    }

    unsafe fn cast(ptr: core::ptr::NonNull<u8>, args: &&[T]) -> core::ptr::NonNull<[T]> {
        core::ptr::NonNull::slice_from_raw_parts(ptr.cast(), args.len())
    }
}

impl<'a, T: Ctor<&'a T>> Ctor<&'a [T]> for [T] {
    type LayoutProvider = SliceLenLayoutProvider;

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

// SAFETY: The layout is compatible with cast
unsafe impl<'a, T: Ctor<&'a mut T>> MaybeLayoutProvider<[T], &'a mut [T]>
    for SliceLenLayoutProvider
{
    fn layout_of(args: &&mut [T]) -> Option<core::alloc::Layout> {
        Layout::array::<T>(args.len()).ok()
    }

    unsafe fn cast(ptr: core::ptr::NonNull<u8>, args: &&mut [T]) -> core::ptr::NonNull<[T]> {
        core::ptr::NonNull::slice_from_raw_parts(ptr.cast(), args.len())
    }
}

impl<'a, T: Ctor<&'a mut T>> Ctor<&'a mut [T]> for [T] {
    type LayoutProvider = SliceLenLayoutProvider;

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
