macro_rules! doc_heap {
    ($($items:tt)*) => {
        /// A heap storage that can reallocate if necessary,
        ///
        /// Usable with the `alloc` feature
        $($items)*
    }
}

#[cfg(any(doc, feature = "nightly"))]
mod nightly;
#[cfg(not(any(doc, feature = "nightly")))]
mod stable;

#[cfg(any(doc, feature = "nightly"))]
pub use nightly::Heap;
#[cfg(not(any(doc, feature = "nightly")))]
pub use stable::Heap;

const INIT_ALLOC_CAPACITY: usize = 4;
