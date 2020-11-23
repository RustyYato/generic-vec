use crate::raw::{RawVec, RawVecInit};

use core::alloc::{AllocError, Layout};
use core::ptr::NonNull;
use std::alloc::{handle_alloc_error, AllocRef, Global};

#[repr(C)]
pub struct Heap<T, A: ?Sized + AllocRef = Global> {
    capacity: usize,
    ptr: NonNull<T>,
    alloc: A,
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
            let layout =
                Layout::from_size_align_unchecked(layout.size() * self.capacity, layout.align());
            self.alloc.dealloc(self.ptr.cast(), layout);
        }
    }
}

impl<T> Heap<T> {
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            capacity: 0,
            alloc: Global,
        }
    }
}

impl<T, A: AllocRef> Heap<T, A> {
    pub fn with_alloc(alloc: A) -> Self {
        Self {
            ptr: NonNull::dangling(),
            capacity: 0,
            alloc,
        }
    }
}

unsafe impl<T, A: ?Sized + AllocRef> RawVec for Heap<T, A> {
    type Item = T;

    fn capacity(&self) -> usize {
        self.capacity
    }

    fn as_ptr(&self) -> *const Self::Item {
        self.ptr.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        self.ptr.as_ptr()
    }

    fn reserve(&mut self, new_capacity: usize) {
        if self.capacity < new_capacity {
            let _ = self.reserve_slow(new_capacity, OnFailure::Abort);
        }
    }

    fn try_reserve(&mut self, new_capacity: usize) -> Result<(), AllocError> {
        if self.capacity < new_capacity {
            self.reserve_slow(new_capacity, OnFailure::Error)
        } else {
            Ok(())
        }
    }
}

impl<T, A: ?Sized + AllocRef> Heap<T, A> {
    #[cold]
    #[inline(never)]
    fn reserve_slow(
        &mut self,
        new_capacity: usize,
        on_failure: OnFailure,
    ) -> Result<(), AllocError> {
        assert!(new_capacity > self.capacity);

        // grow by at least doubling
        let new_capacity = new_capacity.max(
            self.capacity
                .checked_mul(2)
                .expect("Could not grow further"),
        );
        let layout = Layout::new::<T>()
            .repeat(new_capacity)
            .expect("Invalid layout")
            .0;

        let ptr = if self.capacity == 0 {
            self.alloc.alloc(layout)
        } else {
            let new_layout = layout;
            let old_layout = Layout::new::<T>()
                .repeat(self.capacity)
                .expect("Invalid layout")
                .0;

            unsafe { self.alloc.grow(self.ptr.cast(), old_layout, new_layout) }
        };

        let ptr = match (ptr, on_failure) {
            (Ok(ptr), _) => ptr,
            (Err(_), OnFailure::Abort) => handle_alloc_error(layout),
            (Err(_), OnFailure::Error) => return Err(AllocError),
        };

        self.ptr = ptr.cast();

        Ok(())
    }
}
