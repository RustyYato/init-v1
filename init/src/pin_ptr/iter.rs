use core::marker::PhantomData;

use crate::{ptr::IterInit, Init, PinInit};

/// An iterator for `PinInit<[T]>`
pub struct IterPinInit<'a, T> {
    raw: IterInit<'a, T>,
    lt: PhantomData<Init<'a, T>>,
}

impl<'a, T> IterPinInit<'a, T> {
    pub(super) fn new(init: PinInit<'a, [T]>) -> Self {
        Self {
            // SAFETY: the iterator doesn't move any values
            raw: unsafe { init.into_inner_unchecked() }.into_iter(),
            lt: PhantomData,
        }
    }

    /// The number of remaining elements in the iterator
    #[inline]
    pub const fn len(&self) -> usize {
        self.raw.len()
    }

    /// The number of remaining elements in the iterator
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// The next element of the iterator without checking if it's exhausted
    ///
    /// # Safety
    ///
    /// The iterator must not be exhausted
    pub unsafe fn next_unchecked(&mut self) -> PinInit<'a, T> {
        // SAFETY: the caller guarantees that this iterator isn't exhausted
        unsafe { self.raw.next_unchecked() }.pin()
    }
}

impl<'a, T> Iterator for IterPinInit<'a, T> {
    type Item = PinInit<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(Init::pin)
    }
}

impl<'a, T> IntoIterator for PinInit<'a, [T]> {
    type Item = PinInit<'a, T>;
    type IntoIter = IterPinInit<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IterPinInit::new(self)
    }
}

#[cfg(test)]
mod test {
    use crate::Uninit;

    #[test]
    fn test_empty() {
        let uninit = Uninit::<[i32]>::from_ref(&mut [][..]).iter();

        assert_eq!(uninit.count(), 0);
    }
}
