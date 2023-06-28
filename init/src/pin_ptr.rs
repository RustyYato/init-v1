mod iter;
mod raw;

use crate::Init;
pub use crate::Uninit;
pub use iter::IterPinInit;
pub use raw::PinInit;

// SAFETY: we only call drop on a `T`, so trivially correct for `may_dangle`
unsafe impl<#[may_dangle] T: ?Sized> Drop for PinInit<'_, T> {
    fn drop(&mut self) {
        // SAFETY:
        // by the guarantees of `Init` the pointer must be aligned, non-null, and initialized
        // so it is safe to drop this value
        unsafe { self.as_mut_ptr().drop_in_place() }
    }
}

impl<'a, T: ?Sized> PinInit<'a, T> {
    /// Get a shared reference to `T`
    pub fn get(&self) -> &T {
        // SAFETY: The pointer is aligned, non-null, and initialized
        unsafe { &*self.as_ptr() }
    }

    /// Get a mutable reference to `T`
    ///
    /// # Safety
    ///
    /// You may not trivially move `T`
    pub unsafe fn get_mut_unchecked(&mut self) -> &mut T {
        // SAFETY: The pointer is aligned, non-null, and initialized
        // the caller will guarantee that the value isn't moved
        unsafe { &mut *self.as_mut_ptr() }
    }

    /// Get a mutable reference to `T`
    pub fn get_mut(&mut self) -> &mut T
    where
        T: Unpin,
    {
        // SAFETY: Unpin types don't care if they are moved
        unsafe { self.get_mut_unchecked() }
    }

    /// unwrap the `PinInit`
    ///
    /// # Safety
    ///
    /// You may not trivially move `T`
    pub unsafe fn into_inner_unchecked(self) -> Init<'a, T> {
        // SAFETY: The pointer is aligned, non-null, and initialized
        // the caller will guarantee that the value isn't moved
        unsafe { Init::from_raw(self.into_raw()) }
    }

    /// unwrap the `PinInit`
    pub fn into_inner(self) -> Init<'a, T>
    where
        T: Unpin,
    {
        // SAFETY: Unpin types don't care if they are moved
        unsafe { self.into_inner_unchecked() }
    }

    /// Leak the `PinInit` and is as signal that something else is taking ownership of the value
    pub const fn take_ownership(self) {
        core::mem::forget(self)
    }
}

impl<'a, T> PinInit<'a, [T]> {
    /// The length of the slice
    pub const fn len(&self) -> usize {
        crate::hacks::ptr_slice_len(self.as_ptr())
    }

    /// Checks if the slice is empty (has length == 0)
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // /// An iterator over all elements of the slice
    // #[inline]
    // pub fn iter(self) -> IterInit<'a, T> {
    //     IterInit::new(self)
    // }

    /// Convert a slice to an array without checking the length
    ///
    /// # Safety
    ///
    /// The length of the slice must equal `N`
    pub unsafe fn into_array_unchecked<const N: usize>(self) -> PinInit<'a, [T; N]> {
        debug_assert_eq!(self.len(), N);
        // SAFETY: The length of the slice is equal to `N`, so the slice is layout compatible with [T; N]
        unsafe { PinInit::from_raw(self.into_raw().cast()) }
    }
}

impl<'a, T, const N: usize> PinInit<'a, [T; N]> {
    /// Convert to an `Init` without writing to the underlying pointer
    #[inline]
    pub const fn to_slice(self) -> PinInit<'a, [T]> {
        // SAFETY: slices and arrays are layout compatible
        unsafe { PinInit::from_raw(self.into_raw() as *mut [T]) }
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
