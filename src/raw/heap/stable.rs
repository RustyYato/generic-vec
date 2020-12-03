use crate::raw::{AllocError, Storage, StorageWithCapacity};

use core::{
    alloc::Layout,
    mem::{align_of, forget, size_of},
    ptr::NonNull,
};
use std::alloc::{alloc, dealloc, handle_alloc_error, realloc};

doc_heap! {
    #[repr(C)]
    pub struct Heap<T> {
        capacity: usize,
        ptr: NonNull<T>,
    }
}

unsafe impl<T> Send for Heap<T> {}
unsafe impl<T> Sync for Heap<T> {}

enum OnFailure {
    Abort,
    Error,
}

impl<T> Drop for Heap<T> {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::new::<T>();
            let layout = Layout::from_size_align_unchecked(layout.size() * self.capacity, layout.align());
            dealloc(self.ptr.as_ptr().cast(), layout);
        }
    }
}

impl<T> Heap<T> {
    /// Create a new zero-capacity heap vector
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            capacity: if core::mem::size_of::<T>() == 0 { usize::MAX } else { 0 },
        }
    }

    /// Create a new `Heap<T>`storage from the given pointer and capacity
    ///
    /// # Safety
    ///
    /// If the capacity is non-zero
    /// * You must have allocated the pointer from the global allocator
    /// * The pointer must be valid to read-write for the range `ptr..ptr.add(capacity)`
    pub const unsafe fn from_raw_parts(ptr: NonNull<T>, capacity: usize) -> Self { Self { ptr, capacity } }

    /// Convert a `Heap` storage into a pointer and capacity, without
    /// deallocating the storage
    pub const fn into_raw_parts(self) -> (NonNull<T>, usize) {
        let Self { ptr, capacity } = self;
        forget(self);
        (ptr, capacity)
    }
}

impl<T> Default for Heap<T> {
    fn default() -> Self { Self::new() }
}

unsafe impl<T, U> Storage<U> for Heap<T> {
    const IS_ALIGNED: bool = align_of::<T>() >= align_of::<U>();

    fn capacity(&self) -> usize { self.capacity }

    fn as_ptr(&self) -> *const U { self.ptr.as_ptr().cast() }

    fn as_mut_ptr(&mut self) -> *mut U { self.ptr.as_ptr().cast() }

    fn reserve(&mut self, new_capacity: usize) {
        let new_capacity = crate::raw::capacity(new_capacity, size_of::<U>(), size_of::<T>());
        if self.capacity < new_capacity {
            let _ = self.reserve_slow(new_capacity, OnFailure::Abort);
        }
    }

    fn try_reserve(&mut self, new_capacity: usize) -> Result<(), AllocError> {
        let new_capacity = crate::raw::capacity(new_capacity, size_of::<U>(), size_of::<T>());
        if self.capacity < new_capacity {
            self.reserve_slow(new_capacity, OnFailure::Error)
        } else {
            Ok(())
        }
    }
}

pub fn padding_needed_for(layout: Layout, align: usize) -> usize {
    let len = layout.size();

    // Rounded up value is:
    //   len_rounded_up = (len + align - 1) & !(align - 1);
    // and then we return the padding difference: `len_rounded_up - len`.
    //
    // We use modular arithmetic throughout:
    //
    // 1. align is guaranteed to be > 0, so align - 1 is always
    //    valid.
    //
    // 2. `len + align - 1` can overflow by at most `align - 1`,
    //    so the &-mask with `!(align - 1)` will ensure that in the
    //    case of overflow, `len_rounded_up` will itself be 0.
    //    Thus the returned padding, when added to `len`, yields 0,
    //    which trivially satisfies the alignment `align`.
    //
    // (Of course, attempts to allocate blocks of memory whose
    // size and padding overflow in the above manner should cause
    // the allocator to yield an error anyway.)

    let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
    len_rounded_up.wrapping_sub(len)
}

pub fn repeat(layout: Layout, n: usize) -> Result<Layout, ()> {
    // This cannot overflow. Quoting from the invariant of Layout:
    // > `size`, when rounded up to the nearest multiple of `align`,
    // > must not overflow (i.e., the rounded value must be less than
    // > `usize::MAX`)
    let padded_size = layout.size() + padding_needed_for(layout, layout.align());
    let alloc_size = padded_size.checked_mul(n).ok_or(())?;

    // SAFETY: self.align is already known to be valid and alloc_size has been
    // padded already.
    unsafe { Ok(Layout::from_size_align_unchecked(alloc_size, layout.align())) }
}

impl<T> Heap<T> {
    fn with_capacity(capacity: usize) -> Self {
        if core::mem::size_of::<T>() == 0 {
            return Self::new()
        }

        let layout = repeat(Layout::new::<T>(), capacity).expect("Invalid layout");

        let ptr = unsafe { alloc(layout) };

        let ptr = match core::ptr::NonNull::new(ptr) {
            Some(ptr) => ptr,
            None => handle_alloc_error(layout),
        };

        Self {
            ptr: ptr.cast(),
            capacity,
        }
    }
}

impl<T, U> StorageWithCapacity<U> for Heap<T> {
    fn with_capacity(capacity: usize) -> Self { Self::with_capacity(capacity) }
}

impl<T> Heap<T> {
    #[cold]
    #[inline(never)]
    fn reserve_slow(&mut self, new_capacity: usize, on_failure: OnFailure) -> Result<(), AllocError> {
        assert!(new_capacity > self.capacity);

        // grow by at least doubling
        let new_capacity = new_capacity
            .max(self.capacity.checked_mul(2).expect("Could not grow further"))
            .max(super::INIT_ALLOC_CAPACITY);
        let layout = repeat(Layout::new::<T>(), new_capacity).expect("Invalid layout");

        let ptr = if self.capacity == 0 {
            unsafe { alloc(layout) }
        } else {
            let new_layout = layout;
            let old_layout = repeat(Layout::new::<T>(), self.capacity).expect("Invalid layout");

            unsafe { realloc(self.ptr.as_ptr().cast(), old_layout, new_layout.size()) }
        };

        let ptr = match (core::ptr::NonNull::new(ptr), on_failure) {
            (Some(ptr), _) => ptr,
            (None, OnFailure::Abort) => handle_alloc_error(layout),
            (None, OnFailure::Error) => return Err(AllocError),
        };

        self.ptr = ptr.cast();
        self.capacity = new_capacity;

        Ok(())
    }
}
