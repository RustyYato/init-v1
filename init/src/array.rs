//! Array constructors

use core::{alloc::Layout, ptr::NonNull};

use crate::layout_provider::LayoutProvider;

/// A layout provider for arrays, which is based off of the slice layout provider
pub struct ArrayLayoutProvider<L>(L);

/// SAFETY: Arrays and slices have the same layout
unsafe impl<T, A, const N: usize, L: LayoutProvider<[T], A>> LayoutProvider<[T; N], A>
    for ArrayLayoutProvider<L>
{
    /// The layout of the type for the given arguments
    fn layout_of(_: &A) -> Option<Layout> {
        Some(Layout::new::<[T; N]>())
    }

    ///  # Safety
    ///
    /// `Self::layout(args)` must return a layout
    unsafe fn cast(ptr: NonNull<u8>, _: &A) -> NonNull<[T; N]> {
        ptr.cast()
    }

    /// If the arguments is guaranteed to zero out data and have no other side effects
    /// then this returns true
    ///
    /// If this function returns true, it is safe to just write zeros to all bytes
    /// `0..layout.size()` and skip the calling `Ctor::init` or `CtorArgs::init_with`
    fn is_zeroed(args: &A) -> bool {
        L::is_zeroed(args)
    }
}
