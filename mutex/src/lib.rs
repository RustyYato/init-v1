#![feature(intrinsics, core_intrinsics)]

use std::{
    alloc::Layout,
    cell::{Cell, UnsafeCell},
    marker::{PhantomData, PhantomPinned},
    ops::{Add, Deref},
    pin::Pin,
};

use init::{
    layout_provider::{HasLayoutProvider, LayoutProvider},
    PinCtor, PinInit,
};
use libc::{
    pthread_mutex_destroy, pthread_mutex_init, pthread_mutex_lock, pthread_mutex_t,
    pthread_mutex_trylock, pthread_mutex_unlock, pthread_mutexattr_destroy, pthread_mutexattr_init,
    pthread_mutexattr_t,
};

#[repr(transparent)]
struct PThreadMutexAttr {
    value: pthread_mutexattr_t,
    _unpin: PhantomPinned,
}

impl Drop for PThreadMutexAttr {
    fn drop(&mut self) {
        unsafe { pthread_mutexattr_destroy(&mut self.value) };
    }
}

#[repr(transparent)]
pub struct PThreadMutex {
    lock: UnsafeCell<pthread_mutex_t>,
    _unpin: PhantomPinned,
}

unsafe impl Send for PThreadMutex {}
unsafe impl Sync for PThreadMutex {}

impl Drop for PThreadMutex {
    fn drop(&mut self) {
        unsafe { pthread_mutex_destroy(self.lock.get()) };
    }
}

impl PinCtor for PThreadMutexAttr {
    fn pin_init(mut uninit: init::Uninit<'_, Self>, (): ()) -> PinInit<'_, Self> {
        let ptr = uninit.as_mut_ptr().cast();
        let err = unsafe { pthread_mutexattr_init(ptr) };
        assert_eq!(err, 0);
        unsafe { uninit.assume_init().pin() }
    }
}

impl init::layout_provider::HasLayoutProvider for PThreadMutexAttr {
    type LayoutProvider = init::layout_provider::SizedLayoutProvider;
}

impl PinCtor for PThreadMutex {
    fn pin_init(mut uninit: init::Uninit<'_, Self>, (): ()) -> PinInit<'_, Self> {
        init::stack_pin_init((), |attr: Pin<&mut PThreadMutexAttr>| {
            unsafe { pthread_mutex_init(uninit.as_mut_ptr().cast(), &attr.value) };
            unsafe { uninit.assume_init().pin() }
        })
    }
}

impl init::layout_provider::HasLayoutProvider for PThreadMutex {
    type LayoutProvider = init::layout_provider::SizedLayoutProvider;
}

impl PThreadMutex {
    pub const fn new() -> Self {
        Self {
            lock: UnsafeCell::new(libc::PTHREAD_MUTEX_INITIALIZER),
            _unpin: PhantomPinned,
        }
    }

    pub fn lock(self: Pin<&Self>) {
        let x = &self.lock;
        let val = unsafe { pthread_mutex_lock(x.get()) };
        assert_eq!(val, 0);
    }

    pub fn try_lock(self: Pin<&Self>) -> bool {
        let x = &self.lock;
        0 == unsafe { pthread_mutex_trylock(x.get()) }
    }

    /// # Safety
    ///
    /// The mutex must be currently locked
    pub unsafe fn force_unlock(self: Pin<&Self>) {
        let x = &self.lock;
        let val = unsafe { pthread_mutex_unlock(x.get()) };
        assert_eq!(val, 0);
    }
}

#[repr(C)]
pub struct Mutex<T: ?Sized> {
    lock: PThreadMutex,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T: ?Sized> {
    mutex: Pin<&'a Mutex<T>>,
    _not_send: PhantomData<&'static Cell<()>>,
}

unsafe impl<T: Sync> Sync for MutexGuard<'_, T> {}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            lock: PThreadMutex::new(),
            value: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    pub fn as_lock(self: Pin<&Self>) -> Pin<&PThreadMutex> {
        let this = unsafe { Pin::into_inner_unchecked(self) };
        unsafe { Pin::new_unchecked(&this.lock) }
    }

    pub fn lock(self: Pin<&Self>) -> MutexGuard<'_, T> {
        self.as_lock().lock();
        MutexGuard {
            mutex: self,
            _not_send: PhantomData,
        }
    }

    pub fn try_lock(self: Pin<&Self>) -> Option<MutexGuard<'_, T>> {
        if self.as_lock().try_lock() {
            Some(MutexGuard {
                mutex: self,
                _not_send: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<T: ?Sized> MutexGuard<'_, T> {
    pub fn as_ref(&self) -> Pin<&T> {
        unsafe { Pin::new_unchecked(&*self.mutex.value.get()) }
    }

    pub fn as_mut(&mut self) -> Pin<&mut T> {
        unsafe { Pin::new_unchecked(&mut *self.mutex.value.get()) }
    }
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { self.mutex.as_lock().force_unlock() }
    }
}

pub struct NewMutex<A>(pub A);

impl<T: ?Sized + PinCtor> PinCtor for Mutex<T> {
    fn pin_init(uninit: init::Uninit<'_, Self>, (): ()) -> init::PinInit<'_, Self> {
        init::pin_init_struct! {
            uninit => Self {
                lock: (),
                value: init::ext::NewUnsafeCell(())
            }
        }
    }
}

impl<T: ?Sized + PinCtor<A>, A> PinCtor<NewMutex<A>> for Mutex<T> {
    fn pin_init(
        uninit: init::Uninit<'_, Self>,
        NewMutex(args): NewMutex<A>,
    ) -> init::PinInit<'_, Self> {
        init::pin_init_struct! {
            uninit => Self {
                lock: (),
                value: init::ext::NewUnsafeCell(args)
            }
        }
    }
}

pub struct MutexLayoutProvider;

impl<T: ?Sized + HasLayoutProvider<A>, A> HasLayoutProvider<NewMutex<A>> for Mutex<T> {
    type LayoutProvider = MutexLayoutProvider;
}

impl<T: ?Sized + HasLayoutProvider> HasLayoutProvider for Mutex<T> {
    type LayoutProvider = MutexLayoutProvider;
}

unsafe impl<T: ?Sized + HasLayoutProvider<A>, A> LayoutProvider<Mutex<T>, NewMutex<A>>
    for MutexLayoutProvider
{
    fn layout_of(args: &NewMutex<A>) -> Option<std::alloc::Layout> {
        let lock = Layout::new::<PThreadMutex>();
        let value = init::layout_provider::layout_of::<T, A>(&args.0)?;
        Some(lock.extend(value).ok()?.0)
    }

    unsafe fn cast(ptr: std::ptr::NonNull<u8>, args: &NewMutex<A>) -> std::ptr::NonNull<Mutex<T>> {
        unsafe {
            let ptr = init::layout_provider::cast::<T, A>(ptr, &args.0);
            core::ptr::NonNull::new_unchecked(ptr.as_ptr() as *mut Mutex<T>)
        }
    }
}

unsafe impl<T: ?Sized + HasLayoutProvider> LayoutProvider<Mutex<T>> for MutexLayoutProvider {
    fn layout_of((): &()) -> Option<std::alloc::Layout> {
        let lock = Layout::new::<PThreadMutex>();
        let value = init::layout_provider::layout_of::<T, ()>(&())?;
        Some(lock.extend(value).ok()?.0.pad_to_align())
    }

    unsafe fn cast(ptr: std::ptr::NonNull<u8>, (): &()) -> std::ptr::NonNull<Mutex<T>> {
        unsafe {
            let ptr = init::layout_provider::cast::<T, ()>(ptr, &());
            core::ptr::NonNull::new_unchecked(ptr.as_ptr() as *mut Mutex<T>)
        }
    }
}

impl<T: ?Sized + core::fmt::Debug> core::fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        T::fmt(self, f)
    }
}

#[test]
fn mutex() {
    let mutex = init::pin_boxed::pin_boxed::<Mutex<i32>, _>(());
    let _lock = mutex.as_ref().lock();
    assert!(mutex.as_ref().try_lock().is_none());
    drop(_lock);
    let _lock = mutex.as_ref().lock();
}
