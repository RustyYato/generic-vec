use crate::{GenericVec, Storage};
use core::{marker::PhantomData, ops::Range, ptr::NonNull};

/// This struct is created by [`GenericVec::raw_cursor`]. See its documentation for more.
pub struct RawCursor<'a, T, S: ?Sized + Storage<T>> {
    vec: NonNull<GenericVec<T, S>>,
    old_vec_len: usize,
    write_front: *mut T,
    read_front: *mut T,
    read_back: *mut T,
    write_back: *mut T,
    mark: PhantomData<&'a mut GenericVec<T, S>>,
}

unsafe impl<T: Send, S: ?Sized + Storage<T> + Send> Send for RawCursor<'_, T, S> {}
unsafe impl<T: Sync, S: ?Sized + Storage<T> + Sync> Sync for RawCursor<'_, T, S> {}

impl<T, S: ?Sized + Storage<T>> Drop for RawCursor<'_, T, S> {
    fn drop(&mut self) { self.finish() }
}

impl<'a, T, S: ?Sized + Storage<T>> RawCursor<'a, T, S> {
    pub(crate) const IS_ZS: bool = core::mem::size_of::<T>() == 0;
    const ZS_PTR: *mut T = NonNull::<T>::dangling().as_ptr();

    #[inline]
    pub(crate) fn new(vec: &'a mut GenericVec<T, S>, Range { start, end }: Range<usize>) -> Self {
        unsafe {
            let mut raw_vec = NonNull::from(vec);
            let vec = raw_vec.as_mut();
            let old_vec_len = vec.len();

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
                let range = vec[start..end].as_mut_ptr_range();

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

            self.skip_n_front(self.len());

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
    /// The `RawCursor` is empty when there are 0 remaining elements
    #[inline]
    pub fn len(&self) -> usize {
        if Self::IS_ZS {
            (self.read_back as usize).wrapping_sub(self.read_front as usize)
        } else {
            unsafe { self.read_back.offset_from(self.read_front) as usize }
        }
    }

    /// Returns `true` if the `RawCursor` is empty
    #[inline]
    pub fn is_empty(&self) -> bool { self.read_back == self.read_front }

    /// Returns `true` if the `RawCursor` is has no unfilled slots
    /// and the `RawCursor` is empty
    #[inline]
    pub fn is_write_empty(&self) -> bool { self.write_back == self.write_front }

    /// Returns true if there is an unfilled slot at the front
    /// of the `RawCursor`
    pub fn is_write_front_empty(&self) -> bool {
        self.is_write_empty() || (self.write_front == self.read_front && !self.is_empty())
    }

    /// Returns true if there is an unfilled slot at the back
    /// of the `RawCursor`
    pub fn is_write_back_empty(&self) -> bool {
        self.is_write_empty() || (self.write_back == self.read_back && !self.is_empty())
    }

    /// Returns the number of unfilled slots if the `RawCursor` is empty
    /// if the `RawCursor` is not empty, the behavior is unspecified
    pub fn write_len(&self) -> usize {
        if Self::IS_ZS {
            (self.write_back as usize).wrapping_sub(self.write_front as usize)
        } else if self.is_write_empty() {
            0
        } else {
            unsafe { self.write_back.offset_from(self.write_front) as usize }
        }
    }

    /// Returns the number of unfilled slots at the front
    /// of the `RawCursor`
    pub fn write_front_len(&self) -> usize {
        if self.is_empty() {
            self.write_len()
        } else {
            if Self::IS_ZS {
                (self.read_front as usize).wrapping_sub(self.write_front as usize)
            } else if self.is_write_empty() {
                0
            } else {
                unsafe { self.read_front.offset_from(self.write_front) as usize }
            }
        }
    }

    /// Returns the number of unfilled slots at the back
    /// of the `RawCursor`
    pub fn write_back_len(&self) -> usize {
        if self.is_empty() {
            self.write_len()
        } else {
            if Self::IS_ZS {
                (self.write_back as usize).wrapping_sub(self.read_back as usize)
            } else if self.is_write_empty() {
                0
            } else {
                unsafe { self.write_back.offset_from(self.read_back) as usize }
            }
        }
    }

    /// Returns a reference to the next element of the `RawCursor`.
    ///
    /// Note: this does *not* advance the `RawCursor` or
    /// change the number of unfilled slots
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be empty
    #[inline]
    pub unsafe fn front(&self) -> &T {
        if Self::IS_ZS {
            unsafe { &*Self::ZS_PTR }
        } else {
            unsafe { &*self.read_front }
        }
    }

    /// Returns a mutable reference to the next element of the `RawCursor`.
    ///
    /// Note: this does *not* advance the `RawCursor` or
    /// change the number of unfilled slots
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be empty
    #[inline]
    pub unsafe fn front_mut(&mut self) -> &mut T {
        if Self::IS_ZS {
            unsafe { &mut *Self::ZS_PTR }
        } else {
            unsafe { &mut *self.read_front }
        }
    }

    /// Returns a reference to the last element of the `RawCursor`.
    ///
    /// Note: this does *not* advance the `RawCursor` or
    /// change the number of unfilled slots
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be empty
    #[inline]
    pub unsafe fn back(&self) -> &T {
        if Self::IS_ZS {
            unsafe { &*Self::ZS_PTR }
        } else {
            unsafe { &*self.read_back.sub(1) }
        }
    }

    /// Returns a mutable reference to the last element of the `RawCursor`.
    ///
    /// Note: this does *not* advance the `RawCursor` or
    /// change the number of unfilled slots
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be empty
    #[inline]
    pub unsafe fn back_mut(&mut self) -> &mut T {
        if Self::IS_ZS {
            unsafe { &mut *Self::ZS_PTR }
        } else {
            unsafe { &mut *self.read_back.sub(1) }
        }
    }

    /// Removes the next element of the `RawCursor`
    /// and removes it from the underlying [`GenericVec`]
    ///
    /// Advances the `RawCursor` by 1 element
    ///
    /// Creates 1 unfilled slot at the front of the `RawCursor`.
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be empty
    #[inline]
    pub unsafe fn take_front(&mut self) -> T {
        debug_assert!(!self.is_empty(), "Cannot take from a empty `RawCursor`");

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

    /// Removes the last element of the `Cursor
    /// and removes it from the underlying [`GenericVec`]
    ///
    /// Advances the `RawCursor` by 1 element
    ///
    /// Creates 1 unfilled slot at the back of the `RawCursor`.
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be empty
    #[inline]
    pub unsafe fn take_back(&mut self) -> T {
        debug_assert!(!self.is_empty(), "Cannot take from a empty `RawCursor`");

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

    /// Drops the next element of the `RawCursor`
    /// and removes them it the underlying [`GenericVec`]
    ///
    /// Advances the `RawCursor` by 1 element
    ///
    /// Creates 1 unfilled slot at the front of the `RawCursor`.
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be empty
    #[inline]
    pub unsafe fn drop_front(&mut self) {
        debug_assert!(!self.is_empty(), "Cannot drop an element from a empty `RawCursor`");

        unsafe {
            if Self::IS_ZS {
                self.read_front = (self.read_front as usize).wrapping_add(1) as _;
                Self::ZS_PTR.drop_in_place()
            } else {
                let read_front = self.read_front;
                self.read_front = self.read_front.add(1);
                read_front.drop_in_place()
            }
        }
    }

    /// Drops the last element of the `RawCursor`
    /// and removes them it the underlying [`GenericVec`]
    ///
    /// Advances the `RawCursor` by 1 element
    ///
    /// Creates 1 unfilled slot at the back of the `RawCursor`.
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be empty
    #[inline]
    pub unsafe fn drop_back(&mut self) {
        debug_assert!(!self.is_empty(), "Cannot drop an element from a empty `RawCursor`");

        unsafe {
            if Self::IS_ZS {
                self.read_back = (self.read_back as usize).wrapping_sub(1) as _;
                Self::ZS_PTR.drop_in_place();
            } else {
                self.read_back = self.read_back.sub(1);
                self.read_back.drop_in_place()
            }
        }
    }

    /// Drops the next `n` elements of the `RawCursor`
    /// and removes them from the underlying [`GenericVec`]
    ///
    /// Advances the `RawCursor` by `n` elements
    ///
    /// Creates `n` unfilled slots at the front of the `RawCursor`.
    ///
    /// # Safety
    ///
    /// The `RawCursor`'s length must be at least equal to `n`
    #[inline]
    pub unsafe fn drop_n_front(&mut self, n: usize) {
        debug_assert!(
            self.len() >= n,
            "Cannot drop {} elements from a `RawCursor` of length {}",
            n,
            self.len()
        );

        unsafe {
            let ptr = if Self::IS_ZS {
                self.read_front = (self.read_front as usize).wrapping_add(n) as _;
                Self::ZS_PTR
            } else {
                let read_front = self.read_front;
                self.read_front = self.read_front.add(n);
                read_front
            };

            core::ptr::slice_from_raw_parts_mut(ptr, n).drop_in_place()
        }
    }

    /// Drops the last `n` elements of the `RawCursor`
    /// and removes them from the underlying [`GenericVec`]
    ///
    /// Advances the `RawCursor` by `n` elements
    ///
    /// Creates `n` unfilled slots at the back of the `RawCursor`.
    ///
    /// # Safety
    ///
    /// The `RawCursor`'s length must be at least equal to `n`
    #[inline]
    pub unsafe fn drop_n_back(&mut self, n: usize) {
        debug_assert!(
            self.len() >= n,
            "Cannot drop {} elements from a `RawCursor` of length {}",
            n,
            self.len()
        );

        unsafe {
            let ptr = if Self::IS_ZS {
                self.read_back = (self.read_back as usize).wrapping_sub(n) as _;
                Self::ZS_PTR
            } else {
                self.read_back = self.read_back.sub(n);
                self.read_back
            };

            core::ptr::slice_from_raw_parts_mut(ptr, n).drop_in_place()
        }
    }

    /// Writes `value` into the unfilled slot at the front of the
    /// `RawCursor` if there is an unfilled slot at the front of the `RawCursor`
    ///
    /// Fills in 1 unfilled slot at the front of the `RawCursor`
    ///
    /// # Safety
    ///
    /// There must be at least 1 unfilled slot at the front of the `RawCursor`
    #[inline]
    pub unsafe fn write_front(&mut self, value: T) {
        debug_assert!(
            !self.is_write_front_empty(),
            "Cannot write to a empty `RawCursor` or if there are not unfilled slots at the front of the `RawCursor`"
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

    /// Writes `value` into the unfilled slot at the back of the
    /// `RawCursor` if there is an unfilled slot at the back of the `RawCursor`
    ///
    /// Fills in 1 unfilled slot at the back of the `RawCursor`
    ///
    /// # Safety
    ///
    /// There must be at least 1 unfilled slot at the back of the `RawCursor`
    #[inline]
    pub unsafe fn write_back(&mut self, value: T) {
        debug_assert!(
            !self.is_write_back_empty(),
            "Cannot write to a empty `RawCursor` or if there are not unfilled slots at the back of the `RawCursor`"
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

    /// Moves `slice` into the unfilled slots at the front of the
    /// `RawCursor` if there are `slice.len()` unfilled slots at the
    /// front of the `RawCursor`
    ///
    /// Fills in `slice.len()` unfilled slots at the front of the `RawCursor`
    ///
    /// # Safety
    ///
    /// * There must be at least `slice.len()` unfilled slots
    ///   at the front of the `RawCursor`
    /// * You must not drop any of the values in `slice`
    pub unsafe fn write_slice_front(&mut self, slice: &[T]) {
        unsafe {
            let write_front_len = self.write_front_len();
            debug_assert!(
                write_front_len >= slice.len(),
                "Cannot write {} elements, only {} slots remaining",
                slice.len(),
                write_front_len
            );

            if Self::IS_ZS {
                self.write_front = (self.write_front as usize).wrapping_add(slice.len()) as _;
            } else {
                self.write_front.copy_from_nonoverlapping(slice.as_ptr(), slice.len());
                self.write_front = self.write_front.add(slice.len());
            }
        }
    }

    /// Moves `slice` into the unfilled slots at the back of the
    /// `RawCursor` if there are `slice.len()` unfilled slots at the
    /// back of the `RawCursor`
    ///
    /// Fills in `slice.len()` unfilled slots at the back of the `RawCursor`
    ///
    /// # Safety
    ///
    /// * There must be at least `slice.len()` unfilled slots
    ///   at the back of the `RawCursor`
    /// * You must not drop any of the values in `slice`
    pub unsafe fn write_slice_back(&mut self, slice: &[T]) {
        let write_back_len = self.write_back_len();
        debug_assert!(
            write_back_len >= slice.len(),
            "Cannot write {} elements, only {} slots remaining",
            slice.len(),
            write_back_len
        );

        unsafe {
            if Self::IS_ZS {
                self.write_back = (self.write_back as usize).wrapping_sub(slice.len()) as _;
            } else {
                self.write_back = self.write_back.sub(slice.len());
                self.write_back.copy_from_nonoverlapping(slice.as_ptr(), slice.len());
            }
        }
    }

    /// Skips the next element of the `RawCursor`
    /// and keeps it in the underlying [`GenericVec`]
    ///
    /// Advances the `RawCursor` by 1 element
    ///
    /// Does not change the number of unfilled slots.
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be empty
    #[inline]
    pub unsafe fn skip_front(&mut self) {
        debug_assert!(!self.is_empty(), "Cannot skip elements from a empty `RawCursor`");

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

    /// Skips the last element of the `RawCursor`
    /// and keeps it in the underlying [`GenericVec`]
    ///
    /// Advances the `RawCursor` by 1 element
    ///
    /// Does not change the number of unfilled slots.
    ///
    /// # Safety
    ///
    /// The `RawCursor` must not be empty
    #[inline]
    pub unsafe fn skip_back(&mut self) {
        debug_assert!(!self.is_empty(), "Cannot skip elements from a empty `RawCursor`");

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

    /// Skips the next `n` elements of the `RawCursor`
    /// and keeps them in the underlying [`GenericVec`]
    ///
    /// Advances the `RawCursor` by `n` elements
    ///
    /// Does not change the number of unfilled slots.
    ///
    /// # Safety
    ///
    /// The `RawCursor`'s length must be at least equal to `n`
    #[inline]
    pub unsafe fn skip_n_front(&mut self, n: usize) {
        debug_assert!(
            self.len() >= n,
            "Cannot skip {} elements from a `RawCursor` of length {}",
            n,
            self.len()
        );

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

    /// Skips the last `n` elements of the `RawCursor`
    /// and keeps them in the underlying [`GenericVec`]
    ///
    /// Advances the `RawCursor` by `n` elements
    ///
    /// Does not change the number of unfilled slots.
    ///
    /// # Safety
    ///
    /// The `RawCursor`'s length must be at least equal to `n`
    #[inline]
    pub unsafe fn skip_n_back(&mut self, n: usize) {
        debug_assert!(
            self.len() >= n,
            "Cannot skip {} elements from a `RawCursor` of length {}",
            n,
            self.len()
        );

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

    /// Reserve at least space unfilled slots in the `RawCursor`
    ///
    /// # Panic
    ///
    /// * Panics if the `RawCursor` is not empty
    /// * May panic if the underlying [`GenericVec`] cannot
    ///   reserve more space
    pub fn reserve(&mut self, space: usize) {
        assert!(self.is_empty(), "You can only call `reserve` on a empty `RawCursor`");
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
