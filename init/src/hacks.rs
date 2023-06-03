pub(crate) const fn ptr_slice_len<T>(ptr: *const [T]) -> usize {
    // SAFETY:
    //
    // This trick assumes that a pointer to a slice has one of two representations
    // either:
    //
    // [ptr, len] or [len, ptr]
    //
    // And deconstructs a carefully selected slice where the pointer is 0 and
    // the length is usize::MAX (all bits are 1), to detect which layout Rust chose
    // Then it picks the right element from the actual slice.
    //
    // If the layout may change for every instance of `*mut [T]`, then this is broken
    // but that seems impossible to implement for rustc (or any other Rust implementation)
    // so this algorithm is SAFE and correct from that standpoint
    //
    // It is safe to transmute a pointer to usize, it simply drops it's provenance.
    //
    // If Rust picks a different layout that isn't compatible with `[usize; 2]`, then
    // the transmute will fail to compile, so this is SAFE.
    //
    // This will all be optimized away to a direct access with even -O1
    unsafe {
        let [a, b] = core::mem::transmute::<*const [T], [usize; 2]>(
            core::ptr::slice_from_raw_parts(0 as *const T, usize::MAX),
        );

        assert!(a == usize::MAX && b == 0 || a == 0 && b == usize::MAX);

        let [c, d] = core::mem::transmute::<*const [T], [usize; 2]>(ptr);

        if a == usize::MAX {
            c
        } else {
            d
        }
    }
}
