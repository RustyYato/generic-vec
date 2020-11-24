use crate::{raw::RawVecInit, GenericVec, RawVec};

use core::borrow::{Borrow, BorrowMut};
use core::hash::{Hash, Hasher};

impl<A: RawVecInit> Clone for GenericVec<A>
where
    A::Item: Clone,
{
    fn clone(&self) -> Self {
        let mut vec = Self::with_capacity(self.len());
        vec.extend_from_slice(self);
        vec
    }

    fn clone_from(&mut self, source: &Self) {
        self.clear();
        self.extend_from_slice(source);
    }
}

impl<A: crate::raw::RawVecInit> Default for GenericVec<A> {
    fn default() -> Self {
        Self {
            len: 0,
            mark: Default::default(),
            raw: Default::default(),
        }
    }
}

impl<S: ?Sized + AsRef<[A::Item]>, A: ?Sized + RawVec> PartialEq<S> for GenericVec<A>
where
    A::Item: PartialEq,
{
    fn eq(&self, other: &S) -> bool {
        self.as_slice() == other.as_ref()
    }
}

impl<A: ?Sized + RawVec> Eq for GenericVec<A> where A::Item: Eq {}

impl<S: ?Sized + AsRef<[A::Item]>, A: ?Sized + RawVec> PartialOrd<S> for GenericVec<A>
where
    A::Item: PartialOrd,
{
    fn partial_cmp(&self, other: &S) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_ref())
    }
}

impl<A: ?Sized + RawVec> Ord for GenericVec<A>
where
    A::Item: Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other.as_ref())
    }
}

impl<A: ?Sized + RawVec> Hash for GenericVec<A>
where
    A::Item: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}

use core::fmt;
impl<A: ?Sized + RawVec> fmt::Debug for GenericVec<A>
where
    A::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl<A: ?Sized + RawVec> AsRef<[A::Item]> for GenericVec<A> {
    fn as_ref(&self) -> &[A::Item] {
        self
    }
}

impl<A: ?Sized + RawVec> AsMut<[A::Item]> for GenericVec<A> {
    fn as_mut(&mut self) -> &mut [A::Item] {
        self
    }
}

impl<A: ?Sized + RawVec> Borrow<[A::Item]> for GenericVec<A> {
    fn borrow(&self) -> &[A::Item] {
        self
    }
}

impl<A: ?Sized + RawVec> BorrowMut<[A::Item]> for GenericVec<A> {
    fn borrow_mut(&mut self) -> &mut [A::Item] {
        self
    }
}

#[cfg(feature = "nightly")]
impl<T, const N: usize> From<[T; N]> for crate::ArrayVec<T, N> {
    fn from(array: [T; N]) -> Self {
        ArrayVec::<T, N>::with_raw(unsafe { core::mem::transmute(array) })
    }
}
