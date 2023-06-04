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

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[doc(hidden)]
pub mod macros;

pub mod interface;
pub mod layout_provider;

mod ext;
mod ptr;
pub mod slice_writer;

#[cfg(feature = "alloc")]
pub mod boxed;
mod hacks;
mod slice;

pub use interface::{ctor, Ctor, CtorArgs};
pub use ptr::{Init, Uninit};
