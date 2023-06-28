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

pub mod ext;
mod pin_ptr;
pub mod pin_slice_writer;
mod ptr;
pub mod slice_writer;
pub mod source;

pub mod array;
#[cfg(feature = "alloc")]
pub mod boxed;
mod hacks;
pub mod pin_array;
pub mod pin_boxed;
pub mod pin_slice;
pub mod slice;
pub mod try_array;
mod try_pin_array;
pub mod try_pin_slice;
pub mod try_slice;

pub use ctor::{ctor, Ctor, CtorArgs};
pub use pin_ctor::{pin_ctor, PinCtor, PinCtorArgs};
pub use pin_ptr::{IterPinInit, PinInit};
pub use ptr::{Init, IterInit, IterUninit, Uninit};
pub use try_ctor::{try_ctor, TryCtor, TryCtorArgs};
pub use try_pin_ctor::{try_pin_ctor, TryPinCtor, TryPinCtorArgs};

/// Try to initialize a value on the stack
pub fn try_stack_init<Args, F: FnOnce(Init<'_, T>) -> R, T: TryCtor<Args>, R>(
    args: Args,
    f: F,
) -> Result<R, T::Error> {
    let mut slot = core::mem::MaybeUninit::<T>::uninit();
    let uninit = Uninit::from_ref(&mut slot).project();
    let output = match uninit.try_init(args) {
        Ok(value) => Ok(f(value)),
        Err(err) => Err(err),
    };
    output
}

/// Initialize a value on the stack
pub fn stack_init<Args, F: FnOnce(Init<'_, T>) -> R, T: Ctor<Args>, R>(args: Args, f: F) -> R {
    match try_stack_init(try_ctor::of_ctor(args), f) {
        Ok(value) => value,
        Err(inf) => match inf {},
    }
}

/// Try to initialize a value on the stack
pub fn try_stack_pin_init<Args, F: FnOnce(core::pin::Pin<&mut T>) -> R, T: TryPinCtor<Args>, R>(
    args: Args,
    f: F,
) -> Result<R, T::Error> {
    let mut slot = core::mem::MaybeUninit::<T>::uninit();
    let uninit = Uninit::from_ref(&mut slot).project();
    let output = match uninit.try_pin_init(args) {
        Err(err) => Err(err),
        Ok(value) => {
            let value =
                // SAFETY: T is in the pinned type-state
                unsafe { core::pin::Pin::new_unchecked(value.into_inner_unchecked().into_mut()) };
            Ok(f(value))
        }
    };
    output
}

/// Initialize a value on the stack
pub fn stack_pin_init<Args, F: FnOnce(core::pin::Pin<&mut T>) -> R, T: PinCtor<Args>, R>(
    args: Args,
    f: F,
) -> R {
    match try_stack_pin_init(try_pin_ctor::of_pin_ctor(args), f) {
        Ok(value) => value,
        Err(inf) => match inf {},
    }
}
