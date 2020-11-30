use crate::{RawDrain, Storage};

/// This struct is created by [`GenericVec::splice`](crate::GenericVec::splice).
/// See its documentation for more.
pub struct Splice<'a, T, S, I>
where
    S: ?Sized + Storage<T>,
    I: Iterator<Item = T>,
{
    raw: RawDrain<'a, T, S>,
    replace_with: I,
}

impl<'a, T, S: ?Sized + Storage<T>, I: Iterator<Item = T>> Splice<'a, T, S, I> {
    pub(crate) fn new(raw: RawDrain<'a, T, S>, replace_with: I) -> Self { Self { raw, replace_with } }
}

impl<T, S: ?Sized + Storage<T>, I: Iterator<Item = T>> Drop for Splice<'_, T, S, I> {
    fn drop(&mut self) {
        self.for_each(drop);

        let Self { raw, replace_with } = self;

        #[cfg(not(feature = "alloc"))]
        {
            const CAPACITY: usize = 16;

            let mut buffer = crate::uninit_array!(CAPACITY);
            let mut buffer = crate::SliceVec::new(&mut buffer);

            replace_with.for_each(|item| unsafe {
                buffer.push_unchecked(item);

                if !RawDrain::<T, S>::IS_ZS && buffer.is_full() {
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
            let mut temp: std::vec::Vec<_> = replace_with.collect();

            unsafe {
                raw.assert_space(temp.len());
                raw.consume_write_slice_front(&temp);
                temp.set_len(0);
            }
        }
    }
}

impl<'a, T, S: ?Sized + Storage<T>, I: Iterator<Item = T>> Iterator for Splice<'a, T, S, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.raw.is_complete() {
            return None
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
