use crate::raw::{AllocError, Storage, StorageWithCapacity};
use core::marker::PhantomData;

/// A storage that can hold zero sized types
pub struct ZeroSized<T>(PhantomData<T>);

impl<T> Default for ZeroSized<T> {
    #[inline]
    fn default() -> Self { Self::NEW }
}

impl<T> Copy for ZeroSized<T> {}
impl<T> Clone for ZeroSized<T> {
    #[inline]
    fn clone(&self) -> Self { Self::NEW }
}

impl<T> ZeroSized<T> {
    /// Create a new zero-sized allocator, can only be used with zero-sized types
    ///
    /// ```rust
    /// # use generic_vec::raw::ZeroSized;
    /// let _ = ZeroSized::<[i32; 0]>::NEW;
    /// ```
    ///
    /// ```compile_fail
    /// # use generic_vec::raw::ZeroSized;
    /// let _ = ZeroSized::<u8>::NEW;
    /// ```
    pub const NEW: Self = ZeroSized([PhantomData][core::mem::size_of::<T>()]);

    const DANGLING: *mut T = core::mem::align_of::<T>() as *mut T;
}

unsafe impl<T> Storage<T> for ZeroSized<T> {
    const CONST_CAPACITY: Option<usize> = Some(usize::MAX);

    #[inline]
    fn as_ptr(&self) -> *const T { Self::DANGLING }
    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T { Self::DANGLING }

    #[inline]
    fn reserve(&mut self, _: usize) {}
    #[inline]
    fn try_reserve(&mut self, _: usize) -> Result<(), AllocError> { Ok(()) }
    #[inline]
    fn capacity(&self) -> usize { usize::MAX }
}

impl<T> StorageWithCapacity<T> for ZeroSized<T> {
    #[inline]
    fn with_capacity(_: usize) -> Self { Self::NEW }
}
