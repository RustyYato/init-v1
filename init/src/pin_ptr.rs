// mod iter;
mod raw;

// pub use iter::{IterInit, IterUninit};
pub use crate::Uninit;
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

impl<T: ?Sized> PinInit<'_, T> {
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
