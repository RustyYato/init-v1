use core::{marker::PhantomData, ptr::NonNull};

use crate::{Init, Uninit};

struct RawIter<T> {
    // for ZSTs this is never changed, and is the same as the pointer to the initial slice
    // for non-ZSTs this is the current pointer (which is either a member of the slice, or one past the end of the slice)
    start: NonNull<T>,
    // for ZSTs this is the number of remaining elements in the iterator
    // for non-ZSTs this is one-past the end of the slice
    end: *mut T,
}

impl<T> Iterator for RawIter<T> {
    type Item = NonNull<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if core::mem::size_of::<T>() == 0 {
            self.end = (self.end as usize).checked_sub(1)? as *mut T;

            Some(NonNull::dangling())
        } else {
            if self.start.as_ptr() == self.end {
                return None;
            }

            let current = self.start;
            // SAFETY: This is the non-ZST case where `self.start` must be either
            // one past the end or a member of the slice we checked that it't not
            // equal to `self.end` which is one past the end, so `self.start` must
            // be an element of the slice. So it's safe to increment the pointer
            // and we'll stay inbound of the slice or one past the end.
            // Which means it's still guaranteed to be `NonNull`
            self.start = unsafe { NonNull::new_unchecked(self.start.as_ptr().add(1)) };
            Some(current)
        }
    }
}

impl<T> RawIter<T> {
    #[allow(clippy::useless_transmute)]
    const fn empty() -> Self {
        let start = NonNull::dangling();
        RawIter {
            start,
            end: if core::mem::size_of::<T>() == 0 {
                // SAFETY: it's always safe to transmute integers to pointers
                unsafe { core::mem::transmute::<usize, *mut T>(0) }
            } else {
                start.as_ptr()
            },
        }
    }

    #[inline]
    unsafe fn new(ptr: NonNull<[T]>) -> Self {
        let start = ptr.as_ptr().cast::<T>();
        let len = crate::hacks::ptr_slice_len(ptr.as_ptr());

        if core::mem::size_of::<T>() == 0 {
            Self {
                // SAFETY: start is the same as `ptr`, which is `NonNull`
                start: unsafe { NonNull::new_unchecked(start) },
                end: len as *mut T,
            }
        } else {
            Self {
                // SAFETY: start is the same as `ptr`, which is `NonNull`
                start: unsafe { NonNull::new_unchecked(start) },
                // SAFETY: it's safe to add `len` to the start of a slice
                end: unsafe { start.add(len) },
            }
        }
    }

    #[inline]
    #[allow(clippy::transmutes_expressible_as_ptr_casts)]
    const fn len(&self) -> usize {
        if core::mem::size_of::<T>() == 0 {
            // SAFETY: at runtime, it's always safe to transmute a pointer to an integer
            // in const context, this pointer doesn't actually contain a pointer
            unsafe { core::mem::transmute::<*mut T, usize>(self.end) }
        } else {
            // SAFETY: both self.end and self.start come from the same slice
            unsafe { self.end.offset_from(self.start.as_ptr()) as usize }
        }
    }

    #[allow(clippy::transmutes_expressible_as_ptr_casts)]
    fn remaining(&mut self) -> *mut [T] {
        core::ptr::slice_from_raw_parts_mut(self.start.as_ptr(), self.len())
    }

    unsafe fn next_unchecked(&mut self) -> NonNull<T> {
        if core::mem::size_of::<T>() == 0 {
            self.end = (self.end as usize).wrapping_sub(1) as *mut T;

            NonNull::dangling()
        } else {
            let current = self.start;
            // SAFETY: This is the non-ZST case where `self.start` must be either
            // one past the end or a member of the slice we checked that it't not
            // equal to `self.end` which is one past the end, so `self.start` must
            // be an element of the slice. So it's safe to increment the pointer
            // and we'll stay inbound of the slice or one past the end.
            // Which means it's still guaranteed to be `NonNull`
            self.start = unsafe { NonNull::new_unchecked(self.start.as_ptr().add(1)) };
            current
        }
    }
}

/// An iterator for `Uninit<[T]>`
pub struct IterUninit<'a, T> {
    raw: RawIter<T>,
    lt: PhantomData<Uninit<'a, T>>,
}

impl<'a, T> IterUninit<'a, T> {
    pub(super) fn new(uninit: Uninit<'a, [T]>) -> Self {
        Self {
            // SAFETY: uninit satisfies all the requirements for `RawIter`
            raw: unsafe { RawIter::new(uninit.into_raw_non_null()) },
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
        self.raw.len() == 0
    }

    /// The remaining elements in the iterator
    #[inline]
    pub fn remaining(&mut self) -> *mut [T] {
        self.raw.remaining()
    }

    /// The remaining elements in the iterator
    #[inline]
    pub fn into_remaining(self) -> *mut [T] {
        core::mem::ManuallyDrop::new(self).raw.remaining()
    }

    /// The next element of the iterator without checking if it's exhausted
    ///
    /// # Safety
    ///
    /// The iterator must not be exhausted
    pub unsafe fn next_unchecked(&mut self) -> Uninit<'a, T> {
        // SAFETY: the caller guarantees that this iterator isn't exhausted
        let ptr = unsafe { self.raw.next_unchecked() };
        // SAFETY: the raw iterator was created from an `Uninit<'_, T>` and
        // raw only gives out distinct elements of the slice, which means they are
        // all aligned, non-null, dereferencable, and unique
        unsafe { Uninit::from_raw(ptr.as_ptr()) }
    }
}

impl<'a, T> Iterator for IterUninit<'a, T> {
    type Item = Uninit<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(|ptr| {
            // SAFETY: the raw iterator was created from an `Uninit<'_, T>` and
            // raw only gives out distinct elements of the slice, which means they are
            // all aligned, non-null, dereferencable, and unique
            unsafe { Uninit::from_raw(ptr.as_ptr()) }
        })
    }
}

/// An iterator for `Init<[T]>`
pub struct IterInit<'a, T> {
    raw: RawIter<T>,
    lt: PhantomData<Init<'a, T>>,
}

impl<'a, T> IterInit<'a, T> {
    pub(super) fn new(init: Init<'a, [T]>) -> Self {
        Self {
            // SAFETY: init satisfies all the requirements for `RawIter`
            raw: unsafe { RawIter::new(init.into_raw_non_null()) },
            lt: PhantomData,
        }
    }

    /// Take the iterator and replace with an empty iterator
    pub fn take_ownership(&mut self) -> Self {
        let iter = Self {
            raw: RawIter::empty(),
            lt: PhantomData,
        };
        core::mem::replace(self, iter)
    }

    /// The number of remaining elements in the iterator
    #[inline]
    pub const fn len(&self) -> usize {
        self.raw.len()
    }

    /// The number of remaining elements in the iterator
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.raw.len() == 0
    }

    /// The remaining elements in the iterator
    #[inline]
    pub fn remaining(&mut self) -> *mut [T] {
        self.raw.remaining()
    }

    /// The remaining elements in the iterator
    #[inline]
    pub fn into_remaining(self) -> *mut [T] {
        core::mem::ManuallyDrop::new(self).raw.remaining()
    }

    /// The next element of the iterator without checking if it's exhausted
    ///
    /// # Safety
    ///
    /// The iterator must not be exhausted
    pub unsafe fn next_unchecked(&mut self) -> Init<'a, T> {
        // SAFETY: the caller guarantees that this iterator isn't exhausted
        let ptr = unsafe { self.raw.next_unchecked() };
        // SAFETY: the raw iterator was created from an `Init<'_, T>` and
        // raw only gives out distinct elements of the slice, which means they are
        // all aligned, non-null, dereferencable, and unique
        unsafe { Init::from_raw(ptr.as_ptr()) }
    }
}

impl<T> Drop for IterInit<'_, T> {
    fn drop(&mut self) {
        if !core::mem::needs_drop::<T>() {
            return;
        }

        // SAFETY: This only contains elements not yet yielded by the iterator
        unsafe { self.raw.remaining().drop_in_place() }
    }
}

impl<'a, T> Iterator for IterInit<'a, T> {
    type Item = Init<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(|ptr| {
            // SAFETY: the raw iterator was created from an `Init<'_, T>` and
            // raw only gives out distinct elements of the slice, which means they are
            // all aligned, non-null, dereferencable, unique, and initialized
            unsafe { Init::from_raw(ptr.as_ptr()) }
        })
    }
}

impl<'a, T> IntoIterator for Uninit<'a, [T]> {
    type Item = Uninit<'a, T>;
    type IntoIter = IterUninit<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IterUninit::new(self)
    }
}

impl<'a, T> IntoIterator for Init<'a, [T]> {
    type Item = Init<'a, T>;
    type IntoIter = IterInit<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IterInit::new(self)
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
