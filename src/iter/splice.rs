use crate::{RawDrain, RawVec};

// FIXME: splice needs to insert *all* elements of the iterator, it currently does not!
pub struct Splice<'a, A, I>
where
    A: ?Sized + RawVec,
    I: Iterator<Item = A::Item>,
{
    raw: RawDrain<'a, A>,
    replace_with: I,
}

impl<'a, A: ?Sized + RawVec, I: Iterator<Item = A::Item>> Splice<'a, A, I> {
    pub fn new(raw: RawDrain<'a, A>, replace_with: I) -> Self {
        Self { raw, replace_with }
    }
}

impl<A: ?Sized + RawVec, I: Iterator<Item = A::Item>> Drop for Splice<'_, A, I> {
    fn drop(&mut self) {
        self.for_each(drop);
    }
}

impl<'a, A: ?Sized + RawVec, I: Iterator<Item = A::Item>> Iterator for Splice<'a, A, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.raw.is_complete() {
            return None;
        }

        unsafe {
            let front = self.raw.front();

            Some(if let Some(item) = self.replace_with.next() {
                let item = core::mem::replace(front, item);
                self.raw.skip_front();
                item
            } else {
                self.raw.take_front()
            })
        }
    }
}
