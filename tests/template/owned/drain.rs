// use std::string::ToString;

#[test]
fn raw_drain_front() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        unsafe {
            let mut drain = vec.raw_cursor(..);

            assert_eq!(drain.take_front(), "0");
            assert_eq!(drain.take_front(), "2");
            assert_eq!(drain.take_front(), "1");
        }

        assert_eq!(vec, []);

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        unsafe {
            let mut drain = vec.raw_cursor(..);

            assert_eq!(drain.take_front(), "0");
            assert_eq!(drain.take_front(), "2");
        }

        assert_eq!(vec, S!([1]));

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        unsafe {
            let mut drain = vec.raw_cursor(..);

            assert_eq!(drain.take_front(), "1");
            drain.skip_front();
            assert_eq!(drain.take_front(), "2");
        }

        assert_eq!(vec, S!([0, 1]));

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        unsafe {
            let mut drain = vec.raw_cursor(..);

            assert_eq!(drain.take_front(), "0");
        }

        assert_eq!(vec, S!([1, 0, 2, 1]))
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(raw_drain_front));
}

#[test]
fn raw_drain_back() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        unsafe {
            let mut drain = vec.raw_cursor(..);

            assert_eq!(drain.take_back(), "1");
            assert_eq!(drain.take_back(), "2");
            assert_eq!(drain.take_back(), "0");
        }

        assert_eq!(vec, []);

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        unsafe {
            let mut drain = vec.raw_cursor(..);

            assert_eq!(drain.take_back(), "1");
            assert_eq!(drain.take_back(), "2");
        }

        assert_eq!(vec, S!([0]));

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        unsafe {
            let mut drain = vec.raw_cursor(..);

            assert_eq!(drain.take_back(), "1");
            drain.skip_back();
            assert_eq!(drain.take_back(), "0");
        }

        assert_eq!(vec, S!([0, 2]));

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        unsafe {
            let mut drain = vec.raw_cursor(..);

            assert_eq!(drain.take_back(), "1");
        }

        assert_eq!(vec, S!([0, 2, 0, 2]))
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(raw_drain_back));
}

#[test]
fn drain() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));

        vec.extend([0, 1, 2, 3, 4, 5, 6, 7].iter().map(|x| S!(x)));

        assert_eq!(vec, S!([0, 1, 2, 3, 4, 5, 6, 7]));

        vec.drain(4..7);

        assert_eq!(vec, S!([0, 1, 2, 3, 7]));

        assert!(vec.drain(1..3).eq([1, 2].iter().map(|x| S!(x))));

        assert_eq!(vec, S!([0, 3, 7]));
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(drain));
}

#[test]
fn drain_filter() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));

        vec.extend(
            ["0", "00", "000", "0000", "00000", "000000", "0000000", "00000000"]
                .iter()
                .map(|x| S!(x)),
        );

        vec.drain_filter(.., |x| x.len() % 2 == 0);

        assert_eq!(vec, S!(["0", "000", "00000", "0000000",]));

        assert!(vec.drain_filter(.., |x| x.len() % 3 == 0).eq(Some(S!("000"))));

        assert_eq!(vec, S!(["0", "00000", "0000000",]));
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(drain_filter));
}
