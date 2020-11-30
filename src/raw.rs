//! The raw vector typse that back-up the [`GenericVec`](crate::GenericVec)

#[cfg(feature = "nightly")]
pub use core::alloc::AllocError;
#[cfg(feature = "alloc")]
use std::boxed::Box;

/// The `AllocError` error indicates an allocation failure
/// that may be due to resource exhaustion or to
/// something wrong when combining the given input arguments with this
/// allocator.
#[cfg(not(feature = "nightly"))]
pub struct AllocError;

#[cfg(feature = "nightly")]
mod array;
#[cfg(feature = "alloc")]
mod heap;
mod slice;

#[cfg(feature = "nightly")]
pub use array::{Array, UninitArray};
#[cfg(feature = "alloc")]
pub use heap::Heap;

pub use slice::{Slice, UninitSlice};

/// A slice or array storage that contains initialized `Copy` types
#[repr(transparent)]
pub struct Init<T: ?Sized>(pub T);
/// A slice or array storage that contains uninitialized data
#[repr(transparent)]
pub struct Uninit<T: ?Sized>(pub T);

/// A [`Storage`] that can only contain initialized `Storage::Item`
pub unsafe trait StorageInit: Storage {}

/// A type that can hold `Self::Item`s, and potentially
/// reserve space for more `Self::Items`s
pub unsafe trait Storage {
    #[doc(hidden)]
    const CONST_CAPACITY: Option<usize> = None;

    /// The type that this storage can hold
    type Item;
    /// The type that this storage uses to represent `Self::Item`
    type BufferItem;

    /// The number of elements that it is valid to write to this `Storage`
    ///
    /// i.e. `as_mut_ptr()..as_mut_ptr() + capacity()` should be valid to write
    /// `Self::Item`s
    fn capacity(&self) -> usize;

    /// Returns a pointer to the first element
    fn as_ptr(&self) -> *const Self::Item;

    /// Returns a mutable pointer to the first element
    fn as_mut_ptr(&mut self) -> *mut Self::Item;

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
    fn try_reserve(&mut self, new_capacity: usize) -> Result<(), AllocError>;
}

/// A storage that can be initially created with a given capacity
pub trait StorageWithCapacity: Storage + Default {
    /// Creates a new storage with at least the given storage capacity
    fn with_capacity(capacity: usize) -> Self;

    #[doc(hidden)]
    #[inline(always)]
    #[allow(non_snake_case)]
    fn __with_capacity__const_capacity_checked(capacity: usize, _old_capacity: Option<usize>) -> Self {
        Self::with_capacity(capacity)
    }
}

unsafe impl<T: ?Sized + StorageInit> StorageInit for &mut T {}
unsafe impl<T: ?Sized + Storage> Storage for &mut T {
    #[doc(hidden)]
    const CONST_CAPACITY: Option<usize> = T::CONST_CAPACITY;
    type Item = T::Item;
    type BufferItem = T::BufferItem;

    fn capacity(&self) -> usize { T::capacity(self) }
    fn as_ptr(&self) -> *const Self::Item { T::as_ptr(self) }
    fn as_mut_ptr(&mut self) -> *mut Self::Item { T::as_mut_ptr(self) }
    fn reserve(&mut self, new_capacity: usize) { T::reserve(self, new_capacity) }
    fn try_reserve(&mut self, new_capacity: usize) -> Result<(), AllocError> { T::try_reserve(self, new_capacity) }
}

#[cfg(feature = "alloc")]
unsafe impl<T: ?Sized + StorageInit> StorageInit for Box<T> {}
#[cfg(feature = "alloc")]
unsafe impl<T: ?Sized + Storage> Storage for Box<T> {
    #[doc(hidden)]
    const CONST_CAPACITY: Option<usize> = T::CONST_CAPACITY;
    type Item = T::Item;
    type BufferItem = T::BufferItem;

    fn capacity(&self) -> usize { T::capacity(self) }
    fn as_ptr(&self) -> *const Self::Item { T::as_ptr(self) }
    fn as_mut_ptr(&mut self) -> *mut Self::Item { T::as_mut_ptr(self) }
    fn reserve(&mut self, new_capacity: usize) { T::reserve(self, new_capacity) }
    fn try_reserve(&mut self, new_capacity: usize) -> Result<(), AllocError> { T::try_reserve(self, new_capacity) }
}

#[cfg(feature = "alloc")]
impl<T: ?Sized + StorageWithCapacity> StorageWithCapacity for Box<T> {
    fn with_capacity(capacity: usize) -> Self { Box::new(T::with_capacity(capacity)) }

    #[doc(hidden)]
    #[inline(always)]
    #[allow(non_snake_case)]
    fn __with_capacity__const_capacity_checked(capacity: usize, _old_capacity: Option<usize>) -> Self {
        Box::new(T::__with_capacity__const_capacity_checked(capacity, _old_capacity))
    }
}
