//! The core interfaces used to pin-initialize types

use core::{marker::PhantomData, pin::Pin};

use crate::{
    config_value::{ConfigValue, PinCloneTag, PinMoveTag, PinTakeTag},
    PinInit, Uninit,
};

/// A type which is constructable using `Args`
pub trait PinCtor<Args = ()> {
    /// Initialize a the type `Self` using `args: Args`
    fn pin_init(uninit: Uninit<'_, Self>, args: Args) -> PinInit<'_, Self>;

    #[inline]
    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        false
    }
}

/// A type which can construct a `T`
pub trait PinCtorArgs<T: ?Sized> {
    /// Initialize a the type `T` using `self`
    fn pin_init_into(self, uninit: Uninit<'_, T>) -> PinInit<'_, T>;
}

impl<T: ?Sized, Args: PinCtorArgs<T>> PinCtor<Args> for T {
    #[inline]
    fn pin_init(uninit: Uninit<'_, Self>, args: Args) -> PinInit<'_, Self> {
        args.pin_init_into(uninit)
    }
}

struct PinCtorFn<F, T: ?Sized>(F, PhantomData<T>);

impl<T: ?Sized, F: FnOnce(Uninit<'_, T>) -> PinInit<'_, T>> PinCtorArgs<T> for PinCtorFn<F, T> {
    #[inline]
    fn pin_init_into(self, uninit: Uninit<'_, T>) -> PinInit<'_, T> {
        (self.0)(uninit)
    }
}

/// a no-op helper function to guide type inference
///
/// Rust's type inference doesn't understand the indirection from
/// `FnOnce()` to `CtorArgs` to `Ctor`, so use this no-op to guide inference
pub fn pin_ctor<T: ?Sized, F: FnOnce(Uninit<T>) -> PinInit<T>>(f: F) -> impl PinCtorArgs<T> {
    PinCtorFn(f, PhantomData)
}

/// An interface to "move" pinned values in a type-safe way
pub trait PinMoveCtor {
    /// If `pin_move_ctor` can be simulated by a memcpy
    ///
    /// # Safety for implementors
    ///
    /// you may only answer yes if there are no side-effects to moving the value
    /// and if there are no self-references that need to be handled
    const IS_MOVE_TRIVIAL: ConfigValue<Self, PinMoveTag> = ConfigValue::no();

    /// "moves" the value in `p` to `uninit`
    fn pin_move_ctor<'this>(uninit: Uninit<'this, Self>, p: PinInit<Self>) -> PinInit<'this, Self>;
}

/// An interface to "take" pinned values in a type-safe way
pub trait PinTakeCtor: PinMoveCtor {
    /// If `pin_take_ctor` can be simulated by a memcpy
    ///
    /// # Safety for implementors
    ///
    /// you may only answer yes if there are no side-effects to moving the value
    /// and if there are no self-references that need to be handled or owned
    /// resources which need to be properly taken
    const IS_TAKE_TRIVIAL: ConfigValue<Self, PinTakeTag> = ConfigValue::no();

    /// takes the value in `p` to `uninit`, the value in `p` will be left in a
    /// valid (safe), but unspecified state. The implementing type may guarantee what
    /// value the move constructor leaves it's state in
    fn pin_take_ctor<'this>(uninit: Uninit<'this, Self>, p: Pin<&mut Self>)
        -> PinInit<'this, Self>;
}

/// An interface to clone pinned values in a type-safe way
pub trait PinCloneCtor: PinTakeCtor {
    /// If `pin_clone_ctor` can be simulated by a memcpy
    ///
    /// # Safety for implementors
    ///
    /// you may only answer yes if there are no side-effects to moving the value
    /// and if there are no self-references that need to be handled or owned
    /// resources which need to be properly cloned
    const IS_CLONE_TRIVIAL: ConfigValue<Self, PinCloneTag> = ConfigValue::no();

    /// clones the value in `p` to `uninit`
    fn pin_clone_ctor<'this>(uninit: Uninit<'this, Self>, p: Pin<&Self>) -> PinInit<'this, Self>;
}
