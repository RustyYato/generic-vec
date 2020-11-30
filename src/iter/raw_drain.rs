use crate::{GenericVec, RawVec};
use core::{
    marker::PhantomData,
    ops::{Bound, RangeBounds},
    ptr::NonNull,
};

pub struct RawDrain<'a, A: ?Sized + RawVec> {
    vec: *mut GenericVec<A>,
    old_vec_len: usize,
    write_front: *mut A::Item,
    read_front: *mut A::Item,
    read_back: *mut A::Item,
    write_back: *mut A::Item,
    mark: PhantomData<&'a mut GenericVec<A>>,
}

impl<A: ?Sized + RawVec> Drop for RawDrain<'_, A> {
    fn drop(&mut self) {
        unsafe {
            self.skip_n_front(self.remaining());

            if Self::IS_ZS {
                let front_len = self.write_front as usize;
                let back_len = self.old_vec_len.wrapping_sub(self.write_back as usize);
                let len = front_len + back_len;
                (*self.vec).set_len_unchecked(len);
            } else {
                let start = (*self.vec).as_mut_ptr();
                let range = start..start.add(self.old_vec_len);

                let front_len = self.write_front.offset_from(range.start) as usize;
                let back_len = range.end.offset_from(self.write_back) as usize;
                let len = front_len + back_len;

                if self.write_front != self.write_back {
                    self.write_front.copy_from(self.write_back, back_len);
                }

                (*self.vec).set_len_unchecked(len);
            }
        }
    }
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

impl<'a, A: ?Sized + RawVec> RawDrain<'a, A> {
    pub(crate) const IS_ZS: bool = core::mem::size_of::<A::Item>() == 0;
    const ZS_PTR: *mut A::Item = NonNull::<A::Item>::dangling().as_ptr();

    #[inline]
    pub fn new<R>(vec: &'a mut GenericVec<A>, range: R) -> Self
    where
        R: RangeBounds<usize>,
    {
        unsafe {
            let raw_vec = vec as *mut GenericVec<A>;
            let vec = &mut *raw_vec;
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

    #[inline]
    pub fn remaining(&self) -> usize {
        if Self::IS_ZS {
            (self.read_back as usize).wrapping_sub(self.read_front as usize)
        } else {
            unsafe { self.read_back.offset_from(self.read_front) as usize }
        }
    }

    #[inline]
    pub fn is_complete(&self) -> bool { self.read_back == self.read_front }

    #[inline]
    pub unsafe fn front(&mut self) -> &mut A::Item {
        if Self::IS_ZS {
            unsafe { &mut *Self::ZS_PTR }
        } else {
            unsafe { &mut *self.read_front }
        }
    }

    #[inline]
    pub unsafe fn back(&mut self) -> &mut A::Item {
        if Self::IS_ZS {
            unsafe { &mut *Self::ZS_PTR }
        } else {
            unsafe { &mut *self.read_back.sub(1) }
        }
    }

    #[inline]
    pub unsafe fn take_front(&mut self) -> A::Item {
        debug_assert!(!self.is_complete(), "Cannot take from a complete RawDrain");

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

    #[inline]
    pub unsafe fn take_back(&mut self) -> A::Item {
        debug_assert!(!self.is_complete(), "Cannot take from a complete RawDrain");

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

    #[inline]
    pub unsafe fn skip_front(&mut self) {
        debug_assert!(!self.is_complete(), "Cannot skip from a complete RawDrain");

        unsafe {
            if Self::IS_ZS {
                self.skip_n_front(1);
            } else {
                if self.write_front as *const A::Item != self.read_front {
                    self.write_front.copy_from_nonoverlapping(self.read_front, 1);
                }
                self.read_front = self.read_front.add(1);
                self.write_front = self.write_front.add(1);
            }
        }
    }

    #[inline]
    pub unsafe fn skip_back(&mut self) {
        debug_assert!(!self.is_complete(), "Cannot skip from a complete RawDrain");

        unsafe {
            if Self::IS_ZS {
                self.skip_n_back(1);
            } else {
                self.read_back = self.read_back.sub(1);
                self.write_back = self.write_back.sub(1);
                if self.write_back as *const A::Item != self.read_back {
                    self.write_back.copy_from_nonoverlapping(self.read_back, 1);
                }
            }
        }
    }

    #[inline]
    pub unsafe fn skip_n_front(&mut self, n: usize) {
        debug_assert!(self.remaining() >= n);

        unsafe {
            if Self::IS_ZS {
                self.read_front = (self.read_front as usize).wrapping_add(n) as _;
                self.write_front = (self.write_front as usize).wrapping_add(n) as _;
            } else {
                if self.write_front as *const A::Item != self.read_front {
                    self.write_front.copy_from(self.read_front, n);
                }
                self.read_front = self.read_front.add(n);
                self.write_front = self.write_front.add(n);
            }
        }
    }

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
                if self.write_back as *const A::Item != self.read_back {
                    self.write_back.copy_from(self.read_back, n);
                }
            }
        }
    }

    pub unsafe fn consume_write_slice_front(&mut self, slice: &[A::Item]) {
        unsafe {
            if Self::IS_ZS {
                self.write_front = (self.write_front as usize).wrapping_add(slice.len()) as _;
            } else {
                self.write_front.copy_from_nonoverlapping(slice.as_ptr(), slice.len());
                self.write_front = self.write_front.add(slice.len());
            }
        }
    }

    pub fn assert_space(&mut self, space: usize) {
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

                let start = (*self.vec).as_mut_ptr();
                let capacity = (*self.vec).capacity();
                let range = start..start.add(self.old_vec_len);

                let front_len = self.write_front.offset_from(range.start) as usize;
                let back_len = range.end.offset_from(self.write_back) as usize;
                let len = front_len + back_len;

                if len + space > capacity {
                    let wf = self.write_front.offset_from(range.start) as usize;
                    let wb = self.write_back.offset_from(range.start) as usize;
                    let rf = self.read_front.offset_from(range.start) as usize;
                    let rb = self.read_back.offset_from(range.start) as usize;

                    let vec = &mut *self.vec;
                    vec.raw.reserve(len + space);

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
