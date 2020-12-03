// use std::string::ToString;

#[test]
fn raw_drain_front() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        {
            let mut drain = vec.cursor(..);

            assert_eq!(drain.take_front(), "0");
            assert_eq!(drain.take_front(), "2");
            assert_eq!(drain.take_front(), "1");
        }

        assert_eq!(vec, []);

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        {
            let mut drain = vec.cursor(..);

            assert_eq!(drain.take_front(), "0");
            assert_eq!(drain.take_front(), "2");
        }

        assert_eq!(vec, S!([1]));

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        {
            let mut drain = vec.cursor(..);

            assert_eq!(drain.take_front(), "1");
            drain.skip_front();
            assert_eq!(drain.take_front(), "2");
        }

        assert_eq!(vec, S!([0, 1]));

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        {
            let mut drain = vec.cursor(..);

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

        {
            let mut drain = vec.cursor(..);

            assert_eq!(drain.take_back(), "1");
            assert_eq!(drain.take_back(), "2");
            assert_eq!(drain.take_back(), "0");
        }

        assert_eq!(vec, []);

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        {
            let mut drain = vec.cursor(..);

            assert_eq!(drain.take_back(), "1");
            assert_eq!(drain.take_back(), "2");
        }

        assert_eq!(vec, S!([0]));

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        {
            let mut drain = vec.cursor(..);

            assert_eq!(drain.take_back(), "1");
            drain.skip_back();
            assert_eq!(drain.take_back(), "0");
        }

        assert_eq!(vec, S!([0, 2]));

        vec.push(S!("0"));
        vec.push(S!("2"));
        vec.push(S!("1"));

        {
            let mut drain = vec.cursor(..);

            assert_eq!(drain.take_back(), "1");
        }

        assert_eq!(vec, S!([0, 2, 0, 2]))
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(raw_drain_back));
}
