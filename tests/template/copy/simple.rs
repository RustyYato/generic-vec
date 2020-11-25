#[test]
pub fn simple() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));

        assert_eq!(vec.len(), 0);
        assert_eq!(*vec.push(0), 0);
        assert_eq!(*vec.push(2), 2);
        assert_eq!(*vec.push(1), 1);
        assert_eq!(vec, [0, 2, 1]);
        assert_eq!(vec.pop(), 1);
        assert_eq!(vec, [0, 2]);
        assert_eq!(*vec.insert(1, 9), 9);
        assert_eq!(*vec.insert(2, 8), 8);
        assert_eq!(*vec.insert(3, 7), 7);
        assert_eq!(vec, [0, 9, 8, 7, 2]);
        assert_eq!(vec.remove(2), 8);
        assert_eq!(vec.remove(2), 7);
        assert_eq!(vec, [0, 9, 2]);
        assert_eq!(vec.swap_remove(0), 0);
        assert_eq!(vec, [2, 9]);
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(simple));
}

#[test]
#[cfg(feature = "nightly")]
pub fn array_ops() {
    let output = mockalloc::record_allocs(|| {
        new_vec!(mut vec, max(8));

        assert_eq!(vec.len(), 0);
        assert_eq!(*vec.push_array([0, 2, 1]), [0, 2, 1]);
        assert_eq!(vec, [0, 2, 1]);
        assert_eq!(vec.pop_array(), [1]);
        assert_eq!(vec, [0, 2]);
        assert_eq!(*vec.insert_array(1, [9, 8, 7]), [9, 8, 7]);
        assert_eq!(vec, [0, 9, 8, 7, 2]);
        assert_eq!(vec.remove_array(2), [8, 7]);
        assert_eq!(vec, [0, 9, 2]);
    });

    assert_eq!(output.mem_allocated(), output.mem_freed() + leak!(simple));
}
