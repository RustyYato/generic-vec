use crate::raw::{RawVec, RawVecInit};
use core::alloc::AllocError;

#[repr(transparent)]
pub struct Init<T: ?Sized>(T);

pub type Array<T, const N: usize> = Init<[T; N]>;
pub type Slice<T> = Init<[T]>;

impl<T: Default + Copy, const N: usize> Default for Array<T, N> {
    fn default() -> Self {
        Self([T::default(); N])
    }
}

impl<T: Default + Copy, const N: usize> RawVecInit for Array<T, N> {
    fn with_capacity(capacity: usize) -> Self {
        assert!(
            capacity <= N,
            "Cannot allocate more than {0} elements when using an UninitArray<T, {0}> RawVec",
            N,
        );

        Self::default()
    }
}

unsafe impl<T: Copy, const N: usize> RawVec for Array<T, N> {
    type Item = T;

    fn capacity(&self) -> usize {
        N
    }

    fn as_ptr(&self) -> *const Self::Item {
        self.0.as_ptr().cast()
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        self.0.as_mut_ptr().cast()
    }

    fn reserve(&mut self, capacity: usize) {
        assert!(
            capacity <= N,
            "Cannot allocate more space when using an Array RawVec"
        )
    }

    fn try_reserve(&mut self, capacity: usize) -> Result<(), AllocError> {
        if capacity <= N {
            Ok(())
        } else {
            Err(AllocError)
        }
    }
}

unsafe impl<T: Copy> RawVec for Slice<T> {
    type Item = T;

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
            "Cannot allocate more space when using an Array RawVec"
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
