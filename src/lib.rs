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
        const_raw_ptr_deref,
        doc_cfg,
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
//! ### [`SliceVec`] and [`InitSliceVec`]
//!
//! [`SliceVec`] and [`InitSliceVec`] are pretty similar, you give them a slice
//! buffer, and they store all of thier values in that buffer. But have three major
//! differences between them.
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
//! ### [`TypeVec`]
//!
//! [`TypeVec`] is an owned buffer. You can use like so:
//!
//! ```rust
//! use generic_vec::{TypeVec, gvec};
//! let mut vec: TypeVec<u32, [u32; 4]> = gvec![1, 2, 3, 4];
//!
//! assert_eq!(vec, [1, 2, 3, 4]);
//!
//! vec.try_push(5).expect_err("Tried to push past capacity!");
//! ```
//!
//! The second parameter specifies the buffer type, this can be any type
//! you want. Only the size of the type matters. There is also a defaulted
//! third parameter, but you should only use that if you know what you are
//! doing, and after reading the docs for [`UninitBuffer`](raw::UninitBuffer).
//!
//! As a neat side-effect of this framework, you can also get an efficient
//! [`GenericVec`] for zero-sized types, just a `usize` in size! This feature
//! can be on stable `no_std`.
//!
//! ### [`ArrayVec`](type@ArrayVec) and [`InitArrayVec`](type@InitArrayVec)
//!
//! [`ArrayVec`](type@ArrayVec) and [`InitArrayVec`](type@InitArrayVec)
//! are just like the slice versions, but since they own their data,
//! they can be freely moved around, unconstrained. You can also create
//! a new [`ArrayVec`](type@ArrayVec) without passing in an existing buffer,
//! unlike the slice versions.
//!
//! On stable, you can use the [`ArrayVec`](macro@ArrayVec) or
//! [`InitArrayVec`](macro@InitArrayVec) to construct the type. On `nightly`,
//! you can use the type aliases [`ArrayVec`](type@ArrayVec) and
//! [`InitArrayVec`](type@InitArrayVec). The macros will be deprecated once
//! `min_const_generics` hits stable.
//!
//! The only limitation on stable is that you can only use [`InitArrayVec`](type@InitArrayVec)
//! capacity up to 32. i.e. `InitArrayVec![i32; 33]` doesn't work. `ArrayVec` does not suffer
//! from this limitation because it is built atop [`TypeVec`].
//!
//! ```rust
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
//! The distinction between [`ArrayVec`](type@ArrayVec) and [`InitArrayVec`](type@InitArrayVec)
//! is identical to their slice counterparts.
//!
//! ### [`ZSVec`]
//!
//! ```rust
//! use generic_vec::ZSVec;
//!
//! struct MyType;
//!
//! let mut vec = ZSVec::new();
//!
//! vec.push(MyType);
//! vec.push(MyType);
//! vec.push(MyType);
//!
//! assert_eq!(vec.len(), 3);
//! assert_eq!(std::mem::size_of_val(&vec), std::mem::size_of::<usize>());
//! ```
//!
//! ## `alloc`
//!
//! A [`HeapVec`] is just [`Vec`], but built atop [`GenericVec`],
//! meaning you get all the features of [`GenericVec`] for free! But this
//! requries either the `alloc` or `std` feature to be enabled.
//!
//! ```rust
//! use generic_vec::{HeapVec, gvec};
//! let mut vec: HeapVec<u32> = gvec![1, 2, 3, 4];
//! assert_eq!(vec.capacity(), 4);
//! vec.extend(&[5, 6, 7, 8]);
//!
//! assert_eq!(vec, [1, 2, 3, 4, 5, 6, 7, 8]);
//!
//! vec.try_push(5).expect_err("Tried to push past capacity!");
//! ```
//!
//! ## `nightly`
//!
//! On `nightly`
//! * the restriction on [`InitArrayVec`](type@InitArrayVec)'s length goes away.
//! * many functions/methods become `const fn`s
//! * a number of optimizations are enabled
//! * some diagnostics become better
//!
//! Note on the documentation: if the feature exists on [`Vec`], then the documentation
//! is either exactly the same as [`Vec`] or slightly adapted to better fit [`GenericVec`]
//!
//! Note on implementation: large parts of the implementation came straight from [`Vec`]
//! so thanks for the amazing reference `std`!

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
mod slice;

pub mod iter;
pub mod raw;

use raw::Storage;

#[doc(hidden)]
pub use core;

/// A heap backed vector with a growable capacity
#[cfg(any(doc, all(feature = "alloc", feature = "nightly")))]
#[cfg_attr(doc, doc(cfg(all(feature = "alloc", feature = "nightly"))))]
pub type HeapVec<T, A = std::alloc::Global> = GenericVec<T, raw::Heap<T, A>>;

/// A heap backed vector with a growable capacity
#[cfg(all(not(doc), feature = "alloc", not(feature = "nightly")))]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
pub type HeapVec<T> = GenericVec<T, raw::Heap<T>>;

/// An array backed vector backed by potentially uninitialized memory
#[cfg(any(doc, feature = "nightly"))]
#[cfg_attr(doc, doc(cfg(feature = "nightly")))]
pub type ArrayVec<T, const N: usize> = TypeVec<T, [T; N]>;
/// An slice backed vector backed by potentially uninitialized memory
pub type SliceVec<'a, T> = GenericVec<T, &'a mut raw::UninitSlice<T>>;

/// An array backed vector backed by initialized memory
#[cfg(any(doc, feature = "nightly"))]
#[cfg_attr(doc, doc(cfg(feature = "nightly")))]
pub type InitArrayVec<T, const N: usize> = GenericVec<T, [T; N]>;
/// An slice backed vector backed by initialized memory
pub type InitSliceVec<'a, T> = GenericVec<T, &'a mut [T]>;
/// A counter vector that can only store zero-sized types
pub type ZSVec<T> = GenericVec<T, raw::ZeroSized<T>>;
/// An type based vector backed by uninitialized memory with the same layout as `B`
///
/// see: [`UninitBuffer`](raw::UninitBuffer) for details
pub type TypeVec<T, B, A = T> = GenericVec<T, raw::UninitBuffer<B, A>>;

#[doc(hidden)]
pub mod macros {
    pub use core::mem::MaybeUninit;
    impl<T> Uninit for T {}
    pub trait Uninit: Sized {
        const UNINIT: MaybeUninit<Self> = MaybeUninit::uninit();
    }
}

/// Create a new generic vector
///
/// Because this can create any generic vector, you will likely
/// need to add some type annotations when you use it,
///
/// ```rust
/// # use generic_vec::{gvec, ArrayVec};
/// let x: ArrayVec<i32, 2> = gvec![0, 1];
/// assert_eq!(x, [0, 1]);
/// ```
#[macro_export]
#[cfg(feature = "nightly")]
macro_rules! gvec {
    ($expr:expr; $n:expr) => {{
        let len = $n;
        let mut vec = $crate::GenericVec::with_capacity(len);
        vec.grow(len, $expr);
        vec
    }};
    ($($expr:expr),*) => {{
        let expr = [$($expr),*];
        let mut vec = $crate::GenericVec::with_capacity(expr.len());
        unsafe { vec.push_array_unchecked(expr); }
        vec
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! count {
    () => { 0 };
    ($($a:tt $b:tt)*) => { $crate::count!($($a)*) << 1 };
    ($c:tt $($a:tt $b:tt)*) => { ($crate::count!($($a)*) << 1) | 1 };
}

/// Create a new generic vector
///
/// Because this can create any generic vector, you will likely
/// need to add some type annotations when you use it,
///
/// ```rust
/// # use generic_vec::{gvec, TypeVec};
/// let x: TypeVec<i32, [i32; 4]> = gvec![1, 2, 3, 4];
/// assert_eq!(x, [1, 2, 3, 4]);
/// ```
#[macro_export]
#[cfg(not(feature = "nightly"))]
macro_rules! gvec {
    ($expr:expr; $n:expr) => {{
        let len = $n;
        let mut vec = $crate::GenericVec::with_capacity(len);
        vec.grow(len, $expr);
        vec
    }};
    ($($expr:expr),*) => {{
        let mut vec = $crate::GenericVec::with_capacity($crate::count!($(($expr))*));
        unsafe {
            $(vec.push_unchecked($expr);)*
        }
        vec
    }};
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

/// Save the changes to [`GenericVec::spare_capacity_mut`]
///
/// $orig - a mutable reference to a [`GenericVec`]
/// $spare - the [`SliceVec`] that was obtained from [`$orig.spare_capacity_mut()`]
///
/// # Safety
///
/// `$spare` should be the [`SliceVec`] returned by `$orig.spare_capacity_mut()`
#[macro_export]
macro_rules! save_spare {
    ($spare:expr, $orig:expr) => {{
        let spare: $crate::SliceVec<_> = $spare;
        let spare = $crate::core::mem::ManuallyDrop::new(spare);
        let len = spare.len();
        let ptr = spare.as_ptr();
        let orig: &mut $crate::GenericVec<_, _> = $orig;
        $crate::validate_spare(ptr, orig);
        let len = len + orig.len();
        $orig.set_len_unchecked(len);
    }};
}

#[doc(hidden)]
pub fn validate_spare<T>(spare_ptr: *const T, orig: &[T]) {
    debug_assert!(
        unsafe { orig.as_ptr().add(orig.len()) == spare_ptr },
        "Tried to use `save_spare!` with a `SliceVec` that was not obtained from `GenricVec::spare_capacity_mut`. \
         This is undefined behavior on release mode!"
    )
}

/// An array backed vector backed by potentially uninitialized memory
///
/// On `nightly`, it's prefered to use the [`ArrayVec`](type@ArrayVec) type alias
#[macro_export]
macro_rules! ArrayVec {
    ($type:ty; $len:expr) => {
        $crate::GenericVec<$type, $crate::raw::UninitBuffer<[$type; $len]>>
    };
}

/// An array backed vector backed by initialized memory
///
/// On `nightly`, it's prefered to use the [`InitArrayVec`](type@InitArrayVec) type alias
#[macro_export]
macro_rules! InitArrayVec {
    ($type:ty; $len:expr) => {
        $crate::GenericVec<$type, [$type; $len]>
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
    ///
    /// ```rust
    /// use generic_vec::{GenericVec, raw::ZeroSized};
    /// let vec = GenericVec::with_storage(ZeroSized::<[i32; 0]>::NEW);
    /// ```
    pub fn with_storage(storage: S) -> Self {
        assert!(S::IS_ALIGNED, "The storage must be aligned to `T`");
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
    ///
    /// Note: this is only const with the `nightly` feature enabled
    pub const fn with_storage(storage: S) -> Self {
        assert!(S::IS_ALIGNED, "The storage must be aligned to `T`");
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

impl<T, B> TypeVec<T, B, T> {
    /// Create a new [`TypeVec`]
    pub const fn new() -> Self { Self::with_align() }
}

impl<T, B, A> TypeVec<T, B, A> {
    /// Create a new [`TypeVec`] with the given alignment type
    pub const fn with_align() -> Self {
        #[cfg(not(feature = "nightly"))]
        #[allow(clippy::no_effect)]
        {
            [()][(!<raw::UninitBuffer<B, A> as raw::Storage<T>>::IS_ALIGNED) as usize];
        }
        #[cfg(feature = "nightly")]
        {
            assert!(
                <raw::UninitBuffer<B, A> as raw::Storage<T>>::IS_ALIGNED,
                "Your buffer is not sufficiently aligned"
            )
        }

        Self {
            len: 0,
            storage: raw::UninitBuffer::uninit(),
            mark: PhantomData,
        }
    }
}

#[cfg(any(doc, feature = "nightly"))]
#[cfg_attr(doc, doc(cfg(feature = "nightly")))]
impl<T, const N: usize> ArrayVec<T, N> {
    /// Create a new full `ArrayVec`
    pub const fn from_array(array: [T; N]) -> Self {
        Self {
            len: 0,
            mark: PhantomData,
            storage: raw::UninitBuffer::new(array),
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
        unsafe { Storage::<[T; N]>::as_ptr(&this.storage).read() }
    }
}

#[cfg(feature = "nightly")]
#[cfg_attr(doc, doc(cfg(feature = "nightly")))]
impl<T: Copy, const N: usize> InitArrayVec<T, N> {
    /// Create a new full `InitArrayVec`
    pub fn new(storage: [T; N]) -> Self {
        Self {
            len: N,
            mark: PhantomData,
            storage,
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
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

#[cfg(any(doc, all(feature = "nightly", feature = "alloc")))]
#[cfg_attr(doc, doc(cfg(all(feature = "nightly", feature = "alloc"))))]
impl<T, A: std::alloc::Allocator> HeapVec<T, A> {
    /// Create a new empty `HeapVec` with the given allocator
    pub fn with_alloc(alloc: A) -> Self { Self::with_storage(raw::Heap::with_alloc(alloc)) }
}

#[cfg(any(doc, not(feature = "nightly")))]
impl<'a, T> SliceVec<'a, T> {
    /// Create a new empty `SliceVec`
    pub fn new(slice: &'a mut [MaybeUninit<T>]) -> Self { Self::with_storage(raw::UninitSlice::from_mut(slice)) }
}

#[cfg(any(doc, feature = "nightly"))]
impl<'a, T> SliceVec<'a, T> {
    /// Create a new empty `SliceVec`
    ///
    /// Note: this is only const with the `nightly` feature enabled
    pub const fn new(slice: &'a mut [MaybeUninit<T>]) -> Self { Self::with_storage(raw::UninitSlice::from_mut(slice)) }
}

#[cfg(any(doc, not(feature = "nightly")))]
impl<'a, T: Copy> InitSliceVec<'a, T> {
    /// Create a new full `InitSliceVec`
    pub fn new(storage: &'a mut [T]) -> Self {
        Self {
            len: storage.len(),
            storage,
            mark: PhantomData,
        }
    }
}

#[cfg(feature = "nightly")]
impl<'a, T: Copy> InitSliceVec<'a, T> {
    /// Create a new full `InitSliceVec`
    ///
    /// Note: this is only const with the `nightly` feature enabled
    pub const fn new(storage: &'a mut [T]) -> Self {
        Self {
            len: storage.len(),
            storage,
            mark: PhantomData,
        }
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
    #[cfg(not(feature = "nightly"))]
    pub unsafe fn from_raw_parts(len: usize, storage: S) -> Self {
        Self {
            storage,
            len,
            mark: PhantomData,
        }
    }
}

#[cfg(feature = "nightly")]
impl<T, S: Storage<T>> GenericVec<T, S> {
    /// Create a `GenericVec` from a length-storage pair
    ///
    /// Note: this is only const with the `nightly` feature enabled
    ///
    /// # Safety
    ///
    /// the length must be less than `raw.capacity()` and
    /// all elements in the range `0..length`, must be initialized
    ///
    /// # Panic
    ///
    /// If the given storage cannot hold type `T`, then this method will panic
    pub const unsafe fn from_raw_parts(len: usize, storage: S) -> Self {
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
    /// a [`SliceVec<'_, T>`](SliceVec).
    ///
    /// Keep in mind that the [`SliceVec<'_, T>`](SliceVec) will drop all elements
    /// that you push into it when it goes out of scope! If you want
    /// these modifications to persist then you should use [`save_spare`]
    /// to persist these writes.
    ///
    /// ```
    /// let mut vec = generic_vec::TypeVec::<i32, [i32; 16]>::new();
    ///
    /// let mut spare = vec.spare_capacity_mut();
    /// spare.push(0);
    /// spare.push(2);
    /// drop(spare);
    /// assert_eq!(vec, []);
    ///
    /// let mut spare = vec.spare_capacity_mut();
    /// spare.push(0);
    /// spare.push(2);
    /// unsafe { generic_vec::save_spare!(spare, &mut vec) }
    /// assert_eq!(vec, [0, 2]);
    /// ```
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
    pub fn try_reserve(&mut self, additional: usize) -> bool {
        if self.remaining_capacity() < additional {
            match self.len().checked_add(additional) {
                Some(new_capacity) => self.storage.try_reserve(new_capacity),
                None => false,
            }
        } else {
            true
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
    /// # Panic
    ///
    /// May panic or reallocate if the collection is full
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
    /// # Panic
    ///
    /// May panic or reallocate if the collection is full
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
            save_spare!(writer, self);
        }
    }

    /// Resizes the [`GenericVec`] in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the [`GenericVec`] is extended by the difference,
    /// with each additional slot filled with value. If `new_len` is less than `len`,
    /// the [`GenericVec`] is simply truncated.
    ///
    /// If you know that `new_len` is larger than `len`, then use [`GenericVec::grow`]
    ///
    /// If you know that `new_len` is less than `len`, then use [`GenericVec::truncate`]
    ///
    /// This method requires `T` to implement `Clone`, in order to be able to clone
    /// the passed value. If you need more flexibility (or want to rely on Default
    /// instead of `Clone`), use [`GenericVec::resize_with`].
    ///
    /// # Panic
    ///
    /// May panic or reallocate if the collection is full
    ///
    /// # Panic behavor
    ///
    /// If `F` panics, then all added items will be dropped. This is different
    /// from `std`, where on panic, items will stay in the `Vec`. This behavior
    /// is unstable, and may change in the future.
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        match new_len.checked_sub(self.len()) {
            Some(0) => (),
            Some(additional) => self.grow(additional, value),
            None => self.truncate(new_len),
        }
    }

    /// Resizes the [`GenericVec`] in-place so that len is equal to new_len.
    ///
    /// If `new_len` is greater than `len`, the [`GenericVec`] is extended by the
    /// difference, with each additional slot filled with the result of calling
    /// the closure `f`. The return values from `f` will end up in the [`GenericVec`]
    /// in the order they have been generated.
    ///
    /// If `new_len` is less than `len`, the [`GenericVec`] is simply truncated.
    ///
    /// If you know that `new_len` is larger than `len`, then use [`GenericVec::grow_with`]
    ///
    /// If you know that `new_len` is less than `len`, then use [`GenericVec::truncate`]
    ///
    /// This method uses a closure to create new values on every push. If you'd
    /// rather [`Clone`] a given value, use [`GenericVec::resize`]. If you want to
    /// use the [`Default`] trait to generate values, you can pass [`Default::default`]
    /// as the second argument.
    ///
    /// # Panic
    ///
    /// May panic or reallocate if the collection is full
    ///
    /// # Panic behavor
    ///
    /// If `F` panics, then all added items will be dropped. This is different
    /// from `std`, where on panic, items will stay in the `Vec`. This behavior
    /// is unstable, and may change in the future.
    pub fn resize_with<F: FnMut() -> T>(&mut self, new_len: usize, value: F) {
        match new_len.checked_sub(self.len()) {
            Some(0) => (),
            Some(additional) => self.grow_with(additional, value),
            None => self.truncate(new_len),
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
    #[cfg(any(doc, feature = "nightly"))]
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
    #[cfg(any(doc, feature = "nightly"))]
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
    #[cfg(any(doc, feature = "nightly"))]
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
    #[cfg(any(doc, feature = "nightly"))]
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
    #[cfg(any(doc, feature = "nightly"))]
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
    #[cfg(any(doc, feature = "nightly"))]
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
    #[cfg(any(doc, feature = "nightly"))]
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
    #[cfg(any(doc, feature = "nightly"))]
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
    #[cfg(any(doc, feature = "nightly"))]
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
    #[cfg(any(doc, feature = "nightly"))]
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
    #[cfg(any(doc, feature = "nightly"))]
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
    #[cfg(any(doc, feature = "nightly"))]
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
    ///
    /// ```rust
    /// # use generic_vec::{gvec, SliceVec, uninit_array};
    /// # let mut vec_buf = uninit_array!(3);
    /// # let mut vec2_buf = uninit_array!(5);
    /// # let mut vec: SliceVec<_> = SliceVec::new(&mut vec_buf); vec.extend([1, 2, 3].iter().copied());
    /// # let mut vec2: SliceVec<_> = SliceVec::new(&mut vec2_buf); vec2.extend([4, 5, 6].iter().copied());
    /// assert_eq!(vec, [1, 2, 3]);
    /// assert_eq!(vec2, [4, 5, 6]);
    /// vec.split_off_into(1, &mut vec2);
    /// assert_eq!(vec, [1]);
    /// assert_eq!(vec2, [4, 5, 6, 2, 3]);
    /// ```
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
    ///
    /// ```rust
    /// # use generic_vec::{gvec, SliceVec, uninit_array};
    /// # let mut vec_buf = uninit_array!(3);
    /// # let mut vec2_buf = uninit_array!(5);
    /// # let mut vec: SliceVec<_> = SliceVec::new(&mut vec_buf); vec.extend([1, 2, 3].iter().copied());
    /// # let mut vec2: SliceVec<_> = SliceVec::new(&mut vec2_buf); vec2.extend([4, 5, 6].iter().copied());
    /// assert_eq!(vec, [1, 2, 3]);
    /// assert_eq!(vec2, [4, 5, 6]);
    /// vec.split_off_into(1, &mut vec2);
    /// assert_eq!(vec, [1]);
    /// assert_eq!(vec2, [4, 5, 6, 2, 3]);
    /// ```
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

    /// Moves all the elements of `other` into `Self`, leaving `other` empty.
    ///
    /// Does not change the capacity of either collection.
    ///
    /// ```rust
    /// # use generic_vec::{gvec, SliceVec, uninit_array};
    /// # let mut vec_buf = uninit_array!(6);
    /// # let mut vec2_buf = uninit_array!(3);
    /// # let mut vec: SliceVec<_> = SliceVec::new(&mut vec_buf); vec.extend([1, 2, 3].iter().copied());
    /// # let mut vec2: SliceVec<_> = SliceVec::new(&mut vec2_buf); vec2.extend([4, 5, 6].iter().copied());
    /// assert_eq!(vec, [1, 2, 3]);
    /// assert_eq!(vec2, [4, 5, 6]);
    /// vec.append(&mut vec2);
    /// assert_eq!(vec, [1, 2, 3, 4, 5, 6]);
    /// assert_eq!(vec2, []);
    /// ```
    ///
    /// # Panic
    ///
    /// May panic or reallocate if the collection is full
    pub fn append<B: Storage<T> + ?Sized>(&mut self, other: &mut GenericVec<T, B>) { other.split_off_into(0, self) }

    /// Convert the backing storage type, and moves all the elements in `self` to the new vector
    pub fn convert<B: raw::StorageWithCapacity<T>>(mut self) -> GenericVec<T, B>
    where
        S: Sized,
    {
        self.split_off(0)
    }

    /// Creates a raw cursor that can be used to remove elements in the specified range.
    /// Usage of [`RawCursor`](iter::RawCursor) is `unsafe` because it doesn't do any checks.
    /// [`RawCursor`](iter::RawCursor) is meant to be a low level tool to implement fancier
    /// iterators, like [`GenericVec::drain`], [`GenericVec::drain_filter`],
    /// or [`GenericVec::splice`].
    ///
    /// # Panic
    ///
    /// Panics if the starting point is greater than the end point or if the end point
    /// is greater than the length of the vector.
    #[inline]
    pub fn raw_cursor<R>(&mut self, range: R) -> iter::RawCursor<'_, T, S>
    where
        R: RangeBounds<usize>,
    {
        let range = slice::check_range(self.len(), range);
        iter::RawCursor::new(self, range)
    }

    /// Creates a cursor that can be used to remove elements in the specified range.
    ///
    /// # Panic
    ///
    /// Panics if the starting point is greater than the end point or if the end point
    /// is greater than the length of the vector.
    #[inline]
    pub fn cursor<R>(&mut self, range: R) -> iter::Cursor<'_, T, S>
    where
        R: RangeBounds<usize>,
    {
        iter::Cursor::new(self.raw_cursor(range))
    }

    /// Creates a draining iterator that removes the specified range in the
    /// vector and yields the removed items.
    ///
    /// When the iterator is dropped, all elements in the range are removed from
    /// the vector, even if the iterator was not fully consumed. If the iterator
    /// is not dropped (with `mem::forget` for example), it is unspecified how many
    /// elements are removed.
    ///
    /// # Panic
    ///
    /// Panics if the starting point is greater than the end point or if the end point
    /// is greater than the length of the vector.
    #[inline]
    pub fn drain<R>(&mut self, range: R) -> iter::Drain<'_, T, S>
    where
        R: RangeBounds<usize>,
    {
        iter::Drain::new(self.raw_cursor(range))
    }

    /// Creates an iterator which uses a closure to determine if an element should be removed.
    ///
    /// If the closure returns true, then the element is removed and yielded.
    /// If the closure returns false, the element will remain in the vector
    /// and will not be yielded by the iterator.
    ///
    /// # Panic
    ///
    /// Panics if the starting point is greater than the end point or if the end point
    /// is greater than the length of the vector.
    #[inline]
    pub fn drain_filter<R, F>(&mut self, range: R, f: F) -> iter::DrainFilter<'_, T, S, F>
    where
        R: RangeBounds<usize>,
        F: FnMut(&mut T) -> bool,
    {
        iter::DrainFilter::new(self.raw_cursor(range), f)
    }

    /// Creates a splicing iterator that replaces the specified range in the vector with
    /// the given replace_with iterator and yields the removed items. replace_with does
    /// not need to be the same length as range.
    ///
    /// range is removed even if the iterator is not consumed until the end.
    ///
    /// It is unspecified how many elements are removed from the vector if the
    /// [`Splice`](iter::Splice) value is leaked.
    ///
    /// The input iterator replace_with is only consumed when the [`Splice`](iter::Splice)
    /// value is dropped
    ///
    /// # Panic
    ///
    /// Panics if the starting point is greater than the end point or if the end point
    /// is greater than the length of the vector.
    #[inline]
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> iter::Splice<'_, T, S, I::IntoIter>
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
    {
        iter::Splice::new(self.raw_cursor(range), replace_with.into_iter())
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(e)` returns false.
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
    /// * You must not drop any of the elements in `slice`
    /// * There must be at least `slice.len()` remaining capacity in the vector
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
    /// If `T::clone` panics, then all newly added items will be dropped. This is different
    /// from `std`, where on panic, newly added items will stay in the `Vec`. This behavior
    /// is unstable, and may change in the future.
    pub fn extend_from_slice(&mut self, slice: &[T])
    where
        T: Clone,
    {
        self.reserve(slice.len());

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

    /// Removes all but the first of consecutive elements in the vector satisfying
    /// a given equality relation.
    ///
    /// The same_bucket function is passed references to two elements from the
    /// vector and must determine if the elements compare equal. The elements
    /// are passed in opposite order from their order in the slice, so if
    /// same_bucket(a, b) returns true, a is removed.
    ///
    /// If the vector is sorted, this removes all duplicates.
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        let (a, _) = slice::partition_dedup_by(self.as_mut_slice(), same_bucket);
        let new_len = a.len();
        self.truncate(new_len);
    }

    /// Removes all but the first of consecutive elements in the vector that resolve to the same key.
    ///
    /// If the vector is sorted, this removes all duplicates.
    pub fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        #[inline]
        fn key_to_same_bucket<T, F, K>(mut f: F) -> impl FnMut(&mut T, &mut T) -> bool
        where
            F: FnMut(&mut T) -> K,
            K: PartialEq,
        {
            #[inline]
            move |a, b| {
                let a = f(a);
                let b = f(b);
                a == b
            }
        }

        self.dedup_by(key_to_same_bucket(key))
    }

    /// Removes all but the first of consecutive elements in the vector that resolve to the same key.
    ///
    /// If the vector is sorted, this removes all duplicates.
    pub fn dedup<F, K>(&mut self)
    where
        T: PartialEq,
    {
        #[inline]
        fn eq_to_same_buckets<T, F>(mut f: F) -> impl FnMut(&mut T, &mut T) -> bool
        where
            F: FnMut(&T, &T) -> bool,
        {
            #[inline]
            move |a, b| f(a, b)
        }

        self.dedup_by(eq_to_same_buckets(PartialEq::eq))
    }
}
