#![cfg(feature = "nightly")]

use array_vec::ArrayVec;

#[test]
pub fn simple() {
    let mut vec = ArrayVec::<u8, 8>::new();

    assert_eq!(vec.len(), 0);
    vec.push(0);
    vec.push(2);
    vec.push(1);
    assert_eq!(vec.len(), 3);
    assert_eq!(vec.get(0), Some(&0));
    assert_eq!(vec.get(1), Some(&2));
    assert_eq!(vec.get(2), Some(&1));
    vec.pop();
    assert_eq!(vec.len(), 2);
    assert_eq!(vec.get(0), Some(&0));
    assert_eq!(vec.get(1), Some(&2));
}
