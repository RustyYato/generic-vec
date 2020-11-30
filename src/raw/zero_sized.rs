use crate::raw::{AllocError, Storage, StorageWithCapacity};

/// A storage that can hold zero sized types
#[derive(Default, Clone, Copy)]
pub struct ZeroSized;

fn dangling<T>() -> *mut T { core::mem::align_of::<T>() as *mut T }

unsafe impl<T> Storage<T> for ZeroSized {
    const CONST_CAPACITY: Option<usize> = Some(usize::MAX);

    fn is_valid_storage() -> bool { core::mem::size_of::<T>() == 0 }

    fn as_ptr(&self) -> *const T { dangling() }
    fn as_mut_ptr(&mut self) -> *mut T { dangling() }

    fn reserve(&mut self, _: usize) {}
    fn try_reserve(&mut self, _: usize) -> Result<(), AllocError> { Ok(()) }
    fn capacity(&self) -> usize { usize::MAX }
}

impl<T> StorageWithCapacity<T> for ZeroSized {
    fn with_capacity(_: usize) -> Self {
        assert_eq!(core::mem::size_of::<T>(), 0);
        Self
    }
}
