#![no_std]
#![forbid(
    missing_docs,
    clippy::missing_safety_doc,
    unsafe_op_in_unsafe_fn,
    clippy::undocumented_unsafe_blocks
)]
#![feature(dropck_eyepatch, ptr_metadata)]

//! ## init
//!
//! A safe library for initializing values in place without any intermediate copies

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[doc(hidden)]
pub mod macros;

pub mod config_value;
pub mod interface;
pub mod layout_provider;
pub mod pin_interface;

mod ext;
mod pin_ptr;
pub mod pin_slice_writer;
mod ptr;
pub mod slice_writer;
pub mod source;

#[cfg(feature = "alloc")]
pub mod boxed;
mod hacks;
pub mod pin_slice;
pub mod slice;

pub use interface::{ctor, Ctor, CtorArgs};
pub use pin_interface::{pin_ctor, PinCtor, PinCtorArgs};
pub use pin_ptr::{IterPinInit, PinInit};
pub use ptr::{Init, IterInit, IterUninit, Uninit};
