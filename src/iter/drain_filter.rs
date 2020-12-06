use crate::{iter::RawCursor, Storage};

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
    panicking: bool,
}

struct SetOnDrop<'a>(&'a mut bool);

impl<'a> Drop for SetOnDrop<'a> {
    fn drop(&mut self) { *self.0 = true; }
}

impl<'a, T, S, F> DrainFilter<'a, T, S, F>
where
    S: ?Sized + Storage<T>,
    F: FnMut(&mut T) -> bool,
{
    pub(crate) fn new(raw: RawCursor<'a, T, S>, filter: F) -> Self {
        Self {
            raw,
            filter,
            panicking: false,
        }
    }
}

impl<T, S, F> Drop for DrainFilter<'_, T, S, F>
where
    S: ?Sized + Storage<T>,
    F: FnMut(&mut T) -> bool,
{
    fn drop(&mut self) {
        if !self.panicking {
            self.for_each(drop);
        }
    }
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
            if self.raw.is_empty() {
                break None
            }

            unsafe {
                let value = self.raw.front_mut();

                let on_drop = SetOnDrop(&mut self.panicking);
                let do_take = (self.filter)(value);
                core::mem::forget(on_drop);

                if do_take {
                    break Some(self.raw.take_front())
                } else {
                    self.raw.skip_front();
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.raw.len();
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
            if self.raw.is_empty() {
                break None
            }

            unsafe {
                let value = self.raw.back_mut();

                let on_drop = SetOnDrop(&mut self.panicking);
                let do_take = (self.filter)(value);
                core::mem::forget(on_drop);

                if do_take {
                    break Some(self.raw.take_back())
                } else {
                    self.raw.skip_back();
                }
            }
        }
    }
}
