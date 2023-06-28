//! This module defines type-safe compile time configuration value to work around the missing
//! specialization feature in Rust. It can answer any yes or no question, where yes implies an
//! unsafe obligation of the implementor.

use core::marker::PhantomData;

/// A tag to make the config value unique to the [`PinMove`](crate::pin_interface::PinMove) trait
pub enum PinMoveTag {}
/// A tag to make the config value unique to the [`PinTake`](crate::pin_interface::PinTake) trait
pub enum PinTakeTag {}
/// A tag to make the config value unique to the [`PinClone`](crate::pin_interface::PinClone) trait
pub enum PinCloneTag {}

/// A tag to make the config value unique to the [`Move`](crate::interface::Move) trait
pub enum MoveTag {}
/// A tag to make the config value unique to the [`Take`](crate::interface::Take) trait
pub enum TakeTag {}
/// A tag to make the config value unique to the [`Clone`](crate::interface::Clone) trait
pub enum CloneTag {}

/// An answer to queries in PinCtorConfig
#[repr(transparent)]
pub struct ConfigValue<T: ?Sized, Tag: ?Sized>(bool, PhantomData<fn() -> (*mut T, *mut Tag)>);

impl<T: ?Sized, Tag: ?Sized> ConfigValue<T, Tag> {
    /// Answer no to a `PinCtorConfig` option
    pub const fn no() -> Self {
        Self(false, PhantomData)
    }

    /// Answer yes to a `PinCtorConfig` option
    ///
    /// # Safety
    ///
    /// See `PinCtorConfig` option
    pub const unsafe fn yes() -> Self {
        Self(true, PhantomData)
    }

    /// Get the value of this config setting
    pub const fn get(self) -> bool {
        self.0
    }

    ///
    ///
    /// # Safety
    ///
    /// See `PinCtorConfig` option (the guarantee for `U` must be compatible with `T`)
    pub const unsafe fn cast<U: ?Sized>(self) -> ConfigValue<U, Tag> {
        ConfigValue(self.0, PhantomData)
    }

    /// Is only true if both values are true
    pub const fn and(self, other: Self) -> Self {
        ConfigValue(self.0 & other.0, PhantomData)
    }
}
