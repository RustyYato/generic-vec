#![cfg(feature = "nightly")]

use array_vec::ArrayVec;

#[test]
fn splice() {
    let mut vec = ArrayVec::<u8, 10>::new();

    vec.extend([0, 1, 2, 3, 4, 5, 6, 7].iter().copied());

    vec.splice(2..5, [4, 3, 2, 1].iter().copied());

    assert_eq!(vec, [0, 1, 4, 3, 2, 5, 6, 7]);

    let mut vec = ArrayVec::<u8, 10>::new();

    vec.extend([0, 1, 2, 3, 4, 5, 6, 7].iter().copied());

    vec.splice(2..5, [9, 8].iter().copied());

    assert_eq!(vec, [0, 1, 9, 8, 5, 6, 7]);
}
