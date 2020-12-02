#![cfg_attr(not(any(doc, feature = "std")), no_std)]
#![cfg_attr(
    any(doc, feature = "nightly"),
    feature(min_const_generics, unsafe_block_in_unsafe_fn)
)]
#![cfg_attr(
    any(doc, feature = "nightly"),
    feature(
        trusted_len,
        min_specialization,
        exact_size_is_empty,
        allocator_api,
        alloc_layout_extra,
        const_panic,
        const_fn,
        const_mut_refs,
    )
)]
#![cfg_attr(feature = "nightly", forbid(unsafe_op_in_unsafe_fn))]
#![allow(unused_unsafe)]
#![forbid(missing_docs, clippy::missing_safety_doc)]

//! A vector that can store items anywhere: in slices, arrays, or the heap!
//!
//! [`GenericVec`] has complete parity with [`Vec`], and even provides some features
//! that are only in `nightly` on `std` (like [`GenericVec::drain_filter`]), or a more permissive
//! interface like [`GenericVec::retain`]. In fact, you can trivially convert a [`Vec`] to a
//! [`HeapVec`] and back!
//!
//! This crate is `no_std` compatible, just turn off all default features.
//!
//! # Features
//!
//! * `std` (default) - enables you to use an allocator, and
//! * `alloc` - enables you to use an allocator, for heap allocated storages
//!     (like [`Vec`])
//! * `nightly` - enables you to use array (`[T; N]`) based storages
//!
//! # Basic Usage
//!
//! On stable `no_std` you have two choices on for which storage you can use
//! [`SliceVec`] or [`InitSliceVec`]. There are three major differences between
//! them.
//!
//! * You can pass an uninitialized buffer to [`SliceVec`]
//! * You can only use [`Copy`] types with [`InitSliceVec`]
//! * You can freely set the length of the [`InitSliceVec`] as long as you stay
//!     within it's capacity (the length of the slice you pass in)
//!
//! ```rust
//! use generic_vec::{SliceVec, InitSliceVec, uninit_array};
//!
//! let mut uninit_buffer = uninit_array!(16);
//! let mut slice_vec = SliceVec::new(&mut uninit_buffer);
//!
//! assert!(slice_vec.is_empty());
//! slice_vec.push(10);
//! assert_eq!(slice_vec, [10]);
//! ```
//!
//! ```rust
//! # use generic_vec::InitSliceVec;
//! let mut init_buffer = [0xae; 16];
//! let mut slice_vec = InitSliceVec::new(&mut init_buffer);
//!
//! assert!(slice_vec.is_full());
//! assert_eq!(slice_vec.pop(), 0xae);
//! slice_vec.set_len(16);
//! assert!(slice_vec.is_full());
//! ```
//!
//! Of course if you try to push past a `*SliceVec`'s capacity
//! (the length of the slice you passed in), then it will panic.
//!
//! ```rust,should_panic
//! # use generic_vec::InitSliceVec;
//! let mut init_buffer = [0xae; 16];
//! let mut slice_vec = InitSliceVec::new(&mut init_buffer);
//! slice_vec.push(0);
//! ```
//!
//! If you enable the nightly feature then you gain access to
//! [`ArrayVec`] and [`InitArrayVec`]. These are just like the
//! slice versions, but since they own their data, they can be
//! freely moved around, unconstrained. You can also create
//! a new [`ArrayVec`] without passing in an existing buffer.
//!
//! ```rust,ignore
//! use generic_vec::ArrayVec;
//!
//! let mut array_vec = ArrayVec::<i32, 16>::new();
//!
//! array_vec.push(10);
//! array_vec.push(20);
//! array_vec.push(30);
//!
//! assert_eq!(array_vec, [10, 20, 30]);
//! ```
//!
//! The distinction between [`ArrayVec`] and [`InitArrayVec`]
//! is identical to their slice counterparts.
//!
//! Finally a [`HeapVec`] is just [`Vec`], but built atop [`GenericVec`],
//! meaning you get all the features of [`GenericVec`] for free! But this
//! requries either the `alloc` or `std` feature to be enabled.
//!
//! As a neat side-effect of this framework, you can also get an efficient
//! [`GenericVec`] for zero-sized types, just a `usize` in size! This feature
//! can be on stable `no_std`.
//!
//! ```rust
//! use generic_vec::ZSVec;
//!
//! struct MyType;
//!
//! let mut vec = ZSVec::new();
//! vec.push(MyType);
//! vec.push(MyType);
//! vec.push(MyType);
//! assert_eq!(vec.len(), 3);
//! assert_eq!(std::mem::size_of_val(&vec), std::mem::size_of::<usize>());
//! ```

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
// mod set_len;

pub mod iter;
pub mod raw;

use raw::Storage;

/// A heap backed vector with a growable capacity
#[cfg(any(doc, feature = "alloc"))]
#[cfg(feature = "nightly")]
pub type HeapVec<T, A = std::alloc::Global> = GenericVec<T, raw::Heap<T, A>>;

/// A heap backed vector with a growable capacity
#[cfg(any(doc, feature = "alloc"))]
#[cfg(not(feature = "nightly"))]
pub type HeapVec<T> = GenericVec<T, raw::Heap<T>>;

/// An array backed vector backed by potentially uninitialized memory
#[cfg(any(doc, feature = "nightly"))]
pub type ArrayVec<T, const N: usize> = GenericVec<T, raw::UninitArray<T, N>>;
/// An slice backed vector backed by potentially uninitialized memory
pub type SliceVec<'a, T> = GenericVec<T, raw::UninitSlice<'a, T>>;

/// An array backed vector backed by initialized memory
#[cfg(any(doc, feature = "nightly"))]
pub type InitArrayVec<T, const N: usize> = GenericVec<T, raw::Array<T, N>>;
/// An slice backed vector backed by initialized memory
pub type InitSliceVec<'a, T> = GenericVec<T, raw::Slice<'a, T>>;
/// A counter vector that can only store zero-sized types
pub type ZSVec<T> = GenericVec<T, raw::ZeroSized<T>>;

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
        // Safety
        //
        // `MaybeUninit` can represent any bit-pattern, including uninitialized memory
        // so it's fine to cast uninitialized memory to `[MaybeUninit; N]`
        unsafe { $crate::macros::MaybeUninit::<[$crate::macros::MaybeUninit<_>; $n]>::uninit().assume_init() }
    };
}

/// A vector type that can be backed up by a variety of different backends
/// including slices, arrays, and the heap.
#[repr(C)]
pub struct GenericVec<T, S: ?Sized + Storage<T>> {
    mark: PhantomData<T>,
    len: usize,
    storage: S,
}

impl<T, S: ?Sized + Storage<T>> Deref for GenericVec<T, S> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        let len = self.len();
        // The first `len` elements are guaranteed to be initialized
        // as part of the guarantee on `self.set_len_unchecked`
        unsafe { core::slice::from_raw_parts(self.as_ptr(), len) }
    }
}

impl<T, S: ?Sized + Storage<T>> DerefMut for GenericVec<T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let len = self.len();
        // The first `len` elements are guaranteed to be initialized
        // as part of the guarantee on `self.set_len_unchecked`
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), len) }
    }
}

impl<T, S: ?Sized + Storage<T>> Drop for GenericVec<T, S> {
    fn drop(&mut self) {
        // The first `len` elements are guaranteed to be initialized
        // as part of the guarantee on `self.set_len_unchecked`
        // These elements should be dropped when the `GenericVec` gets dropped/
        // The storage will clean it's self up on drop
        unsafe { ptr::drop_in_place(self.as_mut_slice()) }
    }
}

#[cfg(not(feature = "nightly"))]
impl<T, S: Storage<T>> GenericVec<T, S> {
    /// Create a new empty `GenericVec` with the given backend
    pub fn with_storage(storage: S) -> Self {
        Self {
            storage,
            len: 0,
            mark: PhantomData,
        }
    }
}

#[cfg(feature = "nightly")]
impl<T, S: Storage<T>> GenericVec<T, S> {
    /// Create a new empty `GenericVec` with the given backend
    pub const fn with_storage(storage: S) -> Self {
        Self {
            storage,
            len: 0,
            mark: PhantomData,
        }
    }
}

impl<T, S: raw::StorageWithCapacity<T>> GenericVec<T, S> {
    /// Create a new empty `GenericVec` with the backend with at least the given capacity
    pub fn with_capacity(capacity: usize) -> Self { Self::with_storage(S::with_capacity(capacity)) }

    #[inline]
    #[allow(non_snake_case)]
    fn __with_capacity__const_capacity_checked(capacity: usize, old_capacity: Option<usize>) -> Self {
        Self::with_storage(S::__with_capacity__const_capacity_checked(capacity, old_capacity))
    }
}

#[cfg(feature = "nightly")]
impl<T, const N: usize> ArrayVec<T, N> {
    /// Create a new empty `ArrayVec`
    pub const fn new() -> Self {
        Self {
            len: 0,
            mark: PhantomData,
            storage: raw::UninitArray::uninit(),
        }
    }

    /// Create a new full `ArrayVec`
    pub const fn from_array(array: [T; N]) -> Self {
        Self {
            len: 0,
            mark: PhantomData,
            storage: raw::UninitArray::with_array(array),
        }
    }

    /// Convert this `ArrayVec` into an array
    ///
    /// # Panic
    ///
    /// Panics if the the collection is not full
    pub fn into_array(self) -> [T; N] {
        assert!(self.is_full());
        let this = core::mem::ManuallyDrop::new(self);
        unsafe { this.storage.0.as_ptr().read() }
    }
}

#[cfg(feature = "nightly")]
impl<T: Copy, const N: usize> InitArrayVec<T, N> {
    /// Create a new full `InitArrayVec`
    pub fn new(array: [T; N]) -> Self {
        Self {
            len: N,
            mark: PhantomData,
            storage: raw::Array::new(array),
        }
    }
}

#[cfg(feature = "alloc")]
impl<T> HeapVec<T> {
    /// Create a new empty `HeapVec`
    pub const fn new() -> Self {
        Self {
            len: 0,
            mark: PhantomData,
            storage: raw::Heap::new(),
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg(feature = "nightly")]
impl<T, A: std::alloc::AllocRef> HeapVec<T, A> {
    /// Create a new empty `HeapVec` with the given allocator
    pub fn with_alloc(alloc: A) -> Self { Self::with_storage(raw::Heap::with_alloc(alloc)) }
}

#[cfg(not(feature = "nightly"))]
impl<'a, T> SliceVec<'a, T> {
    /// Create a new empty `SliceVec`
    pub fn new(slice: &'a mut [MaybeUninit<T>]) -> Self { Self::with_storage(raw::Uninit(slice)) }
}

#[cfg(feature = "nightly")]
impl<'a, T> SliceVec<'a, T> {
    /// Create a new empty `SliceVec`
    pub const fn new(slice: &'a mut [MaybeUninit<T>]) -> Self { Self::with_storage(raw::Uninit(slice)) }
}

impl<'a, T: Copy> InitSliceVec<'a, T> {
    /// Create a new full `InitSliceVec`
    pub fn new(slice: &'a mut [T]) -> Self {
        let len = slice.len();
        let mut vec = Self::with_storage(raw::Init(slice));
        vec.set_len(len);
        vec
    }
}

impl<T, S: Storage<T>> GenericVec<T, S> {
    /// Convert a `GenericVec` into a length-storage pair
    pub fn into_raw_parts(self) -> (usize, S) {
        let this = core::mem::ManuallyDrop::new(self);
        unsafe { (this.len, core::ptr::read(&this.storage)) }
    }

    /// Create a `GenericVec` from a length-storage pair
    ///
    /// # Safety
    ///
    /// the length must be less than `raw.capacity()` and
    /// all elements in the range `0..length`, must be initialized
    ///
    /// # Panic
    ///
    /// If the given storage cannot hold type `T`, then this method will panic
    pub unsafe fn from_raw_parts(len: usize, storage: S) -> Self {
        Self {
            storage,
            len,
            mark: PhantomData,
        }
    }
}

impl<T> ZSVec<T> {
    /// Create a new counter vector
    pub const NEW: Self = Self {
        len: 0,
        storage: raw::ZeroSized::NEW,
        mark: PhantomData,
    };

    /// Create a new counter vector
    pub const fn new() -> Self { Self::NEW }
}

impl<T, S: ?Sized + Storage<T>> GenericVec<T, S> {
    /// Returns a shared raw pointer to the vector's buffer.
    ///
    /// It's not safe to write to this pointer except for values
    /// inside of an `UnsafeCell`
    pub fn as_ptr(&self) -> *const T { self.storage.as_ptr() }

    /// Returns a unique raw pointer to the vector's buffer.
    pub fn as_mut_ptr(&mut self) -> *mut T { self.storage.as_mut_ptr() }

    /// Returns the number of elements in the vector
    pub fn len(&self) -> usize { self.len }

    /// Returns the number of elements the vector can hold without reallocating or panicing.
    pub fn capacity(&self) -> usize {
        if core::mem::size_of::<T>() == 0 {
            isize::MAX as usize
        } else {
            self.storage.capacity()
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
        S: raw::StorageInit<T>,
    {
        // Safety
        //
        // The storage only contains initialized data, and we check that
        // the given length is smaller than the capacity
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
    pub fn as_slice(&self) -> &[T] { self }

    /// Extracts a mutable slice containing the entire vector.
    ///
    /// Equivalent to &mut s[..].
    pub fn as_mut_slice(&mut self) -> &mut [T] { self }

    /// Returns the underlying storage
    pub fn storage(&self) -> &S { &self.storage }

    /// Returns the underlying storage
    ///
    /// # Safety
    ///
    /// You must not replace the storage
    pub unsafe fn storage_mut(&mut self) -> &mut S { &mut self.storage }

    /// Returns the remaining spare capacity of the vector as
    /// a `SliceVec<'_, T>`.
    ///
    /// Keep in mind that the `SliceVec<'_, T>` will drop all elements
    /// that you push into it!
    pub fn spare_capacity_mut(&mut self) -> SliceVec<'_, T> {
        // Safety
        //
        // The elements from `len..capacity` are guaranteed to be contain
        // `A::BufferItem`s, as per `Storage`'s safety requirements
        unsafe {
            let len = self.len();
            let cap = self.capacity();
            SliceVec::new(core::slice::from_raw_parts_mut(
                self.storage.as_mut_ptr().add(len).cast(),
                cap.wrapping_sub(len),
            ))
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
        #[cold]
        #[inline(never)]
        fn allocation_failure(additional: usize) -> ! {
            panic!("Tried to allocate: {} more space and failed", additional)
        }

        if self.remaining_capacity() < additional {
            self.storage.reserve(match self.len().checked_add(additional) {
                Some(new_capacity) => new_capacity,
                None => allocation_failure(additional),
            })
        }
    }

    /// Try to reserve enough space for at least `additional` elements, and returns `Err(_)`
    /// if it's not possible to reserve enough space
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), raw::AllocError> {
        if self.remaining_capacity() < additional {
            self.storage
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
            // # Safety
            //
            // * the given length is smaller than the current length, so
            //   all the elements must be initialized
            // * the elements from `len..self.len()` are valid,
            //   and should be dropped
            unsafe {
                self.set_len_unchecked(len);
                let ptr = self.as_mut_ptr().add(len);
                let len = diff;
                core::ptr::drop_in_place(core::slice::from_raw_parts_mut(ptr, len));
            }
        }
    }

    /// Grows the `GenericVec` in-place by additional elements.
    ///
    /// This method requires `T` to implement `Clone`, in order to be able to clone
    /// the passed value. If you need more flexibility (or want to rely on Default instead of `Clone`),
    /// use [`GenericVec::grow_with`].
    ///
    /// # Panic behavor
    ///
    /// If `T::clone` panics, then all added items will be dropped. This is different
    /// from `std`, where on panic, items will stay in the `Vec`. This behavior
    /// is unstable, and may change in the future.
    pub fn grow(&mut self, additional: usize, value: T)
    where
        T: Clone,
    {
        self.reserve(additional);
        // # Safety
        //
        // * we reserved enough space
        unsafe { extension::Extension::grow(self, additional, value) }
    }

    /// Grows the `GenericVec` in-place by additional elements.
    ///
    /// This method uses a closure to create new values on every push.
    /// If you'd rather `Clone` a given value, use `GenericVec::resize`.
    /// If you want to use the `Default` trait to generate values, you
    /// can pass `Default::default` as the second argument.
    ///
    /// # Panic behavor
    ///
    /// If `F` panics, then all added items will be dropped. This is different
    /// from `std`, where on panic, items will stay in the `Vec`. This behavior
    /// is unstable, and may change in the future.
    pub fn grow_with<F>(&mut self, additional: usize, mut value: F)
    where
        F: FnMut() -> T,
    {
        // Safety
        //
        // * we reserve enough space for `additional` elements
        // * we use `spare_capacity_mut` to ensure that the items are dropped,
        //   even on panic
        // * the `ptr` always stays in bounds

        self.reserve(additional);
        let mut writer = self.spare_capacity_mut();

        for _ in 0..additional {
            unsafe {
                writer.push_unchecked(value());
            }
        }

        unsafe {
            let writer = core::mem::ManuallyDrop::new(writer);
            let len = writer.len() + self.len();
            self.set_len_unchecked(len);
        }
    }

    /// Clears the vector, removing all values.
    ///
    /// Note that this method has no effect on the allocated capacity of the vector.
    pub fn clear(&mut self) { self.truncate(0); }

    /// Appends an element to the back of a collection.
    ///
    /// # Panic
    ///
    /// May panic or reallocate if the collection is full
    pub fn push(&mut self, value: T) -> &mut T {
        if self.len() == self.capacity() {
            self.reserve(1);
        }

        // Safety
        //
        // * we reserve enough space for 1 more element
        unsafe { self.push_unchecked(value) }
    }

    /// Appends the array to the back of a collection.
    ///
    /// # Panic
    ///
    /// May panic or reallocate if the collection has less than N elements remaining
    #[cfg(feature = "nightly")]
    pub fn push_array<const N: usize>(&mut self, value: [T; N]) -> &mut [T; N] {
        self.reserve(N);

        // Safety
        //
        // * we reserve enough space for N more elements
        unsafe { self.push_array_unchecked(value) }
    }

    /// Inserts an element at position index within the vector,
    /// shifting all elements after it to the right.
    ///
    /// # Panics
    ///
    /// * May panic or reallocate if the collection is full
    /// * Panics if index > len.
    pub fn insert(&mut self, index: usize, value: T) -> &mut T {
        #[cold]
        #[inline(never)]
        fn insert_fail(index: usize, len: usize) -> ! {
            panic!("Tried to insert at {}, but length is {}", index, len);
        }

        if index > self.len() {
            insert_fail(index, self.len())
        }

        if self.is_full() {
            self.reserve(1);
        }

        // Safety
        //
        // * we reserve enough space for 1 more element
        // * we verify that index is in bounds
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
    pub fn insert_array<const N: usize>(&mut self, index: usize, value: [T; N]) -> &mut [T; N] {
        #[cold]
        #[inline(never)]
        fn insert_array_fail(index: usize, size: usize, len: usize) -> ! {
            panic!(
                "Tried to insert array of length {} at {}, but length is {}",
                size, index, len
            );
        }

        if index > self.len() {
            insert_array_fail(index, N, self.len())
        }

        self.reserve(N);

        // Safety
        //
        // * we reserve enough space for N more elements
        // * we verify that index is in bounds
        unsafe { self.insert_array_unchecked(index, value) }
    }

    /// Removes the last element from a vector and returns it
    ///
    /// # Panics
    ///
    /// Panics if the collection is empty
    pub fn pop(&mut self) -> T {
        #[cold]
        #[inline(never)]
        fn pop_fail() -> ! {
            panic!("Tried to pop an element from an empty vector",);
        }

        if self.is_empty() {
            pop_fail()
        }

        // Safety
        //
        // * we verify we are not empty
        unsafe { self.pop_unchecked() }
    }

    /// Removes the last `N` elements from a vector and returns it
    ///
    /// # Panics
    ///
    /// Panics if the collection contains less than `N` elements in it
    #[cfg(feature = "nightly")]
    pub fn pop_array<const N: usize>(&mut self) -> [T; N] {
        #[cold]
        #[inline(never)]
        fn pop_array_fail(size: usize, len: usize) -> ! {
            panic!("Tried to pop an array of size {}, a vector of length {}", size, len);
        }

        if self.len() < N {
            pop_array_fail(N, self.len())
        }

        // Safety
        //
        // * we verify we have at least N elements
        unsafe { self.pop_array_unchecked() }
    }

    /// Removes and returns the element at position index within the vector,
    /// shifting all elements after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> T {
        #[cold]
        #[inline(never)]
        fn remove_fail(index: usize, len: usize) -> ! {
            panic!("Tried to remove an element at {}, but length is {}", index, len);
        }

        if index > self.len() {
            remove_fail(index, self.len())
        }

        // Safety
        //
        // * we verify that the index is in bounds
        unsafe { self.remove_unchecked(index) }
    }

    /// Removes and returns `N` elements at position index within the vector,
    /// shifting all elements after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds or if `index + N > len()`
    #[cfg(feature = "nightly")]
    pub fn remove_array<const N: usize>(&mut self, index: usize) -> [T; N] {
        #[cold]
        #[inline(never)]
        fn remove_array_fail(index: usize, size: usize, len: usize) -> ! {
            panic!(
                "Tried to remove an array length {} at {}, but length is {}",
                size, index, len
            );
        }

        if self.len() < index || self.len().wrapping_sub(index) < N {
            remove_array_fail(index, N, self.len())
        }

        // Safety
        //
        // * we verify that the index is in bounds
        // * we verify that there are at least `N` elements
        //   after the index
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
    pub fn swap_remove(&mut self, index: usize) -> T {
        #[cold]
        #[inline(never)]
        fn swap_remove_fail(index: usize, len: usize) -> ! {
            panic!("Tried to remove an element at {}, but length is {}", index, len);
        }

        if index > self.len() {
            swap_remove_fail(index, self.len())
        }

        // Safety
        //
        // * we verify that the index is in bounds
        unsafe { self.swap_remove_unchecked(index) }
    }

    /// Tries to append an element to the back of a collection.
    /// Returns the `Err(value)` if the collection is full
    ///
    /// Guaranteed to not panic/abort/allocate
    pub fn try_push(&mut self, value: T) -> Result<&mut T, T> {
        if self.is_full() {
            Err(value)
        } else {
            // Safety
            //
            // * we reserve enough space for 1 more element
            unsafe { Ok(self.push_unchecked(value)) }
        }
    }

    /// Tries to append an array to the back of a collection.
    /// Returns the `Err(value)` if the collection doesn't have enough remaining capacity
    /// to hold `N` elements.
    ///
    /// Guaranteed to not panic/abort/allocate
    #[cfg(feature = "nightly")]
    pub fn try_push_array<const N: usize>(&mut self, value: [T; N]) -> Result<&mut [T; N], [T; N]> {
        if self.remaining_capacity() < N {
            Err(value)
        } else {
            // Safety
            //
            // * we reserve enough space for N more elements
            unsafe { Ok(self.push_array_unchecked(value)) }
        }
    }

    /// Inserts an element at position index within the vector,
    /// shifting all elements after it to the right.
    /// Returns the `Err(value)` if the collection is full or index is out of bounds
    ///
    /// Guaranteed to not panic/abort/allocate
    pub fn try_insert(&mut self, index: usize, value: T) -> Result<&mut T, T> {
        if self.is_full() || index > self.len() {
            Err(value)
        } else {
            // Safety
            //
            // * we reserve enough space for 1 more element
            // * we verify that index is in bounds
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
    pub fn try_insert_array<const N: usize>(&mut self, index: usize, value: [T; N]) -> Result<&mut [T; N], [T; N]> {
        if self.capacity().wrapping_sub(self.len()) < N || index > self.len() {
            Err(value)
        } else {
            // Safety
            //
            // * we reserve enough space for N more elements
            // * we verify that index is in bounds
            unsafe { Ok(self.insert_array_unchecked(index, value)) }
        }
    }

    /// Removes the last element from a vector and returns it,
    /// Returns `None` if the collection is empty
    ///
    /// Guaranteed to not panic/abort/allocate
    pub fn try_pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            // Safety
            //
            // * we verify we are not empty
            unsafe { Some(self.pop_unchecked()) }
        }
    }

    /// Removes the last `N` elements from a vector and returns it,
    /// Returns `None` if the collection is has less than N elements
    ///
    /// Guaranteed to not panic/abort/allocate
    #[cfg(feature = "nightly")]
    pub fn try_pop_array<const N: usize>(&mut self) -> Option<[T; N]> {
        if self.is_empty() {
            None
        } else {
            // Safety
            //
            // * we verify we have at least N elements
            unsafe { Some(self.pop_array_unchecked()) }
        }
    }

    /// Removes and returns the element at position index within the vector,
    /// shifting all elements after it to the left.
    /// Returns `None` if collection is empty or `index` is out of bounds.
    ///
    /// Guaranteed to not panic/abort/allocate
    pub fn try_remove(&mut self, index: usize) -> Option<T> {
        if self.len() < index {
            None
        } else {
            // Safety
            //
            // * we verify that the index is in bounds
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
    pub fn try_remove_array<const N: usize>(&mut self, index: usize) -> Option<[T; N]> {
        if self.len() < index || self.len().wrapping_sub(index) < N {
            None
        } else {
            // Safety
            //
            // * we verify that the index is in bounds
            // * we verify that there are at least `N` elements
            //   after the index
            unsafe { Some(self.remove_array_unchecked(index)) }
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
    pub fn try_swap_remove(&mut self, index: usize) -> Option<T> {
        if index < self.len() {
            // Safety
            //
            // * we verify that the index is in bounds
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
    pub unsafe fn push_unchecked(&mut self, value: T) -> &mut T {
        if Some(0) == S::CONST_CAPACITY {
            panic!("Tried to push an element into a zero-capacity vector!")
        }

        debug_assert_ne!(
            self.len(),
            self.capacity(),
            "Tried to `push_unchecked` past capacity! This is UB in release mode"
        );

        // Safety
        //
        // the collection isn't full, so `ptr.add(len)` is valid to write
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
    pub unsafe fn push_array_unchecked<const N: usize>(&mut self, value: [T; N]) -> &mut [T; N] {
        match S::CONST_CAPACITY {
            Some(n) if n < N => {
                panic!("Tried to push an array larger than the maximum capacity of the vector!")
            }
            _ => (),
        }

        // Safety
        //
        // the collection has at least N remaining elements of capacity left,
        // so `ptr.add(len)` is valid to write `N` elements
        unsafe {
            let len = self.len();
            self.set_len_unchecked(len.wrapping_add(N));
            let ptr = self.as_mut_ptr();
            let out = ptr.add(len) as *mut [T; N];
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
    pub unsafe fn insert_unchecked(&mut self, index: usize, value: T) -> &mut T {
        unsafe {
            if Some(0) == S::CONST_CAPACITY {
                panic!("Tried to insert an element into a zero-capacity vector!")
            }

            // Safety
            //
            // * the index is in bounds
            // * the collection is't full so `ptr.add(len)` is valid to write 1 element
            let len = self.len();
            self.set_len_unchecked(len.wrapping_add(1));
            let ptr = self.storage.as_mut_ptr().add(index);
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
    pub unsafe fn insert_array_unchecked<const N: usize>(&mut self, index: usize, value: [T; N]) -> &mut [T; N] {
        match S::CONST_CAPACITY {
            Some(n) if n < N => {
                panic!("Tried to push an array larger than the maximum capacity of the vector!")
            }
            _ => (),
        }

        // Safety
        //
        // * the index is in bounds
        // * the collection has at least N remaining elements of capacity left,
        //   so `ptr.add(len)` is valid to write `N` elements
        unsafe {
            let len = self.len();
            self.set_len_unchecked(len.wrapping_add(N));
            let ptr = self.as_mut_ptr();
            let dist = len.wrapping_sub(index);

            let out = ptr.add(index);
            out.add(N).copy_from(out, dist);
            let out = out as *mut [T; N];
            out.write(value);
            &mut *out
        }
    }

    /// Removes the last element from a vector and returns it
    ///
    /// # Safety
    ///
    /// the collection must not be empty
    pub unsafe fn pop_unchecked(&mut self) -> T {
        if Some(0) == S::CONST_CAPACITY {
            panic!("Tried to remove an element from a zero-capacity vector!")
        }

        let len = self.len();
        debug_assert_ne!(
            len, 0,
            "Tried to `pop_unchecked` an empty array vec! This is UB in release mode"
        );

        // Safety
        //
        // * the collection isn't empty, so `ptr.add(len - 1)` is valid to read
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
    pub unsafe fn pop_array_unchecked<const N: usize>(&mut self) -> [T; N] {
        match S::CONST_CAPACITY {
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
        // Safety
        //
        // * the collection has at least `N` elements, so `ptr.add(len - N)` is valid to read `N` elements
        unsafe {
            let len = len.wrapping_sub(N);
            self.set_len_unchecked(len);
            self.as_mut_ptr().add(len).cast::<[T; N]>().read()
        }
    }

    /// Removes and returns the element at position index within the vector,
    /// shifting all elements after it to the left.
    ///
    /// # Safety
    ///
    /// the collection must not be empty, and
    /// index must be in bounds
    pub unsafe fn remove_unchecked(&mut self, index: usize) -> T {
        if Some(0) == S::CONST_CAPACITY {
            panic!("Tried to remove an element from a zero-capacity vector!")
        }

        let len = self.len();

        debug_assert!(
            index <= len,
            "Tried to remove an element at index {} from a {} length vector! This is UB in release mode",
            index,
            len,
        );

        // Safety
        //
        // * the index is in bounds
        // * the collection isn't empty, so `ptr.add(len - index - 1)` is valid to read
        unsafe {
            self.set_len_unchecked(len.wrapping_sub(1));
            let ptr = self.storage.as_mut_ptr().add(index);
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
    pub unsafe fn remove_array_unchecked<const N: usize>(&mut self, index: usize) -> [T; N] {
        match S::CONST_CAPACITY {
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

        // Safety
        //
        // * the index is in bounds
        // * the collection isn't empty, so `ptr.add(len - index - N)` is valid to read `N` elements
        unsafe {
            self.set_len_unchecked(len.wrapping_sub(N));
            let ptr = self.as_mut_ptr().add(index);
            let value = ptr.cast::<[T; N]>().read();
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
    pub unsafe fn swap_remove_unchecked(&mut self, index: usize) -> T {
        if Some(0) == S::CONST_CAPACITY {
            panic!("Tried to remove an element from a zero-capacity vector!")
        }

        // Safety
        //
        // * the index is in bounds
        // * the collection isn't empty
        unsafe {
            let len = self.len();
            self.set_len_unchecked(len.wrapping_sub(1));
            let ptr = self.storage.as_mut_ptr();
            let at = ptr.add(index);
            let end = ptr.add(len.wrapping_sub(1));
            let value = at.read();
            at.copy_from(end, 1);
            value
        }
    }

    /// Splits the collection into two at the given index.
    ///
    /// Returns a newly allocated vector containing the elements in the range `[at, len)`.
    /// After the call, the original vector will be left containing the elements `[0, at)`
    /// with its previous capacity unchanged.
    pub fn split_off<B>(&mut self, index: usize) -> GenericVec<T, B>
    where
        B: raw::StorageWithCapacity<T>,
    {
        assert!(
            index <= self.len(),
            "Tried to split at index {}, but length is {}",
            index,
            self.len()
        );

        let mut vec = GenericVec::<T, B>::__with_capacity__const_capacity_checked(
            self.len().wrapping_sub(index),
            S::CONST_CAPACITY,
        );

        self.split_off_into(index, &mut vec);

        vec
    }

    /// Splits the collection into two at the given index.
    ///
    /// Appends the elements from the range `[at, len)` to `other`.
    /// After the call, the original vector will be left containing the elements `[0, at)`
    /// with its previous capacity unchanged.
    pub fn split_off_into<B>(&mut self, index: usize, other: &mut GenericVec<T, B>)
    where
        B: raw::Storage<T> + ?Sized,
    {
        assert!(
            index <= self.len(),
            "Tried to split at index {}, but length is {}",
            index,
            self.len()
        );

        unsafe {
            // Safety
            //
            // * the index is in bounds
            // * other has reserved enough space
            // * we ignore all elements after index
            let slice = self.get_unchecked(index..);
            other.reserve(slice.len());
            other.extend_from_slice_unchecked(slice);
            self.set_len_unchecked(index);
        }
    }

    /// Convert the backing storage type, and moves all the elements in `self` to the new vector
    pub fn convert<B: raw::StorageWithCapacity<T>>(mut self) -> GenericVec<T, B>
    where
        S: Sized,
    {
        self.split_off(0)
    }

    /// Creates a raw drain that can be used to remove elements in the specified range.
    /// Usage of [`RawDrain`] is `unsafe` because it doesn't do any checks because is
    /// meant to be a low level tool to implement fancier iterators, like [`GenericVec::drain`],
    /// [`GenericVec::drain_filter`], or [`GenericVec::splice`].
    ///
    /// # Panic
    ///
    /// Panics if the starting point is greater than the end point or if the end point is greater than the length of the vector.
    #[inline]
    pub fn raw_drain<R>(&mut self, range: R) -> RawDrain<'_, T, S>
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
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, T, S>
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
    pub fn drain_filter<R, F>(&mut self, range: R, f: F) -> DrainFilter<'_, T, S, F>
    where
        R: RangeBounds<usize>,
        F: FnMut(&mut T) -> bool,
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
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, T, S, I::IntoIter>
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
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
        F: FnMut(&mut T) -> bool,
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
    pub unsafe fn extend_from_slice_unchecked(&mut self, slice: &[T]) {
        debug_assert!(
            self.remaining_capacity() >= slice.len(),
            "Not enough capacity to hold the slice"
        );

        unsafe {
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
    ///
    /// # Panic behavor
    ///
    /// If `T::clone` panics, then all added items will be dropped. This is different
    /// from `std`, where on panic, items will stay in the `Vec`. This behavior
    /// is unstable, and may change in the future.
    pub fn extend_from_slice(&mut self, slice: &[T])
    where
        T: Clone,
    {
        self.reserve(self.len());

        // Safety
        //
        // We reserved enough space
        unsafe { extension::Extension::extend_from_slice(self, slice) }
    }

    /// Replaces all of the current elements with the ones in the slice
    ///
    /// equivalent to the following
    ///
    /// ```rust
    /// # let slice = [];
    /// # let mut buffer = generic_vec::uninit_array!(0);
    /// # let mut vec = generic_vec::SliceVec::<()>::new(&mut buffer);
    /// vec.clear();
    /// vec.extend_from_slice(&slice);
    /// ```
    ///
    /// # Panic
    ///
    /// May try to panic/reallocate if there is not enough capacity for the slice
    pub fn clone_from(&mut self, source: &[T])
    where
        T: Clone,
    {
        let source = source.as_ref();
        // If the `self` is longer than `source`, remove excess
        self.truncate(source.len());

        // `self` is now at most the same length as `source`
        //
        // * `init.len() == self.len()`
        // * tail is the rest of the `source`, in the case
        //     that `self` is smaller than `source`
        let (init, tail) = source.split_at(self.len());

        // Clone in the beginning, using `slice::clone_from_slice`
        self.clone_from_slice(init);

        // Append the remaining elements
        self.extend_from_slice(tail);
    }
}
