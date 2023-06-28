//! Creating boxes using constructors

use core::{pin::Pin, ptr::NonNull};

use alloc::{
    alloc::{alloc, alloc_zeroed},
    boxed::Box,
};

use crate::{
    layout_provider::{HasLayoutProvider, LayoutProvider},
    CtorArgs, PinCtor, TryCtorArgs, TryPinCtor, Uninit,
};

pub use crate::boxed::{Boxed, TryBoxedError};

/// Create a new value of the heap, initializing it in place
pub fn pin_boxed<T, Args>(args: Args) -> Pin<Box<T>>
where
    T: ?Sized + PinCtor<Args> + HasLayoutProvider<Args>,
{
    pin_boxed_with::<T, Args, T::LayoutProvider>(args)
}

/// Create a new value of the heap, initializing it in place
pub fn pin_boxed_with<T, Args, L>(args: Args) -> Pin<Box<T>>
where
    T: ?Sized + PinCtor<Args>,
    L: LayoutProvider<T, Args>,
{
    match try_pin_boxed_with::<T, _, crate::try_pin_ctor::OfPinCtorLayoutProvider<L>>(
        crate::try_pin_ctor::of_pin_ctor(args),
    ) {
        Ok(bx) => bx,
        Err(err) => err.handle(),
    }
}

/// Create a new value of the heap, initializing it in place
pub fn try_pin_boxed<T, Args>(args: Args) -> Result<Pin<Box<T>>, TryBoxedError<T::Error>>
where
    T: ?Sized + TryPinCtor<Args> + HasLayoutProvider<Args>,
{
    try_pin_boxed_with::<T, Args, T::LayoutProvider>(args)
}
/// Create a new value of the heap, initializing it in place
pub fn try_pin_boxed_with<T, Args, L>(args: Args) -> Result<Pin<Box<T>>, TryBoxedError<T::Error>>
where
    T: ?Sized + TryPinCtor<Args>,
    L: LayoutProvider<T, Args>,
{
    let layout = L::layout_of(&args).ok_or(TryBoxedError::LayoutError)?;
    let is_zeroed = L::is_zeroed(&args);

    let ptr = if layout.size() == 0 {
        layout.align() as *mut u8
    } else if is_zeroed {
        // SAFETY: layout.size() != 0
        unsafe { alloc_zeroed(layout) }
    } else {
        // SAFETY: layout.size() != 0
        unsafe { alloc(layout) }
    };

    let Some(ptr) = NonNull::new(ptr) else {
        return Err(TryBoxedError::AllocError(layout))
    };

    // SAFETY: `lp::layout_of` returned a layout
    let ptr = unsafe { L::cast(ptr, &args) };

    // SAFETY: if the layout provider says the argument just zeros the memory with no side effects
    // then we can skip initialization
    if !is_zeroed {
        // SAFETY: ptr is a freshly allocated non-null, aligned pointer for `T`
        // because the layout given by `LayoutProvider` is correct
        // and `alloc`/`alloc_zeroed`
        let uninit = unsafe { Uninit::from_raw(ptr.as_ptr()) };

        let init = uninit
            .try_pin_init(args)
            .map_err(TryBoxedError::InitError)?;

        // the box will take ownership of the `T`, so we should forget the `Init`
        init.take_ownership();
    }

    // SAFETY: ptr points to an initialized, non-null, aligned pointer to T that was allocated
    // using the global allocator
    // Pin<Box<T>> has the same representation as `*mut T`
    // We can't use `Box::from_raw` -> `Box::into_pin`/`Pin::new_unchecked` because moving a boxed
    // item invalidates internal pointers due to Stacked Borrows/Tree Borrows
    // and is otherwise equivalent to
    // Ok(unsafe { Box::into_pin(Box::from_raw(ptr.as_ptr())) })
    Ok(unsafe { core::mem::transmute(ptr.as_ptr()) })
}

impl<T, Args> CtorArgs<Pin<Box<T>>> for Boxed<Args>
where
    T: ?Sized + PinCtor<Args> + HasLayoutProvider<Args>,
{
    fn init_into(self, uninit: Uninit<'_, Pin<Box<T>>>) -> crate::Init<'_, Pin<Box<T>>> {
        uninit.write(pin_boxed(self.0))
    }

    #[inline]
    #[doc(hidden)]
    fn __is_clone_cheap() -> bool {
        T::__is_args_clone_cheap()
    }
}

impl<T, Args> TryCtorArgs<Pin<Box<T>>> for Boxed<Args>
where
    T: ?Sized + TryPinCtor<Args> + HasLayoutProvider<Args>,
{
    type Error = TryBoxedError<T::Error>;

    fn try_init_into(
        self,
        uninit: Uninit<'_, Pin<Box<T>>>,
    ) -> Result<crate::Init<'_, Pin<Box<T>>>, Self::Error> {
        Ok(uninit.write(try_pin_boxed(self.0)?))
    }

    #[inline]
    #[doc(hidden)]
    fn __is_clone_cheap() -> bool {
        T::__is_args_clone_cheap()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let value = super::pin_boxed::<[u8], _>(crate::slice::pin_ctor::CopyArgsLen(10, ()));

        assert_eq!(*value, [0; 10]);
    }
}
