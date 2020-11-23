use crate::{GenericVec, RawVec};

pub trait Extension<T> {
    unsafe fn extend_from_slice(&mut self, slice: &[T]);

    unsafe fn grow(&mut self, additional: usize, value: T);
}

impl<A: ?Sized + RawVec> Extension<A::Item> for GenericVec<A>
where
    A::Item: Clone,
{
    default unsafe fn extend_from_slice(&mut self, slice: &[A::Item]) {
        let mut len = crate::set_len::SetLenOnDrop::new(&mut self.len);
        let mut ptr = self.raw.as_mut_ptr();

        for value in slice.iter().cloned() {
            unsafe {
                ptr.write(value);
                len += 1;
                ptr = ptr.add(1);
            }
        }
    }

    default unsafe fn grow(&mut self, additional: usize, value: A::Item) {
        let mut len = crate::set_len::SetLenOnDrop::new(&mut self.len);
        let mut ptr = self.raw.as_mut_ptr();

        for _ in 1..additional {
            unsafe {
                ptr.write(value.clone());
                len += 1;
                ptr = ptr.add(1);
            }
        }

        if additional > 0 {
            unsafe {
                ptr.write(value);
            }
        }
    }
}

impl<A: ?Sized + RawVec> Extension<A::Item> for GenericVec<A>
where
    A::Item: Copy,
{
    unsafe fn extend_from_slice(&mut self, slice: &[A::Item]) {
        unsafe { self.extend_from_slice_unchecked(slice) }
    }

    default unsafe fn grow(&mut self, additional: usize, value: A::Item) {
        self.len += additional;
        let mut ptr = self.raw.as_mut_ptr();

        for _ in 0..additional {
            unsafe {
                ptr.write(value);
                ptr = ptr.add(1);
            }
        }
    }
}
