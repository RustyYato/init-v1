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
pub mod layout_provider;

pub mod ctor;
pub mod pin_ctor;
pub mod try_ctor;
pub mod try_pin_ctor;

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
pub mod try_slice;

pub use ctor::{ctor, Ctor, CtorArgs};
pub use pin_ctor::{pin_ctor, PinCtor, PinCtorArgs};
pub use pin_ptr::{IterPinInit, PinInit};
pub use ptr::{Init, IterInit, IterUninit, Uninit};
pub use try_ctor::{try_ctor, TryCtor, TryCtorArgs};
pub use try_pin_ctor::{try_pin_ctor, TryPinCtor, TryPinCtorArgs};
