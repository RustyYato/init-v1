#[allow(clippy::useless_transmute)]
pub(crate) const fn ptr_slice_len<T>(ptr: *const [T]) -> usize {
    // SAFETY: as zero-sized slice doesn't alias anything, so we can freely dereference it
    unsafe { (*(ptr as *const [()])).len() }
}
