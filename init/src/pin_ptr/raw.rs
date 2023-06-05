use core::{marker::PhantomData, ptr::NonNull};

struct Invariant<'a>(PhantomData<fn() -> *mut &'a ()>);

/// `PinInit` is a pointer to initialized memory.
/// `PinInit` owns a value of type `T`, and that value of type `T` cannot
/// be moved anywhere directly.
/// NOTE: the underlying memory isn't owned by the `PinInit`, so it may be
/// freed before the `T` is dropped iff the `PinInit` is leaked.
///
/// ## Guarantees
///
/// all `PinInit` pointers are for the lifetime `'a`:
/// * aligned
/// * non-null
/// * dereferencable (for reads and writes, but reads may yield uninitialized memory)
/// * initialized for type `T`
/// * not aliased by any unrelated pointers
/// * the value may not be trivially moved
#[repr(transparent)]
pub struct PinInit<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _brand: Invariant<'a>,
    ty: PhantomData<T>,
}

impl<'a, T: ?Sized> PinInit<'a, T> {
    /// Create an `PinInit` from a raw pointer
    ///
    /// # Safety
    ///
    /// You must uphold the guarantees of `PinInit`
    #[inline]
    pub const unsafe fn from_raw(ptr: *mut T) -> Self {
        Self {
            // SAFETY: the pointer must be non-null because the caller guarantees it
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _brand: Invariant(PhantomData),
            ty: PhantomData,
        }
    }

    /// Convert an `PinInit` into a raw pointer
    #[inline]
    pub const fn into_raw(self) -> *mut T {
        self.into_raw_non_null().as_ptr()
    }

    /// Convert an `PinUninit` into a raw pointer
    ///
    /// This pointer may only be written to before it is read from
    #[inline]
    pub const fn into_raw_non_null(self) -> NonNull<T> {
        let ptr = self.ptr;
        core::mem::forget(self);
        ptr
    }

    /// Get the underlying raw pointer from an `PinInit`
    ///
    /// This pointer may only be used for reads, no writes
    #[inline]
    pub const fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Get the underlying raw pointer from an `PinInit`
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }
}
