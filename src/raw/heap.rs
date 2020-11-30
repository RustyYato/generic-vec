macro_rules! doc_heap {
    ($($items:tt)*) => {
        /// Heap
        $($items)*
    }
}

#[cfg(feature = "nightly")]
mod nightly;
#[cfg(not(feature = "nightly"))]
mod stable;

#[cfg(feature = "nightly")]
pub use nightly::Heap;
#[cfg(not(feature = "nightly"))]
pub use stable::Heap;

const INIT_ALLOC_CAPACITY: usize = 4;
