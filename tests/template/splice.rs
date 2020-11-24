#[test]
fn splice_exact_or_more() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(9));

        vec.extend([0, 1, 2, 3, 4, 5, 6, 7].iter().copied());

        #[cfg(feature = "alloc")]
        {
            vec.splice(2..5, [4, 3, 2, 1].iter().copied());
            assert_eq!(vec, [0, 1, 4, 3, 2, 1, 5, 6, 7]);
        }

        #[cfg(not(feature = "alloc"))]
        {
            vec.splice(2..5, [4, 3, 2].iter().copied());
            assert_eq!(vec, [0, 1, 4, 3, 2, 5, 6, 7]);
        }
    });

    assert_eq!(
        output.mem_allocated(),
        output.mem_freed() + leak!(splice_exact)
    );
}

#[test]
fn splice_less() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(10));

        vec.extend([0, 1, 2, 3, 4, 5, 6, 7].iter().copied());

        vec.splice(2..5, [9, 8].iter().copied());

        assert_eq!(vec, [0, 1, 9, 8, 5, 6, 7]);
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(splice));
}

#[test]
fn splice_from_zero() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(10));

        vec.splice(0..0, [0, 1, 2, 3, 4, 5, 6, 7].iter().copied());

        assert_eq!(vec, [0, 1, 2, 3, 4, 5, 6, 7]);
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(splice));
}
