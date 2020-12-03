#[allow(unused_imports)]
use crate::{iter::RawCursor, GenericVec, Storage};

/// This struct is created by [`GenericVec::cursor`]. See its documentation for more.
pub struct Cursor<'a, T, S: ?Sized + Storage<T>> {
    raw: RawCursor<'a, T, S>,
}

impl<'a, T, S: ?Sized + Storage<T>> Cursor<'a, T, S> {
    #[inline]
    pub(crate) fn new(raw: RawCursor<'a, T, S>) -> Self { Self { raw } }

    /// Get a mutable reference to the underlying `RawCursor`
    ///
    /// Updating the state of the underlying `RawCursor` does
    /// update the state of this `Cursor`
    pub fn as_raw_cursor_mut(&mut self) -> &mut RawCursor<'a, T, S> { &mut self.raw }

    /// The number of remaining elements in range of this `Cursor`
    ///
    /// The `Cursor` is empty when there are 0 remaining elements
    #[inline]
    pub fn len(&self) -> usize { self.raw.len() }

    /// Returns `true` if the `Cursor` is empty
    #[inline]
    pub fn is_empty(&self) -> bool { self.raw.is_empty() }

    /// Returns `true` if the `Cursor` is has no unfilled slots
    /// and the `Cursor` is empty
    #[inline]
    pub fn is_write_empty(&self) -> bool { self.raw.is_write_empty() }

    /// Returns true if there is an unfilled slot at the front
    /// of the `Cursor`
    #[inline]
    pub fn is_write_front_empty(&self) -> bool { self.raw.is_write_front_empty() }

    /// Returns true if there is an unfilled slot at the back
    /// of the `Cursor`
    #[inline]
    pub fn is_write_back_empty(&self) -> bool { self.raw.is_write_back_empty() }

    /// Returns the number of unfilled slots if the `Cursor` is empty
    /// if the `Cursor` is not empty, the behavior is unspecified
    #[inline]
    pub fn write_len(&self) -> usize { self.raw.write_len() }

    /// Returns the number of unfilled slots at the front
    /// of the `Cursor`
    #[inline]
    pub fn write_front_len(&self) -> usize { self.raw.write_front_len() }

    /// Returns the number of unfilled slots at the back
    /// of the `Cursor`
    #[inline]
    pub fn write_back_len(&self) -> usize { self.raw.write_back_len() }

    /// Returns a reference to the next element of the `Cursor`.
    /// Returns `None` if the `Cursor` is empty
    ///
    /// Note: this does *not* advance the `Cursor` or
    /// change the number of unfilled slots
    #[inline]
    pub fn front(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some(self.raw.front()) }
        }
    }

    /// Returns a mutable reference to the next element of the `Cursor`.
    /// Returns `None` if the `Cursor` is empty
    ///
    /// Note: this does *not* advance the `Cursor` or
    /// change the number of unfilled slots
    #[inline]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some(self.raw.front_mut()) }
        }
    }

    /// Returns a reference to the last element of the `Cursor`.
    /// Returns `None` if the `Cursor` is empty
    ///
    /// Note: this does *not* advance the `Cursor` or
    /// change the number of unfilled slots
    #[inline]
    pub fn back(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some(self.raw.back()) }
        }
    }

    /// Returns a mutable reference to the last element of the `Cursor`.
    /// Returns `None` if the `Cursor` is empty
    ///
    /// Note: this does *not* advance the `Cursor` or
    /// change the number of unfilled slots
    #[inline]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some(self.raw.back_mut()) }
        }
    }

    /// Removes the next element of the `Cursor`
    /// and removes it from the underlying [`GenericVec`]
    ///
    /// Advances the `Cursor` by 1 element
    ///
    /// Creates 1 unfilled slot at the front of the `Cursor`.
    ///
    /// # Panic
    ///
    /// Panics if the `Cursor` is empty
    #[inline]
    pub fn take_front(&mut self) -> T {
        assert!(!self.is_empty(), "Cannot take from a empty `Cursor`");
        unsafe { self.raw.take_front() }
    }

    /// Removes the last element of the `Cursor
    /// and removes it from the underlying [`GenericVec`]
    ///
    /// Advances the `Cursor` by 1 element
    ///
    /// Creates 1 unfilled slot at the back of the `Cursor`.
    ///
    /// # Panic
    ///
    /// Panics if the `Cursor` is empty
    #[inline]
    pub fn take_back(&mut self) -> T {
        assert!(!self.is_empty(), "Cannot take from a empty `Cursor`");
        unsafe { self.raw.take_back() }
    }

    /// Drops the next element of the `Cursor`
    /// and removes them it the underlying [`GenericVec`]
    ///
    /// Advances the `Cursor` by 1 element
    ///
    /// Creates 1 unfilled slot at the front of the `Cursor`.
    ///
    /// # Panic
    ///
    /// Panics if the `Cursor` is empty
    #[inline]
    pub fn drop_front(&mut self) {
        assert!(!self.is_empty(), "Cannot drop an element from a empty `Cursor`");

        unsafe { self.raw.drop_front() }
    }

    /// Drops the last element of the `Cursor`
    /// and removes them it the underlying [`GenericVec`]
    ///
    /// Advances the `Cursor` by 1 element
    ///
    /// Creates 1 unfilled slot at the back of the `Cursor`.
    ///
    /// # Panic
    ///
    /// Panics if the `Cursor` is empty
    #[inline]
    pub fn drop_back(&mut self) {
        assert!(!self.is_empty(), "Cannot drop an element from a empty `Cursor`");

        unsafe { self.raw.drop_back() }
    }

    /// Drops the next `n` elements of the `Cursor`
    /// and removes them from the underlying [`GenericVec`]
    ///
    /// Advances the `Cursor` by `n` elements
    ///
    /// Creates `n` unfilled slots at the front of the `Cursor`.
    ///
    /// # Panic
    ///
    /// Panics if the `Cursor`'s length is less than `n`
    #[inline]
    pub fn drop_n_front(&mut self, n: usize) {
        assert!(
            self.len() >= n,
            "Cannot drop {} elements from a `Cursor` of length {}",
            n,
            self.len()
        );

        unsafe { self.raw.drop_n_front(n) }
    }

    /// Drops the last `n` elements of the `Cursor`
    /// and removes them from the underlying [`GenericVec`]
    ///
    /// Advances the `Cursor` by `n` elements
    ///
    /// Creates `n` unfilled slots at the back of the `Cursor`.
    ///
    /// # Panic
    ///
    /// Panics if the `Cursor`'s length is less than `n`
    #[inline]
    pub fn drop_n_back(&mut self, n: usize) {
        assert!(
            self.len() >= n,
            "Cannot drop {} elements from a `Cursor` of length {}",
            n,
            self.len()
        );

        unsafe { self.raw.drop_n_back(n) }
    }

    /// Returns `Ok(())` and writes `value` into the unfilled slot
    /// at the front of the `Cursor` if there is an unfilled slot
    /// at the front of the `Cursor`
    ///
    /// If there are no unfilled slots at the front of the `Cursor`
    /// then return `Err(value)`
    ///
    /// Fills in 1 unfilled slot at the front of the `Cursor` on success
    #[inline]
    pub fn try_write_front(&mut self, value: T) -> Result<(), T> {
        if self.is_write_front_empty() {
            unsafe { self.raw.write_front(value) }
            Ok(())
        } else {
            Err(value)
        }
    }

    /// Writes `value` into the unfilled slot at the front of the
    /// `Cursor` if there is an unfilled slot at the front of the `Cursor`
    ///
    /// Fills in 1 unfilled slot at the front of the `Cursor`
    ///
    /// # Panic
    ///
    /// Panics if there are no unfilled slots at the front of the `Cursor`
    #[inline]
    pub fn write_front(&mut self, value: T) {
        assert!(
            !self.is_write_front_empty(),
            "Cannot write to a empty `Cursor` or if there are not unfilled slots at the front of the `Cursor`"
        );

        unsafe { self.raw.write_front(value) }
    }

    /// Returns `Ok(())` and writes `value` into the unfilled slot
    /// at the back of the `Cursor` if there is an unfilled slot
    /// at the back of the `Cursor`
    ///
    /// If there are no unfilled slots at the back of the `Cursor`
    /// then return `Err(value)`
    ///
    /// Fills in 1 unfilled slot at the back of the `Cursor` on success
    #[inline]
    pub fn try_write_back(&mut self, value: T) -> Result<(), T> {
        if self.is_write_back_empty() {
            Err(value)
        } else {
            unsafe { self.raw.write_back(value) }
            Ok(())
        }
    }

    /// Writes `value` into the unfilled slot at the back of the
    /// `Cursor` if there is an unfilled slot at the back of the `Cursor`
    ///
    /// Fills in 1 unfilled slot at the back of the `Cursor`
    ///
    /// # Panic
    ///
    /// Panics if there are no unfilled slots at the back of the `Cursor`
    #[inline]
    pub fn write_back(&mut self, value: T) {
        assert!(
            !self.is_write_back_empty(),
            "Cannot write to a empty `Cursor` or if there are not unfilled slots at the back of the `Cursor`"
        );

        unsafe { self.raw.write_back(value) }
    }

    /// Copies `slice` into the unfilled slots at the front of the
    /// `Cursor` if there are `slice.len()` unfilled slots at the
    /// front of the `Cursor`
    ///
    /// Fills in `slice.len()` unfilled slots at the front of the `Cursor`
    ///
    /// # Panic
    ///
    /// Panics if there are less than `slice.len()` unfilled slots
    /// at the front of the `Cursor`
    pub fn write_slice_front(&mut self, slice: &[T])
    where
        T: Copy,
    {
        let write_front_len = self.write_front_len();
        assert!(
            write_front_len >= slice.len(),
            "Cannot write {} elements, only {} slots remaining",
            slice.len(),
            write_front_len
        );

        unsafe { self.raw.write_slice_front(slice) }
    }

    /// Copies `slice` into the unfilled slots at the back of the
    /// `Cursor` if there are `slice.len()` unfilled slots at the
    /// back of the `Cursor`
    ///
    /// Fills in `slice.len()` unfilled slots at the back of the `Cursor`
    ///
    /// # Panic
    ///
    /// Panics if there are less than `slice.len()` unfilled slots
    /// at the back of the `Cursor`
    pub fn write_slice_back(&mut self, slice: &[T])
    where
        T: Copy,
    {
        let write_back_len = self.write_back_len();
        assert!(
            write_back_len >= slice.len(),
            "Cannot write {} elements, only {} slots remaining",
            slice.len(),
            write_back_len
        );

        unsafe { self.raw.write_slice_back(slice) }
    }

    /// Skips the next element of the `Cursor`
    /// and keeps it in the underlying [`GenericVec`]
    ///
    /// Advances the `Cursor` by 1 element
    ///
    /// Does not change the number of unfilled slots.
    ///
    /// # Panic
    ///
    /// Panics if the `Cursor` is empty
    #[inline]
    pub fn skip_front(&mut self) {
        assert!(!self.is_empty(), "Cannot skip elements from a empty `Cursor`");
        unsafe { self.raw.skip_front() }
    }

    /// Skips the last element of the `Cursor`
    /// and keeps it in the underlying [`GenericVec`]
    ///
    /// Advances the `Cursor` by 1 element
    ///
    /// Does not change the number of unfilled slots.
    ///
    /// # Panic
    ///
    /// Panics if the `Cursor` is empty
    #[inline]
    pub fn skip_back(&mut self) {
        assert!(!self.is_empty(), "Cannot skip elements from a empty `Cursor`");
        unsafe { self.raw.skip_back() }
    }

    /// Skips the next `n` elements of the `Cursor`
    /// and keeps them in the underlying [`GenericVec`]
    ///
    /// Advances the `Cursor` by `n` elements
    ///
    /// Does not change the number of unfilled slots.
    ///
    /// # Panic
    ///
    /// Panics if the `Cursor`'s length is less than `n`
    #[inline]
    pub fn skip_n_front(&mut self, n: usize) {
        assert!(
            self.len() >= n,
            "Cannot skip {} elements from a `Cursor` of length {}",
            n,
            self.len()
        );

        unsafe { self.raw.skip_n_front(n) }
    }

    /// Skips the last `n` elements of the `Cursor`
    /// and keeps them in the underlying [`GenericVec`]
    ///
    /// Advances the `Cursor` by `n` elements
    ///
    /// Does not change the number of unfilled slots.
    ///
    /// # Panic
    ///
    /// Panics if the `Cursor`'s length is less than `n`
    #[inline]
    pub fn skip_n_back(&mut self, n: usize) {
        assert!(
            self.len() >= n,
            "Cannot skip {} elements from a `Cursor` of length {}",
            n,
            self.len()
        );

        unsafe { self.raw.skip_n_back(n) }
    }

    /// Reserve at least space unfilled slots in the `Cursor`
    ///
    /// # Panic
    ///
    /// * Panics if the `Cursor` is not empty
    /// * May panic if the underlying [`GenericVec`] cannot
    ///   reserve more space
    pub fn reserve(&mut self, space: usize) {
        assert!(self.is_empty(), "You can only call `reserve` on a empty `Cursor`");
        self.raw.reserve(space);
    }
}
