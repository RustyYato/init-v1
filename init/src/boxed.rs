//! Creating boxes using constructors

use core::{alloc::Layout, ptr::NonNull};

use alloc::{
    alloc::{alloc, alloc_zeroed, handle_alloc_error},
    boxed::Box,
};

use crate::{
    layout_provider::{HasLayoutProvider, LayoutProvider},
    Ctor, CtorArgs, TryCtor, TryCtorArgs, Uninit,
};

/// Create a new value of the heap, initializing it in place
pub fn boxed<T, Args>(args: Args) -> Box<T>
where
    T: ?Sized + Ctor<Args> + HasLayoutProvider<Args>,
{
    match try_boxed(crate::try_ctor::of_ctor(args)) {
        Ok(bx) => bx,
        Err(err) => err.handle(),
    }
}

/// Create a new value of the heap, initializing it in place
pub fn boxed_with<T, Args, L>(args: Args) -> Box<T>
where
    T: ?Sized + Ctor<Args>,
    L: LayoutProvider<T, Args>,
{
    match try_boxed_with::<T, _, crate::try_ctor::OfCtorLayoutProvider<L>>(
        crate::try_ctor::of_ctor(args),
    ) {
        Ok(bx) => bx,
        Err(err) => err.handle(),
    }
}

/// Safety, an error type for handling Box creation failure
pub enum TryBoxedError<E> {
    /// The layout wasn't calculated or was too large to fit in [`Layout`]
    LayoutError,
    /// The allocation failed for the given layout
    AllocError(Layout),
    /// Initialization failed with the given error
    InitError(E),
}

impl<E> TryBoxedError<E> {
    /// Handle all the allocation errors, and return the initialization errors
    pub fn handle_alloc_and_layout(self) -> E {
        match self {
            TryBoxedError::LayoutError => {
                #[cold]
                #[inline(never)]
                fn layout_calculation_failed() -> ! {
                    panic!("Could not construct layout")
                }

                layout_calculation_failed()
            }
            TryBoxedError::AllocError(layout) => handle_alloc_error(layout),
            TryBoxedError::InitError(error) => error,
        }
    }
}

impl TryBoxedError<core::convert::Infallible> {
    /// Handle all the allocation errors, and assert that there are no initialization errors
    pub fn handle(self) -> ! {
        #[allow(unreachable_code)]
        match self.handle_alloc_and_layout() {}
    }
}

/// Create a new value of the heap, initializing it in place
pub fn try_boxed<T, Args>(args: Args) -> Result<Box<T>, TryBoxedError<T::Error>>
where
    T: ?Sized + TryCtor<Args> + HasLayoutProvider<Args>,
{
    try_boxed_with::<T, Args, T::LayoutProvider>(args)
}

/// Create a new value of the heap, initializing it in place
pub fn try_boxed_with<T, Args, L>(args: Args) -> Result<Box<T>, TryBoxedError<T::Error>>
where
    T: ?Sized + TryCtor<Args>,
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

        let init = uninit.try_init(args).map_err(TryBoxedError::InitError)?;

        // the box will take ownership of the `T`, so we should forget the `Init`
        init.take_ownership();
    }

    // SAFETY: ptr points to an initialized, non-null, aligned pointer to T that was allocated
    // using the global allocator
    Ok(unsafe { Box::from_raw(ptr.as_ptr()) })
}

/// Converts an initializer argument to one that can initialize a [`Box`]
pub struct Boxed<Args>(pub Args);

impl<T, Args> CtorArgs<Box<T>> for Boxed<Args>
where
    T: ?Sized + Ctor<Args> + HasLayoutProvider<Args>,
{
    fn init_into(self, uninit: Uninit<'_, Box<T>>) -> crate::Init<'_, Box<T>> {
        uninit.write(boxed(self.0))
    }

    #[inline]
    #[doc(hidden)]
    fn __is_clone_cheap() -> bool {
        T::__is_args_clone_cheap()
    }
}

impl<T, Args> TryCtorArgs<Box<T>> for Boxed<Args>
where
    T: ?Sized + TryCtor<Args> + HasLayoutProvider<Args>,
{
    type Error = TryBoxedError<T::Error>;

    fn try_init_into(
        self,
        uninit: Uninit<'_, Box<T>>,
    ) -> Result<crate::Init<'_, Box<T>>, Self::Error> {
        Ok(uninit.write(try_boxed(self.0)?))
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
        let value = super::boxed::<[u8], _>(crate::slice::ctor::CopyArgsLen(10, ()));

        assert_eq!(*value, [0; 10]);
    }
}
