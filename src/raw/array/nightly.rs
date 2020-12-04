use crate::raw::{Storage, StorageWithCapacity};

unsafe impl<T: Copy, const N: usize> crate::raw::StorageInit<T> for [T; N] {}
unsafe impl<T: Default + Copy, const N: usize> StorageWithCapacity<T> for [T; N]
where
    Self: Default,
{
    fn with_capacity(capacity: usize) -> Self {
        if capacity > N {
            crate::raw::capacity::fixed_capacity_reserve_error(N, capacity)
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

unsafe impl<T: Copy, const N: usize> Storage<T> for [T; N] {
    #[doc(hidden)]
    const CONST_CAPACITY: Option<usize> = Some(N);
    const IS_ALIGNED: bool = true;

    fn capacity(&self) -> usize { N }

    fn as_ptr(&self) -> *const T { self[..].as_ptr().cast() }

    fn as_mut_ptr(&mut self) -> *mut T { self[..].as_mut_ptr().cast() }

    fn reserve(&mut self, capacity: usize) {
        if capacity > N {
            crate::raw::capacity::fixed_capacity_reserve_error(N, capacity)
        }
    }

    fn try_reserve(&mut self, capacity: usize) -> bool { capacity <= N }
}
