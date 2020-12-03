use crate::{iter::RawCursor, Storage};

use core::iter::FusedIterator;

/// This struct is created by [`GenericVec::drain`](crate::GenericVec::drain).
/// See its documentation for more.
pub struct Drain<'a, T, S: ?Sized + Storage<T>> {
    raw: RawCursor<'a, T, S>,
}

impl<'a, T, S: ?Sized + Storage<T>> Drain<'a, T, S> {
    pub(crate) fn new(raw: RawCursor<'a, T, S>) -> Self { Self { raw } }
}

impl<T, S: ?Sized + Storage<T>> FusedIterator for Drain<'_, T, S> {}

#[cfg(feature = "nightly")]
impl<T, S: ?Sized + Storage<T>> ExactSizeIterator for Drain<'_, T, S> {
    fn is_empty(&self) -> bool { self.raw.is_empty() }
}

impl<T, S: ?Sized + Storage<T>> Drop for Drain<'_, T, S> {
    fn drop(&mut self) { self.for_each(drop); }
}

impl<T, S: ?Sized + Storage<T>> Iterator for Drain<'_, T, S> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.raw.is_empty() {
            None
        } else {
            unsafe { Some(self.raw.take_front()) }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.raw.len();
        (len, Some(len))
    }
}

impl<T, S: ?Sized + Storage<T>> DoubleEndedIterator for Drain<'_, T, S> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.raw.is_empty() {
            None
        } else {
            unsafe { Some(self.raw.take_back()) }
        }
    }
}
