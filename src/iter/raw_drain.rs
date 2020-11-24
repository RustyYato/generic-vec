use crate::{GenericVec, RawVec};
use core::{marker::PhantomData, slice::SliceIndex};

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

impl<'a, A: ?Sized + RawVec> RawDrain<'a, A> {
    #[inline]
    pub fn new<R>(vec: &'a mut GenericVec<A>, range: R) -> Self
    where
        R: SliceIndex<[A::Item], Output = [A::Item]>,
    {
        unsafe {
            let raw_vec = vec as *mut GenericVec<A>;
            let vec = &mut *raw_vec;
            let ptr = vec.as_mut_ptr();
            let range = vec[range].as_mut_ptr_range();

            let old_vec_len = vec.len();
            vec.set_len_unchecked(range.start.offset_from(ptr) as usize);

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

    // #[inline]
    // fn start_ptr(&mut self) -> *mut A::Item {
    //     unsafe {
    //         let ptr = self.vec_len as *mut _ as *mut u8;
    //         let ptr = ptr.add(core::mem::size_of_val(self.vec_len));
    //         let offset = ptr.align_offset(core::mem::align_of::<A::Item>());
    //         ptr.add(offset).cast::<A::Item>()
    //     }
    // }

    #[inline]
    pub fn remaining(&self) -> usize {
        unsafe { self.read_back.offset_from(self.read_front) as usize }
    }

    #[inline]
    pub fn is_complete(&self) -> bool {
        self.read_back == self.read_front
    }

    #[inline]
    pub unsafe fn front(&mut self) -> &mut A::Item {
        unsafe { &mut *self.read_front }
    }

    #[inline]
    pub unsafe fn back(&mut self) -> &mut A::Item {
        unsafe { &mut *self.read_back.sub(1) }
    }

    #[inline]
    pub unsafe fn take_front(&mut self) -> A::Item {
        debug_assert!(!self.is_complete(), "Cannot take from a complete RawDrain");

        unsafe {
            let read_front = self.read_front;
            self.read_front = self.read_front.add(1);
            read_front.read()
        }
    }

    #[inline]
    pub unsafe fn take_back(&mut self) -> A::Item {
        debug_assert!(!self.is_complete(), "Cannot take from a complete RawDrain");

        unsafe {
            self.read_back = self.read_back.sub(1);
            self.read_back.read()
        }
    }

    #[inline]
    pub unsafe fn skip_front(&mut self) {
        debug_assert!(!self.is_complete(), "Cannot skip from a complete RawDrain");

        unsafe {
            if self.write_front as *const A::Item != self.read_front {
                self.write_front
                    .copy_from_nonoverlapping(self.read_front, 1);
            }
            self.read_front = self.read_front.add(1);
            self.write_front = self.write_front.add(1);
        }
    }

    #[inline]
    pub unsafe fn skip_back(&mut self) {
        debug_assert!(!self.is_complete(), "Cannot skip from a complete RawDrain");

        unsafe {
            self.read_back = self.read_back.sub(1);
            self.write_back = self.write_back.sub(1);
            if self.write_back as *const A::Item != self.read_back {
                self.write_back.copy_from_nonoverlapping(self.read_back, 1);
            }
        }
    }

    #[inline]
    pub unsafe fn skip_n_front(&mut self, n: usize) {
        debug_assert!(self.remaining() >= n);

        unsafe {
            if self.write_front as *const A::Item != self.read_front {
                self.write_front.copy_from(self.read_front, n);
            }
            self.read_front = self.read_front.add(n);
            self.write_front = self.write_front.add(n);
        }
    }

    #[inline]
    pub unsafe fn skip_n_back(&mut self, n: usize) {
        debug_assert!(self.remaining() >= n);

        unsafe {
            self.read_back = self.read_back.sub(n);
            self.write_back = self.write_back.sub(n);
            if self.write_back as *const A::Item != self.read_back {
                self.write_back.copy_from(self.read_back, n);
            }
        }
    }

    pub unsafe fn consume_write_slice_front(&mut self, slice: &[A::Item]) {
        unsafe {
            self.write_front
                .copy_from_nonoverlapping(slice.as_ptr(), slice.len());
            self.write_front = self.write_front.add(slice.len());
        }
    }

    pub fn assert_space(&mut self, space: usize) {
        unsafe {
            let write_space = self.write_back.offset_from(self.write_front) as usize;

            if write_space >= space {
                return;
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

            let new_write_back = self.write_back.add(space.wrapping_sub(write_space));
            new_write_back.copy_from(self.write_back, back_len);
            self.write_back = new_write_back;
            self.old_vec_len += space;
        }
    }
}
