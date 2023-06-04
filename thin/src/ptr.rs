//! A raw thin pointer abstraction

use core::{alloc::Layout, marker::PhantomData, ptr::NonNull};

use init::{
    layout_provider::{LayoutProvider, MaybeLayoutProvider, NoLayoutProvider},
    Ctor, Init,
};

/// The Pointee::Metadata for a given type
pub type Metadata<T> = <T as core::ptr::Pointee>::Metadata;

/// The raw thin pointer
///
/// This is  guaranteed to have the same representation as a `*mut ()` but may point to any `T`
#[repr(transparent)]
pub struct RawThinPtr<T: ?Sized, M = Metadata<T>> {
    raw: NonNull<()>,
    ty: PhantomData<fn() -> WithHeader<T, M>>,
}

impl<T: ?Sized, M> Copy for RawThinPtr<T, M> {}
impl<T: ?Sized, M> Clone for RawThinPtr<T, M> {
    fn clone(&self) -> Self {
        *self
    }
}

/// A type which stores the pointer metadata inline with the data, instead of alongside the pointer
#[repr(C)]
pub struct WithHeader<T: ?Sized, M = Metadata<T>> {
    metadata: M,
    value: T,
}

/// A constructor for `WithHeader`
pub struct PushHeader<Args>(pub Args);

/// The layout provider for `WithHeader`
pub struct WithHeaderLayoutProvider;

impl<T: ?Sized + Ctor<Args>, Args> LayoutProvider<WithHeader<T>, PushHeader<Args>>
    for WithHeaderLayoutProvider
{
}

// SAFETY: the layout given by layout_of matches the algorithm used to calculate the layout of
// repr(C) structs
unsafe impl<T: ?Sized + Ctor<Args>, Args> MaybeLayoutProvider<WithHeader<T>, PushHeader<Args>>
    for WithHeaderLayoutProvider
{
    fn layout_of(args: &PushHeader<Args>) -> Option<core::alloc::Layout> {
        let data_layout = init::layout_provider::layout_of::<T, Args>(&args.0)?;
        let metadata_layout = Layout::new::<Metadata<T>>();
        let (layout, _) = metadata_layout.extend(data_layout).ok()?;
        Some(layout.pad_to_align())
    }

    unsafe fn cast(ptr: NonNull<u8>, args: &PushHeader<Args>) -> NonNull<WithHeader<T>> {
        // SAFETY: `Self::layout_of` only returns a layout if `T::layout_of` returns Some
        let ptr = unsafe { init::layout_provider::cast::<T, Args>(ptr, &args.0) };
        // SAFETY: `ptr` is non-null
        unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut WithHeader<T>) }
    }
}

impl<T: ?Sized + Ctor<Args>, Args> Ctor<PushHeader<Args>> for WithHeader<T> {
    type LayoutProvider = WithHeaderLayoutProvider;

    #[inline]
    fn init(
        uninit: init::Uninit<'_, Self>,
        PushHeader(args): PushHeader<Args>,
    ) -> init::Init<'_, Self> {
        init::init_struct! {
            uninit => Self {
                value: args,
                metadata: Literal(core::ptr::metadata(value.as_ptr())),
            }
        }
    }
}

impl<T: ?Sized> RawThinPtr<T> {
    /// Create a raw pointer from an `Init`
    ///
    /// Note: to safely call any function on this `RawThinPtr` marked unsafe
    /// you must ensure that the `RawThinPtr` does not outlive the `Init` ptr's
    /// original lifetime
    pub fn from_init(ptr: Init<WithHeader<T>>) -> Self {
        Self {
            raw: ptr.into_raw_non_null().cast(),
            ty: PhantomData,
        }
    }

    /// Create a raw pointer from an `Init`
    ///
    /// Note: to safely call any function on this `RawThinPtr` marked unsafe
    /// you must ensure that the `RawThinPtr` does not outlive the `Init` ptr's
    /// original lifetime
    pub fn from_raw(ptr: NonNull<WithHeader<T>>) -> Self {
        Self {
            // SAFETY: guaranteed by caller
            raw: ptr.cast(),
            ty: PhantomData,
        }
    }

    /// Get the metadata of the pointer
    ///
    /// # Safety
    ///
    /// The pointer must still be valid
    pub unsafe fn metadata(self) -> Metadata<T> {
        // SAFETY: Guaranteed by caller
        unsafe { *self.raw.cast::<Metadata<T>>().as_ptr() }
    }

    /// Get a read-only pointer to the underlying data
    ///
    /// # Safety
    ///
    /// The pointer must still be valid
    pub unsafe fn as_ptr(self) -> *const T {
        // SAFETY: Guaranteed by caller
        unsafe { self.as_mut_ptr() }
    }

    /// Get a mutable pointer to the underlying data
    ///
    /// # Safety
    ///
    /// The pointer must still be valid
    pub unsafe fn as_mut_ptr(self) -> *mut T {
        // SAFETY: Guaranteed by caller
        let ptr = unsafe { self.as_mut_with_header_ptr() };
        // SAFETY: Guaranteed by caller
        unsafe { core::ptr::addr_of_mut!((*ptr).value) }
    }

    /// Get a mutable pointer to the underlying data
    ///
    /// # Safety
    ///
    /// The pointer must still be valid
    pub unsafe fn as_mut_with_header_ptr(self) -> *mut WithHeader<T> {
        // SAFETY: Guaranteed by caller
        let metadata = unsafe { self.metadata() };
        let ptr = core::ptr::from_raw_parts_mut::<T>(self.raw.as_ptr(), metadata);
        ptr as *mut WithHeader<T>
    }
}

struct Literal<T>(pub T);

impl<T> init::CtorArgs<T> for Literal<T> {
    type LayoutProvider = NoLayoutProvider;

    fn init_with(self, uninit: init::Uninit<'_, T>) -> init::Init<'_, T> {
        uninit.write(self.0)
    }
}
