use core::mem;

// taken from `std`, https://doc.rust-lang.org/nightly/std/primitive.slice.html#method.partition_dedup_by
// TODO: once it stablizes remove this
pub(crate) fn partition_dedup_by<T, F>(slice: &mut [T], mut same_bucket: F) -> (&mut [T], &mut [T])
where
    F: FnMut(&mut T, &mut T) -> bool,
{
    // Although we have a mutable reference to `self`, we cannot make
    // *arbitrary* changes. The `same_bucket` calls could panic, so we
    // must ensure that the slice is in a valid state at all times.
    //
    // The way that we handle this is by using swaps; we iterate
    // over all the elements, swapping as we go so that at the end
    // the elements we wish to keep are in the front, and those we
    // wish to reject are at the back. We can then split the slice.
    // This operation is still `O(n)`.
    //
    // Example: We start in this state, where `r` represents "next
    // read" and `w` represents "next_write`.
    //
    //           r
    //     +---+---+---+---+---+---+
    //     | 0 | 1 | 1 | 2 | 3 | 3 |
    //     +---+---+---+---+---+---+
    //           w
    //
    // Comparing self[r] against self[w-1], this is not a duplicate, so
    // we swap self[r] and self[w] (no effect as r==w) and then increment both
    // r and w, leaving us with:
    //
    //               r
    //     +---+---+---+---+---+---+
    //     | 0 | 1 | 1 | 2 | 3 | 3 |
    //     +---+---+---+---+---+---+
    //               w
    //
    // Comparing self[r] against self[w-1], this value is a duplicate,
    // so we increment `r` but leave everything else unchanged:
    //
    //                   r
    //     +---+---+---+---+---+---+
    //     | 0 | 1 | 1 | 2 | 3 | 3 |
    //     +---+---+---+---+---+---+
    //               w
    //
    // Comparing self[r] against self[w-1], this is not a duplicate,
    // so swap self[r] and self[w] and advance r and w:
    //
    //                       r
    //     +---+---+---+---+---+---+
    //     | 0 | 1 | 2 | 1 | 3 | 3 |
    //     +---+---+---+---+---+---+
    //                   w
    //
    // Not a duplicate, repeat:
    //
    //                           r
    //     +---+---+---+---+---+---+
    //     | 0 | 1 | 2 | 3 | 1 | 3 |
    //     +---+---+---+---+---+---+
    //                       w
    //
    // Duplicate, advance r. End of slice. Split at w.

    let len = slice.len();
    if len <= 1 {
        return (slice, &mut [])
    }

    let ptr = slice.as_mut_ptr();
    let mut next_read: usize = 1;
    let mut next_write: usize = 1;

    // SAFETY: the `while` condition guarantees `next_read` and `next_write`
    // are less than `len`, thus are inside `self`. `prev_ptr_write` points to
    // one element before `ptr_write`, but `next_write` starts at 1, so
    // `prev_ptr_write` is never less than 0 and is inside the slice.
    // This fulfils the requirements for dereferencing `ptr_read`, `prev_ptr_write`
    // and `ptr_write`, and for using `ptr.add(next_read)`, `ptr.add(next_write - 1)`
    // and `prev_ptr_write.offset(1)`.
    //
    // `next_write` is also incremented at most once per loop at most meaning
    // no element is skipped when it may need to be swapped.
    //
    // `ptr_read` and `prev_ptr_write` never point to the same element. This
    // is required for `&mut *ptr_read`, `&mut *prev_ptr_write` to be safe.
    // The explanation is simply that `next_read >= next_write` is always true,
    // thus `next_read > next_write - 1` is too.
    unsafe {
        // Avoid bounds checks by using raw pointers.
        while next_read < len {
            let ptr_read = ptr.add(next_read);
            let prev_ptr_write = ptr.add(next_write - 1);
            if !same_bucket(&mut *ptr_read, &mut *prev_ptr_write) {
                if next_read != next_write {
                    let ptr_write = prev_ptr_write.offset(1);
                    mem::swap(&mut *ptr_read, &mut *ptr_write);
                }
                next_write += 1;
            }
            next_read += 1;
        }
    }

    slice.split_at_mut(next_write)
}

#[inline(never)]
#[cold]
#[track_caller]
pub(super) fn slice_start_index_overflow_fail() -> ! {
    panic!("attempted to index slice from after maximum usize");
}

#[inline(never)]
#[cold]
#[track_caller]
pub(super) fn slice_end_index_overflow_fail() -> ! {
    panic!("attempted to index slice up to maximum usize");
}

#[inline(never)]
#[cold]
#[track_caller]
pub(super) fn slice_index_order_fail(index: usize, end: usize) -> ! {
    panic!("slice index starts at {} but ends at {}", index, end);
}

#[inline(never)]
#[cold]
#[track_caller]
pub(super) fn slice_end_index_len_fail(index: usize, len: usize) -> ! {
    panic!("range end index {} out of range for slice of length {}", index, len);
}

use core::ops::{Bound, Range, RangeBounds};

pub(crate) fn check_range<R: RangeBounds<usize>>(len: usize, range: R) -> Range<usize> {
    let start = match range.start_bound() {
        Bound::Included(&start) => start,
        Bound::Excluded(start) => start
            .checked_add(1)
            .unwrap_or_else(|| slice_start_index_overflow_fail()),
        Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
        Bound::Included(end) => end.checked_add(1).unwrap_or_else(|| slice_end_index_overflow_fail()),
        Bound::Excluded(&end) => end,
        Bound::Unbounded => len,
    };

    if start > end {
        slice_index_order_fail(start, end);
    }
    if end > len {
        slice_end_index_len_fail(end, len);
    }

    start..end
}
