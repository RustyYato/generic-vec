use crate::raw::{
    capacity::{capacity, fixed_capacity_reserve_error, Round},
    AllocError, Storage,
};

use core::mem::{align_of, size_of, MaybeUninit};

/// An uninitialized slice storage
#[repr(transparent)]
pub struct UninitSlice<T>([MaybeUninit<T>]);

unsafe impl<T> Send for UninitSlice<T> {}
unsafe impl<T> Sync for UninitSlice<T> {}

#[cfg(not(feature = "nightly"))]
impl<T> UninitSlice<T> {
    /// Create a new `UninitSlice` storage
    pub fn from_mut(buffer: &mut [MaybeUninit<T>]) -> &mut Self { unsafe { &mut *(buffer as *mut [_] as *mut Self) } }

    /// Get the backing value of the this `Uninit` storage
    ///
    /// # Safety
    ///
    /// You may not write uninitialized memory to this slice
    pub unsafe fn to_mut(&mut self) -> &mut [MaybeUninit<T>] { unsafe { &mut *(self as *mut Self as *mut [_]) } }
}

#[cfg(feature = "nightly")]
impl<T> UninitSlice<T> {
    /// Create a new `UninitSlice` storage
    pub const fn from_mut(buffer: &mut [MaybeUninit<T>]) -> &mut Self {
        unsafe { &mut *(buffer as *mut [_] as *mut Self) }
    }

    /// Get the backing value of the this `Uninit` storage
    ///
    /// # Safety
    ///
    /// You may not write uninitialized memory to this slice
    pub const unsafe fn to_mut(&mut self) -> &mut [MaybeUninit<T>] { unsafe { &mut *(self as *mut Self as *mut [_]) } }
}

unsafe impl<T, U> Storage<U> for UninitSlice<T> {
    const IS_ALIGNED: bool = align_of::<T>() >= align_of::<U>();

    fn capacity(&self) -> usize { capacity(self.0.len(), size_of::<T>(), size_of::<U>(), Round::Down) }

    fn as_ptr(&self) -> *const U { self.0.as_ptr().cast() }

    fn as_mut_ptr(&mut self) -> *mut U { self.0.as_mut_ptr().cast() }

    fn reserve(&mut self, new_capacity: usize) {
        let new_capacity = capacity(new_capacity, size_of::<U>(), size_of::<T>(), Round::Up);
        if new_capacity > self.0.len() {
            fixed_capacity_reserve_error(self.0.len(), new_capacity)
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
            fixed_capacity_reserve_error(self.len(), capacity)
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
