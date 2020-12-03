use crate::raw::{AllocError, Init, Storage, Uninit};

use core::mem::{align_of, size_of, MaybeUninit};

/// An uninitialized slice storage
pub type UninitSlice<'a, T> = Uninit<&'a mut [MaybeUninit<T>]>;
/// An initialized slice storage, can only store `Copy` types
pub type Slice<'a, T> = Init<&'a mut [T]>;

unsafe impl<T> Send for UninitSlice<'_, T> {}
unsafe impl<T> Sync for UninitSlice<'_, T> {}

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

unsafe impl<T: Copy> crate::raw::StorageInit<T> for Slice<'_, T> {}
unsafe impl<T: Copy> Storage<T> for Slice<'_, T> {
    const IS_ALIGNED: bool = true;

    fn capacity(&self) -> usize { self.0.len() }

    fn as_ptr(&self) -> *const T { self.0.as_ptr().cast() }

    fn as_mut_ptr(&mut self) -> *mut T { self.0.as_mut_ptr().cast() }

    fn reserve(&mut self, capacity: usize) {
        if capacity > self.0.len() {
            crate::raw::fixed_capacity_reserve_error(self.0.len(), capacity)
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
