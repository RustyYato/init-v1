#![no_std]
#![forbid(missing_docs, clippy::missing_safety_doc)]
#![feature(dropck_eyepatch)]

//! ## init
//!
//! A safe library for initializing values in place without any intermediate copies

mod interface;
mod ptr;

pub use ptr::{Init, Uninit};
