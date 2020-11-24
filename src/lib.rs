#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(
    feature = "nightly",
    feature(min_const_generics, unsafe_block_in_unsafe_fn)
)]
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
    ops::{Deref, DerefMut},
    ptr,
};

mod extension;
mod impls;
mod set_len;

pub mod iter;
pub mod raw;

use raw::RawVec;

#[cfg(feature = "alloc")]
#[cfg(feature = "nightly")]
pub type Vec<T, A = std::alloc::Global> = GenericVec<raw::Heap<T, A>>;
#[cfg(feature = "alloc")]
#[cfg(not(feature = "nightly"))]
pub type Vec<T> = GenericVec<raw::Heap<T>>;

#[cfg(feature = "nightly")]
pub type ArrayVec<T, const N: usize> = GenericVec<raw::UninitArray<T, N>>;
pub type SliceVec<'a, T> = GenericVec<raw::UninitSlice<'a, T>>;

#[cfg(feature = "nightly")]
pub type InitArrayVec<T, const N: usize> = GenericVec<raw::Array<T, N>>;
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

#[macro_export]
macro_rules! uninit_array {
    (const $n:expr) => {
        [$crate::macros::Uninit::UNINIT; $n]
    };

    ($n:expr) => {
        unsafe {
            $crate::macros::MaybeUninit::<[$crate::macros::MaybeUninit<_>; $n]>::uninit()
                .assume_init()
        }
    };
}

#[repr(C)]
pub struct GenericVec<A: ?Sized + RawVec> {
    len: usize,
    mark: PhantomData<A::Item>,
    raw: A,
}

impl<A: ?Sized + RawVec> Deref for GenericVec<A> {
    type Target = [A::Item];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.len) }
    }
}

impl<A: ?Sized + RawVec> DerefMut for GenericVec<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }
}

impl<A: ?Sized + RawVec> Drop for GenericVec<A> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.as_mut_slice()) }
    }
}

impl<A: RawVec> GenericVec<A> {
    pub fn with_raw(raw: A) -> Self {
        Self {
            raw,
            len: 0,
            mark: PhantomData,
        }
    }
}

impl<A: raw::RawVecWithCapacity> GenericVec<A> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_raw(A::with_capacity(capacity))
    }

    #[inline]
    #[allow(non_snake_case)]
    fn __with_capacity__const_capacity_checked(
        capacity: usize,
        old_capacity: Option<usize>,
    ) -> Self {
        Self {
            len: 0,
            raw: A::__with_capacity__const_capacity_checked(capacity, old_capacity),
            mark: PhantomData,
        }
    }
}

#[cfg(feature = "nightly")]
impl<T, const N: usize> ArrayVec<T, N> {
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
    pub fn with_alloc(alloc: A) -> Self {
        Self::with_raw(raw::Heap::with_alloc(alloc))
    }
}

impl<'a, T> SliceVec<'a, T> {
    pub fn new(slice: &'a mut [MaybeUninit<T>]) -> Self {
        Self::with_raw(raw::Uninit(slice))
    }
}

impl<'a, T: Copy> InitSliceVec<'a, T> {
    pub fn new(slice: &'a mut [T]) -> Self {
        let len = slice.len();
        let mut vec = Self::with_raw(raw::Init(slice));
        vec.set_len(len);
        vec
    }
}

impl<A: ?Sized + RawVec> GenericVec<A> {
    pub fn as_ptr(&self) -> *const A::Item {
        self.raw.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut A::Item {
        self.raw.as_mut_ptr()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub unsafe fn set_len_unchecked(&mut self, len: usize) {
        self.len = len;
    }

    pub fn set_len(&mut self, len: usize)
    where
        A: raw::RawVecInit,
    {
        self.len = len;
    }

    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }

    pub fn as_slice(&self) -> &[A::Item] {
        self
    }

    pub fn as_mut_slice(&mut self) -> &mut [A::Item] {
        self
    }

    pub unsafe fn raw_buffer(&self) -> &A {
        &self.raw
    }

    pub unsafe fn raw_buffer_mut(&mut self) -> &mut A {
        &mut self.raw
    }

    pub fn remaining(&mut self) -> &mut [A::BufferItem] {
        unsafe {
            let cap = self.raw.capacity();
            core::slice::from_raw_parts_mut(
                self.raw.as_mut_ptr().add(self.len).cast(),
                cap.wrapping_sub(self.len),
            )
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        if let Some(new_capacity) = self.len.checked_add(additional) {
            self.raw.reserve(new_capacity)
        }
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), raw::AllocError> {
        if let Some(new_capacity) = self.len.checked_add(additional) {
            self.raw.try_reserve(new_capacity)
        } else {
            Ok(())
        }
    }

    pub fn truncate(&mut self, len: usize) {
        if let Some(diff) = self.len.checked_sub(len) {
            self.len = len;

            unsafe {
                core::slice::from_raw_parts_mut(self.as_mut_ptr().add(len), diff);
            }
        }
    }

    pub fn grow(&mut self, additional: usize, value: A::Item)
    where
        A::Item: Clone,
    {
        self.reserve(additional);
        unsafe { extension::Extension::grow(self, additional, value) }
    }

    pub fn clear(&mut self) {
        self.truncate(0);
    }

    pub fn push(&mut self, value: A::Item) -> &mut A::Item {
        if self.len() == self.capacity() {
            self.reserve(1);
        }

        unsafe { self.push_unchecked(value) }
    }

    pub fn try_push(&mut self, value: A::Item) -> Result<&mut A::Item, A::Item> {
        if self.len() == self.capacity() {
            Err(value)
        } else {
            unsafe { Ok(self.push_unchecked(value)) }
        }
    }

    pub unsafe fn insert_unchecked(&mut self, index: usize, value: A::Item) -> &mut A::Item {
        unsafe {
            let ptr = self.raw.as_mut_ptr().add(index);
            ptr.add(1).copy_from(ptr, self.len.wrapping_sub(index));
            ptr.write(value);
            &mut *ptr
        }
    }

    pub unsafe fn insert(&mut self, index: usize, value: A::Item) -> &mut A::Item {
        if self.len() == self.capacity() {
            self.reserve(1);
        }

        unsafe { self.insert_unchecked(index, value) }
    }

    pub fn try_insert(&mut self, index: usize, value: A::Item) -> Result<&mut A::Item, A::Item> {
        if self.len() == self.capacity() {
            Err(value)
        } else {
            unsafe { Ok(self.insert_unchecked(index, value)) }
        }
    }

    pub fn pop(&mut self) -> Option<A::Item> {
        if self.len() == 0 {
            None
        } else {
            unsafe { Some(self.pop_unchecked()) }
        }
    }

    pub unsafe fn remove_unchecked(&mut self, index: usize) -> A::Item {
        unsafe {
            let ptr = self.raw.as_mut_ptr();
            let value = ptr::read(self.get_unchecked(index));
            ptr.copy_from(ptr.add(1), self.len.wrapping_sub(index).wrapping_sub(1));
            value
        }
    }

    pub fn remove(&mut self, index: usize) -> A::Item {
        assert!(
            index < self.len,
            "Tried to remove item at index {}, but length is {}",
            index,
            self.len
        );

        unsafe { self.remove_unchecked(index) }
    }

    pub fn try_remove(&mut self, index: usize) -> Option<A::Item> {
        if self.len() < index {
            unsafe { Some(self.remove_unchecked(index)) }
        } else {
            None
        }
    }

    pub unsafe fn swap_remove_unchecked(&mut self, index: usize) -> A::Item {
        unsafe {
            let ptr = self.raw.as_mut_ptr();
            let at = ptr.add(index);
            let end = ptr.add(self.len.wrapping_sub(1));
            let value = at.read();
            at.copy_from(end, 1);
            value
        }
    }

    pub fn swap_remove(&mut self, index: usize) -> A::Item {
        assert!(
            index < self.len,
            "Tried to remove item at index {}, but length is {}",
            index,
            self.len
        );

        unsafe { self.swap_remove_unchecked(index) }
    }

    pub fn try_swap_remove(&mut self, index: usize) -> Option<A::Item> {
        if index < self.len {
            unsafe { Some(self.swap_remove_unchecked(index)) }
        } else {
            None
        }
    }

    pub unsafe fn push_unchecked(&mut self, value: A::Item) -> &mut A::Item {
        debug_assert_ne!(
            self.len,
            self.capacity(),
            "Tried to `push_unchecked` past capacity! This is UB in release mode"
        );
        unsafe {
            let ptr = self.as_mut_ptr().add(self.len);
            self.len += 1;
            ptr.write(value);
            &mut *ptr
        }
    }

    pub unsafe fn pop_unchecked(&mut self) -> A::Item {
        debug_assert_ne!(
            self.len,
            self.capacity(),
            "Tried to `pop_unchecked` an empty array vec! This is UB in release mode"
        );
        unsafe {
            self.len -= 1;
            self.as_mut_ptr().add(self.len).read()
        }
    }

    pub fn split_off<B: raw::RawVecWithCapacity<Item = A::Item>>(
        &mut self,
        index: usize,
    ) -> GenericVec<B> {
        assert!(
            index <= self.len,
            "Tried to split at index {}, but length is {}",
            index,
            self.len
        );

        let mut vec = GenericVec::<B>::__with_capacity__const_capacity_checked(
            self.len.wrapping_sub(index),
            A::CONST_CAPACITY,
        );

        unsafe {
            vec.extend_from_slice_unchecked(self.get_unchecked(index..));
            self.len = index;
        }

        vec
    }

    pub fn convert<B: raw::RawVecWithCapacity<Item = A::Item>>(mut self) -> GenericVec<B>
    where
        A: Sized,
    {
        self.split_off(0)
    }

    pub fn consume<B: raw::RawVec<Item = A::Item>>(&mut self, other: &mut GenericVec<B>) {
        unsafe {
            self.reserve(other.len);
            self.extend_from_slice_unchecked(other);
            other.len = 0;
        }
    }

    #[inline]
    pub fn raw_drain<R>(&mut self, range: R) -> RawDrain<'_, A>
    where
        R: core::slice::SliceIndex<[A::Item], Output = [A::Item]>,
    {
        RawDrain::new(self, range)
    }

    #[inline]
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, A>
    where
        R: core::slice::SliceIndex<[A::Item], Output = [A::Item]>,
    {
        self.raw_drain(range).into()
    }

    #[inline]
    pub fn drain_filter<R, F>(&mut self, range: R, f: F) -> DrainFilter<'_, A, F>
    where
        R: core::slice::SliceIndex<[A::Item], Output = [A::Item]>,
        F: FnMut(&mut A::Item) -> bool,
    {
        DrainFilter::new(self.raw_drain(range), f)
    }

    #[inline]
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, A, I::IntoIter>
    where
        R: core::slice::SliceIndex<[A::Item], Output = [A::Item]>,
        I: IntoIterator<Item = A::Item>,
    {
        Splice::new(self.raw_drain(range), replace_with.into_iter())
    }

    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&mut A::Item) -> bool,
    {
        fn not<F: FnMut(&mut T) -> bool, T>(mut f: F) -> impl FnMut(&mut T) -> bool {
            move |value| !f(value)
        }
        self.drain_filter(.., not(f));
    }

    pub unsafe fn extend_from_slice_unchecked(&mut self, slice: &[A::Item]) {
        unsafe {
            self.as_mut_ptr()
                .add(self.len)
                .copy_from_nonoverlapping(slice.as_ptr(), slice.len());
        }
        self.len += slice.len();
    }

    pub fn extend_from_slice(&mut self, slice: &[A::Item])
    where
        A::Item: Clone,
    {
        self.reserve(self.len());

        unsafe { extension::Extension::extend_from_slice(self, slice) }
    }
}
