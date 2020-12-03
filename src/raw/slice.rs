use crate::raw::{AllocError, Storage};

use core::mem::{align_of, size_of, MaybeUninit};

/// An uninitialized slice storage
#[repr(transparent)]
pub struct UninitSlice<'a, T>(&'a mut [MaybeUninit<T>]);

unsafe impl<T> Send for UninitSlice<'_, T> {}
unsafe impl<T> Sync for UninitSlice<'_, T> {}

#[cfg(not(feature = "nightly"))]
impl<'a, T> UninitSlice<'a, T> {
    /// Create a new `UninitSlice` storage
    pub fn new(buffer: &'a mut [MaybeUninit<T>]) -> Self { Self(buffer) }

    /// Reborrow an `UninitSlice` storage
    pub fn as_ref(&mut self) -> UninitSlice<'_, T> { UninitSlice(self.0) }

    /// Get the backing value of the this `Uninit` storage
    ///
    /// # Safety
    ///
    /// You may not write uninitialized memory to this slice
    pub unsafe fn into_inner(self) -> &'a mut [MaybeUninit<T>] { self.0 }
}

#[cfg(feature = "nightly")]
impl<'a, T> UninitSlice<'a, T> {
    /// Create a new `UninitSlice` storage
    pub const fn new(buffer: &'a mut [MaybeUninit<T>]) -> Self { Self(buffer) }

    /// Reborrow an `UninitSlice` storage
    pub const fn as_ref(&mut self) -> UninitSlice<'_, T> { UninitSlice(self.0) }

    /// Get the backing value of the this `Uninit` storage
    ///
    /// # Safety
    ///
    /// You may not write uninitialized memory to this slice
    pub const unsafe fn into_inner(self) -> &'a mut [MaybeUninit<T>] { self.0 }
}

unsafe impl<T, U> Storage<U> for UninitSlice<'_, T> {
    const IS_ALIGNED: bool = align_of::<T>() >= align_of::<U>();

    fn capacity(&self) -> usize { crate::raw::capacity(self.0.len(), size_of::<T>(), size_of::<U>()) }

    fn as_ptr(&self) -> *const U { self.0.as_ptr().cast() }

    fn as_mut_ptr(&mut self) -> *mut U { self.0.as_mut_ptr().cast() }

    fn reserve(&mut self, new_capacity: usize) {
        let new_capacity = crate::raw::capacity(new_capacity, size_of::<U>(), size_of::<T>());
        if new_capacity > self.0.len() {
            crate::raw::fixed_capacity_reserve_error(self.0.len(), new_capacity)
        }
    }

    fn try_reserve(&mut self, capacity: usize) -> Result<(), AllocError> {
        if capacity <= Storage::<U>::capacity(self) {
            Ok(())
        } else {
            Err(AllocError)
        }
    }
}

unsafe impl<T: Copy> crate::raw::StorageInit<T> for [T] {}
unsafe impl<T: Copy> Storage<T> for [T] {
    const IS_ALIGNED: bool = true;

    fn capacity(&self) -> usize { self.len() }

    fn as_ptr(&self) -> *const T { self.as_ptr().cast() }

    fn as_mut_ptr(&mut self) -> *mut T { self.as_mut_ptr().cast() }

    fn reserve(&mut self, capacity: usize) {
        if capacity > self.len() {
            crate::raw::fixed_capacity_reserve_error(self.len(), capacity)
        }
    }

    fn try_reserve(&mut self, capacity: usize) -> Result<(), AllocError> {
        if capacity <= self.capacity() {
            Ok(())
        } else {
            Err(AllocError)
        }
    }
}
