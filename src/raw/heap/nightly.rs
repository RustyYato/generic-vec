use crate::raw::{AllocError, Storage, StorageWithCapacity};

use core::{
    alloc::Layout,
    mem::{size_of, ManuallyDrop},
    ptr::NonNull,
};
use std::alloc::handle_alloc_error;

#[cfg(feature = "nightly")]
use std::alloc::{AllocRef, Global};

doc_heap! {
    #[repr(C)]
    pub struct Heap<T, A: ?Sized + AllocRef = Global> {
        capacity: usize,
        ptr: NonNull<T>,
        alloc: A,
    }
}

unsafe impl<T, A: AllocRef + Send> Send for Heap<T, A> {}
unsafe impl<T, A: AllocRef + Sync> Sync for Heap<T, A> {}

enum OnFailure {
    Abort,
    Error,
}

impl<T, A: ?Sized + AllocRef> Drop for Heap<T, A> {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::new::<T>();
            let layout = Layout::from_size_align_unchecked(layout.size() * self.capacity, layout.align());
            self.alloc.dealloc(self.ptr.cast(), layout);
        }
    }
}

impl<T> Heap<T> {
    /// Create a new zero-capacity heap vector
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            capacity: if core::mem::size_of::<T>() == 0 { usize::MAX } else { 0 },
            alloc: Global,
        }
    }

    /// Create a new `Heap<T>`storage from the given pointer and capacity
    ///
    /// # Safety
    ///
    /// If the capacity is non-zero
    /// * You must have allocated the pointer from the [`Global`] allocator
    /// * The pointer must be valid to read-write for the range `ptr..ptr.add(capacity)`
    pub const unsafe fn from_raw_parts(ptr: NonNull<T>, capacity: usize) -> Self {
        Self {
            ptr,
            capacity,
            alloc: Global,
        }
    }

    /// Convert a `Heap` storage into a pointer and capacity, without
    /// deallocating the storage
    pub const fn into_raw_parts(self) -> (NonNull<T>, usize) {
        let Self { ptr, capacity, .. } = self;
        core::mem::forget(self);
        (ptr, capacity)
    }
}

impl<T, A: AllocRef> Heap<T, A> {
    /// Create a new zero-capacity heap vector with the given allocator
    pub const fn with_alloc(alloc: A) -> Self {
        Self {
            ptr: NonNull::dangling(),
            capacity: if core::mem::size_of::<T>() == 0 { usize::MAX } else { 0 },
            alloc,
        }
    }

    /// Create a new `Heap<T>`storage from the given pointer and capacity
    ///
    /// # Safety
    ///
    /// If the capacity is non-zero
    /// * You must have allocated the pointer from the given allocator
    /// * The pointer must be valid to read-write for the range `ptr..ptr.add(capacity)`
    pub const unsafe fn from_raw_parts_in(ptr: NonNull<T>, capacity: usize, alloc: A) -> Self {
        Self { ptr, capacity, alloc }
    }

    /// Convert a `Heap` storage into a pointer and capacity, without
    /// deallocating the storage
    pub fn into_raw_parts_with_alloc(self) -> (NonNull<T>, usize, A) {
        #[repr(C)]
        #[allow(dead_code)]
        struct HeapRepr<T, A: AllocRef> {
            capacity: usize,
            ptr: NonNull<T>,
            alloc: A,
        }

        let HeapRepr { ptr, capacity, alloc } = unsafe { core::mem::transmute_copy(&ManuallyDrop::new(self)) };

        (ptr, capacity, alloc)
    }
}

impl<T, A: AllocRef + Default> Default for Heap<T, A> {
    fn default() -> Self { Self::with_alloc(Default::default()) }
}

unsafe impl<T, U, A: ?Sized + AllocRef> Storage<U> for Heap<T, A> {
    fn is_valid_storage() -> bool { crate::raw::is_identical::<T, U>() }

    fn capacity(&self) -> usize { crate::raw::capacity(self.capacity, size_of::<T>(), size_of::<U>()) }

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

impl<T, A: Default + AllocRef> Heap<T, A> {
    fn with_capacity(capacity: usize) -> Self {
        if core::mem::size_of::<T>() == 0 {
            return Self::default()
        }

        let layout = Layout::new::<T>().repeat(capacity).expect("Invalid layout").0;
        let alloc = A::default();

        let ptr = unsafe { alloc.alloc(layout) };

        let ptr = match ptr {
            Ok(ptr) => ptr,
            Err(_) => handle_alloc_error(layout),
        };

        Self {
            ptr: ptr.cast(),
            capacity,
            alloc,
        }
    }
}

impl<T, U, A: Default + AllocRef> StorageWithCapacity<U> for Heap<T, A> {
    fn with_capacity(capacity: usize) -> Self { Self::with_capacity(capacity) }
}

impl<T, A: ?Sized + AllocRef> Heap<T, A> {
    #[cold]
    #[inline(never)]
    fn reserve_slow(&mut self, new_capacity: usize, on_failure: OnFailure) -> Result<(), AllocError> {
        assert!(new_capacity > self.capacity);

        // grow by at least doubling
        let new_capacity = new_capacity
            .max(self.capacity.checked_mul(2).expect("Could not grow further"))
            .max(super::INIT_ALLOC_CAPACITY);
        let layout = Layout::new::<T>().repeat(new_capacity).expect("Invalid layout").0;

        let ptr = if self.capacity == 0 {
            self.alloc.alloc(layout)
        } else {
            let new_layout = layout;
            let old_layout = Layout::new::<T>().repeat(self.capacity).expect("Invalid layout").0;

            unsafe { self.alloc.grow(self.ptr.cast(), old_layout, new_layout) }
        };

        let ptr = match (ptr, on_failure) {
            (Ok(ptr), _) => ptr,
            (Err(_), OnFailure::Abort) => handle_alloc_error(layout),
            (Err(_), OnFailure::Error) => return Err(AllocError),
        };

        self.ptr = ptr.cast();
        self.capacity = new_capacity;

        Ok(())
    }
}
