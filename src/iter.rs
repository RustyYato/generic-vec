//! The [`Iterator`] types that can be created from a [`GenericVec`]

mod drain;
mod drain_filter;
mod into_iter;
mod raw_drain;
mod splice;

pub use drain::Drain;
pub use drain_filter::DrainFilter;
pub use into_iter::IntoIter;
pub use raw_drain::RawDrain;
pub use splice::Splice;

use core::iter::FromIterator;

use crate::{raw::StorageWithCapacity, GenericVec};

impl<V, A: StorageWithCapacity> FromIterator<V> for GenericVec<A>
where
    Self: Extend<V>,
{
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let mut array = Self::default();
        array.extend(iter);
        array
    }
}

impl<A: ?Sized + crate::raw::Storage> Extend<A::Item> for GenericVec<A> {
    fn extend<T: IntoIterator<Item = A::Item>>(&mut self, iter: T) {
        #[allow(clippy::drop_ref)]
        iter.into_iter().for_each(|item| drop(self.push(item)));
    }
}
