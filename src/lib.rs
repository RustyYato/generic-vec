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

#[cfg(feature = "alloc")]
#[cfg(feature = "nightly")]
use std::boxed::Box;

#[cfg(feature = "nightly")]
use core::convert::TryFrom;

use core::{
    marker::PhantomData,
    mem,
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
#[cfg(feature = "nightly")]
pub type SliceVec<T> = GenericVec<raw::UninitSlice<T>>;

#[cfg(feature = "nightly")]
pub type InitArrayVec<T, const N: usize> = GenericVec<raw::Array<T, N>>;
#[cfg(feature = "nightly")]
pub type InitSliceVec<T> = GenericVec<raw::Slice<T>>;

use iter::{Drain, DrainFilter, RawDrain, Splice};

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
        Self {
            len: 0,
            mark: PhantomData,
            raw: raw::Heap::with_alloc(alloc),
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg(feature = "nightly")]
impl<'a, T, const N: usize> TryFrom<Box<SliceVec<T>>> for Box<ArrayVec<T, N>> {
    type Error = Box<SliceVec<T>>;

    fn try_from(vec: Box<SliceVec<T>>) -> Result<Self, Self::Error> {
        if vec.raw.capacity() == N {
            Ok(unsafe { Box::from_raw(Box::into_raw(vec) as *mut ArrayVec<T, N>) })
        } else {
            Err(vec)
        }
    }
}

#[cfg(feature = "nightly")]
impl<'a, T, const N: usize> TryFrom<&'a mut SliceVec<T>> for &'a mut ArrayVec<T, N> {
    type Error = &'a mut SliceVec<T>;

    fn try_from(vec: &'a mut SliceVec<T>) -> Result<Self, Self::Error> {
        if vec.raw.capacity() == N {
            Ok(unsafe { &mut *(vec as *mut SliceVec<T> as *mut ArrayVec<T, N>) })
        } else {
            Err(vec)
        }
    }
}

#[cfg(feature = "nightly")]
impl<'a, T, const N: usize> TryFrom<&'a SliceVec<T>> for &'a ArrayVec<T, N> {
    type Error = &'a SliceVec<T>;

    fn try_from(vec: &'a SliceVec<T>) -> Result<Self, Self::Error> {
        if vec.raw.capacity() == N {
            Ok(unsafe { &*(vec as *const SliceVec<T> as *const ArrayVec<T, N>) })
        } else {
            Err(vec)
        }
    }
}

// TODO: insert, remove, swap_remove, split_off, docs

impl<A: raw::RawVecInit> GenericVec<A> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            len: 0,
            raw: A::with_capacity(capacity),
            mark: PhantomData,
        }
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

    pub fn capacity(&self) -> usize {
        mem::size_of_val(&self.raw) / mem::size_of::<A::Item>()
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

    pub fn reserve(&mut self, additional: usize) {
        if let Some(new_capacity) = self.len.checked_add(additional) {
            self.raw.reserve(new_capacity)
        }
    }

    #[cfg(feature = "nightly")]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), core::alloc::AllocError> {
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

    pub fn pop(&mut self) -> Option<A::Item> {
        if self.len() == 0 {
            None
        } else {
            unsafe { Some(self.pop_unchecked()) }
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
