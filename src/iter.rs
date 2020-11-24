mod drain;
mod drain_filter;
mod into_iter;
mod raw_drain;
mod splice;

pub use {
    drain::Drain, drain_filter::DrainFilter, into_iter::IntoIter, raw_drain::RawDrain,
    splice::Splice,
};

use core::iter::FromIterator;

use crate::{raw::RawVecWithCapacity, GenericVec};

impl<V, A: RawVecWithCapacity> FromIterator<V> for GenericVec<A>
where
    Self: Extend<V>,
{
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let mut array = Self::default();
        array.extend(iter);
        array
    }
}

impl<A: ?Sized + crate::raw::RawVec> Extend<A::Item> for GenericVec<A> {
    fn extend<T: IntoIterator<Item = A::Item>>(&mut self, iter: T) {
        iter.into_iter().for_each(|item| drop(self.push(item)));
    }
}
