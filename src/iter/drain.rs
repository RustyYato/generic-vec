use crate::{RawDrain, Storage};

use core::iter::FusedIterator;

/// This struct is created by [`GenericVec::drain`](crate::GenericVec::drain).
/// See its documentation for more.
pub struct Drain<'a, A: ?Sized + Storage> {
    raw: RawDrain<'a, A>,
}

impl<'a, A: ?Sized + Storage> From<RawDrain<'a, A>> for Drain<'a, A> {
    fn from(raw: RawDrain<'a, A>) -> Self { Self { raw } }
}

impl<A: ?Sized + Storage> FusedIterator for Drain<'_, A> {}

#[cfg(feature = "nightly")]
impl<A: ?Sized + Storage> ExactSizeIterator for Drain<'_, A> {
    fn is_empty(&self) -> bool { self.raw.is_complete() }
}

impl<A: ?Sized + Storage> Drop for Drain<'_, A> {
    fn drop(&mut self) { self.for_each(drop); }
}

impl<A: ?Sized + Storage> Iterator for Drain<'_, A> {
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.raw.is_complete() {
            None
        } else {
            unsafe { Some(self.raw.take_front()) }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.raw.remaining();
        (len, Some(len))
    }
}

impl<A: ?Sized + Storage> DoubleEndedIterator for Drain<'_, A> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.raw.is_complete() {
            None
        } else {
            unsafe { Some(self.raw.take_back()) }
        }
    }
}
