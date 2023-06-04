#![no_std]
#![forbid(
    missing_docs,
    clippy::missing_safety_doc,
    unsafe_op_in_unsafe_fn,
    clippy::undocumented_unsafe_blocks
)]
#![feature(dropck_eyepatch)]

//! ## init
//!
//! A safe library for initializing values in place without any intermediate copies

mod interface;
mod ptr;
pub mod slice_writer;

mod slice;

mod hacks;

pub use interface::{ctor, Ctor, CtorArgs};
pub use ptr::{Init, Uninit};
