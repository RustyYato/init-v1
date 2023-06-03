use crate::{slice_writer::SliceWriter, Ctor};

impl<T: Ctor> Ctor for [T] {
    #[inline]
    fn init(uninit: crate::Uninit<'_, Self>, (): ()) -> crate::Init<'_, Self> {
        uninit.init(CopyArgs(()))
    }
}

#[repr(transparent)]
struct CopyArgs<Args>(pub Args);

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
struct CloneArgs<Args>(pub Args);

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
