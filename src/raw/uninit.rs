//! `UninitBuffer` represents some uninitialized memory
//!

use core::mem::{align_of, size_of, MaybeUninit};

use super::{Storage, StorageWithCapacity};

#[repr(C)]
struct AlignedBuffer<T, A> {
    align: MaybeUninit<[A; 0]>,
    value: T,
}

/// An uninitialized storage. This storage can store values
/// of any type that has an alignment smaller of equal to `T` or `A`.
///
/// `UninitBuffer` has a max capacity of
/// `round_up(size_of::<T>(), align_::<A>()) / size_of::<Element>()`
/// elements.
///
/// i.e. `UninitBuffer<[i32; 12]>` can store 12 `i32`s, but
/// `UninitBuffer<[i32; 1], u64>` can store 2 `i32`s. Because `u64 is
/// aligned to 8 bytes, so `round_up(4 bytes, 8 bytes) / 4 bytes == 2`
///
/// You can query the capacity with [`UninitBuffer::capacity`]
///
/// ```rust
/// # use generic_vec::raw::UninitBuffer;
/// assert_eq!(UninitBuffer::<[i32; 12]>::capacity::<i32>(), 12);
/// assert_eq!(UninitBuffer::<[i32; 1], u64>::capacity::<i32>(), 2);
/// assert_eq!(UninitBuffer::<[i32; 1]>::capacity::<u64>(), 0);
/// ```
///
/// ## In memory representation
///
/// This type is represented in memory as
///
/// ```
/// #[repr(C)]
/// struct AlignedBuffer<T, A> {
///     align: [A; 0],
///     value: T,
/// }
/// ```
///
/// The size of the buffer is determined by type paramter `T`, and
/// the alignment is the maximum alignment of `T` and `A`. This means
/// that `A` should be used to ensure a certain alignment.
#[repr(transparent)]
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
    /// Get the capacity of this buffer for a given element type
    pub const fn capacity<U>() -> usize { size::<U, T, A>() }

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

unsafe impl<U, T, A> StorageWithCapacity<U> for UninitBuffer<T, A> {
    fn with_capacity(capacity: usize) -> Self {
        let max_capacity = size::<U, T, A>();
        if capacity > max_capacity {
            crate::raw::capacity::fixed_capacity_reserve_error(max_capacity, capacity)
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
            crate::raw::capacity::fixed_capacity_reserve_error(capacity, new_capacity)
        }
    }

    fn try_reserve(&mut self, capacity: usize) -> bool { capacity <= size::<U, T, A>() }
}
