use crate::{RawCursor, Storage};

use core::mem::ManuallyDrop;

/// This struct is created by [`GenericVec::splice`](crate::GenericVec::splice).
/// See its documentation for more.
pub struct Splice<'a, T, S, I>
where
    S: ?Sized + Storage<T>,
    I: Iterator<Item = T>,
{
    raw: RawCursor<'a, T, S>,
    replace_with: I,
}

impl<'a, T, S: ?Sized + Storage<T>, I: Iterator<Item = T>> Splice<'a, T, S, I> {
    pub(crate) fn new(raw: RawCursor<'a, T, S>, replace_with: I) -> Self { Self { raw, replace_with } }
}

impl<T, S: ?Sized + Storage<T>, I: Iterator<Item = T>> Drop for Splice<'_, T, S, I> {
    fn drop(&mut self) {
        self.for_each(drop);

        let Self { raw, replace_with } = self;

        if let Some(storage) = crate::raw::ZeroSized::TRY_NEW {
            unsafe {
                let mut vec = ManuallyDrop::new(crate::ZSVec::with_storage(storage));
                vec.extend(replace_with);
                let len = vec.len();
                raw.assert_space(len);
                raw.consume_write_slice_front(vec.as_slice());
                return
            }
        }

        if raw.at_back_of_vec() {
            self.raw.finish();
            unsafe { self.raw.vec_mut().extend(replace_with) }
            return
        }

        while !raw.is_write_complete() {
            match replace_with.next() {
                Some(value) => unsafe { raw.write_front(value) },
                None => return,
            }
        }

        #[cfg(not(feature = "alloc"))]
        {
            const CAPACITY: usize = 16;

            let mut buffer = crate::uninit_array!(CAPACITY);
            let mut buffer = crate::SliceVec::new(&mut buffer);

            replace_with.for_each(|item| unsafe {
                buffer.push_unchecked(item);

                if buffer.is_full() {
                    unsafe {
                        raw.assert_space(buffer.len());
                        raw.consume_write_slice_front(&buffer);
                        buffer.set_len_unchecked(0);
                    }
                }
            });

            unsafe {
                raw.assert_space(buffer.len());
                raw.consume_write_slice_front(&buffer);
                core::mem::forget(buffer);
            }
        }

        #[cfg(feature = "alloc")]
        {
            let mut temp: std::vec::Vec<T> = replace_with.collect();

            unsafe {
                raw.assert_space(temp.len());
                raw.consume_write_slice_front(&temp);
                temp.set_len(0);
            }
        }
    }
}

impl<T, S: ?Sized + Storage<T>, I: Iterator<Item = T>> ExactSizeIterator for Splice<'_, T, S, I> {}

impl<'a, T, S: ?Sized + Storage<T>, I: Iterator<Item = T>> Iterator for Splice<'a, T, S, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.raw.is_complete() {
            None
        } else {
            Some(unsafe { self.raw.take_front() })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.raw.remaining();
        (size, Some(size))
    }
}

impl<'a, T, S: ?Sized + Storage<T>, I: Iterator<Item = T>> DoubleEndedIterator for Splice<'a, T, S, I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.raw.is_complete() {
            None
        } else {
            Some(unsafe { self.raw.take_back() })
        }
    }
}
