[![Crates.io](https://img.shields.io/crates/v/generic-vec.svg)](https://crates.io/crates/generic-vec)
[![Workflow Status](https://github.com/rustyyato/generic-vec/workflows/PR%20test/badge.svg)](https://github.com/rustyyato/generic-vec/actions?query=workflow%3A%22PR%2Btest%22)
![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# generic-vec

A vector that can store items anywhere: in slices, arrays, or the heap!

[`GenericVec`] has complete parity with [`Vec`], and even provides some features
that are only in `nightly` on `std` (like [`GenericVec::drain_filter`]), or a more permissive
interface like [`GenericVec::retain`]. In fact, you can trivially convert a [`Vec`] to a
[`HeapVec`] and back!

This crate is `no_std` compatible.

## Features

* `std` (default) - enables you to use an allocator, and
* `alloc` - enables you to use an allocator, for heap allocated storages
    (like [`Vec`])
* `nightly` - enables you to use array (`[T; N]`) based storages

## Basic Usage

On stable `no_std` you have two choices on for which storage you can use
[`SliceVec`] or [`InitSliceVec`]. There are three major differences between
them.

* You can pass an uninitialized buffer to [`SliceVec`]
* You can only use [`Copy`] types with [`InitSliceVec`]
* You can freely set the length of the [`InitSliceVec`] as long as you stay
    within it's capacity

```rust
use generic_vec::{SliceVec, InitSliceVec, uninit_array};

let mut uninit_buffer = uninit_array!(16);
let mut slice_vec = SliceVec::new(&mut uninit_buffer);

assert!(slice_vec.is_empty());
slice_vec.push(10);
assert_eq!(slice_vec, [10]);
```

```rust
let mut init_buffer = [0xae; 16];
let mut slice_vec = InitSliceVec::new(&mut init_buffer);

assert!(slice_vec.is_full());
assert_eq!(slice_vec.pop(), 0xae);
slice_vec.set_len(16);
assert!(slice_vec.is_full());
```

Of course if you try to push past a `*SliceVec`'s capacity
(the length of the slice you passed in), then it will panic.

```rust
let mut init_buffer = [0xae; 16];
let mut slice_vec = InitSliceVec::new(&mut init_buffer);
slice_vec.push(0);
```

If you enable the nightly feature then you gain access to
[`ArrayVec`] and [`InitArrayVec`]. These are just like the
slice versions, but since they own their data, they can be
freely moved around, unconstrained. You can also create
a new [`ArrayVec`] without passing in an existing buffer.

```rust
use generic_vec::ArrayVec;

let mut array_vec = ArrayVec::<i32, 16>::new();

array_vec.push(10);
array_vec.push(20);
array_vec.push(30);

assert_eq!(array_vec, [10, 20, 30]);
```

The ditinction between [`ArrayVec`] and [`InitArrayVec`]
is identical to their slice counterparts.

Finally a [`HeapVec`] is just [`Vec`], but built atop [`GenericVec`],
meaning you get all the features of [`GenericVec`] for free! But this
requries either the `alloc` or `std` feature to be enabled.


Current version: 0.1.0-alpha

License: MIT/Apache 2.0
