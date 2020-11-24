#[cfg(feature = "nightly")]
pub use core::alloc::AllocError;

#[cfg(not(feature = "nightly"))]
pub struct AllocError;

#[cfg(feature = "nightly")]
mod array;
#[cfg(feature = "alloc")]
mod heap;
mod slice;

#[cfg(feature = "nightly")]
pub use array::{Array, UninitArray};
#[cfg(feature = "alloc")]
pub use heap::Heap;

pub use slice::{Slice, UninitSlice};

#[repr(transparent)]
pub struct Init<T: ?Sized>(T);
#[repr(transparent)]
pub struct Uninit<T: ?Sized>(T);

pub unsafe trait RawVec {
    type Item;

    fn capacity(&self) -> usize;
    fn as_ptr(&self) -> *const Self::Item;
    fn as_mut_ptr(&mut self) -> *mut Self::Item;
    fn reserve(&mut self, new_capacity: usize);
    fn try_reserve(&mut self, new_capacity: usize) -> Result<(), AllocError>;
}

pub trait RawVecInit: RawVec + Default {
    fn with_capacity(capacity: usize) -> Self;
}
