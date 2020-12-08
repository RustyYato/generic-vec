use generic_vec::{InitSliceVec, SliceVec};

#[test]
fn split_off() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));
        vec.extend(0..8);
        let mut other = generic_vec::uninit_array!(4);
        let mut other = SliceVec::new(&mut other);
        vec.split_off_into(4, &mut other);
        assert_eq!(vec, [0, 1, 2, 3]);
        assert_eq!(other, [4, 5, 6, 7]);
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(simple));
}

#[test]
fn consume_extend() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(4));
        let mut other = [0, 1, 2, 3];
        let mut other = InitSliceVec::new(&mut other);
        other.split_off_into(0, &mut vec);
        assert_eq!(vec, [0, 1, 2, 3]);
        assert_eq!(other, []);
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(simple));
}

#[test]
fn grow() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(4));
        vec.grow(4, 0);
        assert_eq!(vec, [0; 4]);
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(simple));
}
