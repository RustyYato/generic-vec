//! The raw vector typse that back-up the [`GenericVec`](crate::GenericVec)

#[cfg(feature = "alloc")]
use std::boxed::Box;

mod array;
#[cfg(feature = "alloc")]
mod heap;
mod slice;
mod uninit;
mod zero_sized;

mod capacity;

#[cfg(feature = "alloc")]
pub use heap::Heap;

pub use slice::UninitSlice;
pub use uninit::UninitBuffer;
pub use zero_sized::ZeroSized;

/// A [`Storage`] that can only contain initialized `Storage::Item`
pub unsafe trait StorageInit<T>: Storage<T> {}

/// A type that can hold `T`s, and potentially
/// reserve space for more `Self::Items`s
pub unsafe trait Storage<T> {
    #[doc(hidden)]
    const CONST_CAPACITY: Option<usize> = None;

    /// Is the pointer from `as_ptr` guaranteed to be aligned to `T`
    ///
    /// Ideally this would be a `where` clause to prevent alignment issues
    /// at compile time, but that can't happen until const-generics allows
    /// predicates in where bounds (like `where align_of::<T>() >= align_of::<U>()`)
    const IS_ALIGNED: bool;

    /// The number of elements that it is valid to write to this `Storage`
    ///
    /// i.e. `as_mut_ptr()..as_mut_ptr() + capacity()` should be valid to write
    /// `T`s
    fn capacity(&self) -> usize;

    /// Returns a pointer to the first element
    fn as_ptr(&self) -> *const T;

    /// Returns a mutable pointer to the first element
    fn as_mut_ptr(&mut self) -> *mut T;

    /// Reserves space for at least `new_capacity` elements
    ///
    /// # Safety
    ///
    /// After this call successfully ends, the `capacity` must be at least
    /// `new_capacity`
    ///
    /// # Panic/Abort
    ///
    /// Maybe panic or abort if it is impossible to set the `capacity` to at
    /// least `new_capacity`
    fn reserve(&mut self, new_capacity: usize);

    /// Tries to reserve space for at least `new_capacity` elements
    ///
    /// Returns `Ok(())` on success, `Err(AllocError)` if it is impossible to
    /// set the `capacity` to at least `new_capacity`
    ///
    /// # Safety
    ///
    /// If `Ok(())` is returned, the `capacity` must be at least `new_capacity`
    fn try_reserve(&mut self, new_capacity: usize) -> bool;
}

/// A storage that can be initially created with a given capacity
///
/// # Safety
///
/// The storage must have a capacity of at least `capacity` after
/// `StorageWithCapacity::with_capacity` is called.
pub unsafe trait StorageWithCapacity<T>: Storage<T> + Default {
    /// Creates a new storage with at least the given storage capacity
    fn with_capacity(capacity: usize) -> Self;

    #[doc(hidden)]
    #[inline(always)]
    #[allow(non_snake_case)]
    fn __with_capacity__const_capacity_checked(capacity: usize, _old_capacity: Option<usize>) -> Self {
        Self::with_capacity(capacity)
    }
}

unsafe impl<T, S: ?Sized + StorageInit<T>> StorageInit<T> for &mut S {}
unsafe impl<T, S: ?Sized + Storage<T>> Storage<T> for &mut S {
    #[doc(hidden)]
    const CONST_CAPACITY: Option<usize> = S::CONST_CAPACITY;
    const IS_ALIGNED: bool = S::IS_ALIGNED;
    #[inline]
    fn capacity(&self) -> usize { S::capacity(self) }
    #[inline]
    fn as_ptr(&self) -> *const T { S::as_ptr(self) }
    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T { S::as_mut_ptr(self) }
    #[inline]
    fn reserve(&mut self, new_capacity: usize) { S::reserve(self, new_capacity) }
    #[inline]
    fn try_reserve(&mut self, new_capacity: usize) -> bool { S::try_reserve(self, new_capacity) }
}

#[cfg(feature = "alloc")]
unsafe impl<T, S: ?Sized + StorageInit<T>> StorageInit<T> for Box<S> {}
#[cfg(feature = "alloc")]
unsafe impl<T, S: ?Sized + Storage<T>> Storage<T> for Box<S> {
    #[doc(hidden)]
    const CONST_CAPACITY: Option<usize> = S::CONST_CAPACITY;
    const IS_ALIGNED: bool = S::IS_ALIGNED;

    #[inline]
    fn capacity(&self) -> usize { S::capacity(self) }
    #[inline]
    fn as_ptr(&self) -> *const T { S::as_ptr(self) }
    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T { S::as_mut_ptr(self) }
    #[inline]
    fn reserve(&mut self, new_capacity: usize) { S::reserve(self, new_capacity) }
    #[inline]
    fn try_reserve(&mut self, new_capacity: usize) -> bool { S::try_reserve(self, new_capacity) }
}

#[cfg(feature = "alloc")]
unsafe impl<T, S: ?Sized + StorageWithCapacity<T>> StorageWithCapacity<T> for Box<S> {
    #[inline(always)]
    fn with_capacity(capacity: usize) -> Self { Box::new(S::with_capacity(capacity)) }

    #[doc(hidden)]
    #[inline(always)]
    #[allow(non_snake_case)]
    fn __with_capacity__const_capacity_checked(capacity: usize, _old_capacity: Option<usize>) -> Self {
        Box::new(S::__with_capacity__const_capacity_checked(capacity, _old_capacity))
    }
}
