use core::mem::{align_of, size_of, MaybeUninit};

use super::{AllocError, Storage, StorageWithCapacity};

#[repr(C)]
struct AlignedBuffer<T, A> {
    align: MaybeUninit<[A; 0]>,
    value: T,
}

/// An uninitialized storage
///
/// This type is represented as, generally the second type parameter can only
/// be used to align the storage
///
/// ```
/// #[repr(C)]
/// struct AlignedBuffer<T, A> {
///     align: [A; 0],
///     value: T,
/// }
/// ```
///
/// By default the alignment type is set to `u8`, so that it doesn't get in the way.
/// But if you need a higher alignment, then you can change the second type.
/// This allows you to do things like `UninitBuffer<[u8; 4], u32>` to get a
/// 4 byte aligned buffer of 4 bytes or `UninitBuffer<[CustomType; 12], usize>` to
/// get an array of `CustomType`, and guarantee it is pointer-aligned.
#[repr(C)]
pub struct UninitBuffer<T, A = u8>(MaybeUninit<AlignedBuffer<T, A>>);

unsafe impl<T, A> Send for UninitBuffer<T, A> {}
unsafe impl<T, A> Sync for UninitBuffer<T, A> {}

const fn size<U, T, A>() -> usize {
    if size_of::<U>() == 0 {
        usize::MAX
    } else {
        size_of::<AlignedBuffer<T, A>>() / size_of::<U>()
    }
}

impl<T, A> UninitBuffer<T, A> {
    /// Create a new uninitialized array storage
    pub const fn uninit() -> Self { Self(MaybeUninit::uninit()) }

    /// Create a new uninitialized array storage with the given value
    pub const fn new(value: T) -> Self {
        Self(MaybeUninit::new(AlignedBuffer {
            align: MaybeUninit::uninit(),
            value,
        }))
    }
}

impl<T, A> Default for UninitBuffer<T, A> {
    fn default() -> Self { Self::uninit() }
}

impl<T, A> Clone for UninitBuffer<T, A> {
    fn clone(&self) -> Self { Self::default() }
}

impl<U, T, A> StorageWithCapacity<U> for UninitBuffer<T, A> {
    fn with_capacity(capacity: usize) -> Self {
        let max_capacity = size::<U, T, A>();
        if capacity > max_capacity {
            crate::raw::fixed_capacity_reserve_error(max_capacity, capacity)
        }

        Self::default()
    }

    #[inline]
    #[doc(hidden)]
    #[allow(non_snake_case)]
    fn __with_capacity__const_capacity_checked(capacity: usize, old_capacity: Option<usize>) -> Self {
        match old_capacity {
            Some(old_capacity) if old_capacity <= size::<U, T, A>() => Self::default(),
            _ => StorageWithCapacity::<U>::with_capacity(capacity),
        }
    }
}

unsafe impl<U, T, A> Storage<U> for UninitBuffer<T, A> {
    #[doc(hidden)]
    const CONST_CAPACITY: Option<usize> = Some(size::<U, T, A>());
    const IS_ALIGNED: bool = align_of::<AlignedBuffer<T, A>>() >= align_of::<U>();

    fn capacity(&self) -> usize { size::<U, T, A>() }

    fn as_ptr(&self) -> *const U { self.0.as_ptr().cast() }

    fn as_mut_ptr(&mut self) -> *mut U { self.0.as_mut_ptr().cast() }

    fn reserve(&mut self, new_capacity: usize) {
        let capacity = size::<U, T, A>();
        if new_capacity > capacity {
            crate::raw::fixed_capacity_reserve_error(capacity, new_capacity)
        }
    }

    fn try_reserve(&mut self, capacity: usize) -> Result<(), AllocError> {
        if capacity <= size::<U, T, A>() {
            Ok(())
        } else {
            Err(AllocError)
        }
    }
}
