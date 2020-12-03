use crate::raw::{AllocError, Init, Storage, StorageWithCapacity, Uninit};
use core::mem::{align_of, size_of, MaybeUninit};

/// An uninitialized array storage
pub type UninitArray<T, const N: usize> = Uninit<MaybeUninit<[T; N]>>;
/// An initialized array storage
pub type Array<T, const N: usize> = Init<[T; N]>;

unsafe impl<T, const N: usize> Send for UninitArray<T, N> {}
unsafe impl<T, const N: usize> Sync for UninitArray<T, N> {}

impl<T, const N: usize> UninitArray<T, N> {
    /// Create a new uninitialized array storage
    pub const fn uninit() -> Self { Self(MaybeUninit::uninit()) }

    /// Create a new uninitialized array storage, with the given array
    pub const fn with_array(array: [T; N]) -> Self { Self(MaybeUninit::new(array)) }
}

impl<T, const N: usize> Array<T, N> {
    /// Create a new initialized array storage, with the given array
    pub const fn new(array: [T; N]) -> Self { Self(array) }
}

impl<T, const N: usize> Default for UninitArray<T, N> {
    fn default() -> Self { Self::uninit() }
}

impl<T: Default + Copy, const N: usize> Default for Array<T, N> {
    fn default() -> Self { Self([T::default(); N]) }
}

impl<T, const N: usize> Clone for UninitArray<T, N> {
    fn clone(&self) -> Self { Self::default() }
}

impl<T: Copy, const N: usize> Copy for Array<T, N> {}
impl<T: Copy, const N: usize> Clone for Array<T, N> {
    fn clone(&self) -> Self { *self }
}

impl<U, T, const N: usize> StorageWithCapacity<U> for UninitArray<T, N> {
    fn with_capacity(capacity: usize) -> Self {
        if capacity > N {
            crate::raw::fixed_capacity_reserve_error(N, capacity)
        }

        Self::default()
    }

    #[inline]
    #[doc(hidden)]
    #[allow(non_snake_case)]
    fn __with_capacity__const_capacity_checked(capacity: usize, old_capacity: Option<usize>) -> Self {
        match old_capacity {
            Some(old_capacity) if old_capacity <= N => Self::default(),
            _ => StorageWithCapacity::<U>::with_capacity(capacity),
        }
    }
}

unsafe impl<U, T, const N: usize> Storage<U> for UninitArray<T, N> {
    #[doc(hidden)]
    const CONST_CAPACITY: Option<usize> = Some(crate::raw::capacity(N, size_of::<T>(), size_of::<U>()));
    const IS_ALIGNED: bool = align_of::<T>() >= align_of::<U>();

    fn capacity(&self) -> usize { <Self as Storage<U>>::CONST_CAPACITY.unwrap() }

    fn as_ptr(&self) -> *const U { self.0.as_ptr().cast() }

    fn as_mut_ptr(&mut self) -> *mut U { self.0.as_mut_ptr().cast() }

    fn reserve(&mut self, new_capacity: usize) {
        let new_capacity = crate::raw::capacity(new_capacity, size_of::<U>(), size_of::<T>());
        if new_capacity > N {
            crate::raw::fixed_capacity_reserve_error(N, new_capacity)
        }
    }

    fn try_reserve(&mut self, capacity: usize) -> Result<(), AllocError> {
        if capacity <= N {
            Ok(())
        } else {
            Err(AllocError)
        }
    }
}

unsafe impl<T: Copy, const N: usize> crate::raw::StorageInit<T> for Array<T, N> {}
impl<T: Default + Copy, const N: usize> StorageWithCapacity<T> for Array<T, N> {
    fn with_capacity(capacity: usize) -> Self {
        if capacity > N {
            crate::raw::fixed_capacity_reserve_error(N, capacity)
        }

        Self::default()
    }

    #[inline]
    #[doc(hidden)]
    #[allow(non_snake_case)]
    fn __with_capacity__const_capacity_checked(capacity: usize, old_capacity: Option<usize>) -> Self {
        match old_capacity {
            Some(old_capacity) if old_capacity <= N => Self::default(),
            _ => Self::with_capacity(capacity),
        }
    }
}

unsafe impl<T: Copy, const N: usize> Storage<T> for Array<T, N> {
    #[doc(hidden)]
    const CONST_CAPACITY: Option<usize> = Some(N);
    const IS_ALIGNED: bool = true;

    fn capacity(&self) -> usize { N }

    fn as_ptr(&self) -> *const T { self.0.as_ptr().cast() }

    fn as_mut_ptr(&mut self) -> *mut T { self.0.as_mut_ptr().cast() }

    fn reserve(&mut self, capacity: usize) {
        if capacity > N {
            crate::raw::fixed_capacity_reserve_error(N, capacity)
        }
    }

    fn try_reserve(&mut self, capacity: usize) -> Result<(), AllocError> {
        if capacity <= N {
            Ok(())
        } else {
            Err(AllocError)
        }
    }
}
