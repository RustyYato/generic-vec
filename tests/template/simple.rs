#[test]
pub fn simple() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));

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
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(simple));
}
