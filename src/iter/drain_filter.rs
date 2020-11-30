use crate::{RawDrain, Storage};

use core::iter::FusedIterator;

/// This struct is created by [`GenericVec::drain_filter`](crate::GenericVec::drain_filter).
/// See its documentation for more.
pub struct DrainFilter<'a, A, F>
where
    A: ?Sized + Storage,
    F: FnMut(&mut A::Item) -> bool,
{
    raw: RawDrain<'a, A>,
    filter: F,
}

impl<'a, A, F> DrainFilter<'a, A, F>
where
    A: ?Sized + Storage,
    F: FnMut(&mut A::Item) -> bool,
{
    pub(crate) fn new(raw: RawDrain<'a, A>, filter: F) -> Self { Self { raw, filter } }
}

impl<A, F> Drop for DrainFilter<'_, A, F>
where
    A: ?Sized + Storage,
    F: FnMut(&mut A::Item) -> bool,
{
    fn drop(&mut self) { self.for_each(drop); }
}

impl<A, F> FusedIterator for DrainFilter<'_, A, F>
where
    A: ?Sized + Storage,
    F: FnMut(&mut A::Item) -> bool,
{
}
impl<A, F> Iterator for DrainFilter<'_, A, F>
where
    A: ?Sized + Storage,
    F: FnMut(&mut A::Item) -> bool,
{
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.raw.is_complete() {
                break None
            }

            unsafe {
                let value = self.raw.front();
                if (self.filter)(value) {
                    break Some(self.raw.take_front())
                } else {
                    self.raw.skip_front();
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.raw.remaining();
        (0, Some(len))
    }
}

impl<A, F> DoubleEndedIterator for DrainFilter<'_, A, F>
where
    A: ?Sized + Storage,
    F: FnMut(&mut A::Item) -> bool,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if self.raw.is_complete() {
                break None
            }

            unsafe {
                let value = self.raw.back();
                if (self.filter)(value) {
                    break Some(self.raw.take_back())
                } else {
                    self.raw.skip_back();
                }
            }
        }
    }
}
