use crate::{GenericVec, Storage};
use core::{
    marker::PhantomData,
    ops::{Bound, RangeBounds},
    ptr::NonNull,
};

/// This struct is created by [`GenericVec::raw_drain`]. See its documentation for more.
pub struct RawCursor<'a, T, S: ?Sized + Storage<T>> {
    vec: NonNull<GenericVec<T, S>>,
    old_vec_len: usize,
    write_front: *mut T,
    read_front: *mut T,
    read_back: *mut T,
    write_back: *mut T,
    mark: PhantomData<&'a mut GenericVec<T, S>>,
}

impl<T, S: ?Sized + Storage<T>> Drop for RawCursor<'_, T, S> {
    fn drop(&mut self) { self.finish() }
}

#[inline(never)]
#[cold]
#[track_caller]
pub(super) fn slice_start_index_overflow_fail() -> ! {
    panic!("attempted to index slice from after maximum usize");
}

#[inline(never)]
#[cold]
#[track_caller]
pub(super) fn slice_end_index_overflow_fail() -> ! {
    panic!("attempted to index slice up to maximum usize");
}

#[inline(never)]
#[cold]
#[track_caller]
pub(super) fn slice_index_order_fail(index: usize, end: usize) -> ! {
    panic!("slice index starts at {} but ends at {}", index, end);
}

#[inline(never)]
#[cold]
#[track_caller]
pub(super) fn slice_end_index_len_fail(index: usize, len: usize) -> ! {
    panic!("range end index {} out of range for slice of length {}", index, len);
}

pub(crate) fn check_range<R: RangeBounds<usize>>(len: usize, range: R) -> core::ops::Range<usize> {
    let start = match range.start_bound() {
        Bound::Included(&start) => start,
        Bound::Excluded(start) => start
            .checked_add(1)
            .unwrap_or_else(|| slice_start_index_overflow_fail()),
        Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
        Bound::Included(end) => end.checked_add(1).unwrap_or_else(|| slice_end_index_overflow_fail()),
        Bound::Excluded(&end) => end,
        Bound::Unbounded => len,
    };

    if start > end {
        slice_index_order_fail(start, end);
    }
    if end > len {
        slice_end_index_len_fail(end, len);
    }

    start..end
}

impl<'a, T, S: ?Sized + Storage<T>> RawCursor<'a, T, S> {
    pub(crate) const IS_ZS: bool = core::mem::size_of::<T>() == 0;
    const ZS_PTR: *mut T = NonNull::<T>::dangling().as_ptr();

    #[inline]
    pub(crate) fn new<R>(vec: &'a mut GenericVec<T, S>, range: R) -> Self
    where
        R: RangeBounds<usize>,
    {
        unsafe {
            let mut raw_vec = NonNull::from(vec);
            let vec = raw_vec.as_mut();
            let old_vec_len = vec.len();

            let range = check_range(old_vec_len, range);
            let (start, end) = (range.start, range.end);

            if Self::IS_ZS {
                vec.set_len_unchecked(start);

                Self {
                    vec: raw_vec,
                    old_vec_len,
                    write_front: start as _,
                    read_front: start as _,
                    read_back: end as _,
                    write_back: end as _,
                    mark: PhantomData,
                }
            } else {
                let range = vec[range].as_mut_ptr_range();

                vec.set_len_unchecked(start);

                Self {
                    vec: raw_vec,
                    old_vec_len,
                    write_front: range.start,
                    read_front: range.start,
                    read_back: range.end,
                    write_back: range.end,
                    mark: PhantomData,
                }
            }
        }
    }

    /// Skip all the remaining elements, and ensure that the [`GenericVec`] is
    /// valid
    pub fn finish(&mut self) {
        unsafe {
            if self.old_vec_len == 0 {
                return
            }

            self.skip_n_front(self.remaining());

            if Self::IS_ZS {
                let front_len = self.write_front as usize;
                let back_len = self.old_vec_len.wrapping_sub(self.write_back as usize);
                let len = front_len + back_len;
                self.vec.as_mut().set_len_unchecked(len);
            } else {
                let start = self.vec.as_mut().as_mut_ptr();
                let range = start..start.add(self.old_vec_len);

                let front_len = self.write_front.offset_from(range.start) as usize;
                let back_len = range.end.offset_from(self.write_back) as usize;
                let len = front_len + back_len;

                if self.write_front != self.write_back {
                    self.write_front.copy_from(self.write_back, back_len);
                }

                self.vec.as_mut().set_len_unchecked(len);
            }
        }
    }

    /// Check if the both write pointers are and the end of the vector
    pub(crate) fn at_back_of_vec(&self) -> bool {
        unsafe {
            let vec = self.vec.as_ref();
            let end = vec.as_ptr().add(self.old_vec_len);
            end == self.write_back && end == self.write_front
        }
    }

    /// Get a mutable reference to the underlying vector
    pub(crate) unsafe fn vec_mut(&mut self) -> &mut GenericVec<T, S> { unsafe { self.vec.as_mut() } }

    /// The number of remaining elements in range of this `RawCursor`
    ///
    /// The `RawCursor` is complete when there are 0 remaining elements
    #[inline]
    pub fn remaining(&self) -> usize {
        if Self::IS_ZS {
            (self.read_back as usize).wrapping_sub(self.read_front as usize)
        } else {
            unsafe { self.read_back.offset_from(self.read_front) as usize }
        }
    }

    /// Returns `true` if the `RawCursor` is complete
    #[inline]
    pub fn is_complete(&self) -> bool { self.read_back == self.read_front }

    /// Returns `true` if the `RawCursor` is complete
    #[inline]
    pub fn is_write_complete(&self) -> bool { self.write_back == self.write_front }

    /// Returns a reference to the next element if the `RawCursor`
    ///
    /// Note: this does *not* advance the `RawCursor`
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be complete
    #[inline]
    pub unsafe fn front(&mut self) -> &mut T {
        if Self::IS_ZS {
            unsafe { &mut *Self::ZS_PTR }
        } else {
            unsafe { &mut *self.read_front }
        }
    }

    /// Returns a reference to the last element if the `RawCursor`
    ///
    /// Note: this does *not* advance the `RawCursor`
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be complete
    #[inline]
    pub unsafe fn back(&mut self) -> &mut T {
        if Self::IS_ZS {
            unsafe { &mut *Self::ZS_PTR }
        } else {
            unsafe { &mut *self.read_back.sub(1) }
        }
    }

    /// Removes the next element of the `RawCursor`, and the underlying [`GenericVec`]
    /// and advances the `RawCursor`
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be complete
    #[inline]
    pub unsafe fn take_front(&mut self) -> T {
        debug_assert!(!self.is_complete(), "Cannot take from a complete RawCursor");

        unsafe {
            if Self::IS_ZS {
                self.read_front = (self.read_front as usize).wrapping_add(1) as _;
                Self::ZS_PTR.read()
            } else {
                let read_front = self.read_front;
                self.read_front = self.read_front.add(1);
                read_front.read()
            }
        }
    }

    /// Removes the last element of the `RawCursor`, and the underlying [`GenericVec`]
    /// and advances the `RawCursor`
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be complete
    #[inline]
    pub unsafe fn take_back(&mut self) -> T {
        debug_assert!(!self.is_complete(), "Cannot take from a complete RawCursor");

        unsafe {
            if Self::IS_ZS {
                self.read_back = (self.read_back as usize).wrapping_sub(1) as _;
                Self::ZS_PTR.read()
            } else {
                self.read_back = self.read_back.sub(1);
                self.read_back.read()
            }
        }
    }

    /// Check if it is safe to write to the front of the `RawCursor`
    pub fn can_write_front(&self) -> bool { !self.is_write_complete() && self.write_front != self.read_front }

    /// Check if it is safe to write to the back of the `RawCursor`
    pub fn can_write_back(&self) -> bool { !self.is_write_complete() && self.write_back != self.read_back }

    /// Returns the number of times you can safely call [`RawCursor::write_front`]
    pub fn write_slots_front(&self) -> usize {
        if Self::IS_ZS {
            (self.read_front as usize).wrapping_sub(self.write_front as usize)
        } else if self.is_write_complete() {
            0
        } else {
            unsafe { self.read_front.offset_from(self.write_front) as usize }
        }
    }

    /// Returns the number of times you can safely call [`RawCursor::write_back`]
    pub fn write_slots_back(&self) -> usize {
        if Self::IS_ZS {
            (self.write_back as usize).wrapping_sub(self.read_back as usize)
        } else if self.is_write_complete() {
            0
        } else {
            unsafe { self.write_back.offset_from(self.read_back) as usize }
        }
    }

    /// Skips the next element of the `RawCursor`, and keeps the element in the
    /// underlying [`GenericVec`] and advances the `RawCursor`
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be write-complete
    #[inline]
    pub unsafe fn write_front(&mut self, value: T) {
        debug_assert!(
            self.can_write_front(),
            "Cannot write to a complete `RawCursor` or if there are not empty slots at the front of the `RawCursor`"
        );

        unsafe {
            if Self::IS_ZS {
                core::mem::forget(value);
                self.write_front = (self.write_front as usize).wrapping_add(1) as _;
            } else {
                self.write_front.write(value);
                self.write_front = self.write_front.add(1);
            }
        }
    }

    /// Skips the next element of the `RawCursor`, and keeps the element in the
    /// underlying [`GenericVec`] and advances the `RawCursor`
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be write-complete
    #[inline]
    pub unsafe fn write_back(&mut self, value: T) {
        debug_assert!(
            self.can_write_back(),
            "Cannot write to a complete `RawCursor` or if there are not empty slots at the back of the `RawCursor`"
        );

        unsafe {
            if Self::IS_ZS {
                core::mem::forget(value);
                self.write_back = (self.write_back as usize).wrapping_sub(1) as _;
            } else {
                self.write_back = self.write_back.sub(1);
                self.write_back.write(value);
            }
        }
    }

    /// Skips the next element of the `RawCursor`, and keeps the element in the
    /// underlying [`GenericVec`] and advances the `RawCursor`
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be complete
    #[inline]
    pub unsafe fn skip_front(&mut self) {
        debug_assert!(!self.is_complete(), "Cannot skip from a complete RawCursor");

        unsafe {
            if Self::IS_ZS {
                self.skip_n_front(1);
            } else {
                if self.write_front as *const T != self.read_front {
                    self.write_front.copy_from_nonoverlapping(self.read_front, 1);
                }
                self.read_front = self.read_front.add(1);
                self.write_front = self.write_front.add(1);
            }
        }
    }

    /// Skips the last element of the `RawCursor`, and keeps the element in the
    /// underlying [`GenericVec`] and advances the `RawCursor`
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be complete
    #[inline]
    pub unsafe fn skip_back(&mut self) {
        debug_assert!(!self.is_complete(), "Cannot skip from a complete RawCursor");

        unsafe {
            if Self::IS_ZS {
                self.skip_n_back(1);
            } else {
                self.read_back = self.read_back.sub(1);
                self.write_back = self.write_back.sub(1);
                if self.write_back as *const T != self.read_back {
                    self.write_back.copy_from_nonoverlapping(self.read_back, 1);
                }
            }
        }
    }

    /// Skips the next `n` elements of the `RawCursor`, and keeps them in the
    /// underlying [`GenericVec`] and advances the `RawCursor`
    ///
    /// # Safety
    ///
    /// The `RawCursor` have at least n remaining elements
    #[inline]
    pub unsafe fn skip_n_front(&mut self, n: usize) {
        debug_assert!(self.remaining() >= n);

        unsafe {
            if Self::IS_ZS {
                self.read_front = (self.read_front as usize).wrapping_add(n) as _;
                self.write_front = (self.write_front as usize).wrapping_add(n) as _;
            } else {
                if self.write_front as *const T != self.read_front {
                    self.write_front.copy_from(self.read_front, n);
                }
                self.read_front = self.read_front.add(n);
                self.write_front = self.write_front.add(n);
            }
        }
    }

    /// Skips the last `n` elements of the `RawCursor`, and keeps them in the
    /// underlying [`GenericVec`] and advances the `RawCursor`
    ///
    /// # Safety
    ///
    /// The `RawCursor` have at least n remaining elements
    #[inline]
    pub unsafe fn skip_n_back(&mut self, n: usize) {
        debug_assert!(self.remaining() >= n);

        unsafe {
            if Self::IS_ZS {
                self.read_back = (self.read_back as usize).wrapping_sub(n) as _;
                self.write_back = (self.write_back as usize).wrapping_sub(n) as _;
            } else {
                self.read_back = self.read_back.sub(n);
                self.write_back = self.write_back.sub(n);
                if self.write_back as *const T != self.read_back {
                    self.write_back.copy_from(self.read_back, n);
                }
            }
        }
    }

    // TODO: this doc is bad, improve it
    /// Write the value into empty space at the front of the `RawCursor`
    ///
    /// # Safety
    ///
    /// The `RawCursor` must have taken at least 1 more element than it has
    /// written from the front
    pub unsafe fn consume_write_front(&mut self, value: T) {
        if Self::IS_ZS {
            core::mem::forget(value);
            self.write_front = (self.write_front as usize).wrapping_add(1) as _;
        } else {
            unsafe {
                self.write_front.write(value);
                self.write_front = self.write_front.add(1);
            }
        }
    }

    // TODO: this doc is bad, improve it
    /// Write the value into empty space at the back of the `RawCursor`
    ///
    /// # Safety
    ///
    /// The `RawCursor` must have taken at least 1 more element than it has
    /// written from the back
    pub unsafe fn consume_write_back(&mut self, value: T) {
        if Self::IS_ZS {
            core::mem::forget(value);
            self.write_back = (self.write_back as usize).wrapping_sub(1) as _;
        } else {
            unsafe {
                self.write_back = self.write_back.sub(1);
                self.write_back.write(value);
            }
        }
    }

    // TODO: this doc is bad, improve it
    /// Write the slice into empty space at the front of the `RawCursor`
    ///
    /// # Safety
    ///
    /// The `RawCursor` must have taken at least `slice.len()` more element than it has
    /// written from the front
    pub unsafe fn consume_write_slice_front(&mut self, slice: &[T]) {
        unsafe {
            if Self::IS_ZS {
                self.write_front = (self.write_front as usize).wrapping_add(slice.len()) as _;
            } else {
                self.write_front.copy_from_nonoverlapping(slice.as_ptr(), slice.len());
                self.write_front = self.write_front.add(slice.len());
            }
        }
    }

    // TODO: this doc is bad, improve it
    /// Write the slice into empty space at the back of the `RawCursor`
    ///
    /// # Safety
    ///
    /// The `RawCursor` must have taken at least `slice.len()` more element than it has
    /// written from the back
    pub unsafe fn consume_write_slice_back(&mut self, slice: &[T]) {
        unsafe {
            if Self::IS_ZS {
                self.write_back = (self.write_back as usize).wrapping_sub(slice.len()) as _;
            } else {
                self.write_back = self.write_back.sub(slice.len());
                self.write_back.copy_from_nonoverlapping(slice.as_ptr(), slice.len());
            }
        }
    }

    /// Assert that there is at least `space` elements of room left to write into
    /// the `RawCursor`, and the underlying [`GenericVec`]
    ///
    /// # Safety
    ///
    /// the `RawCursor` must be complete
    pub unsafe fn assert_space(&mut self, space: usize) {
        debug_assert!(
            self.is_complete(),
            "You can only call `assert_space` on a complete `RawCursor`, this is UB in release mode!"
        );
        unsafe {
            if Self::IS_ZS {
                let write_space = (self.write_back as usize).wrapping_sub(self.write_front as usize);

                if let Some(increase_by) = space.checked_sub(write_space) {
                    self.write_back = (self.write_back as usize).wrapping_add(increase_by) as _;
                    self.old_vec_len += increase_by;
                }
            } else {
                let write_space = self.write_back.offset_from(self.write_front) as usize;

                if write_space >= space {
                    return
                }

                let capacity = self.vec.as_ref().capacity();
                let start = self.vec.as_mut().as_mut_ptr();
                let range = start..start.add(self.old_vec_len);

                let front_len = self.write_front.offset_from(range.start) as usize;
                let back_len = range.end.offset_from(self.write_back) as usize;
                let len = front_len + back_len;

                if len + space > capacity {
                    let wf = self.write_front.offset_from(range.start) as usize;
                    let wb = self.write_back.offset_from(range.start) as usize;
                    let rf = self.read_front.offset_from(range.start) as usize;
                    let rb = self.read_back.offset_from(range.start) as usize;

                    let vec = self.vec.as_mut();
                    vec.storage.reserve(len + space);

                    let start = vec.as_mut_ptr();
                    self.write_front = start.add(wf);
                    self.write_back = start.add(wb);
                    self.read_front = start.add(rf);
                    self.read_back = start.add(rb);
                }

                let increase_by = space.wrapping_sub(write_space);
                let new_write_back = self.write_back.add(increase_by);
                new_write_back.copy_from(self.write_back, back_len);
                self.write_back = new_write_back;
                self.old_vec_len += increase_by;
            }
        }
    }
}
