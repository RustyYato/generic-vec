use generic_vec::SliceVec;

#[test]
fn split_off() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));
        vec.extend((0..8).map(|x| S!(x)));
        let mut other = generic_vec::uninit_array!(4);
        let mut other = SliceVec::new(&mut other);
        vec.split_off_into(4, &mut other);
        assert_eq!(vec, S!([0, 1, 2, 3]));
        assert_eq!(other, S!([4, 5, 6, 7]));
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(simple));
}

#[test]
fn consume_extend() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(4));
        let mut other = generic_vec::uninit_array!(4);
        let mut other = SliceVec::new(&mut other);
        other.extend((0..4).map(|x| S!(x)));
        other.split_off_into(0, &mut vec);
        assert_eq!(vec, S!([0, 1, 2, 3]));
        assert_eq!(other, []);
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(simple));
}

#[test]
fn grow() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(4));
        vec.grow(4, S!(0));
        assert_eq!(vec, [S!(0), S!(0), S!(0), S!(0)]);
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(simple));
}
