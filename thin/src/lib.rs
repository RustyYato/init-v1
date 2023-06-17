#![no_std]
#![forbid(
    // missing_docs,
    // clippy::missing_safety_doc,
    unsafe_op_in_unsafe_fn,
    // clippy::undocumented_unsafe_blocks
)]
#![feature(ptr_metadata, slice_range)]

//! A thin pointer library which uses `init` for safe initialization

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod boxed;
pub mod ptr;

#[cfg(feature = "alloc")]
pub mod pin_vec;
#[cfg(feature = "alloc")]
pub mod vec;

mod core_ext;

pub fn asm1(v: &mut vec::ThinVec<i32>) -> Option<i32> {
    v.pop().map(init::Init::into_inner)
}
pub fn asm2(v: &mut alloc::vec::Vec<i32>) -> Option<i32> {
    v.pop()
}
