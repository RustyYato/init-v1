// #![no_std]
// #![forbid(
//     missing_docs,
//     clippy::missing_safety_doc,
//     unsafe_op_in_unsafe_fn,
//     clippy::undocumented_unsafe_blocks
// )]
#![feature(ptr_metadata, slice_range)]

//! A thin pointer library which uses `init` for safe initialization

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod boxed;
pub mod ptr;
pub mod vec;

mod core_ext;
