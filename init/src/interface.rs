use crate::{Init, Uninit};

/// A type which is constructable using `Args`
pub trait Ctor<Args> {
    /// Initialize a the type `Self` using `args: Args`
    fn init(uninit: Uninit<'_, Self>, args: Args) -> Init<'_, Self>;
}

/// A type which can construct a `T`
pub trait CtorArgs<T: ?Sized> {
    /// Initialize a the type `T` using `self`
    fn init_with(self, uninit: Uninit<'_, T>) -> Init<'_, T>;
}

impl<T: ?Sized, Args: CtorArgs<T>> Ctor<Args> for T {
    #[inline]
    fn init(uninit: Uninit<'_, Self>, args: Args) -> Init<'_, Self> {
        args.init_with(uninit)
    }
}
