use crate::{raw::StorageWithCapacity, GenericVec, Storage};

use core::{
    borrow::{Borrow, BorrowMut},
    hash::{Hash, Hasher},
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

impl<A: StorageWithCapacity> Clone for GenericVec<A>
where
    A::Item: Clone,
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

impl<A: crate::raw::StorageWithCapacity> Default for GenericVec<A> {
    fn default() -> Self { Self::with_raw(Default::default()) }
}

impl<S: ?Sized + AsRef<[A::Item]>, A: ?Sized + Storage> PartialEq<S> for GenericVec<A>
where
    A::Item: PartialEq,
{
    fn eq(&self, other: &S) -> bool { self.as_slice() == other.as_ref() }
}

impl<A: ?Sized + Storage> Eq for GenericVec<A> where A::Item: Eq {}

impl<S: ?Sized + AsRef<[A::Item]>, A: ?Sized + Storage> PartialOrd<S> for GenericVec<A>
where
    A::Item: PartialOrd,
{
    fn partial_cmp(&self, other: &S) -> Option<core::cmp::Ordering> { self.as_slice().partial_cmp(other.as_ref()) }
}

impl<A: ?Sized + Storage> Ord for GenericVec<A>
where
    A::Item: Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering { self.as_slice().cmp(other.as_ref()) }
}

impl<A: ?Sized + Storage> Hash for GenericVec<A>
where
    A::Item: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) { self.as_slice().hash(state) }
}

use core::fmt;
impl<A: ?Sized + Storage> fmt::Debug for GenericVec<A>
where
    A::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.as_slice().fmt(f) }
}

impl<A: ?Sized + Storage> AsRef<[A::Item]> for GenericVec<A> {
    fn as_ref(&self) -> &[A::Item] { self }
}

impl<A: ?Sized + Storage> AsMut<[A::Item]> for GenericVec<A> {
    fn as_mut(&mut self) -> &mut [A::Item] { self }
}

impl<A: ?Sized + Storage> Borrow<[A::Item]> for GenericVec<A> {
    fn borrow(&self) -> &[A::Item] { self }
}

impl<A: ?Sized + Storage> BorrowMut<[A::Item]> for GenericVec<A> {
    fn borrow_mut(&mut self) -> &mut [A::Item] { self }
}

#[cfg(feature = "nightly")]
impl<T, const N: usize> From<[T; N]> for crate::ArrayVec<T, N> {
    fn from(array: [T; N]) -> Self { Self::from_array(array) }
}

#[cfg(feature = "nightly")]
impl<T: Copy, const N: usize> From<[T; N]> for crate::InitArrayVec<T, N> {
    fn from(array: [T; N]) -> Self { crate::InitArrayVec::<T, N>::new(array) }
}

impl<A: Storage + ?Sized, I> Index<I> for GenericVec<A>
where
    I: SliceIndex<[A::Item]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output { self.as_slice().index(index) }
}

impl<A: Storage + ?Sized, I> IndexMut<I> for GenericVec<A>
where
    I: SliceIndex<[A::Item]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output { self.as_mut_slice().index_mut(index) }
}
