#[test]
fn splice() {
    new_vec!(mut vec, max(8));

    vec.extend([0, 1, 2, 3, 4, 5, 6, 7].iter().copied());

    vec.splice(2..5, [4, 3, 2, 1].iter().copied());

    assert_eq!(vec, [0, 1, 4, 3, 2, 5, 6, 7]);

    new_vec!(mut vec, max(10));

    vec.extend([0, 1, 2, 3, 4, 5, 6, 7].iter().copied());

    vec.splice(2..5, [9, 8].iter().copied());

    assert_eq!(vec, [0, 1, 9, 8, 5, 6, 7]);
}
