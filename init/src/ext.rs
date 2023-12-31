//! Constructors and layout providers for external types

use core::{alloc::Layout, cell::UnsafeCell, pin::Pin};

use crate::{
    config_value::{CloneTag, ConfigValue, MoveTag, PinCloneTag, PinMoveTag, PinTakeTag, TakeTag},
    ctor::{CloneCtor, MoveCtor, TakeCtor},
    layout_provider::{HasLayoutProvider, LayoutProvider},
    pin_ctor::{PinCloneCtor, PinMoveCtor, PinTakeCtor},
    Ctor, PinCtor,
};

/// A layout provider for scalar primitives
pub struct ScalarLayoutProvider;

macro_rules! primitive {
    ($($ty:ident $(($zero:expr))?)*) => {$(

        impl HasLayoutProvider for $ty {
            type LayoutProvider = ScalarLayoutProvider;
        }

        // SAFETY: sized types have a known layout
        unsafe impl LayoutProvider<$ty> for ScalarLayoutProvider {
            #[inline]
            fn layout_of((): &()) -> Option<core::alloc::Layout> {
                Some(Layout::new::<$ty>())
            }

            #[inline]
            unsafe fn cast(ptr: core::ptr::NonNull<u8>, (): &()) -> core::ptr::NonNull<$ty> {
                ptr.cast()
            }

            #[inline]
            fn is_zeroed(_: &()) -> bool {
                true
            }
        }

        impl HasLayoutProvider<$ty> for $ty {
            type LayoutProvider = ScalarLayoutProvider;
        }

        // SAFETY: sized types have a known layout
        unsafe impl LayoutProvider<$ty, $ty> for ScalarLayoutProvider {
            #[inline]
            fn layout_of(_: &$ty) -> Option<core::alloc::Layout> {
                Some(Layout::new::<$ty>())
            }

            #[inline]
            unsafe fn cast(ptr: core::ptr::NonNull<u8>, _: &$ty) -> core::ptr::NonNull<$ty> {
                ptr.cast()
            }

            #[inline]
            fn is_zeroed(arg: &$ty) -> bool {
                let _value = 0;
                $(let _value = $zero;)?
                *arg == _value
            }
        }

        impl HasLayoutProvider<&$ty> for $ty {
            type LayoutProvider = ScalarLayoutProvider;
        }

        // SAFETY: sized types have a known layout
        unsafe impl LayoutProvider<$ty, &$ty> for ScalarLayoutProvider {
            #[inline]
            fn layout_of(_: &&$ty) -> Option<core::alloc::Layout> {
                Some(Layout::new::<$ty>())
            }

            #[inline]
            unsafe fn cast(ptr: core::ptr::NonNull<u8>, _: &&$ty) -> core::ptr::NonNull<$ty> {
                ptr.cast()
            }

            #[inline]
            fn is_zeroed(arg: &&$ty) -> bool {
                let _value = 0;
                $(let _value = $zero;)?
                **arg == _value
            }
        }

        impl HasLayoutProvider<&mut $ty> for $ty {
            type LayoutProvider = ScalarLayoutProvider;
        }

        // SAFETY: sized types have a known layout
        unsafe impl LayoutProvider<$ty, &mut $ty> for ScalarLayoutProvider {
            #[inline]
            fn layout_of(_: &&mut $ty) -> Option<core::alloc::Layout> {
                Some(Layout::new::<$ty>())
            }

            #[inline]
            unsafe fn cast(ptr: core::ptr::NonNull<u8>, _: &&mut $ty) -> core::ptr::NonNull<$ty> {
                ptr.cast()
            }

            #[inline]
            fn is_zeroed(arg: &&mut $ty) -> bool {
                let _value = 0;
                $(let _value = $zero;)?
                **arg == _value
            }
        }

        impl Ctor for $ty {
            #[inline]
            fn init(uninit: crate::Uninit<'_, Self>, (): ()) -> crate::Init<'_, Self> {
                let _value = 0;
                $(let _value = $zero;)?
                uninit.write(_value)
            }

            #[inline]
            #[doc(hidden)]
            fn __is_args_clone_cheap() -> bool {
                true
            }
        }

        impl Ctor<$ty> for $ty {
            #[inline]
            fn init(uninit: crate::Uninit<'_, Self>, arg: $ty) -> crate::Init<'_, Self> {
                uninit.write(arg)
            }

            #[inline]
            #[doc(hidden)]
            fn __is_args_clone_cheap() -> bool {
                true
            }
        }

        impl PinCtor for $ty {
            #[inline]
            fn pin_init(uninit: crate::Uninit<'_, Self>, (): ()) -> crate::PinInit<'_, Self> {
                let _value = 0;
                $(let _value = $zero;)?
                uninit.write(_value).pin()
            }

            #[inline]
            #[doc(hidden)]
            fn __is_args_clone_cheap() -> bool {
                true
            }
        }

        impl PinCtor<$ty> for $ty {
            #[inline]
            fn pin_init(uninit: crate::Uninit<'_, Self>, arg: $ty) -> crate::PinInit<'_, Self> {
                uninit.write(arg).pin()
            }

            #[inline]
            #[doc(hidden)]
            fn __is_args_clone_cheap() -> bool {
                true
            }
        }

        impl MoveCtor for $ty {
            const IS_MOVE_TRIVIAL: ConfigValue<Self, MoveTag> = {
                // SAFETY: all primitive types are trivially movable
                unsafe { ConfigValue::yes() }
            };
            #[inline]
            fn move_ctor<'this>(
                uninit: crate::Uninit<'this, Self>,
                p: crate::Init<Self>,
            ) -> crate::Init<'this, Self> {
                uninit.write(*p.get())
            }
        }

        impl TakeCtor for $ty {
            const IS_TAKE_TRIVIAL: ConfigValue<Self, TakeTag> = {
                // SAFETY: all primitive types are trivially takable
                unsafe { ConfigValue::yes() }
            };

            #[inline]
            fn take_ctor<'this>(
                uninit: crate::Uninit<'this, Self>,
                p: &mut Self,
            ) -> crate::Init<'this, Self> {
                uninit.write(*p)
            }
        }

        impl CloneCtor for $ty {
            const IS_CLONE_TRIVIAL: ConfigValue<Self, CloneTag> = {
                // SAFETY: all primitive types are trivially clone-able
                unsafe { ConfigValue::yes() }
            };

            #[inline]
            fn clone_ctor<'this>(uninit: crate::Uninit<'this, Self>, p: &Self) -> crate::Init<'this, Self> {
                uninit.write(*p)
            }
        }

        impl PinMoveCtor for $ty {
            const IS_MOVE_TRIVIAL: ConfigValue<Self, PinMoveTag> = {
                // SAFETY: all primitive types are trivially movable
                unsafe { ConfigValue::yes() }
            };

            #[inline]
            fn pin_move_ctor<'this>(
                uninit: crate::Uninit<'this, Self>,
                p: crate::PinInit<Self>,
            ) -> crate::PinInit<'this, Self> {
                uninit.write(*p.get()).pin()
            }
        }

        impl PinTakeCtor for $ty {
            const IS_TAKE_TRIVIAL: ConfigValue<Self, PinTakeTag> = {
                // SAFETY: all primitive types are trivially takable
                unsafe { ConfigValue::yes() }
            };

            #[inline]
            fn pin_take_ctor<'this>(
                uninit: crate::Uninit<'this, Self>,
                p: Pin<&mut Self>,
            ) -> crate::PinInit<'this, Self> {
                uninit.write(*p).pin()
            }
        }

        impl PinCloneCtor for $ty {
            const IS_CLONE_TRIVIAL: ConfigValue<Self, PinCloneTag> = {
                // SAFETY: all primitive types are trivially clone-able
                unsafe { ConfigValue::yes() }
            };

            #[inline]
            fn pin_clone_ctor<'this>(
                uninit: crate::Uninit<'this, Self>,
                p: Pin<&Self>,
            ) -> crate::PinInit<'this, Self> {
                uninit.write(*p).pin()
            }
        }
    )*};
}

primitive!(u8 u16 u32 u64 u128 usize);
primitive!(i8 i16 i32 i64 i128 isize);
primitive!(f32(0.0) f64(0.0) bool(false) char('\0'));

impl HasLayoutProvider for () {
    type LayoutProvider = ScalarLayoutProvider;
}

// SAFETY: sized types have a known layout
unsafe impl LayoutProvider<()> for ScalarLayoutProvider {
    #[inline]
    fn layout_of((): &()) -> Option<core::alloc::Layout> {
        Some(Layout::new::<()>())
    }

    #[inline]
    unsafe fn cast(ptr: core::ptr::NonNull<u8>, (): &()) -> core::ptr::NonNull<()> {
        ptr.cast()
    }

    #[inline]
    fn is_zeroed(_: &()) -> bool {
        true
    }
}

impl Ctor for () {
    #[inline]
    fn init(uninit: crate::Uninit<'_, Self>, (): ()) -> crate::Init<'_, Self> {
        uninit.write(())
    }
}

impl PinCtor for () {
    #[inline]
    fn pin_init(uninit: crate::Uninit<'_, Self>, (): ()) -> crate::PinInit<'_, Self> {
        uninit.write(()).pin()
    }
}

impl MoveCtor for () {
    const IS_MOVE_TRIVIAL: ConfigValue<Self, MoveTag> = {
        // SAFETY: all primitive types are trivially movable
        unsafe { ConfigValue::yes() }
    };
    #[inline]
    fn move_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        _: crate::Init<Self>,
    ) -> crate::Init<'this, Self> {
        uninit.write(())
    }
}

impl TakeCtor for () {
    const IS_TAKE_TRIVIAL: ConfigValue<Self, TakeTag> = {
        // SAFETY: all primitive types are trivially takable
        unsafe { ConfigValue::yes() }
    };

    #[inline]
    fn take_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        (): &mut Self,
    ) -> crate::Init<'this, Self> {
        uninit.write(())
    }
}

impl CloneCtor for () {
    const IS_CLONE_TRIVIAL: ConfigValue<Self, CloneTag> = {
        // SAFETY: all primitive types are trivially clone-able
        unsafe { ConfigValue::yes() }
    };

    #[inline]
    fn clone_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        (): &Self,
    ) -> crate::Init<'this, Self> {
        uninit.write(())
    }
}

impl PinMoveCtor for () {
    const IS_MOVE_TRIVIAL: ConfigValue<Self, PinMoveTag> = {
        // SAFETY: all primitive types are trivially movable
        unsafe { ConfigValue::yes() }
    };

    #[inline]
    fn pin_move_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        _: crate::PinInit<Self>,
    ) -> crate::PinInit<'this, Self> {
        uninit.write(()).pin()
    }
}

impl PinTakeCtor for () {
    const IS_TAKE_TRIVIAL: ConfigValue<Self, PinTakeTag> = {
        // SAFETY: all primitive types are trivially takable
        unsafe { ConfigValue::yes() }
    };

    #[inline]
    fn pin_take_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        _: Pin<&mut Self>,
    ) -> crate::PinInit<'this, Self> {
        uninit.write(()).pin()
    }
}

impl PinCloneCtor for () {
    const IS_CLONE_TRIVIAL: ConfigValue<Self, PinCloneTag> = {
        // SAFETY: all primitive types are trivially clone-able
        unsafe { ConfigValue::yes() }
    };

    #[inline]
    fn pin_clone_ctor<'this>(
        uninit: crate::Uninit<'this, Self>,
        _: Pin<&Self>,
    ) -> crate::PinInit<'this, Self> {
        uninit.write(()).pin()
    }
}
/// A constructor for an [`UnsafeCell`]
pub struct NewUnsafeCell<T>(pub T);

impl<T: ?Sized + crate::Ctor<A>, A> crate::Ctor<NewUnsafeCell<A>> for UnsafeCell<T> {
    fn init(
        uninit: crate::Uninit<'_, Self>,
        NewUnsafeCell(args): NewUnsafeCell<A>,
    ) -> crate::Init<'_, Self> {
        // SAFETY: UnsafeCell has the same layout as `T`
        let value = unsafe { crate::Uninit::from_raw(uninit.as_ptr() as *mut T) };
        value.init(args).take_ownership();
        // SAFETY: ^^^ The value was initialized
        unsafe { uninit.assume_init() }
    }
}

impl<T: ?Sized + crate::PinCtor<A>, A> crate::PinCtor<NewUnsafeCell<A>> for UnsafeCell<T> {
    fn pin_init(
        uninit: crate::Uninit<'_, Self>,
        NewUnsafeCell(args): NewUnsafeCell<A>,
    ) -> crate::PinInit<'_, Self> {
        // SAFETY: UnsafeCell has the same layout as `T` and it will remain in the pinned type-state
        let value = unsafe { crate::Uninit::from_raw(uninit.as_ptr() as *mut T) };
        value.pin_init(args).take_ownership();
        // SAFETY: ^^^ The value was initialized
        unsafe { uninit.assume_init().pin() }
    }
}

/// The layout provider for [`UnsafeCell`]
pub struct UnsafeCellLayoutProvider;

impl<T: ?Sized + HasLayoutProvider<A>, A> HasLayoutProvider<NewUnsafeCell<A>> for UnsafeCell<T> {
    type LayoutProvider = UnsafeCellLayoutProvider;
}

// SAFETY: [`UnsafeCell`] has the same layout as `T` so it's safe to just defer to `T`'s layout provider
unsafe impl<T: ?Sized + HasLayoutProvider<A>, A> LayoutProvider<UnsafeCell<T>, NewUnsafeCell<A>>
    for UnsafeCellLayoutProvider
{
    fn layout_of(args: &NewUnsafeCell<A>) -> Option<Layout> {
        crate::layout_provider::layout_of::<T, A>(&args.0)
    }

    unsafe fn cast(
        ptr: core::ptr::NonNull<u8>,
        args: &NewUnsafeCell<A>,
    ) -> core::ptr::NonNull<UnsafeCell<T>> {
        // SAFETY: see impl level safety doc
        unsafe {
            let ptr = crate::layout_provider::cast::<T, A>(ptr, &args.0);
            core::ptr::NonNull::new_unchecked(ptr.as_ptr() as *mut UnsafeCell<T>)
        }
    }

    fn is_zeroed(args: &NewUnsafeCell<A>) -> bool {
        crate::layout_provider::is_zeroed::<T, A>(&args.0)
    }
}
