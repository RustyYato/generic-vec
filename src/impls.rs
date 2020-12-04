use crate::{raw::StorageWithCapacity, GenericVec, Storage};

#[allow(unused_imports)]
use core::{
    borrow::{Borrow, BorrowMut},
    hash::{Hash, Hasher},
    ops::{Index, IndexMut},
    ptr::NonNull,
    slice::SliceIndex,
};

#[cfg(feature = "alloc")]
use std::vec::Vec;

impl<T, S: StorageWithCapacity<T>> Clone for GenericVec<T, S>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut vec = Self::with_capacity(self.len());
        vec.extend_from_slice(self);
        vec
    }

    fn clone_from(&mut self, source: &Self) { self.clone_from(source); }
}

impl<T, S: StorageWithCapacity<T>> Default for GenericVec<T, S> {
    fn default() -> Self { Self::with_storage(Default::default()) }
}

impl<T, O: ?Sized + AsRef<[T]>, S: ?Sized + Storage<T>> PartialEq<O> for GenericVec<T, S>
where
    T: PartialEq,
{
    fn eq(&self, other: &O) -> bool { self.as_slice() == other.as_ref() }
}

impl<T, S: ?Sized + Storage<T>> Eq for GenericVec<T, S> where T: Eq {}

impl<T, O: ?Sized + AsRef<[T]>, S: ?Sized + Storage<T>> PartialOrd<O> for GenericVec<T, S>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &O) -> Option<core::cmp::Ordering> { self.as_slice().partial_cmp(other.as_ref()) }
}

impl<T, S: ?Sized + Storage<T>> Ord for GenericVec<T, S>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering { self.as_slice().cmp(other.as_ref()) }
}

impl<T, S: ?Sized + Storage<T>> Hash for GenericVec<T, S>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) { self.as_slice().hash(state) }
}

use core::fmt;
impl<T, S: ?Sized + Storage<T>> fmt::Debug for GenericVec<T, S>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.as_slice().fmt(f) }
}

impl<T, S: ?Sized + Storage<T>> AsRef<[T]> for GenericVec<T, S> {
    fn as_ref(&self) -> &[T] { self }
}

impl<T, S: ?Sized + Storage<T>> AsMut<[T]> for GenericVec<T, S> {
    fn as_mut(&mut self) -> &mut [T] { self }
}

impl<T, S: ?Sized + Storage<T>> Borrow<[T]> for GenericVec<T, S> {
    fn borrow(&self) -> &[T] { self }
}

impl<T, S: ?Sized + Storage<T>> BorrowMut<[T]> for GenericVec<T, S> {
    fn borrow_mut(&mut self) -> &mut [T] { self }
}

#[cfg(any(doc, feature = "nightly"))]
impl<T, const N: usize> From<[T; N]> for crate::ArrayVec<T, N> {
    fn from(array: [T; N]) -> Self { Self::from_array(array) }
}

#[cfg(any(doc, feature = "nightly"))]
impl<T: Copy, const N: usize> From<[T; N]> for crate::InitArrayVec<T, N> {
    fn from(array: [T; N]) -> Self { crate::InitArrayVec::<T, N>::new(array) }
}

#[cfg(not(doc))]
#[cfg(feature = "alloc")]
#[cfg(not(feature = "nightly"))]
impl<T> From<Vec<T>> for crate::HeapVec<T> {
    fn from(vec: Vec<T>) -> Self {
        let mut vec = core::mem::ManuallyDrop::new(vec);

        let len = vec.len();
        let cap = vec.capacity();
        let ptr = unsafe { NonNull::new_unchecked(vec.as_mut_ptr()) };

        unsafe { crate::HeapVec::from_raw_parts(len, crate::raw::Heap::from_raw_parts(ptr, cap)) }
    }
}

#[cfg(any(doc, feature = "alloc"))]
#[cfg(any(doc, feature = "nightly"))]
impl<T, A: std::alloc::AllocRef> From<Vec<T, A>> for crate::HeapVec<T, A> {
    fn from(vec: Vec<T, A>) -> Self {
        let (ptr, len, cap, alloc) = vec.into_raw_parts_with_alloc();

        unsafe {
            crate::HeapVec::from_raw_parts(
                len,
                crate::raw::Heap::from_raw_parts_in(NonNull::new_unchecked(ptr), cap, alloc),
            )
        }
    }
}

#[cfg(not(doc))]
#[cfg(feature = "alloc")]
#[cfg(not(feature = "nightly"))]
impl<T> From<crate::HeapVec<T>> for Vec<T> {
    fn from(vec: crate::HeapVec<T>) -> Self {
        let (length, alloc) = vec.into_raw_parts();
        let (ptr, capacity) = alloc.into_raw_parts();

        unsafe { Vec::from_raw_parts(ptr.as_ptr(), length, capacity) }
    }
}

#[cfg(any(doc, feature = "alloc"))]
#[cfg(any(doc, feature = "nightly"))]
impl<T, A: std::alloc::AllocRef> From<crate::HeapVec<T, A>> for Vec<T, A> {
    fn from(vec: crate::HeapVec<T, A>) -> Self {
        let (length, alloc) = vec.into_raw_parts();
        let (ptr, capacity, alloc) = alloc.into_raw_parts_with_alloc();

        unsafe { Vec::from_raw_parts_in(ptr.as_ptr(), length, capacity, alloc) }
    }
}

impl<T, S: Storage<T> + ?Sized, I> Index<I> for GenericVec<T, S>
where
    I: SliceIndex<[T]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output { self.as_slice().index(index) }
}

impl<T, S: Storage<T> + ?Sized, I> IndexMut<I> for GenericVec<T, S>
where
    I: SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output { self.as_mut_slice().index_mut(index) }
}
