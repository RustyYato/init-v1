pub(crate) fn write<T>(x: &mut T, value: T) {
    // SAFETY: it's always safe to write to a mutable reference
    unsafe { core::ptr::write(x, value) }
}
