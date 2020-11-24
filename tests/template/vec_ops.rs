use generic_vec::SliceVec;

#[test]
fn split_off() {
    new_vec!(mut vec, max(8));
    vec.extend(0..8);
    let mut other = generic_vec::uninit_array!(4);
    let mut other = SliceVec::new(&mut other);
    vec.split_off_into(4, &mut other);
    assert_eq!(vec, [0, 1, 2, 3]);
    assert_eq!(other, [4, 5, 6, 7]);
}
