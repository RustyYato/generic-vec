use crate::raw::{Storage, StorageWithCapacity};
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

    /// Try to create a new zero-sized allocator, will be `None`
    /// if the type is not zero sized
    ///
    /// ```rust
    /// # use generic_vec::raw::ZeroSized;
    /// assert!(ZeroSized::<[i32; 0]>::TRY_NEW.is_some());
    /// assert!(ZeroSized::<u8>::TRY_NEW.is_none());
    /// ```
    pub const TRY_NEW: Option<Self> = [None, Some(ZeroSized(PhantomData))][(core::mem::size_of::<T>() == 0) as usize];

    const DANGLING: *mut T = core::mem::align_of::<T>() as *mut T;
}

unsafe impl<T> Storage<T> for ZeroSized<T> {
    const CONST_CAPACITY: Option<usize> = Some(usize::MAX);
    const IS_ALIGNED: bool = true;

    #[inline]
    fn as_ptr(&self) -> *const T { Self::DANGLING }
    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T { Self::DANGLING }

    #[inline]
    fn reserve(&mut self, _: usize) {}
    #[inline]
    fn try_reserve(&mut self, _: usize) -> bool { true }
    #[inline]
    fn capacity(&self) -> usize { usize::MAX }
}

unsafe impl<T> StorageWithCapacity<T> for ZeroSized<T> {
    #[inline]
    fn with_capacity(_: usize) -> Self { Self::NEW }
}
