use crate::raw::{AllocError, Storage, StorageWithCapacity};
use core::marker::PhantomData;

/// A storage that can hold zero sized types
pub struct ZeroSized<T>(PhantomData<T>);

impl<T> Default for ZeroSized<T> {
    fn default() -> Self { Self::NEW }
}

impl<T> Copy for ZeroSized<T> {}
impl<T> Clone for ZeroSized<T> {
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
}

fn dangling<T>() -> *mut T { core::mem::align_of::<T>() as *mut T }

unsafe impl<T> Storage<T> for ZeroSized<T> {
    const CONST_CAPACITY: Option<usize> = Some(usize::MAX);

    fn as_ptr(&self) -> *const T { dangling() }
    fn as_mut_ptr(&mut self) -> *mut T { dangling() }

    fn reserve(&mut self, _: usize) {}
    fn try_reserve(&mut self, _: usize) -> Result<(), AllocError> { Ok(()) }
    fn capacity(&self) -> usize { usize::MAX }
}

impl<T> StorageWithCapacity<T> for ZeroSized<T> {
    fn with_capacity(_: usize) -> Self { Self::NEW }
}
