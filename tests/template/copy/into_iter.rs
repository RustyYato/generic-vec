#[test]
pub fn into_iter() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));
        vec.extend(0..8);

        assert!((0..8).eq(vec));
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(simple));
}
