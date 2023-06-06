use init::Init;

use crate::ptr::RawThinPtr;

use super::VecData;

pub struct Drain<'a, T> {
    pub(super) ptr: RawThinPtr<VecData<T>>,
    pub(super) iter: init::IterInit<'a, T>,
    pub(super) new_len: usize,
    pub(super) offset: usize,
}

impl<T> Drop for Drain<'_, T> {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.ptr.as_mut_ptr();
            let remaining_len = self.new_len - self.offset;
            let data = core::ptr::addr_of_mut!((*ptr).data)
                .cast::<T>()
                .add(self.offset);

            let remaining = self.iter.take_ownership().into_remaining();
            let len: usize = core::ptr::metadata(remaining) + remaining_len;
            let end_ptr = remaining.cast::<T>();
            (*ptr).len = self.new_len + core::ptr::metadata(remaining);

            if data == end_ptr || len == 0 {
                return;
            }

            data.copy_from(end_ptr, len)
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
