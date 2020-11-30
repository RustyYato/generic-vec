use crate::{GenericVec, Storage};

pub trait Extension<T> {
    unsafe fn extend_from_slice(&mut self, slice: &[T]);

    unsafe fn grow(&mut self, additional: usize, value: T);
}

fn clone_extend_from_slice<A: ?Sized + Storage>(vec: &mut GenericVec<A>, slice: &[A::Item])
where
    A::Item: Clone,
{
    let mut len = crate::set_len::SetLenOnDrop::new(&mut vec.len);
    let mut ptr = vec.raw.as_mut_ptr();

    for value in slice.iter().cloned() {
        unsafe {
            ptr.write(value);
            len += 1;
            ptr = ptr.add(1);
        }
    }
}

fn clone_grow<A: ?Sized + Storage>(vec: &mut GenericVec<A>, additional: usize, value: A::Item)
where
    A::Item: Clone,
{
    let mut len = crate::set_len::SetLenOnDrop::new(&mut vec.len);
    let mut ptr = vec.raw.as_mut_ptr();

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

impl<A: ?Sized + Storage> Extension<A::Item> for GenericVec<A>
where
    A::Item: Clone,
{
    #[cfg(feature = "nightly")]
    default unsafe fn extend_from_slice(&mut self, slice: &[A::Item]) { clone_extend_from_slice(self, slice) }

    #[cfg(not(feature = "nightly"))]
    unsafe fn extend_from_slice(&mut self, slice: &[A::Item]) { clone_extend_from_slice(self, slice) }

    #[cfg(feature = "nightly")]
    default unsafe fn grow(&mut self, additional: usize, value: A::Item) { clone_grow(self, additional, value) }

    #[cfg(not(feature = "nightly"))]
    unsafe fn grow(&mut self, additional: usize, value: A::Item) { clone_grow(self, additional, value) }
}

#[cfg(feature = "nightly")]
impl<A: ?Sized + Storage> Extension<A::Item> for GenericVec<A>
where
    A::Item: Copy,
{
    unsafe fn extend_from_slice(&mut self, slice: &[A::Item]) { unsafe { self.extend_from_slice_unchecked(slice) } }

    default unsafe fn grow(&mut self, additional: usize, value: A::Item) {
        let len = self.len();
        unsafe {
            self.set_len_unchecked(len.wrapping_add(additional));
        }
        let mut ptr = self.raw.as_mut_ptr();

        for _ in 0..additional {
            unsafe {
                ptr.write(value);
                ptr = ptr.add(1);
            }
        }
    }
}
