//! Constructors for slices

use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

use crate::{
    layout_provider::{HasLayoutProvider, LayoutProvider},
    pin_slice_writer::PinSliceWriter,
    TryPinCtor,
};

impl<T: TryPinCtor> TryPinCtor for [T] {
    type Error = T::Error;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        (): (),
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        uninit.try_pin_init(CopyArgs(()))
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

impl<T> TryPinCtor<UninitSliceLen> for [MaybeUninit<T>] {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        _: UninitSliceLen,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        Ok(uninit.uninit_slice().pin())
    }
}

/// A slice constructor which copies the argument and uses it to construct each element of the slice
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct CopyArgs<Args>(pub Args);

impl<T: TryPinCtor<Args>, Args: Copy> TryPinCtor<CopyArgs<Args>> for [T] {
    type Error = T::Error;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        CopyArgs(args): CopyArgs<Args>,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        let mut writer = PinSliceWriter::new(uninit);

        while !writer.is_complete() {
            // SAFETY: The write isn't complete
            unsafe { writer.try_pin_init_unchecked(args)? }
        }

        // SAFETY: the writer is complete
        Ok(unsafe { writer.finish_unchecked() })
    }
}

/// A slice constructor which clones the argument and uses it to construct each element of the slice
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct CloneArgs<Args>(pub Args);

impl<T: TryPinCtor<Args>, Args: Clone> TryPinCtor<CloneArgs<Args>> for [T] {
    type Error = T::Error;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        CloneArgs(args): CloneArgs<Args>,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        let mut writer = PinSliceWriter::new(uninit);

        if T::__is_args_clone_cheap() {
            while !writer.is_complete() {
                // SAFETY: The write isn't complete
                unsafe { writer.try_pin_init_unchecked(args.clone())? }
            }
        } else {
            loop {
                match writer.remaining_len() {
                    0 => break,
                    1 => {
                        writer.try_pin_init(args)?;
                        break;
                    }
                    _ => writer.try_pin_init(args.clone())?,
                }
            }
        }

        Ok(writer.finish())
    }
}

/// A slice constructor which copies the argument and uses it to construct each element of the slice
///
/// It also has a `LayoutProvider` which allocates enough spaces for `self.0` items
#[derive(Debug, Clone, Copy)]
pub struct CopyArgsLen<Args>(pub usize, pub Args);

impl<T: TryPinCtor<Args>, Args: Copy> HasLayoutProvider<CopyArgsLen<Args>> for [T]
where
    T: HasLayoutProvider<Args>,
{
    type LayoutProvider = SliceLenLayoutProvider;
}

// SAFETY: The layout is compatible with cast
unsafe impl<T, Args: Copy> LayoutProvider<[T], CopyArgsLen<Args>> for SliceLenLayoutProvider
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

impl<T: TryPinCtor<Args>, Args: Copy> TryPinCtor<CopyArgsLen<Args>> for [T] {
    type Error = T::Error;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        CopyArgsLen(_, args): CopyArgsLen<Args>,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        uninit.try_pin_init(CopyArgs(args))
    }
}

/// A slice constructor which clones the argument and uses it to construct each element of the slice
///
/// It also has a `LayoutProvider` which allocates enough spaces for `self.0` items
#[derive(Debug, Clone, Copy)]
pub struct CloneArgsLen<Args>(pub usize, pub Args);

impl<T, Args: Clone> HasLayoutProvider<CloneArgsLen<Args>> for [T]
where
    T: HasLayoutProvider<Args>,
{
    type LayoutProvider = SliceLenLayoutProvider;
}

// SAFETY: The layout is compatible with cast
unsafe impl<T, Args: Clone> LayoutProvider<[T], CloneArgsLen<Args>> for SliceLenLayoutProvider
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

impl<T: TryPinCtor<Args>, Args: Clone> TryPinCtor<CloneArgsLen<Args>> for [T] {
    type Error = T::Error;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        CloneArgsLen(_, args): CloneArgsLen<Args>,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        uninit.try_pin_init(CloneArgs(args))
    }
}

/// A layout provider for slices
pub struct SliceLenLayoutProvider;

/// An initializer argument to initialize a slice with the items of the iterator
///
/// NOTE: this will take at most enough elements as needed to fill up the slice, and no more
///
/// The initializer will error if not enough elements are passed in to completely fill up the slice
pub struct IterInit<I>(pub I);

/// An error for the [`IterInit`] type
pub enum IterInitError<E> {
    /// If not enough elements were in the iterator to fill up the slice
    NotEnoughItems,
    /// If any item in the slice failed to initialize
    InitError(E),
}

impl<T: TryPinCtor<I::Item>, I: Iterator> TryPinCtor<IterInit<I>> for [T] {
    type Error = IterInitError<T::Error>;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        IterInit(args): IterInit<I>,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        let mut writer = PinSliceWriter::new(uninit);

        args.take(writer.remaining_len())
            // SAFETY: we take up to the remaining length of the writer elements, which means if the closure is called
            // there is enough space in the writer to initialize an element
            .try_for_each(|arg| unsafe { writer.try_pin_init_unchecked(arg) })
            .map_err(IterInitError::InitError)?;

        writer.try_finish().ok_or(IterInitError::NotEnoughItems)
    }
}

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

// SAFETY: The layout is compatible with cast
unsafe impl<T, I> LayoutProvider<[T], IterLenInit<I>> for SliceLenLayoutProvider {
    fn layout_of(args: &IterLenInit<I>) -> Option<Layout> {
        Layout::array::<T>(args.0).ok()
    }

    unsafe fn cast(ptr: NonNull<u8>, args: &IterLenInit<I>) -> NonNull<[T]> {
        NonNull::slice_from_raw_parts(ptr.cast(), args.0)
    }
}

impl<T: TryPinCtor<I::Item>, I: Iterator> TryPinCtor<IterLenInit<I>> for [T] {
    type Error = IterInitError<T::Error>;

    fn try_pin_init(
        uninit: crate::Uninit<'_, Self>,
        IterLenInit(_, args): IterLenInit<I>,
    ) -> Result<crate::PinInit<'_, Self>, Self::Error> {
        uninit.try_pin_init(IterInit(args))
    }
}
