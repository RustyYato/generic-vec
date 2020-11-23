use array_vec::ArrayVec;

#[test]
fn raw_drain_front() {
    let mut vec = ArrayVec::<u8, 8>::new();

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
}

#[test]
fn raw_drain_back() {
    let mut vec = ArrayVec::<u8, 8>::new();

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
}
