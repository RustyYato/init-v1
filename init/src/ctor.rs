//! The core interfaces used to initialize types

use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{
    config_value::{CloneTag, ConfigValue, MoveTag, TakeTag},
    Init, Uninit,
};

/// A type which is constructable using `Args`
pub trait Ctor<Args = ()> {
    /// Initialize a the type `Self` using `args: Args`
    fn init(uninit: Uninit<'_, Self>, args: Args) -> Init<'_, Self>;

    #[inline]
    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        false
    }
}

/// A type which can construct a `T`
pub trait CtorArgs<T: ?Sized> {
    /// Initialize a the type `T` using `self`
    fn init_into(self, uninit: Uninit<'_, T>) -> Init<'_, T>;

    #[inline]
    #[doc(hidden)]
    fn __is_clone_cheap() -> bool {
        false
    }
}

impl<T: ?Sized, Args: CtorArgs<T>> Ctor<Args> for T {
    #[inline]
    fn init(uninit: Uninit<'_, Self>, args: Args) -> Init<'_, Self> {
        args.init_into(uninit)
    }

    #[inline]
    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        Args::__is_clone_cheap()
    }
}

impl<T> Ctor for MaybeUninit<T> {
    #[inline]
    fn init(uninit: Uninit<'_, Self>, (): ()) -> Init<'_, Self> {
        uninit.uninit()
    }

    #[inline]
    #[doc(hidden)]
    fn __is_args_clone_cheap() -> bool {
        true
    }
}

struct CtorFn<F, T: ?Sized>(F, PhantomData<T>);

impl<T: ?Sized, F: FnOnce(Uninit<'_, T>) -> Init<'_, T>> CtorArgs<T> for CtorFn<F, T> {
    #[inline]
    fn init_into(self, uninit: Uninit<'_, T>) -> Init<'_, T> {
        (self.0)(uninit)
    }
}

/// a no-op helper function to guide type inference
///
/// Rust's type inference doesn't understand the indirection from
/// `FnOnce()` to `CtorArgs` to `Ctor`, so use this no-op to guide inference
pub fn ctor<T: ?Sized, F: FnOnce(Uninit<T>) -> Init<T>>(f: F) -> impl CtorArgs<T> {
    CtorFn(f, PhantomData)
}

/// An interface to "move" values without any temporaries
pub trait MoveCtor {
    /// If `pin_move_ctor` can be simulated by a memcpy
    ///
    /// # Safety for implementors
    ///
    /// you may only answer yes if there are no side-effects to moving the value
    /// and if there are no self-references that need to be handled
    const IS_MOVE_TRIVIAL: ConfigValue<Self, MoveTag> = ConfigValue::no();

    /// "moves" the value in `p` to `uninit`
    fn move_ctor<'this>(uninit: Uninit<'this, Self>, p: Init<Self>) -> Init<'this, Self>;
}

/// An interface to "take" values without any temporaries
pub trait TakeCtor: MoveCtor {
    /// If `pin_take_ctor` can be simulated by a memcpy
    ///
    /// # Safety for implementors
    ///
    /// you may only answer yes if there are no side-effects to moving the value
    /// and if there are no self-references that need to be handled or owned
    /// resources which need to be properly taken
    const IS_TAKE_TRIVIAL: ConfigValue<Self, TakeTag> = ConfigValue::no();

    /// takes the value in `p` to `uninit`, the value in `p` will be left in a
    /// valid (safe), but unspecified state. The implementing type may guarantee what
    /// value the move constructor leaves it's state in
    fn take_ctor<'this>(uninit: Uninit<'this, Self>, p: &mut Self) -> Init<'this, Self>;
}

/// An interface to clone values without any temporaries
pub trait CloneCtor: TakeCtor {
    /// If `pin_clone_ctor` can be simulated by a memcpy
    ///
    /// # Safety for implementors
    ///
    /// you may only answer yes if there are no side-effects to moving the value
    /// and if there are no self-references that need to be handled or owned
    /// resources which need to be properly cloned
    const IS_CLONE_TRIVIAL: ConfigValue<Self, CloneTag> = ConfigValue::no();

    /// clones the value in `p` to `uninit`
    fn clone_ctor<'this>(uninit: Uninit<'this, Self>, p: &Self) -> Init<'this, Self>;
}
