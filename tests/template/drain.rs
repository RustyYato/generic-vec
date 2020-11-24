#[test]
fn raw_drain_front() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));

        vec.push(0);
        vec.push(2);
        vec.push(1);

        unsafe {
            let mut drain = vec.raw_drain(..);

            assert_eq!(drain.take_front(), 0);
            assert_eq!(drain.take_front(), 2);
            assert_eq!(drain.take_front(), 1);
        }

        assert_eq!(vec.len(), 0);

        vec.push(0);
        vec.push(2);
        vec.push(1);

        unsafe {
            let mut drain = vec.raw_drain(..);

            assert_eq!(drain.take_front(), 0);
            assert_eq!(drain.take_front(), 2);
        }

        assert_eq!(vec.len(), 1);
        assert_eq!(vec.get(0), Some(&1));

        vec.push(0);
        vec.push(2);
        vec.push(1);

        unsafe {
            let mut drain = vec.raw_drain(..);

            assert_eq!(drain.take_front(), 1);
            drain.skip_front();
            assert_eq!(drain.take_front(), 2);
        }

        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(0), Some(&0));
        assert_eq!(vec.get(1), Some(&1));

        vec.push(0);
        vec.push(2);
        vec.push(1);

        unsafe {
            let mut drain = vec.raw_drain(..);

            assert_eq!(drain.take_front(), 0);
        }

        assert_eq!(&vec[..], [1, 0, 2, 1])
    });

    assert_eq!(
        output.mem_allocated(),
        output.mem_freed() + leak!(raw_drain_front)
    );
}

#[test]
fn raw_drain_back() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));

        vec.push(0);
        vec.push(2);
        vec.push(1);

        unsafe {
            let mut drain = vec.raw_drain(..);

            assert_eq!(drain.take_back(), 1);
            assert_eq!(drain.take_back(), 2);
            assert_eq!(drain.take_back(), 0);
        }

        assert_eq!(vec.len(), 0);

        vec.push(0);
        vec.push(2);
        vec.push(1);

        unsafe {
            let mut drain = vec.raw_drain(..);

            assert_eq!(drain.take_back(), 1);
            assert_eq!(drain.take_back(), 2);
        }

        assert_eq!(vec.len(), 1);
        assert_eq!(vec.get(0), Some(&0));

        vec.push(0);
        vec.push(2);
        vec.push(1);

        unsafe {
            let mut drain = vec.raw_drain(..);

            assert_eq!(drain.take_back(), 1);
            drain.skip_back();
            assert_eq!(drain.take_back(), 0);
        }

        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(0), Some(&0));
        assert_eq!(vec.get(1), Some(&2));

        vec.push(0);
        vec.push(2);
        vec.push(1);

        unsafe {
            let mut drain = vec.raw_drain(..);

            assert_eq!(drain.take_back(), 1);
        }

        assert_eq!(&vec[..], [0, 2, 0, 2])
    });

    assert_eq!(
        output.mem_allocated(),
        output.mem_freed() + leak!(raw_drain_back)
    );
}
