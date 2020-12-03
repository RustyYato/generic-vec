use crate::{RawCursor, Storage};

use core::iter::FusedIterator;

/// This struct is created by [`GenericVec::drain_filter`](crate::GenericVec::drain_filter).
/// See its documentation for more.
pub struct DrainFilter<'a, T, S, F>
where
    S: ?Sized + Storage<T>,
    F: FnMut(&mut T) -> bool,
{
    raw: RawCursor<'a, T, S>,
    filter: F,
}

impl<'a, T, S, F> DrainFilter<'a, T, S, F>
where
    S: ?Sized + Storage<T>,
    F: FnMut(&mut T) -> bool,
{
    pub(crate) fn new(raw: RawCursor<'a, T, S>, filter: F) -> Self { Self { raw, filter } }
}

impl<T, S, F> Drop for DrainFilter<'_, T, S, F>
where
    S: ?Sized + Storage<T>,
    F: FnMut(&mut T) -> bool,
{
    fn drop(&mut self) { self.for_each(drop); }
}

impl<T, S, F> FusedIterator for DrainFilter<'_, T, S, F>
where
    S: ?Sized + Storage<T>,
    F: FnMut(&mut T) -> bool,
{
}
impl<T, S, F> Iterator for DrainFilter<'_, T, S, F>
where
    S: ?Sized + Storage<T>,
    F: FnMut(&mut T) -> bool,
{
    type Item = T;

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

impl<T, S, F> DoubleEndedIterator for DrainFilter<'_, T, S, F>
where
    S: ?Sized + Storage<T>,
    F: FnMut(&mut T) -> bool,
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
