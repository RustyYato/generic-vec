use crate::raw::{Init, RawVec, RawVecInit, Uninit};
use core::{alloc::AllocError, mem::MaybeUninit};

pub type UninitArray<T, const N: usize> = Uninit<[MaybeUninit<T>; N]>;

pub type Array<T, const N: usize> = Init<[T; N]>;

impl<T, const N: usize> UninitArray<T, N> {
    pub const fn uninit() -> Self {
        impl<T> ConstUninit for T {}
        trait ConstUninit: Sized {
            const UNINIT: MaybeUninit<Self> = MaybeUninit::uninit();
        }

        Uninit([ConstUninit::UNINIT; N])
    }
}

impl<T, const N: usize> Default for UninitArray<T, N> {
    fn default() -> Self {
        Self::uninit()
    }
}

impl<T: Default + Copy, const N: usize> Default for Array<T, N> {
    fn default() -> Self {
        Self([T::default(); N])
    }
}

impl<T, const N: usize> Clone for UninitArray<T, N> {
    fn clone(&self) -> Self {
        Self::uninit()
    }
}

impl<T: Copy, const N: usize> Copy for Array<T, N> {}
impl<T: Copy, const N: usize> Clone for Array<T, N> {
    fn clone(&self) -> Self {
        *self
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
            "Cannot allocate more space when using an array-backed RawVec"
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
            "Cannot allocate more space when using an array-backed RawVec"
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
