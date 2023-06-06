//! The core interfaces used to initialize types

mod source;

use core::{marker::PhantomData, mem::MaybeUninit, pin::Pin};

use crate::{
    layout_provider::{HasLayoutProvider, SizedLayoutProvider},
    Init, PinInit, Uninit,
};

pub use source::SourceLayoutProvider;

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
pub trait CtorArgs<T: ?Sized>: Sized {
    /// Initialize a the type `T` using `self`
    fn init_with(self, uninit: Uninit<'_, T>) -> Init<'_, T>;
}

/// A type which can construct a `T`
pub trait PinCtorArgs<T: ?Sized>: Sized {
    /// Initialize a the type `T` using `self`
    fn pin_init_with(self, uninit: Uninit<'_, T>) -> PinInit<'_, T>;
}

impl<T: ?Sized, Args: CtorArgs<T>> Ctor<Args> for T {
    #[inline]
    fn init(uninit: Uninit<'_, Self>, args: Args) -> Init<'_, Self> {
        args.init_with(uninit)
    }
}

impl<T: ?Sized, Args: PinCtorArgs<T>> PinCtor<Args> for T {
    #[inline]
    fn pin_init(uninit: Uninit<'_, Self>, args: Args) -> PinInit<'_, Self> {
        args.pin_init_with(uninit)
    }
}

impl<T> HasLayoutProvider for MaybeUninit<T> {
    type LayoutProvider = SizedLayoutProvider;
}

impl<T> Ctor for MaybeUninit<T> {
    #[inline]
    fn init(uninit: Uninit<'_, Self>, (): ()) -> Init<'_, Self> {
        uninit.uninit()
    }
}

struct CtorFn<F, T: ?Sized>(F, PhantomData<T>);

impl<T: ?Sized, F: FnOnce(Uninit<'_, T>) -> Init<'_, T>> CtorArgs<T> for CtorFn<F, T> {
    #[inline]
    fn init_with(self, uninit: Uninit<'_, T>) -> Init<'_, T> {
        (self.0)(uninit)
    }
}

impl<T: ?Sized, F: FnOnce(Uninit<'_, T>) -> PinInit<'_, T>> PinCtorArgs<T> for CtorFn<F, T> {
    #[inline]
    fn pin_init_with(self, uninit: Uninit<'_, T>) -> PinInit<'_, T> {
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

/// a no-op helper function to guide type inference
///
/// Rust's type inference doesn't understand the indirection from
/// `FnOnce()` to `CtorArgs` to `Ctor`, so use this no-op to guide inference
pub fn pin_ctor<T: ?Sized, F: FnOnce(Uninit<T>) -> PinInit<T>>(f: F) -> impl PinCtorArgs<T> {
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

#[doc(hidden)]
pub enum PinMoveTag {}
#[doc(hidden)]
pub enum PinTakeTag {}
#[doc(hidden)]
pub enum PinCloneTag {}

#[doc(hidden)]
pub enum MoveTag {}
#[doc(hidden)]
pub enum TakeTag {}
#[doc(hidden)]
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
}
