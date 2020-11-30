#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(min_const_generics, unsafe_block_in_unsafe_fn))]
#![cfg_attr(
    feature = "nightly",
    feature(
        trusted_len,
        min_specialization,
        exact_size_is_empty,
        allocator_api,
        alloc_layout_extra
    )
)]
#![cfg_attr(feature = "nightly", forbid(unsafe_op_in_unsafe_fn))]
#![allow(unused_unsafe)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc as std;

use core::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut, RangeBounds},
    ptr,
};

mod extension;
mod impls;
mod set_len;

pub mod iter;
pub mod raw;

use raw::RawVec;

/// A heap backed vector with a growable capacity
#[cfg(feature = "alloc")]
#[cfg(feature = "nightly")]
pub type Vec<T, A = std::alloc::Global> = GenericVec<raw::Heap<T, A>>;

/// A heap backed vector with a growable capacity
#[cfg(feature = "alloc")]
#[cfg(not(feature = "nightly"))]
pub type Vec<T> = GenericVec<raw::Heap<T>>;

/// An array backed vector backed by potentially uninitialized memory
#[cfg(feature = "nightly")]
pub type ArrayVec<T, const N: usize> = GenericVec<raw::UninitArray<T, N>>;
/// An slice backed vector backed by potentially uninitialized memory
pub type SliceVec<'a, T> = GenericVec<raw::UninitSlice<'a, T>>;

/// An array backed vector backed by initialized memory
#[cfg(feature = "nightly")]
pub type InitArrayVec<T, const N: usize> = GenericVec<raw::Array<T, N>>;
/// An slice backed vector backed by initialized memory
pub type InitSliceVec<'a, T> = GenericVec<raw::Slice<'a, T>>;

use iter::{Drain, DrainFilter, RawDrain, Splice};

#[doc(hidden)]
pub mod macros {
    pub use core::mem::MaybeUninit;
    impl<T> Uninit for T {}
    pub trait Uninit: Sized {
        const UNINIT: MaybeUninit<Self> = MaybeUninit::uninit();
    }
}

/// a helper macro to safely create an array of uninitialized memory of any size
///
///  use the const prefix if you need to initialize a `const` or `static`,
/// otherwise don't use the const modifier
#[macro_export]
macro_rules! uninit_array {
    (const $n:expr) => {
        [$crate::macros::Uninit::UNINIT; $n]
    };

    ($n:expr) => {
        unsafe { $crate::macros::MaybeUninit::<[$crate::macros::MaybeUninit<_>; $n]>::uninit().assume_init() }
    };
}

/// A vector type that can be backed up by a variety of different backends
/// including slices, arrays, and the heap.
#[repr(C)]
pub struct GenericVec<A: ?Sized + RawVec> {
    mark: PhantomData<A::Item>,
    len: usize,
    raw: A,
}

impl<A: ?Sized + RawVec> Deref for GenericVec<A> {
    type Target = [A::Item];

    fn deref(&self) -> &Self::Target {
        let len = self.len();
        unsafe { core::slice::from_raw_parts(self.as_ptr(), len) }
    }
}

impl<A: ?Sized + RawVec> DerefMut for GenericVec<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let len = self.len();
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), len) }
    }
}

impl<A: ?Sized + RawVec> Drop for GenericVec<A> {
    fn drop(&mut self) { unsafe { ptr::drop_in_place(self.as_mut_slice()) } }
}

impl<A: RawVec> GenericVec<A> {
    /// Create a new empty GenericVec with the given backend
    pub fn with_raw(raw: A) -> Self {
        Self {
            raw,
            len: 0,
            mark: PhantomData,
        }
    }
}

impl<A: raw::RawVecWithCapacity> GenericVec<A> {
    /// Create a new empty GenericVec with the backend with at least the given capacity
    pub fn with_capacity(capacity: usize) -> Self { Self::with_raw(A::with_capacity(capacity)) }

    #[inline]
    #[allow(non_snake_case)]
    fn __with_capacity__const_capacity_checked(capacity: usize, old_capacity: Option<usize>) -> Self {
        Self::with_raw(A::__with_capacity__const_capacity_checked(capacity, old_capacity))
    }
}

#[cfg(feature = "nightly")]
impl<T, const N: usize> ArrayVec<T, N> {
    /// Create a new empty `ArrayVec`
    pub const fn new() -> Self {
        Self {
            len: 0,
            mark: PhantomData,
            raw: raw::UninitArray::uninit(),
        }
    }
}

#[cfg(feature = "nightly")]
impl<T: Copy, const N: usize> InitArrayVec<T, N> {
    /// Create a new full `InitArrayVec`
    pub fn new(array: [T; N]) -> Self {
        Self {
            len: N,
            mark: PhantomData,
            raw: raw::Array::new(array),
        }
    }
}

#[cfg(feature = "alloc")]
impl<T> Vec<T> {
    /// Create a new empty `Vec`
    pub const fn new() -> Self {
        Self {
            len: 0,
            mark: PhantomData,
            raw: raw::Heap::new(),
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg(feature = "nightly")]
impl<T, A: std::alloc::AllocRef> Vec<T, A> {
    /// Create a new empty `Vec` with the given allocator
    pub fn with_alloc(alloc: A) -> Self { Self::with_raw(raw::Heap::with_alloc(alloc)) }
}

impl<'a, T> SliceVec<'a, T> {
    /// Create a new empty `SliceVec`
    pub fn new(slice: &'a mut [MaybeUninit<T>]) -> Self { Self::with_raw(raw::Uninit(slice)) }
}

impl<'a, T: Copy> InitSliceVec<'a, T> {
    /// Create a new full `InitSliceVec`
    pub fn new(slice: &'a mut [T]) -> Self {
        let len = slice.len();
        let mut vec = Self::with_raw(raw::Init(slice));
        vec.set_len(len);
        vec
    }
}

impl<A: ?Sized + RawVec> GenericVec<A> {
    /// Returns a shared raw pointer to the vector's buffer.
    ///
    /// It's not safe to write to this pointer except for values
    /// inside of an `UnsafeCell`
    pub fn as_ptr(&self) -> *const A::Item { self.raw.as_ptr() }

    /// Returns a unique raw pointer to the vector's buffer.
    pub fn as_mut_ptr(&mut self) -> *mut A::Item { self.raw.as_mut_ptr() }

    /// Returns the number of elements in the vector
    pub fn len(&self) -> usize { self.len }

    /// Returns the number of elements the vector can hold without reallocating or panicing.
    pub fn capacity(&self) -> usize {
        if core::mem::size_of::<A::Item>() == 0 {
            isize::MAX as usize
        } else {
            self.raw.capacity()
        }
    }

    /// Returns true if and only if the vector contains no elements.
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Returns true if and only if the vector's length is equal to it's capacity.
    pub fn is_full(&self) -> bool { self.len() == self.capacity() }

    /// Returns the length of the spare capacity of the `GenericVec`
    pub fn remaining_capacity(&self) -> usize { self.capacity().wrapping_sub(self.len()) }

    /// Set the length of a vector
    ///
    /// # Safety
    ///
    /// * new_len must be less than or equal to `capacity()`.
    /// * The elements at `old_len..new_len` must be initialized.
    pub unsafe fn set_len_unchecked(&mut self, len: usize) { self.len = len; }

    /// Set the length of a vector
    pub fn set_len(&mut self, len: usize)
    where
        A: raw::RawVecInit,
    {
        unsafe {
            assert!(
                len <= self.capacity(),
                "Tried to set the length to larger than the capacity"
            );
            self.set_len_unchecked(len);
        }
    }

    /// Extracts a slice containing the entire vector.
    ///
    /// Equivalent to &s[..].
    pub fn as_slice(&self) -> &[A::Item] { self }

    /// Extracts a mutable slice containing the entire vector.
    ///
    /// Equivalent to &mut s[..].
    pub fn as_mut_slice(&mut self) -> &mut [A::Item] { self }

    /// Returns the underlying raw buffer
    pub fn raw_buffer(&self) -> &A { &self.raw }

    /// Returns the underlying raw buffer
    ///
    /// # Safety
    ///
    /// You must not replace the raw buffer
    pub unsafe fn raw_buffer_mut(&mut self) -> &mut A { &mut self.raw }

    /// Returns the remaining spare capacity of the vector as a slice
    /// of `[MaybeUninit<T>]` or `[T]`
    pub fn spare_capacity_mut(&mut self) -> &mut [A::BufferItem] {
        unsafe {
            let len = self.len();
            let cap = self.capacity();
            core::slice::from_raw_parts_mut(self.raw.as_mut_ptr().add(len).cast(), cap.wrapping_sub(len))
        }
    }

    /// Reserve enough space for at least `additional` elements
    ///
    /// # Panics
    ///
    /// May panic or abort if it isn't possible to allocate enough space for
    /// `additional` more elements
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        if self.remaining_capacity() < additional {
            self.raw.reserve(
                self.len()
                    .checked_add(additional)
                    .expect("Allocation overflow detected"),
            )
        }
    }

    /// Try to reserve enough space for at least `additional` elements, and returns `Err(_)`
    /// if it's not possible to reserve enough space
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), raw::AllocError> {
        if self.remaining_capacity() < additional {
            self.raw
                .try_reserve(self.len().checked_add(additional).ok_or(raw::AllocError)?)
        } else {
            Ok(())
        }
    }

    /// Shortens the vector, keeping the first len elements and dropping the rest.
    ///
    /// If len is greater than the vector's current length, this has no effect.
    ///
    /// Note that this method has no effect on the allocated capacity of the vector.
    pub fn truncate(&mut self, len: usize) {
        if let Some(diff) = self.len().checked_sub(len) {
            unsafe {
                self.set_len_unchecked(len);
                core::slice::from_raw_parts_mut(self.as_mut_ptr().add(len), diff);
            }
        }
    }

    // Grows the `GenericVec` in-place by additional elements.
    //
    // This method requires `T` to implement `Clone`, in order to be able to clone
    // the passed value. If you need more flexibility (or want to rely on Default instead of `Clone`),
    // use [`GenericVec::grow_with`].
    pub fn grow(&mut self, additional: usize, value: A::Item)
    where
        A::Item: Clone,
    {
        self.reserve(additional);
        unsafe { extension::Extension::grow(self, additional, value) }
    }

    // Grows the `GenericVec` in-place by additional elements.
    //
    // This method uses a closure to create new values on every push.
    // If you'd rather `Clone` a given value, use `GenericVec::resize`.
    // If you want to use the `Default` trait to generate values, you
    // can pass `Default::default` as the second argument.
    pub fn grow_with<F>(&mut self, additional: usize, mut value: F)
    where
        F: FnMut() -> A::Item,
    {
        self.reserve(additional);

        let mut len = set_len::SetLenOnDrop::new(&mut self.len);
        let mut ptr = unsafe { self.raw.as_mut_ptr().add(*len) };
        let end = unsafe { ptr.add(additional) };

        while ptr != end {
            unsafe {
                ptr.write(value());
                ptr = ptr.add(1);
                len += 1;
            }
        }
    }

    // Clears the vector, removing all values.
    //
    // Note that this method has no effect on the allocated capacity of the vector.
    pub fn clear(&mut self) { self.truncate(0); }

    /// Appends an element to the back of a collection.
    ///
    /// # Panic
    ///
    /// May panic or reallocate if the collection is full
    pub fn push(&mut self, value: A::Item) -> &mut A::Item {
        if self.len() == self.capacity() {
            self.reserve(1);
        }

        unsafe { self.push_unchecked(value) }
    }

    /// Appends the array to the back of a collection.
    ///
    /// # Panic
    ///
    /// May panic or reallocate if the collection has less than N elements remaining
    #[cfg(feature = "nightly")]
    pub fn push_array<const N: usize>(&mut self, value: [A::Item; N]) -> &mut [A::Item; N] {
        self.reserve(N);
        unsafe { self.push_array_unchecked(value) }
    }

    /// Inserts an element at position index within the vector,
    /// shifting all elements after it to the right.
    ///
    /// # Panics
    ///
    /// * May panic or reallocate if the collection is full
    /// * Panics if index > len.
    pub fn insert(&mut self, index: usize, value: A::Item) -> &mut A::Item {
        assert!(
            index <= self.len(),
            "Tried to insert at {}, but length is {}",
            index,
            self.len(),
        );

        if self.is_full() {
            self.reserve(1);
        }

        unsafe { self.insert_unchecked(index, value) }
    }

    /// Inserts the array at position index within the vector,
    /// shifting all elements after it to the right.
    ///
    /// # Panics
    ///
    /// * May panic or reallocate if the collection has less than N elements remaining
    /// * Panics if index > len.
    #[cfg(feature = "nightly")]
    pub fn insert_array<const N: usize>(&mut self, index: usize, value: [A::Item; N]) -> &mut [A::Item; N] {
        assert!(
            index <= self.len(),
            "Tried to insert at {}, but length is {}",
            index,
            self.len(),
        );

        self.reserve(N);
        unsafe { self.insert_array_unchecked(index, value) }
    }

    /// Removes the last element from a vector and returns it
    ///
    /// # Panics
    ///
    /// Panics if the collection is empty
    pub fn pop(&mut self) -> A::Item {
        assert_ne!(self.len(), 0, "Tried to pop an element from an empty vector",);

        unsafe { self.pop_unchecked() }
    }

    /// Removes the last `N` elements from a vector and returns it
    ///
    /// # Panics
    ///
    /// Panics if the collection contains less than `N` elements in it
    #[cfg(feature = "nightly")]
    pub fn pop_array<const N: usize>(&mut self) -> [A::Item; N] {
        assert!(
            self.len() >= N,
            "Tried to pop {} elements, but length is {}",
            N,
            self.len()
        );

        unsafe { self.pop_array_unchecked() }
    }

    /// Removes and returns the element at position index within the vector,
    /// shifting all elements after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> A::Item {
        assert!(
            index < self.len(),
            "Tried to remove item at index {}, but length is {}",
            index,
            self.len()
        );

        unsafe { self.remove_unchecked(index) }
    }

    /// Removes and returns `N` elements at position index within the vector,
    /// shifting all elements after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds or if `index + N > len()`
    #[cfg(feature = "nightly")]
    pub fn remove_array<const N: usize>(&mut self, index: usize) -> [A::Item; N] {
        assert!(
            self.len() >= index && self.len().wrapping_sub(index) >= N,
            "Tried to remove {} elements at index {}, but length is {}",
            N,
            index,
            self.len()
        );

        unsafe { self.remove_array_unchecked(index) }
    }

    /// Removes an element from the vector and returns it.
    ///
    /// The removed element is replaced by the last element of the vector.
    ///
    /// This does not preserve ordering, but is O(1).
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn swap_remove(&mut self, index: usize) -> A::Item {
        assert!(
            index < self.len(),
            "Tried to remove item at index {}, but length is {}",
            index,
            self.len()
        );

        unsafe { self.swap_remove_unchecked(index) }
    }

    /// Tries to append an element to the back of a collection.
    /// Returns the `Err(value)` if the collection is full
    ///
    /// Guaranteed to not panic/abort/allocate
    pub fn try_push(&mut self, value: A::Item) -> Result<&mut A::Item, A::Item> {
        if self.is_full() {
            Err(value)
        } else {
            unsafe { Ok(self.push_unchecked(value)) }
        }
    }

    /// Tries to append an array to the back of a collection.
    /// Returns the `Err(value)` if the collection doesn't have enough remaining capacity
    /// to hold `N` elements.
    ///
    /// Guaranteed to not panic/abort/allocate
    #[cfg(feature = "nightly")]
    pub fn try_push_array<const N: usize>(&mut self, value: [A::Item; N]) -> Result<&mut [A::Item; N], [A::Item; N]> {
        if self.remaining_capacity() < N {
            Err(value)
        } else {
            unsafe { Ok(self.push_array_unchecked(value)) }
        }
    }

    /// Inserts an element at position index within the vector,
    /// shifting all elements after it to the right.
    /// Returns the `Err(value)` if the collection is full or index is out of bounds
    ///
    /// Guaranteed to not panic/abort/allocate
    pub fn try_insert(&mut self, index: usize, value: A::Item) -> Result<&mut A::Item, A::Item> {
        if self.is_full() || index > self.len() {
            Err(value)
        } else {
            unsafe { Ok(self.insert_unchecked(index, value)) }
        }
    }

    /// Inserts an array at position index within the vector,
    /// shifting all elements after it to the right.
    /// Returns the `Err(value)` if the collection doesn't have enough remaining capacity
    /// to hold `N` elements or index is out of bounds
    ///
    /// Guaranteed to not panic/abort/allocate
    #[cfg(feature = "nightly")]
    pub fn try_insert_array<const N: usize>(
        &mut self,
        index: usize,
        value: [A::Item; N],
    ) -> Result<&mut [A::Item; N], [A::Item; N]> {
        if self.capacity().wrapping_sub(self.len()) < N || index > self.len() {
            Err(value)
        } else {
            unsafe { Ok(self.insert_array_unchecked(index, value)) }
        }
    }

    /// Removes the last element from a vector and returns it,
    /// Returns `None` if the collection is empty
    ///
    /// Guaranteed to not panic/abort/allocate
    pub fn try_pop(&mut self) -> Option<A::Item> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some(self.pop_unchecked()) }
        }
    }

    /// Removes the last `N` elements from a vector and returns it,
    /// Returns `None` if the collection is has less than N elements
    ///
    /// Guaranteed to not panic/abort/allocate
    #[cfg(feature = "nightly")]
    pub fn try_pop_array<const N: usize>(&mut self) -> Option<[A::Item; N]> {
        if self.len() == 0 {
            None
        } else {
            unsafe { Some(self.pop_array_unchecked()) }
        }
    }

    /// Removes and returns the element at position index within the vector,
    /// shifting all elements after it to the left.
    /// Returns `None` if collection is empty or `index` is out of bounds.
    ///
    /// Guaranteed to not panic/abort/allocate
    pub fn try_remove(&mut self, index: usize) -> Option<A::Item> {
        if self.len() < index {
            None
        } else {
            unsafe { Some(self.remove_unchecked(index)) }
        }
    }

    /// Removes and returns the element at position index within the vector,
    /// shifting all elements after it to the left.
    /// Returns `None` if the collection is has less than N elements
    /// or `index` is out of bounds.
    ///
    /// Guaranteed to not panic/abort/allocate
    #[cfg(feature = "nightly")]
    pub fn try_remove_array<const N: usize>(&mut self, index: usize) -> Option<[A::Item; N]> {
        if self.len() < index || self.len().wrapping_sub(index) < N {
            unsafe { Some(self.remove_array_unchecked(index)) }
        } else {
            None
        }
    }

    /// Removes an element from the vector and returns it.
    /// Returns `None` if collection is empty or `index` is out of bounds.
    ///
    /// The removed element is replaced by the last element of the vector.
    ///
    /// This does not preserve ordering, but is O(1).
    ///
    /// Guaranteed to not panic/abort/allocate
    pub fn try_swap_remove(&mut self, index: usize) -> Option<A::Item> {
        if index < self.len() {
            unsafe { Some(self.swap_remove_unchecked(index)) }
        } else {
            None
        }
    }

    /// Appends an element to the back of a collection.
    ///
    /// # Safety
    ///
    /// the collection must not be full
    pub unsafe fn push_unchecked(&mut self, value: A::Item) -> &mut A::Item {
        match A::CONST_CAPACITY {
            Some(0) => panic!("Tried to push an element into a zero-capacity vector!"),
            _ => (),
        }

        debug_assert_ne!(
            self.len(),
            self.capacity(),
            "Tried to `push_unchecked` past capacity! This is UB in release mode"
        );
        unsafe {
            let len = self.len();
            self.set_len_unchecked(len.wrapping_add(1));
            let ptr = self.as_mut_ptr().add(len);
            ptr.write(value);
            &mut *ptr
        }
    }

    /// Appends the array to the back of a collection.
    ///
    /// # Safety
    ///
    /// the collection's remaining capacity must be at least N
    #[cfg(feature = "nightly")]
    pub unsafe fn push_array_unchecked<const N: usize>(&mut self, value: [A::Item; N]) -> &mut [A::Item; N] {
        match A::CONST_CAPACITY {
            Some(n) if n < N => {
                panic!("Tried to push an array larger than the maximum capacity of the vector!")
            }
            _ => (),
        }

        unsafe {
            let len = self.len();
            self.set_len_unchecked(len.wrapping_add(N));
            let ptr = self.as_mut_ptr();
            let out = ptr.add(len) as *mut [A::Item; N];
            out.write(value);
            &mut *out
        }
    }

    /// Inserts an element at position index within the vector,
    /// shifting all elements after it to the right.
    ///
    /// # Safety
    ///
    /// * the collection is must not be full
    /// * hte index must be in bounds
    pub unsafe fn insert_unchecked(&mut self, index: usize, value: A::Item) -> &mut A::Item {
        unsafe {
            match A::CONST_CAPACITY {
                Some(0) => panic!("Tried to insert an element into a zero-capacity vector!"),
                _ => (),
            }

            let len = self.len();
            self.set_len_unchecked(len.wrapping_add(1));
            let ptr = self.raw.as_mut_ptr().add(index);
            ptr.add(1).copy_from(ptr, len.wrapping_sub(index));
            ptr.write(value);
            &mut *ptr
        }
    }

    /// Inserts an array at position index within the vector,
    /// shifting all elements after it to the right.
    ///
    /// # Safety
    ///
    /// * the collection's remaining capacity must be at least N
    /// * hte index must be in bounds
    #[cfg(feature = "nightly")]
    pub unsafe fn insert_array_unchecked<const N: usize>(
        &mut self,
        index: usize,
        value: [A::Item; N],
    ) -> &mut [A::Item; N] {
        match A::CONST_CAPACITY {
            Some(n) if n < N => {
                panic!("Tried to push an array larger than the maximum capacity of the vector!")
            }
            _ => (),
        }

        unsafe {
            let len = self.len();
            self.set_len_unchecked(len.wrapping_add(N));
            let ptr = self.as_mut_ptr();
            let dist = len.wrapping_sub(index);

            let out = ptr.add(index);
            out.add(N).copy_from(out, dist);
            let out = out as *mut [A::Item; N];
            out.write(value);
            &mut *out
        }
    }

    /// Removes the last element from a vector and returns it
    ///
    /// # Safety
    ///
    /// the collection must not be empty
    pub unsafe fn pop_unchecked(&mut self) -> A::Item {
        match A::CONST_CAPACITY {
            Some(0) => panic!("Tried to remove an element from a zero-capacity vector!"),
            _ => (),
        }

        let len = self.len();
        debug_assert_ne!(
            len, 0,
            "Tried to `pop_unchecked` an empty array vec! This is UB in release mode"
        );
        unsafe {
            let len = len.wrapping_sub(1);
            self.set_len_unchecked(len);
            self.as_mut_ptr().add(len).read()
        }
    }

    /// Removes the last `N` elements from a vector and returns it
    ///
    /// # Safety
    ///
    /// The collection must contain at least `N` elements in it
    #[cfg(feature = "nightly")]
    pub unsafe fn pop_array_unchecked<const N: usize>(&mut self) -> [A::Item; N] {
        match A::CONST_CAPACITY {
            Some(n) if n < N => panic!("Tried to remove {} elements from a {} capacity vector!", N, n),
            _ => (),
        }

        let len = self.len();
        debug_assert!(
            len > N,
            "Tried to remove {} elements from a {} length vector! This is UB in release mode",
            N,
            len,
        );
        unsafe {
            let len = len.wrapping_sub(N);
            self.set_len_unchecked(len);
            self.as_mut_ptr().add(len).cast::<[A::Item; N]>().read()
        }
    }

    /// Removes and returns the element at position index within the vector,
    /// shifting all elements after it to the left.
    ///
    /// # Safety
    ///
    /// the collection must not be empty, and
    /// index must be in bounds
    pub unsafe fn remove_unchecked(&mut self, index: usize) -> A::Item {
        unsafe {
            match A::CONST_CAPACITY {
                Some(0) => panic!("Tried to remove an element from a zero-capacity vector!"),
                _ => (),
            }

            let len = self.len();

            debug_assert!(
                index <= len,
                "Tried to remove an element at index {} from a {} length vector! This is UB in release mode",
                index,
                len,
            );

            self.set_len_unchecked(len.wrapping_sub(1));
            let ptr = self.raw.as_mut_ptr().add(index);
            let value = ptr.read();
            ptr.copy_from(ptr.add(1), len.wrapping_sub(index).wrapping_sub(1));
            value
        }
    }

    /// Removes and returns the element at position index within the vector,
    /// shifting all elements after it to the left.
    ///
    /// # Safety
    ///
    /// the collection must contain at least N elements, and
    /// index must be in bounds
    #[cfg(feature = "nightly")]
    pub unsafe fn remove_array_unchecked<const N: usize>(&mut self, index: usize) -> [A::Item; N] {
        match A::CONST_CAPACITY {
            Some(n) if n < N => panic!("Tried to remove {} elements from a {} capacity vector!", N, n),
            _ => (),
        }

        let len = self.len();
        debug_assert!(
            index <= len,
            "Tried to remove elements at index {} from a {} length vector! This is UB in release mode",
            index,
            len,
        );
        debug_assert!(
            len.wrapping_sub(index) > N,
            "Tried to remove {} elements from a {} length vector! This is UB in release mode",
            N,
            len,
        );
        unsafe {
            self.set_len_unchecked(len.wrapping_sub(N));
            let ptr = self.as_mut_ptr().add(index);
            let value = ptr.cast::<[A::Item; N]>().read();
            if N != 0 {
                ptr.copy_from(ptr.add(N), len.wrapping_sub(index).wrapping_sub(N));
            }
            value
        }
    }

    /// Removes an element from the vector and returns it.
    ///
    /// The removed element is replaced by the last element of the vector.
    ///
    /// This does not preserve ordering, but is O(1).
    ///
    /// # Safety
    ///
    /// the `index` must be in bounds
    pub unsafe fn swap_remove_unchecked(&mut self, index: usize) -> A::Item {
        unsafe {
            match A::CONST_CAPACITY {
                Some(0) => panic!("Tried to remove an element from a zero-capacity vector!"),
                _ => (),
            }

            let len = self.len();
            self.set_len_unchecked(len.wrapping_sub(1));
            let ptr = self.raw.as_mut_ptr();
            let at = ptr.add(index);
            let end = ptr.add(len.wrapping_sub(1));
            let value = at.read();
            at.copy_from(end, 1);
            value
        }
    }

    /// Splits the collection into two at the given index.
    ///
    /// Returns a newly allocated vector containing the elements in the range [at, len).
    /// After the call, the original vector will be left containing the elements [0, at)
    /// with its previous capacity unchanged.
    pub fn split_off<B>(&mut self, index: usize) -> GenericVec<B>
    where
        B: raw::RawVecWithCapacity<Item = A::Item>,
    {
        assert!(
            index <= self.len(),
            "Tried to split at index {}, but length is {}",
            index,
            self.len()
        );

        let mut vec =
            GenericVec::<B>::__with_capacity__const_capacity_checked(self.len().wrapping_sub(index), A::CONST_CAPACITY);

        self.split_off_into(index, &mut vec);

        vec
    }

    /// Splits the collection into two at the given index.
    ///
    /// Appends the elements from the range [at, len) to `other`.
    /// After the call, the original vector will be left containing the elements [0, at)
    /// with its previous capacity unchanged.
    pub fn split_off_into<B>(&mut self, index: usize, other: &mut GenericVec<B>)
    where
        B: raw::RawVec<Item = A::Item> + ?Sized,
    {
        assert!(
            index <= self.len(),
            "Tried to split at index {}, but length is {}",
            index,
            self.len()
        );

        unsafe {
            let slice = self.get_unchecked(index..);
            other.reserve(slice.len());
            other.extend_from_slice_unchecked(slice);
            self.set_len_unchecked(index);
        }
    }

    /// Convert the backing buffer type, and moves all the elements in `self` to the new vector
    pub fn convert<B: raw::RawVecWithCapacity<Item = A::Item>>(mut self) -> GenericVec<B>
    where
        A: Sized,
    {
        self.split_off(0)
    }

    #[inline]
    pub fn raw_drain<R>(&mut self, range: R) -> RawDrain<'_, A>
    where
        R: RangeBounds<usize>,
    {
        RawDrain::new(self, range)
    }

    /// Creates a draining iterator that removes the specified range in the
    /// vector and yields the removed items.
    ///
    /// When the iterator is dropped, all elements in the range are removed from
    /// the vector, even if the iterator was not fully consumed. If the iterator
    /// is not dropped (with mem::forget for example), it is unspecified how many
    /// elements are removed.
    ///
    /// # Panic
    ///
    /// Panics if the starting point is greater than the end point or if the end point is greater than the length of the vector.
    #[inline]
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, A>
    where
        R: RangeBounds<usize>,
    {
        self.raw_drain(range).into()
    }

    /// Creates an iterator which uses a closure to determine if an element should be removed.
    ///
    /// If the closure returns true, then the element is removed and yielded.
    /// If the closure returns false, the element will remain in the vector
    /// and will not be yielded by the iterator.
    #[inline]
    pub fn drain_filter<R, F>(&mut self, range: R, f: F) -> DrainFilter<'_, A, F>
    where
        R: RangeBounds<usize>,
        F: FnMut(&mut A::Item) -> bool,
    {
        DrainFilter::new(self.raw_drain(range), f)
    }

    /// Creates a splicing iterator that replaces the specified range in the vector with the given replace_with iterator and yields the removed items. replace_with does not need to be the same length as range.
    ///
    /// range is removed even if the iterator is not consumed until the end.
    ///
    /// It is unspecified how many elements are removed from the vector if the Splice value is leaked.
    ///
    /// The input iterator replace_with is only consumed when the Splice value is dropped
    #[inline]
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, A, I::IntoIter>
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = A::Item>,
    {
        Splice::new(self.raw_drain(range), replace_with.into_iter())
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements e such that f(&e) returns false.
    /// This method operates in place, visiting each element exactly once in
    /// the original order, and preserves the order of the retained elements.
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&mut A::Item) -> bool,
    {
        fn not<F: FnMut(&mut T) -> bool, T>(mut f: F) -> impl FnMut(&mut T) -> bool { move |value| !f(value) }
        self.drain_filter(.., not(f));
    }

    /// Shallow copies and appends all elements in a slice to the `GenericVec`.
    ///
    /// # Safety
    ///
    /// You must not drop any of the elements in `slice`, and
    /// there must be at least `slice.len()` remaining capacity in the vector
    pub unsafe fn extend_from_slice_unchecked(&mut self, slice: &[A::Item]) {
        unsafe {
            debug_assert!(
                self.remaining_capacity() >= slice.len(),
                "Not enough capacity to hold the slice"
            );

            let len = self.len();
            self.as_mut_ptr()
                .add(len)
                .copy_from_nonoverlapping(slice.as_ptr(), slice.len());
            self.set_len_unchecked(len.wrapping_add(slice.len()));
        }
    }

    /// Clones and appends all elements in a slice to the `GenericVec`.
    ///
    /// Iterates over the slice other, clones each element, and then appends
    /// it to this `GenericVec`. The other vector is traversed in-order.
    ///
    /// Note that this function is same as extend except that it is specialized
    /// to work with slices instead. If and when Rust gets specialization this
    /// function will likely be deprecated (but still available).
    pub fn extend_from_slice(&mut self, slice: &[A::Item])
    where
        A::Item: Clone,
    {
        self.reserve(self.len());

        unsafe { extension::Extension::extend_from_slice(self, slice) }
    }
}
