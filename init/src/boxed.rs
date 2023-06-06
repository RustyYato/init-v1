//! Creating boxes using constructors

use core::ptr::NonNull;

use alloc::{
    alloc::{alloc, alloc_zeroed, handle_alloc_error},
    boxed::Box,
};

use crate::{
    layout_provider::{self as lp, HasLayoutProvider},
    Ctor, Uninit,
};

/// Create a new value of the heap, initializing it in place
pub fn boxed<T, Args>(args: Args) -> Box<T>
where
    T: ?Sized + Ctor<Args> + HasLayoutProvider<Args>,
{
    let layout = lp::layout_of::<T, Args>(&args).expect("Could not extract layout from arguments");
    let is_zeroed = lp::is_zeroed::<T, Args>(&args);

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
        handle_alloc_error(layout)
    };

    // SAFETY: `lp::layout_of` returned a layout
    let ptr = unsafe { lp::cast::<T, Args>(ptr, &args) };

    // SAFETY: if the layout provider says the argument just zeros the memory with no side effects
    // then we can skip initialization
    if !is_zeroed {
        // SAFETY: ptr is a freshly allocated non-null, aligned pointer for `T`
        // because the layout given by `LayoutProvider` is correct
        // and `alloc`/`alloc_zeroed`
        let uninit = unsafe { Uninit::from_raw(ptr.as_ptr()) };

        let init = uninit.init(args);

        // the box will take ownership of the `T`, so we should forget the `Init`
        core::mem::forget(init);
    }

    // SAFETY: ptr points to an initialized, non-null, aligned pointer to T that was allocated
    // using the global allocator
    unsafe { Box::from_raw(ptr.as_ptr()) }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let value = super::boxed::<[u8], _>(crate::slice::CopyArgsLen(10, ()));

        assert_eq!(*value, [0; 10]);
    }
}
