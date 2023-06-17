mod iter;
mod raw;

use core::mem::MaybeUninit;

pub use iter::{IterInit, IterUninit};
pub use raw::{Init, Uninit};

use crate::PinInit;

// SAFETY: we only call drop on a `T`, so trivially correct for `may_dangle`
unsafe impl<#[may_dangle] T: ?Sized> Drop for Init<'_, T> {
    fn drop(&mut self) {
        // SAFETY:
        // by the guarantees of `Init` the pointer must be aligned, non-null, and initialized
        // so it is safe to drop this value
        unsafe { self.as_mut_ptr().drop_in_place() }
    }
}

impl<'a, T: ?Sized> Uninit<'a, T> {
    /// Convert the `Uninit` to an `Init` without checking if it was initialized
    ///
    /// # Safety
    ///
    /// The pointer mut be initialized
    #[inline]
    pub const unsafe fn assume_init(self) -> Init<'a, T> {
        // SAFETY: caller guarantees that pointer is initialized
        // All other guarantees come from the `Uninit` type
        unsafe { Init::from_raw(self.into_raw()) }
    }

    /// Initialize self using a constructor
    pub fn init<Args>(self, args: Args) -> Init<'a, T>
    where
        T: crate::Ctor<Args>,
    {
        crate::Ctor::init(self, args)
    }

    /// Try to initialize self using a constructor
    pub fn try_init<Args>(self, args: Args) -> Result<Init<'a, T>, T::Error>
    where
        T: crate::TryCtor<Args>,
    {
        crate::TryCtor::try_init(self, args)
    }

    /// Initialize self using a constructor
    pub fn pin_init<Args>(self, args: Args) -> PinInit<'a, T>
    where
        T: crate::PinCtor<Args>,
    {
        crate::PinCtor::pin_init(self, args)
    }
}

impl<'a, T> Uninit<'a, T> {
    /// Initialize the pointer to the given value and convert to an `Init`
    #[inline]
    pub fn write(mut self, value: T) -> Init<'a, T> {
        // SAFETY:
        // by the guarantees of `Uninit` the pointer must be aligned,
        // non-null, dereferencable, and not aliasing any unrelated pointers
        // which means we can safely write to it
        unsafe { self.as_mut_ptr().write(value) }
        // SAFETY: we just initialized the pointer ^^^
        unsafe { self.assume_init() }
    }
}

impl<'a, T> Uninit<'a, MaybeUninit<T>> {
    /// Convert to an `Init` without writing to the underlying pointer
    #[inline]
    pub const fn uninit(self) -> Init<'a, MaybeUninit<T>> {
        // SAFETY: `MaybeUninit` may safely point to uninitialized memory
        unsafe { self.assume_init() }
    }
}

impl<'a, T> Uninit<'a, [MaybeUninit<T>]> {
    /// Convert to an `Init` without writing to the underlying pointer
    #[inline]
    pub const fn uninit_slice(self) -> Init<'a, [MaybeUninit<T>]> {
        // SAFETY: `MaybeUninit` may safely point to uninitialized memory
        unsafe { self.assume_init() }
    }
}

impl<'a, T> Uninit<'a, [T]> {
    /// The length of the slice
    pub const fn len(&self) -> usize {
        crate::hacks::ptr_slice_len(self.as_ptr())
    }

    /// Checks if the slice is empty (has length == 0)
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// An iterator over all elements of the slice
    #[inline]
    pub fn iter(self) -> IterUninit<'a, T> {
        IterUninit::new(self)
    }
}

impl<'a, T> Uninit<'a, [T]> {
    /// Copy the data from `slice` and convert to an `Init`
    ///
    /// # Panics
    ///
    /// Panics if the length of `init` doesn't equal `self.len()`
    ///
    /// # Safety
    ///
    /// `init` must be leaked after this function is successfully called
    #[inline]
    pub unsafe fn copy_from_slice_unchecked(mut self, init: &[T]) -> Init<'a, [T]> {
        fn copy_from_slice_failed(my_len: usize, init_len: usize) -> ! {
            panic!("Could not copy from slice because lengths didn't match, expected length: {my_len} but got {init_len}")
        }

        if self.len() != init.len() {
            copy_from_slice_failed(self.len(), init.len())
        }

        // SAFETY: The lengths of the two slices are equal, checked above
        // and it is safe to write to the pointer given by `Uninit::as_mut_ptr`
        unsafe {
            self.as_mut_ptr()
                .cast::<T>()
                .copy_from_nonoverlapping(init.as_ptr(), init.len());
        }

        // SAFETY: just initialized the pointer ^^^
        unsafe { self.assume_init() }
    }
}

impl<'a, T: Copy> Uninit<'a, [T]> {
    /// Copy the data from `slice` and convert to an `Init`
    ///
    /// # Panics
    ///
    /// Panics if the length of `init` doesn't equal `self.len()`
    #[inline]
    pub fn copy_from_slice(self, init: &[T]) -> Init<'a, [T]> {
        // SAFETY: T is copy so it doesn't do anything on `Drop`
        unsafe { self.copy_from_slice_unchecked(init) }
    }
}

impl<'a, T: ?Sized> Init<'a, T> {
    /// Pin a initialized pointer
    pub fn pin(self) -> PinInit<'a, T> {
        // SAFETY: the pointer is:
        // * aligned
        // * non-null
        // * dereferencable (for reads and writes, but reads may yield uninitialized memory)
        // * initialized for type `T`
        // * not aliased by any unrelated pointers
        // `PinInit` will take care of the not-moving guarantee
        unsafe { PinInit::from_raw(self.into_raw()) }
    }

    /// Get a shared reference to `T`
    pub fn get(&self) -> &T {
        // SAFETY: The pointer is aligned, non-null, and initialized
        unsafe { &*self.as_ptr() }
    }

    /// Get a mutable reference to `T`
    pub fn get_mut(&mut self) -> &mut T {
        // SAFETY: The pointer is aligned, non-null, and initialized
        unsafe { &mut *self.as_mut_ptr() }
    }
}

impl<'a, T> Init<'a, [T]> {
    /// The length of the slice
    pub const fn len(&self) -> usize {
        crate::hacks::ptr_slice_len(self.as_ptr())
    }

    /// Checks if the slice is empty (has length == 0)
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// An iterator over all elements of the slice
    #[inline]
    pub fn iter(self) -> IterInit<'a, T> {
        IterInit::new(self)
    }
}

impl<T> Init<'_, T> {
    /// Read the underlying value from the `Init`
    pub fn into_inner(self) -> T {
        // SAFETY: the pointer is valid for reads
        unsafe { self.into_raw().read() }
    }
}

#[cfg(test)]
mod test {
    use crate::Uninit;

    #[test]
    fn test_slice_len() {
        let mut data: [i32; 3] = [0, 1, 2];
        let uninit = Uninit::from_ref(&mut data[..]);
        // the pointer cannot be `3`, because the pointer `3` isn't aligned for i32
        // and the `uninit` is aligned
        assert_eq!(uninit.len(), 3);
    }
}
