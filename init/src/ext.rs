use core::alloc::Layout;

use crate::{
    layout_provider::{LayoutProvider, MaybeLayoutProvider},
    Ctor,
};

pub struct ScalarLayoutProvider;

macro_rules! primitive {
    ($($ty:ident $(($zero:expr))?)*) => {$(

        impl LayoutProvider<$ty> for ScalarLayoutProvider {}
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

        impl LayoutProvider<$ty, $ty> for ScalarLayoutProvider {}
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

        impl LayoutProvider<$ty, &$ty> for ScalarLayoutProvider {}
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

        impl LayoutProvider<$ty, &mut $ty> for ScalarLayoutProvider {}
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
            type LayoutProvider = ScalarLayoutProvider;

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
            type LayoutProvider = ScalarLayoutProvider;

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

        impl Ctor<&$ty> for $ty {
            type LayoutProvider = ScalarLayoutProvider;

            #[inline]
            fn init<'a>(uninit: crate::Uninit<'a, Self>, arg: &$ty) -> crate::Init<'a, Self> {
                uninit.write(*arg)
            }

            #[inline]
            #[doc(hidden)]
            fn __is_args_clone_cheap() -> bool {
                true
            }
        }

        impl Ctor<&mut $ty> for $ty {
            type LayoutProvider = ScalarLayoutProvider;

            #[inline]
            fn init<'a>(uninit: crate::Uninit<'a, Self>, arg: &mut $ty) -> crate::Init<'a, Self> {
                uninit.write(*arg)
            }
        }
    )*};
}

primitive!(u8 u16 u32 u64 u128 usize);
primitive!(i8 i16 i32 i64 i128 isize);
primitive!(f32(0.0) f64(0.0) bool(false) char('\0'));

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
    type LayoutProvider = ScalarLayoutProvider;

    #[inline]
    fn init(uninit: crate::Uninit<'_, Self>, (): ()) -> crate::Init<'_, Self> {
        uninit.write(())
    }
}
