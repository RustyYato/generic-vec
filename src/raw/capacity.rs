#[cold]
#[inline(never)]
pub(in crate::raw) fn fixed_capacity_reserve_error(capacity: usize, new_capacity: usize) -> ! {
    panic!(
        "Tried to reserve {}, but used a fixed capacity storage of {}",
        new_capacity, capacity
    )
}

#[cfg(not(any(
    target_pointer_width = "8",
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
)))]
compile_error!("Cannot correctly calculate capacity on an 128-bit or larger architecture");

pub(in crate::raw) enum Round {
    Up,
    Down,
}

pub(in crate::raw) const fn capacity(old_capacity: usize, size_self: usize, size_other: usize, round: Round) -> usize {
    #[cfg(target_pointer_width = "8")]
    type PointerNext = u16;
    #[cfg(target_pointer_width = "16")]
    type PointerNext = u32;
    #[cfg(target_pointer_width = "32")]
    type PointerNext = u64;
    #[cfg(target_pointer_width = "64")]
    type PointerNext = u128;

    if size_other == 0 {
        return usize::MAX
    }

    let size = (old_capacity as PointerNext) * (size_self as PointerNext);

    let size = match round {
        // this can't overflow
        //
        // old_capacity : n-bit number   0..pow2(n)
        // size_self    : n-bit number   0..pow2(n)
        // size_other   : n-bit number   1..pow2(n)
        // PointerNext  : 2*n-bit number 0..pow2(n)
        //
        // size = 0..=(pow2(n)-1) * 0..=(pow2(n)-1)
        //      = 0..=(pow2(n) * pow2(n) - 2 * pow2(n) - 1)
        //      = 0..=(pow2(2 * n) - 2 * pow2(n) - 1)
        //
        // size + size_other - 1 = 0..=(pow2(2 * n) - 2 * pow2(n) - 1) + 1..=pow2(n) - 1..=1
        //                       = 0..=(pow2(2 * n) - 2 * pow2(n) - 1) + 0..=pow2(n)
        //                       = 0..=(pow2(2 * n) - 2 * pow2(n) - 1 + pow2(n))
        //                       = 0..=(pow2(2 * n) - pow2(n) - 1) < pow2(2 * n)
        //
        Round::Up => size.wrapping_add(size_other as PointerNext).wrapping_sub(1),
        Round::Down => size,
    };

    // this can't overflow pow2(n)
    //
    // size = 0..=(pow2(2 * n) - pow2(n) - 1)
    // size / size_other = 0..=(pow2(2 * n) - pow2(n) - 1) / pow2(n)
    //                   = 0..=(pow2(n) - 1 - 1 / pow2(n)) < pow2(n)
    //
    (size / (size_other as PointerNext)) as usize
}
