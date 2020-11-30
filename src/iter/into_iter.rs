use crate::{GenericVec, Storage};
#[cfg(feature = "nightly")]
use core::iter::TrustedLen;
use core::{
    iter::{ExactSizeIterator, FusedIterator},
    mem::ManuallyDrop,
    ptr,
};

/// This struct is created by [`GenericVec::into_iter`](crate::GenericVec::into_iter).
/// See its documentation for more.
pub struct IntoIter<A: ?Sized + Storage> {
    index: usize,
    vec: ManuallyDrop<GenericVec<A>>,
}

impl<A: ?Sized + Storage> Drop for IntoIter<A> {
    fn drop(&mut self) {
        unsafe {
            // TODO: handle panics

            struct DropAlloc<'a, A: ?Sized + Storage>(&'a mut GenericVec<A>);

            impl<A: ?Sized + Storage> Drop for DropAlloc<'_, A> {
                fn drop(&mut self) {
                    unsafe {
                        ptr::drop_in_place(&mut self.0.raw);
                    }
                }
            }

            let mut drop_alloc = DropAlloc(&mut self.vec);
            let vec = &mut drop_alloc.0;

            ptr::drop_in_place(&mut vec.get_unchecked(self.index..));
        }
    }
}

impl<A: Storage> IntoIterator for GenericVec<A> {
    type IntoIter = IntoIter<A>;
    type Item = A::Item;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            index: 0,
            vec: ManuallyDrop::new(self),
        }
    }
}

impl<'a, A: ?Sized + Storage> IntoIterator for &'a mut GenericVec<A> {
    type IntoIter = core::slice::IterMut<'a, A::Item>;
    type Item = &'a mut A::Item;

    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

impl<'a, A: ?Sized + Storage> IntoIterator for &'a GenericVec<A> {
    type IntoIter = core::slice::Iter<'a, A::Item>;
    type Item = &'a A::Item;

    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<A: ?Sized + Storage> FusedIterator for IntoIter<A> {}
impl<A: ?Sized + Storage> ExactSizeIterator for IntoIter<A> {
    #[cfg(feature = "nightly")]
    fn is_empty(&self) -> bool { self.index == self.vec.len() }
}

#[cfg(feature = "nightly")]
unsafe impl<A: ?Sized + Storage> TrustedLen for IntoIter<A> {}

impl<A: ?Sized + Storage> Iterator for IntoIter<A> {
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.vec.len() {
            None
        } else {
            unsafe {
                let value = self.vec.get_unchecked(self.index);
                self.index += 1;
                Some(ptr::read(value))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.vec.len().wrapping_sub(self.index);
        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let n = self.len().min(n);
        let old_index = self.index;
        self.index += n;

        unsafe {
            ptr::drop_in_place(self.vec.get_unchecked_mut(old_index..self.index));
        }

        self.next()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }
}

impl<A: ?Sized + Storage> DoubleEndedIterator for IntoIter<A> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index == self.vec.len() {
            None
        } else {
            unsafe { Some(self.vec.pop_unchecked()) }
        }
    }
}
