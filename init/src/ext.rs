use core::{alloc::Layout, pin::Pin};

use crate::{
    interface::{CloneCtor, MoveCtor, PinCloneCtor, PinMoveCtor, PinTakeCtor, TakeCtor},
    layout_provider::{HasLayoutProvider, MaybeLayoutProvider},
    Ctor,
};

pub struct ScalarLayoutProvider;

macro_rules! primitive {
    ($($ty:ident $(($zero:expr))?)*) => {$(

        impl HasLayoutProvider for $ty {
            type LayoutProvider = ScalarLayoutProvider;
        }

        // SAFETY: sized types have a known layout
        unsafe impl MaybeLayoutProvider<$ty> for ScalarLayoutProvider {
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
        unsafe impl MaybeLayoutProvider<$ty, $ty> for ScalarLayoutProvider {
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
        unsafe impl MaybeLayoutProvider<$ty, &$ty> for ScalarLayoutProvider {
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
        unsafe impl MaybeLayoutProvider<$ty, &mut $ty> for ScalarLayoutProvider {
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

        impl MoveCtor for $ty {
            const IS_MOVE_TRIVIAL: crate::interface::ConfigValue<Self, crate::interface::MoveTag> = {
                // SAFETY: all primitive types are trivially movable
                unsafe { crate::interface::ConfigValue::yes() }
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
            const IS_TAKE_TRIVIAL: crate::interface::ConfigValue<Self, crate::interface::TakeTag> = {
                // SAFETY: all primitive types are trivially takable
                unsafe { crate::interface::ConfigValue::yes() }
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
            const IS_CLONE_TRIVIAL: crate::interface::ConfigValue<Self, crate::interface::CloneTag> = {
                // SAFETY: all primitive types are trivially clone-able
                unsafe { crate::interface::ConfigValue::yes() }
            };

            #[inline]
            fn clone_ctor<'this>(uninit: crate::Uninit<'this, Self>, p: &Self) -> crate::Init<'this, Self> {
                uninit.write(*p)
            }
        }

        impl PinMoveCtor for $ty {
            const IS_MOVE_TRIVIAL: crate::interface::ConfigValue<Self, crate::interface::PinMoveTag> = {
                // SAFETY: all primitive types are trivially movable
                unsafe { crate::interface::ConfigValue::yes() }
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
            const IS_TAKE_TRIVIAL: crate::interface::ConfigValue<Self, crate::interface::PinTakeTag> = {
                // SAFETY: all primitive types are trivially takable
                unsafe { crate::interface::ConfigValue::yes() }
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
            const IS_CLONE_TRIVIAL: crate::interface::ConfigValue<Self, crate::interface::PinCloneTag> = {
                // SAFETY: all primitive types are trivially clone-able
                unsafe { crate::interface::ConfigValue::yes() }
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
unsafe impl MaybeLayoutProvider<()> for ScalarLayoutProvider {
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
