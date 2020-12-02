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
pub struct IntoIter<T, S: ?Sized + Storage<T>> {
    index: usize,
    vec: ManuallyDrop<GenericVec<T, S>>,
}

impl<T, S: ?Sized + Storage<T>> Drop for IntoIter<T, S> {
    fn drop(&mut self) {
        unsafe {
            struct DropAlloc<'a, S: ?Sized>(&'a mut S);

            impl<S: ?Sized> Drop for DropAlloc<'_, S> {
                fn drop(&mut self) {
                    unsafe {
                        core::ptr::drop_in_place(self.0);
                    }
                }
            }

            let len = self.vec.len();
            let index = self.index;

            let drop_alloc = DropAlloc(&mut self.vec.storage);
            let data = drop_alloc.0.as_mut_ptr().add(index);
            core::ptr::slice_from_raw_parts_mut(data, len.wrapping_sub(index)).drop_in_place();
        }
    }
}

impl<T, S: Storage<T>> IntoIterator for GenericVec<T, S> {
    type IntoIter = IntoIter<T, S>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            index: 0,
            vec: ManuallyDrop::new(self),
        }
    }
}

impl<'a, T, S: ?Sized + Storage<T>> IntoIterator for &'a mut GenericVec<T, S> {
    type IntoIter = core::slice::IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

impl<'a, T, S: ?Sized + Storage<T>> IntoIterator for &'a GenericVec<T, S> {
    type IntoIter = core::slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<T, S: ?Sized + Storage<T>> FusedIterator for IntoIter<T, S> {}
impl<T, S: ?Sized + Storage<T>> ExactSizeIterator for IntoIter<T, S> {
    #[cfg(feature = "nightly")]
    fn is_empty(&self) -> bool { self.index == self.vec.len() }
}

#[cfg(feature = "nightly")]
unsafe impl<T, S: ?Sized + Storage<T>> TrustedLen for IntoIter<T, S> {}

impl<T, S: ?Sized + Storage<T>> IntoIter<T, S> {
    /// Get a slice to the remaining elements in the iterator
    pub fn as_slice(&self) -> &[T] {
        let index = self.index;
        let len = self.vec.len();
        let ptr = self.vec.as_ptr();
        unsafe { core::slice::from_raw_parts(ptr.add(index), len.wrapping_sub(index)) }
    }

    /// Get a mutable slice to the remaining elements in the iterator
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let index = self.index;
        let len = self.vec.len();
        let ptr = self.vec.as_mut_ptr();
        unsafe { core::slice::from_raw_parts_mut(ptr.add(index), len.wrapping_sub(index)) }
    }
}

impl<T, S: ?Sized + Storage<T>> Iterator for IntoIter<T, S> {
    type Item = T;

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

impl<T, S: ?Sized + Storage<T>> DoubleEndedIterator for IntoIter<T, S> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index == self.vec.len() {
            None
        } else {
            unsafe { Some(self.vec.pop_unchecked()) }
        }
    }
}
