use crate::{GenericVec, Storage};

pub trait Extension<T> {
    unsafe fn extend_from_slice(&mut self, slice: &[T]);

    unsafe fn grow(&mut self, additional: usize, value: T);
}

fn clone_extend_from_slice<T, S: ?Sized + Storage<T>>(vec: &mut GenericVec<T, S>, slice: &[T])
where
    T: Clone,
{
    let mut len = crate::set_len::SetLenOnDrop::new(&mut vec.len);
    let mut ptr = vec.storage.as_mut_ptr();

    for value in slice.iter().cloned() {
        // Safety
        //
        // `clone_extend_from_slice` is only called from `Extension::extend_from_slice`
        // which has the pre-condition that there must be at least enough remaining capacity
        // for the slice. So it is safe to write the contents of the slice
        unsafe {
            ptr.write(value);
            len += 1;
            ptr = ptr.add(1);
        }
    }
}

fn clone_grow<T, S: ?Sized + Storage<T>>(vec: &mut GenericVec<T, S>, additional: usize, value: T)
where
    T: Clone,
{
    let mut len = crate::set_len::SetLenOnDrop::new(&mut vec.len);
    let mut ptr = vec.storage.as_mut_ptr();

    // Safety
    //
    // `clone_grow` is only called from `Extension::grow`
    // which has the pre-condition that there must be at least enough remaining capacity
    // for the `additional` new elements
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

impl<T, S: ?Sized + Storage<T>> Extension<T> for GenericVec<T, S>
where
    T: Clone,
{
    #[cfg(feature = "nightly")]
    default unsafe fn extend_from_slice(&mut self, slice: &[T]) { clone_extend_from_slice(self, slice) }

    #[cfg(not(feature = "nightly"))]
    unsafe fn extend_from_slice(&mut self, slice: &[T]) { clone_extend_from_slice(self, slice) }

    #[cfg(feature = "nightly")]
    default unsafe fn grow(&mut self, additional: usize, value: T) { clone_grow(self, additional, value) }

    #[cfg(not(feature = "nightly"))]
    unsafe fn grow(&mut self, additional: usize, value: T) { clone_grow(self, additional, value) }
}

#[cfg(feature = "nightly")]
impl<T, S: ?Sized + Storage<T>> Extension<T> for GenericVec<T, S>
where
    T: Copy,
{
    unsafe fn extend_from_slice(&mut self, slice: &[T]) {
        // Safety
        //
        // * `Extension::extend_from_slice`'s precondition ensure that
        //   there is enough capacity for `slice`
        // * `T: Copy`, so there is nothing to drop
        unsafe { self.extend_from_slice_unchecked(slice) }
    }

    default unsafe fn grow(&mut self, additional: usize, value: T) {
        // Safety
        //
        // * `Extension::grow`'s precondition ensure that
        //   there is enough capacity for `additional` elements
        let len = self.len();
        unsafe {
            self.set_len_unchecked(len.wrapping_add(additional));
        }
        let mut ptr = self.storage.as_mut_ptr();

        for _ in 0..additional {
            unsafe {
                ptr.write(value);
                ptr = ptr.add(1);
            }
        }
    }
}
