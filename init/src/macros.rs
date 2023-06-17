pub use core;

use crate::Uninit;

#[macro_export]
macro_rules! init_struct {
    ($u:ident => $ty:path {
        $($(
            $field_name:ident : $field_value:expr
        ),+ $(,)?)?
    }) => {{
        let mut uninit: $crate::Uninit<_> = $u;
        let ptr = uninit.as_mut_ptr();
        // ensure that all fields are accounted for, and no deref fields are used
        let $ty { $($($field_name: _,)*)? };
        $($(
            // SAFETY: ptr is a dereferencable pointer (guaranteed by `Uninit`)
            let field_ptr = unsafe { $crate::macros::core::ptr::addr_of_mut!((*ptr).$field_name) };
            // SAFETY: ptr came from uninit
            let field_uninit = unsafe { $crate::Uninit::from_raw(field_ptr) };
            // ensure that uninit and field_uninit have the same lifetime so the user
            // can't invalidate the `Init`
            $crate::macros::bind_lifetimes(&uninit, &field_uninit);
            #[allow(unused_mut)]
            let mut $field_name = field_uninit.init($field_value);
        )*)?
        // leak all fields, since the struct will take ownership of them
        $crate::macros::core::mem::forget((
            $($($field_name,)*)?
        ));
        // SAFETY: all fields were initialized
        unsafe { uninit.assume_init() }
    }};
}

#[macro_export]
macro_rules! try_init_struct {
    ($u:ident => $ty:path {
        $($(
            $field_name:ident : $field_value:expr
        ),+ $(,)?)?
    }) => {{
        let mut uninit: $crate::Uninit<_> = $u;
        let ptr = uninit.as_mut_ptr();
        // ensure that all fields are accounted for, and no deref fields are used
        let $ty { $($($field_name: _,)*)? };
        $($(
            // SAFETY: ptr is a dereferencable pointer (guaranteed by `Uninit`)
            let field_ptr = unsafe { $crate::macros::core::ptr::addr_of_mut!((*ptr).$field_name) };
            // SAFETY: ptr came from uninit
            let field_uninit = unsafe { $crate::Uninit::from_raw(field_ptr) };
            // ensure that uninit and field_uninit have the same lifetime so the user
            // can't invalidate the `Init`
            $crate::macros::bind_lifetimes(&uninit, &field_uninit);
            #[allow(unused_mut)]
            let mut $field_name = field_uninit.try_init($field_value)?;
        )*)?
        // leak all fields, since the struct will take ownership of them
        $crate::macros::core::mem::forget((
            $($($field_name,)*)?
        ));
        // SAFETY: all fields were initialized
        unsafe { uninit.assume_init() }
    }};
}

pub fn bind_lifetimes<'a, T: ?Sized, U: ?Sized>(_: &'a Uninit<'_, T>, _: &Uninit<'a, U>) {
    //
}
