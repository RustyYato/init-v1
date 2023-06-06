use init::Init;

use crate::ptr::RawThinPtr;

use super::VecData;

pub struct Drain<'a, T> {
    pub(super) ptr: RawThinPtr<VecData<T>>,
    pub(super) iter: init::IterInit<'a, T>,
    pub(super) tail_len: usize,
    pub(super) tail_offset: usize,
}

impl<T> Drop for Drain<'_, T> {
    fn drop(&mut self) {
        unsafe {
            // FIXME : this code only works for trivially movable types

            let ptr = self.ptr.as_mut_ptr();

            let len = (*ptr).len;

            if core::mem::size_of::<T>() == 0 {
                panic!()
            }

            let data = core::ptr::addr_of_mut!((*ptr).data).cast::<T>();

            let dest = data.add(len);

            let mut remaining = self.iter.take_ownership().into_remaining();

            let rem_len = remaining.len();
            let rem_start = remaining.as_mut_ptr().cast::<T>();
            let rem_end = rem_start.add(rem_len);

            // the vector will take ownership of the remaining elements
            remaining.take_ownership();

            let tail_len = self.tail_len;
            let tail_start = data.add(tail_len);

            (*ptr).len += rem_len + tail_len;

            if rem_len == 0 && tail_len == 0 {
                return;
            }

            if tail_start == rem_end {
                // one copy
                dest.copy_from(rem_start, rem_len + tail_len);
                return;
            }

            if rem_len != 0 {
                dest.copy_from(rem_start, rem_len)
            }

            if tail_len != 0 {
                dest.copy_from(tail_start, tail_len)
            }
        }
    }
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = Init<'a, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_drain() {
        let mut tv = crate::vec::ThinVec::<i32>::new();

        tv.emplace(10);
        tv.emplace(20);
        tv.emplace(30);
        tv.emplace(40);
        tv.emplace(50);

        assert_eq!(tv.as_slice(), [10, 20, 30, 40, 50]);

        tv.drain(..);

        assert_eq!(tv.as_slice(), [10, 20, 30, 40, 50]);

        tv.drain(1..).next();

        assert_eq!(tv.as_slice(), [10, 30, 40, 50]);

        tv.drain(2..4).for_each(drop);

        assert_eq!(tv.as_slice(), [10, 30]);

        tv.drain(..).next_back();

        assert_eq!(tv.as_slice(), [10]);
    }

    #[test]
    pub fn test_drain_leak_amplification() {
        let mut tv = crate::vec::ThinVec::<i32>::new();

        tv.emplace(10);
        tv.emplace(20);
        tv.emplace(30);
        tv.emplace(40);
        tv.emplace(50);

        assert_eq!(tv.as_slice(), [10, 20, 30, 40, 50]);

        core::mem::forget(tv.drain(1..3));

        assert_eq!(tv.as_slice(), [10]);
    }
}
