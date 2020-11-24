use crate::raw::{AllocError, Init, RawVec, Uninit};

use core::mem::MaybeUninit;

pub type UninitSlice<'a, T> = Uninit<&'a mut [MaybeUninit<T>]>;
pub type Slice<'a, T> = Init<&'a mut [T]>;

unsafe impl<T> RawVec for UninitSlice<'_, T> {
    type Item = T;
    type BufferItem = MaybeUninit<T>;

    fn capacity(&self) -> usize {
        self.0.len()
    }

    fn as_ptr(&self) -> *const Self::Item {
        self.0.as_ptr().cast()
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        self.0.as_mut_ptr().cast()
    }

    fn reserve(&mut self, capacity: usize) {
        assert!(
            capacity <= self.0.len(),
            "Cannot allocate more space when using an slice-backed RawVec"
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

unsafe impl<T: Copy> crate::raw::RawVecInit for Slice<'_, T> {}
unsafe impl<T: Copy> RawVec for Slice<'_, T> {
    type Item = T;
    type BufferItem = T;

    fn capacity(&self) -> usize {
        self.0.len()
    }

    fn as_ptr(&self) -> *const Self::Item {
        self.0.as_ptr().cast()
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        self.0.as_mut_ptr().cast()
    }

    fn reserve(&mut self, capacity: usize) {
        assert!(
            capacity <= self.0.len(),
            "Cannot allocate more space when using an slice-backed RawVec"
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
