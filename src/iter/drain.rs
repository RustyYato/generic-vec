use crate::{RawDrain, RawVec};

use core::iter::FusedIterator;

pub struct Drain<'a, A: ?Sized + RawVec> {
    raw: RawDrain<'a, A>,
}

impl<'a, A: ?Sized + RawVec> From<RawDrain<'a, A>> for Drain<'a, A> {
    fn from(raw: RawDrain<'a, A>) -> Self {
        Self { raw }
    }
}

impl<A: ?Sized + RawVec> FusedIterator for Drain<'_, A> {}

#[cfg(feature = "nightly")]
impl<A: ?Sized + RawVec> ExactSizeIterator for Drain<'_, A> {
    fn is_empty(&self) -> bool {
        self.raw.is_complete()
    }
}

impl<A: ?Sized + RawVec> Drop for Drain<'_, A> {
    fn drop(&mut self) {
        self.for_each(drop);
    }
}

impl<A: ?Sized + RawVec> Iterator for Drain<'_, A> {
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

impl<A: ?Sized + RawVec> DoubleEndedIterator for Drain<'_, A> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.raw.is_complete() {
            None
        } else {
            unsafe { Some(self.raw.take_back()) }
        }
    }
}
