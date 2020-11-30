use crate::{raw::StorageWithCapacity, GenericVec, Storage};

use core::{
    borrow::{Borrow, BorrowMut},
    hash::{Hash, Hasher},
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

impl<T, S: StorageWithCapacity<T>> Clone for GenericVec<T, S>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut vec = Self::with_capacity(self.len());
        vec.extend_from_slice(self);
        vec
    }

    fn clone_from(&mut self, source: &Self) {
        self.truncate(source.len());
        let (init, tail) = source.split_at(self.len());
        self.clone_from_slice(init);
        self.extend_from_slice(tail);
    }
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

#[cfg(feature = "nightly")]
impl<T, const N: usize> From<[T; N]> for crate::ArrayVec<T, N> {
    fn from(array: [T; N]) -> Self { Self::from_array(array) }
}

#[cfg(feature = "nightly")]
impl<T: Copy, const N: usize> From<[T; N]> for crate::InitArrayVec<T, N> {
    fn from(array: [T; N]) -> Self { crate::InitArrayVec::<T, N>::new(array) }
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
