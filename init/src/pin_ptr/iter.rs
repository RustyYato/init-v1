use core::marker::PhantomData;

use crate::{ptr::IterInit, Init, PinInit};

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

    #[inline]
    pub const fn len(&self) -> usize {
        self.raw.len()
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
