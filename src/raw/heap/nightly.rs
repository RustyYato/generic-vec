use crate::raw::{
    capacity::{capacity, Round},
    Storage, StorageWithCapacity,
};

use core::{
    alloc::Layout,
    mem::{align_of, size_of, ManuallyDrop},
    ptr::NonNull,
};
use std::alloc::handle_alloc_error;

use std::alloc::{Allocator, Global};

doc_heap! {
    #[repr(C)]
    #[cfg_attr(doc, doc(cfg(feature = "alloc")))]
    ///
    /// The allocator type paramter is only available on `nightly`
    pub struct Heap<T, A: ?Sized + Allocator = Global> {
        capacity: usize,
        ptr: NonNull<T>,
        allocator: A,
    }
}

unsafe impl<T, A: Allocator + Send> Send for Heap<T, A> {}
unsafe impl<T, A: Allocator + Sync> Sync for Heap<T, A> {}

enum OnFailure {
    Abort,
    Error,
}

impl<T, A: ?Sized + Allocator> Drop for Heap<T, A> {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::new::<T>();
            let layout = Layout::from_size_align_unchecked(layout.size() * self.capacity, layout.align());
            self.allocator.deallocate(self.ptr.cast(), layout);
        }
    }
}

impl<T> Heap<T> {
    /// Create a new zero-capacity heap vector
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            capacity: if core::mem::size_of::<T>() == 0 { usize::MAX } else { 0 },
            allocator: Global,
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
            allocator: Global,
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

#[cfg_attr(doc, doc(cfg(feature = "nightly")))]
impl<T, A: Allocator> Heap<T, A> {
    /// Create a new zero-capacity heap vector with the given allocator
    pub const fn with_alloc(allocator: A) -> Self {
        Self {
            ptr: NonNull::dangling(),
            capacity: if core::mem::size_of::<T>() == 0 { usize::MAX } else { 0 },
            allocator,
        }
    }

    /// Create a new `Heap<T>`storage from the given pointer and capacity
    ///
    /// # Safety
    ///
    /// If the capacity is non-zero
    /// * You must have allocated the pointer from the given allocator
    /// * The pointer must be valid to read-write for the range `ptr..ptr.add(capacity)`
    pub const unsafe fn from_raw_parts_in(ptr: NonNull<T>, capacity: usize, allocator: A) -> Self {
        Self {
            ptr,
            capacity,
            allocator,
        }
    }

    /// Convert a `Heap` storage into a pointer and capacity, without
    /// deallocating the storage
    pub fn into_raw_parts_with_alloc(self) -> (NonNull<T>, usize, A) {
        #[repr(C)]
        #[allow(dead_code)]
        struct HeapRepr<T, A: Allocator> {
            capacity: usize,
            ptr: NonNull<T>,
            allocator: A,
        }

        let HeapRepr {
            ptr,
            capacity,
            allocator,
        } = unsafe { core::mem::transmute_copy(&ManuallyDrop::new(self)) };

        (ptr, capacity, allocator)
    }
}

impl<T, A: Allocator + Default> Default for Heap<T, A> {
    fn default() -> Self { Self::with_alloc(Default::default()) }
}

unsafe impl<T, U, A: ?Sized + Allocator> Storage<U> for Heap<T, A> {
    const IS_ALIGNED: bool = align_of::<T>() >= align_of::<U>();

    fn capacity(&self) -> usize { capacity(self.capacity, size_of::<T>(), size_of::<U>(), Round::Down) }

    fn as_ptr(&self) -> *const U { self.ptr.as_ptr().cast() }

    fn as_mut_ptr(&mut self) -> *mut U { self.ptr.as_ptr().cast() }

    fn reserve(&mut self, new_capacity: usize) {
        let new_capacity = capacity(new_capacity, size_of::<U>(), size_of::<T>(), Round::Up);
        if self.capacity < new_capacity {
            let _ = self.reserve_slow(new_capacity, OnFailure::Abort);
        }
    }

    fn try_reserve(&mut self, new_capacity: usize) -> bool {
        let new_capacity = capacity(new_capacity, size_of::<U>(), size_of::<T>(), Round::Up);
        if self.capacity < new_capacity {
            self.reserve_slow(new_capacity, OnFailure::Error)
        } else {
            true
        }
    }
}

impl<T, A: Default + Allocator> Heap<T, A> {
    fn with_capacity(capacity: usize) -> Self {
        if core::mem::size_of::<T>() == 0 {
            return Self::default()
        }

        let layout = Layout::new::<T>().repeat(capacity).expect("Invalid layout").0;
        let allocator = A::default();

        let ptr = unsafe { allocator.allocate(layout) };

        let ptr = match ptr {
            Ok(ptr) => ptr,
            Err(_) => handle_alloc_error(layout),
        };

        Self {
            ptr: ptr.cast(),
            capacity,
            allocator,
        }
    }
}

unsafe impl<T, U, A: Default + Allocator> StorageWithCapacity<U> for Heap<T, A> {
    fn with_capacity(cap: usize) -> Self {
        Self::with_capacity(capacity(cap, size_of::<U>(), size_of::<T>(), Round::Up))
    }
}

impl<T, A: ?Sized + Allocator> Heap<T, A> {
    #[cold]
    #[inline(never)]
    fn reserve_slow(&mut self, new_capacity: usize, on_failure: OnFailure) -> bool {
        assert!(new_capacity > self.capacity);

        // grow by at least doubling
        let new_capacity = new_capacity
            .max(self.capacity.checked_mul(2).expect("Could not grow further"))
            .max(super::INIT_ALLOC_CAPACITY);
        let layout = Layout::new::<T>().repeat(new_capacity).expect("Invalid layout").0;

        let ptr = if self.capacity == 0 {
            self.allocator.allocate(layout)
        } else {
            let new_layout = layout;
            let old_layout = Layout::new::<T>().repeat(self.capacity).expect("Invalid layout").0;

            unsafe { self.allocator.grow(self.ptr.cast(), old_layout, new_layout) }
        };

        let ptr = match (ptr, on_failure) {
            (Ok(ptr), _) => ptr,
            (Err(_), OnFailure::Abort) => handle_alloc_error(layout),
            (Err(_), OnFailure::Error) => return false,
        };

        self.ptr = ptr.cast();
        self.capacity = new_capacity;

        true
    }
}
