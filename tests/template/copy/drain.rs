#[test]
fn drain() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));

        vec.extend([0, 1, 2, 3, 4, 5, 6, 7].iter().copied());

        assert_eq!(vec, [0, 1, 2, 3, 4, 5, 6, 7]);

        vec.drain(4..7);

        assert_eq!(vec, [0, 1, 2, 3, 7]);

        assert!(vec.drain(1..3).rev().eq([2, 1].iter().copied()));

        assert_eq!(vec, [0, 3, 7]);
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(drain));
}

#[test]
fn drain_filter() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));

        vec.extend([0, 1, 2, 3, 4, 5, 6, 7].iter().copied());

        vec.drain_filter(.., |&mut x| x % 2 == 0);

        assert_eq!(vec, [1, 3, 5, 7]);

        assert!(vec.drain_filter(.., |&mut x| x % 3 == 0).eq([3].iter().copied()));

        assert_eq!(vec, [1, 5, 7]);
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(drain_filter));
}
