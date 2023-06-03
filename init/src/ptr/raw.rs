use core::{marker::PhantomData, ptr::NonNull};

struct Invariant<'a>(PhantomData<&'a mut &'a mut ()>);

/// `Uninit` is a pointer to uninitialized memory
///
/// ## Guarantees
///
/// all `Uninit` pointers are for the lifetime `'a`:
/// * aligned
/// * non-null
/// * dereferencable (for reads and writes, but reads may yield uninitialized memory)
#[repr(transparent)]
pub struct Uninit<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _brand: Invariant<'a>,
}

/// `Init` is a pointer to initialized memory.
/// `Init` owns a value of type `T`
///
/// ## Guarantees
///
/// all `Init` pointers are for the lifetime `'a`:
/// * aligned
/// * non-null
/// * dereferencable (for reads and writes, but reads may yield uninitialized memory)
/// * initialized for type `T`
#[repr(transparent)]
pub struct Init<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _brand: Invariant<'a>,
    ty: PhantomData<T>,
}

impl<'a, T: ?Sized> Uninit<'a, T> {
    /// Create an `Uninit` from a raw pointer
    ///
    /// # Safety
    ///
    /// You must uphold the guarantees of `Uninit`
    #[inline]
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr),
            _brand: Invariant(PhantomData),
        }
    }

    /// Convert an `Uninit` into a raw pointer
    ///
    /// This pointer may only be written to before it is read from
    pub fn into_raw(self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Get the underlying raw pointer from an `Uninit`
    ///
    /// This pointer may only be written to before it is read from
    pub fn as_mut_raw(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<'a, T: ?Sized> Init<'a, T> {
    /// Create an `Init` from a raw pointer
    ///
    /// # Safety
    ///
    /// You must uphold the guarantees of `Init`
    #[inline]
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr),
            _brand: Invariant(PhantomData),
            ty: PhantomData,
        }
    }

    /// Convert an `Init` into a raw pointer
    pub fn into_raw(self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Get the underlying raw pointer from an `Init`
    ///
    /// This pointer may only be used for reads, no writes
    pub fn as_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Get the underlying raw pointer from an `Init`
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }
}
