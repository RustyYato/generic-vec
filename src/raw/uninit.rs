use crate::raw::{RawVec, RawVecInit};
use core::{alloc::AllocError, mem::MaybeUninit};

#[repr(transparent)]
pub struct Uninit<T: ?Sized>(T);

pub type UninitArray<T, const N: usize> = Uninit<[MaybeUninit<T>; N]>;
pub type UninitSlice<T> = Uninit<[MaybeUninit<T>]>;

impl<T> ConstUninit for T {}
trait ConstUninit: Sized {
    const UNINIT: MaybeUninit<Self> = MaybeUninit::uninit();
}

impl<T, const N: usize> UninitArray<T, N> {
    pub const fn uninit() -> Self {
        Uninit([ConstUninit::UNINIT; N])
    }
}

impl<T, const N: usize> Default for UninitArray<T, N> {
    fn default() -> Self {
        Self::uninit()
    }
}

impl<T, const N: usize> Clone for UninitArray<T, N> {
    fn clone(&self) -> Self {
        Self::uninit()
    }
}

impl<T, const N: usize> RawVecInit for UninitArray<T, N> {
    fn with_capacity(capacity: usize) -> Self {
        assert!(
            capacity <= N,
            "Cannot allocate more than {0} elements when using an UninitArray<T, {0}> RawVec",
            N,
        );

        Self::default()
    }
}

unsafe impl<T, const N: usize> RawVec for UninitArray<T, N> {
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

unsafe impl<T> RawVec for UninitSlice<T> {
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
