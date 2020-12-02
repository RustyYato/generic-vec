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

#[cfg(any(doc, feature = "nightly"))]
mod array;
#[cfg(any(doc, feature = "alloc"))]
mod heap;
mod slice;
mod zero_sized;

#[cfg(any(doc, feature = "nightly"))]
pub use array::{Array, UninitArray};
#[cfg(any(doc, feature = "alloc"))]
pub use heap::Heap;

pub use slice::{Slice, UninitSlice};
pub use zero_sized::ZeroSized;

/// A slice or array storage that contains initialized `Copy` types
#[repr(transparent)]
pub struct Init<T: ?Sized>(pub T);
/// A slice or array storage that contains uninitialized data
#[repr(transparent)]
pub struct Uninit<T: ?Sized>(pub(crate) T);

#[cold]
#[inline(never)]
#[cfg(feature = "nightly")]
const fn capacity_calculation_overflow() -> ! { panic!("Tried to calculate the current capacity, but overflowed") }

#[cold]
#[inline(never)]
fn fixed_capacity_reserve_error(capacity: usize, new_capacity: usize) -> ! {
    panic!(
        "Tried to reserve {}, but used a fixed capacity storage of {}",
        new_capacity, capacity
    )
}

#[cfg(not(any(
    target_pointer_width = "8",
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
)))]
compile_error!("Cannot correctly calculate capacity on an 128-bit or larger architecture");

const fn capacity(old_capacity: usize, size_self: usize, size_other: usize) -> usize {
    #[cfg(target_pointer_width = "8")]
    type PointerNext = u16;
    #[cfg(target_pointer_width = "16")]
    type PointerNext = u32;
    #[cfg(target_pointer_width = "32")]
    type PointerNext = u64;
    #[cfg(target_pointer_width = "64")]
    type PointerNext = u128;

    let size = (old_capacity as PointerNext) * (size_self as PointerNext) / (size_other as PointerNext);

    #[cfg(not(feature = "nightly"))]
    {
        [size as usize][(size > usize::MAX as PointerNext) as usize]
    }

    #[cfg(feature = "nightly")]
    {
        if size > usize::MAX as PointerNext {
            capacity_calculation_overflow()
        }

        size as usize
    }
}

impl<T> Uninit<T> {
    /// Create a new `Uninit` storage
    pub fn new(value: T) -> Self { Self(value) }

    /// Get the backing value of the this `Uninit` storage
    ///
    /// # Safety
    ///
    /// This `Uninit` storage must be backed by a potentially
    /// uninitialized source.
    pub unsafe fn into_inner(self) -> T { self.0 }
}

/// A [`Storage`] that can only contain initialized `Storage::Item`
pub unsafe trait StorageInit<T>: Storage<T> {}

/// Check if type `U` smaller than `T` and less aligned than `T`
pub const fn is_compatible<T, U>() -> bool {
    use core::mem::{align_of, size_of};

    size_of::<T>() >= size_of::<U>() && align_of::<T>() >= align_of::<U>()
}

/// Check if type `U` is layout identical to `T`
pub const fn is_identical<T, U>() -> bool {
    use core::mem::{align_of, size_of};

    size_of::<T>() == size_of::<U>() && align_of::<T>() == align_of::<U>()
}

/// A type that can hold `T`s, and potentially
/// reserve space for more `Self::Items`s
pub unsafe trait Storage<T> {
    #[doc(hidden)]
    const CONST_CAPACITY: Option<usize> = None;

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
    fn try_reserve(&mut self, new_capacity: usize) -> Result<(), AllocError>;
}

/// A storage that can be initially created with a given capacity
pub trait StorageWithCapacity<T>: Storage<T> + Default {
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

    #[inline]
    fn capacity(&self) -> usize { S::capacity(self) }
    #[inline]
    fn as_ptr(&self) -> *const T { S::as_ptr(self) }
    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T { S::as_mut_ptr(self) }
    #[inline]
    fn reserve(&mut self, new_capacity: usize) { S::reserve(self, new_capacity) }
    #[inline]
    fn try_reserve(&mut self, new_capacity: usize) -> Result<(), AllocError> { S::try_reserve(self, new_capacity) }
}

#[cfg(feature = "alloc")]
unsafe impl<T, S: ?Sized + StorageInit<T>> StorageInit<T> for Box<S> {}
#[cfg(feature = "alloc")]
unsafe impl<T, S: ?Sized + Storage<T>> Storage<T> for Box<S> {
    #[doc(hidden)]
    const CONST_CAPACITY: Option<usize> = S::CONST_CAPACITY;

    #[inline]
    fn capacity(&self) -> usize { S::capacity(self) }
    #[inline]
    fn as_ptr(&self) -> *const T { S::as_ptr(self) }
    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T { S::as_mut_ptr(self) }
    #[inline]
    fn reserve(&mut self, new_capacity: usize) { S::reserve(self, new_capacity) }
    #[inline]
    fn try_reserve(&mut self, new_capacity: usize) -> Result<(), AllocError> { S::try_reserve(self, new_capacity) }
}

#[cfg(feature = "alloc")]
impl<T, S: ?Sized + StorageWithCapacity<T>> StorageWithCapacity<T> for Box<S> {
    fn with_capacity(capacity: usize) -> Self { Box::new(S::with_capacity(capacity)) }

    #[doc(hidden)]
    #[inline(always)]
    #[allow(non_snake_case)]
    fn __with_capacity__const_capacity_checked(capacity: usize, _old_capacity: Option<usize>) -> Self {
        Box::new(S::__with_capacity__const_capacity_checked(capacity, _old_capacity))
    }
}
