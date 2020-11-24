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
        unsafe { MaybeUninit::uninit().assume_init() }
    }
}

impl<T: Default + Copy, const N: usize> Default for Array<T, N> {
    fn default() -> Self {
        Self([T::default(); N])
    }
}

impl<T, const N: usize> Clone for UninitArray<T, N> {
    fn clone(&self) -> Self {
        Self::default()
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

    #[inline]
    #[doc(hidden)]
    #[allow(non_snake_case)]
    fn __with_capacity__const_capacity_checked(
        capacity: usize,
        old_capacity: Option<usize>,
    ) -> Self {
        match old_capacity {
            Some(old_capacity) if old_capacity <= N => Self::default(),
            _ => Self::with_capacity(capacity),
        }
    }
}

unsafe impl<T, const N: usize> RawVec for UninitArray<T, N> {
    #[doc(hidden)]
    const CONST_CAPACITY: Option<usize> = Some(N);
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

    #[inline]
    #[doc(hidden)]
    #[allow(non_snake_case)]
    fn __with_capacity__const_capacity_checked(
        capacity: usize,
        old_capacity: Option<usize>,
    ) -> Self {
        match old_capacity {
            Some(old_capacity) if old_capacity <= N => Self::default(),
            _ => Self::with_capacity(capacity),
        }
    }
}

unsafe impl<T: Copy, const N: usize> RawVec for Array<T, N> {
    #[doc(hidden)]
    const CONST_CAPACITY: Option<usize> = Some(N);
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
