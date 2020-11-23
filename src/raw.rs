use core::alloc::AllocError;

#[cfg(feature = "alloc")]
mod alloc;
mod init;
mod uninit;

#[cfg(feature = "alloc")]
pub use alloc::Heap;
pub use init::{Array, Init, Slice};
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
