use crate::{slice_writer::SliceWriter, Ctor};

impl<T: Ctor> Ctor for [T] {
    #[inline]
    fn init(uninit: crate::Uninit<'_, Self>, (): ()) -> crate::Init<'_, Self> {
        uninit.init(CopyArgs(()))
    }
}

#[repr(transparent)]
pub struct CopyArgs<Args>(pub Args);

impl<T: Ctor<Args>, Args: Copy> Ctor<CopyArgs<Args>> for [T] {
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

impl<'a, T: Ctor<&'a T>> Ctor<&'a [T]> for [T] {
    #[inline]
    fn init<'u>(uninit: crate::Uninit<'u, Self>, source: &'a [T]) -> crate::Init<'u, Self> {
        assert_eq!(uninit.len(), source.len());

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

impl<'a, T: Ctor<&'a mut T>> Ctor<&'a mut [T]> for [T] {
    #[inline]
    fn init<'u>(uninit: crate::Uninit<'u, Self>, source: &'a mut [T]) -> crate::Init<'u, Self> {
        assert_eq!(uninit.len(), source.len());

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
