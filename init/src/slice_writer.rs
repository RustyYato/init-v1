//! A helper type to incrementally initialize a slice, see [`SliceWriter`] for details

use core::mem::ManuallyDrop;

use crate::{ptr::IterUninit, Ctor, Init, TryCtor, Uninit};

/// A helper type to incrementally initialize a slice
///
/// This type has three parts, a pointer to the start, the total length of the slice (len)
/// and the number of initialized elements (init).  
///
/// This type has the invariant that `init <= len`, and that all elements
/// `0..init` must be initialized.
///
/// This type does not support partially initializing a slice, the slice must
/// be completely initialized or have all previously initialized elements dropped. (modulo leaks)
pub struct SliceWriter<'a, T> {
    len: usize,
    init: usize,
    iter: IterUninit<'a, T>,
}

impl<'a, T> Drop for SliceWriter<'a, T> {
    fn drop(&mut self) {
        // SAFETY:
        // `get_remaining` is only called in `finish` and at `drop`, and it's
        // `self` is leaked in `finish`, which prevents this `drop` from being called
        // so `get_remaining` is called at most once.
        unsafe { self.get_remaining() };
    }
}

impl<'a, T> SliceWriter<'a, T> {
    /// Create a new slice writer to the uninitialized memory
    pub fn new(uninit: Uninit<'a, [T]>) -> Self {
        let len = uninit.len();
        Self {
            iter: uninit.iter(),
            init: 0,
            len,
        }
    }

    /// Returns true iff are all elements in the slice initialized
    pub fn is_complete(&self) -> bool {
        self.remaining_len() == 0
    }

    /// Returns true iff are all elements in the slice initialized
    pub fn remaining_len(&self) -> usize {
        self.iter.len()
    }

    /// Returns true iff any element panicked while initializing
    pub fn is_poisoned(&self) -> bool {
        self.len - self.iter.len() != self.init
    }

    /// Write the next element of the slice (write goes in order, from 0 -> len)
    pub fn init<Args>(&mut self, args: Args)
    where
        T: Ctor<Args>,
    {
        assert!(!self.is_complete());
        // SAFETY: this writer isn't complete
        unsafe { self.init_unchecked(args) }
    }

    /// Write the next element of the slice (write goes in order, from 0 -> len)
    pub fn try_init<Args>(&mut self, args: Args) -> Result<(), T::Error>
    where
        T: TryCtor<Args>,
    {
        assert!(!self.is_complete());
        // SAFETY: this writer isn't complete
        unsafe { self.try_init_unchecked(args) }
    }

    /// Write the next element of the slice (write goes in order, from 0 -> len)
    ///
    /// # Safety
    ///
    /// This writer must not be complete
    pub unsafe fn init_unchecked<Args>(&mut self, args: Args)
    where
        T: Ctor<Args>,
    {
        debug_assert!(!self.is_complete());
        // SAFETY: The caller guarantees that this writer isn't complete,
        // which ensure that the iterator isn't empty
        let init = unsafe { self.iter.next_unchecked() }.init(args);
        // We take ownership of the newly constructed value
        core::mem::forget(init);
        self.init += 1;
    }

    /// Write the next element of the slice (write goes in order, from 0 -> len)
    ///
    /// # Safety
    ///
    /// This writer must not be complete
    pub unsafe fn try_init_unchecked<Args>(&mut self, args: Args) -> Result<(), T::Error>
    where
        T: TryCtor<Args>,
    {
        debug_assert!(!self.is_complete());
        // SAFETY: The caller guarantees that this writer isn't complete,
        // which ensure that the iterator isn't empty
        let init = unsafe { self.iter.next_unchecked() }.try_init(args)?;
        // We take ownership of the newly constructed value
        core::mem::forget(init);
        self.init += 1;
        Ok(())
    }

    /// # Safety
    ///
    /// Must be called at most once per `SliceWriter`
    unsafe fn get_remaining(&mut self) -> Init<'a, [T]> {
        // SAFETY: SliceWriter guarantees that the slice at `self.ptr` has at least `self.init` values initialized

        let remaining = self.iter.remaining();

        // SAFETY:
        // current_ptr - (len - iter.len()) == start of slice for non ZSTs
        // for ZSTs `iter.remaining()` is properly aligned and `sub` is a no-op
        // so this is safe
        let start_ptr = unsafe { remaining.cast::<T>().sub(self.len - self.iter.len()) };

        let slice = core::ptr::slice_from_raw_parts_mut(start_ptr, self.init);

        // SAFETY: This pointer is derived from an `Uninit`, and `get_remaining` is called at most once
        // so it is guaranteed to be unique, non-null, aligned, and dereferencable
        // The `SliceWriter` also guarantees that `init` will alway count the number of initialized
        // elements in the slice, so every element of `slice` is initialized
        unsafe { Init::from_raw(slice) }
    }

    /// Write the next element of the slice (write goes in order, from 0 -> len)
    pub fn finish(self) -> Init<'a, [T]> {
        self.try_finish().unwrap_or_else(|| incomplete_error())
    }

    /// Write the next element of the slice (write goes in order, from 0 -> len)
    pub fn try_finish(self) -> Option<Init<'a, [T]>> {
        if self.is_complete() {
            // SAFETY: This writer is complete
            Some(unsafe { self.finish_unchecked() })
        } else {
            None
        }
    }
    /// Write the next element of the slice (write goes in order, from 0 -> len)
    ///
    /// # Safety
    ///
    /// This writer must be complete
    pub unsafe fn finish_unchecked(self) -> Init<'a, [T]> {
        if !self.is_complete() {
            // SAFETY: caller guarantees that writer is complete
            //
            // This allows the poisoned check to be elided if LLVM can guarantee there were no panics
            unsafe { core::hint::unreachable_unchecked() }
        }

        if self.is_poisoned() {
            poisoned_error()
        }

        // SAFETY:
        // `get_remaining` is only called here and at `drop`, and it's
        // unsound to call any function after calling drop, so it could not have been called yet
        // and self is leaked, so Self::drop isn't called, so `get_remaining` is called
        // at most once for this `SliceWriter`
        unsafe { ManuallyDrop::new(self).get_remaining() }
    }
}

#[cold]
#[inline(never)]
fn poisoned_error() -> ! {
    panic!("Tried to finish an incomplete writer")
}

#[cold]
#[inline(never)]
fn incomplete_error() -> ! {
    panic!("Tried to finish a poisoned writer")
}
