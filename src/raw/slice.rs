use crate::raw::{AllocError, Init, Storage, Uninit};

use core::mem::{size_of, MaybeUninit};

/// An uninitialized slice storage
pub type UninitSlice<'a, T> = Uninit<&'a mut [MaybeUninit<T>]>;
/// An initialized slice storage, can only store `Copy` types
pub type Slice<'a, T> = Init<&'a mut [T]>;

unsafe impl<T, U> Storage<U> for UninitSlice<'_, T> {
    fn is_valid_storage() -> bool { crate::raw::is_compatible::<T, U>() }

    fn capacity(&self) -> usize {
        self.0
            .len()
            .checked_mul(size_of::<T>() / size_of::<U>())
            .expect("Overflow calculating capacity")
    }

    fn as_ptr(&self) -> *const U { self.0.as_ptr().cast() }

    fn as_mut_ptr(&mut self) -> *mut U { self.0.as_mut_ptr().cast() }

    fn reserve(&mut self, capacity: usize) {
        assert!(
            capacity <= self.0.len(),
            "Cannot allocate more space when using an slice-backed vector"
        )
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
    fn is_valid_storage() -> bool { true }

    fn capacity(&self) -> usize { self.0.len() }

    fn as_ptr(&self) -> *const T { self.0.as_ptr().cast() }

    fn as_mut_ptr(&mut self) -> *mut T { self.0.as_mut_ptr().cast() }

    fn reserve(&mut self, capacity: usize) {
        assert!(
            capacity <= self.0.len(),
            "Cannot allocate more space when using an slice-backed vector"
        )
    }

    fn try_reserve(&mut self, capacity: usize) -> Result<(), AllocError> {
        if capacity <= self.capacity() {
            Ok(())
        } else {
            Err(AllocError)
        }
    }
}
