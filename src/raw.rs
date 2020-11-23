#[cfg(feature = "nightly")]
pub use core::alloc::AllocError;

#[cfg(not(feature = "nightly"))]
pub struct AllocError;

#[cfg(feature = "alloc")]
mod heap;
#[cfg(feature = "nightly")]
mod init;
#[cfg(feature = "nightly")]
mod uninit;

#[cfg(feature = "alloc")]
pub use heap::Heap;
#[cfg(feature = "nightly")]
pub use init::{Array, Init, Slice};
#[cfg(feature = "nightly")]
pub use uninit::{Uninit, UninitArray, UninitSlice};

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
