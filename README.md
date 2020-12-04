[![Crates.io](https://img.shields.io/crates/v/generic-vec.svg)](https://crates.io/crates/generic-vec)
[![Docs.rs](https://docs.rs/generic-vec/badge.svg)](https://docs.rs/generic-vec)
[![Workflow Status](https://github.com/rustyyato/generic-vec/workflows/main/badge.svg)](https://github.com/rustyyato/generic-vec/actions?query=workflow%3A%22main%22)
![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# generic-vec

A vector that can store items anywhere: in slices, arrays, or the heap!

`GenericVec` has complete parity with `Vec`, and even provides some features
that are only in `nightly` on `std` (like `GenericVec::drain_filter`), or a more permissive
interface like `GenericVec::retain`. In fact, you can trivially convert a `Vec` to a
`HeapVec` and back!

This crate is `no_std` compatible, just turn off all default features.

## Features

* `std` (default) - enables you to use an allocator, and
* `alloc` - enables you to use an allocator, for heap allocated storages
    (like `Vec`)
* `nightly` - enables you to use array (`[T; N]`) based storages

## Basic Usage

On stable `no_std` you have four choices on for which storage you can use
`SliceVec`, `InitSliceVec`, `TypeVec`, and `ZSVec`.

`SliceVec` and `InitSliceVec` are pretty similar, you give them a slice
buffer, and they store all of thier values in that buffer. But have three major
differences between them.

* You can pass an uninitialized buffer to `SliceVec`
* You can only use `Copy` types with `InitSliceVec`
* You can freely set the length of the `InitSliceVec` as long as you stay
    within it's capacity (the length of the slice you pass in)

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

`TypeVec` is an owned buffer. You can use like so:

```rust
use generic_vec::{TypeVec, gvec};
let mut vec: TypeVec<u32, [u32; 4]> = gvec![1, 2, 3, 4];

assert_eq!(vec, [1, 2, 3, 4]);

vec.try_push(5).expect_err("Tried to push past capacity!");
```

The second parameter specifies the buffer type, this can be any type
you want. Only the size of the type matters. There is also a defaulted
third parameter, but you should only use that if you know what you are
doing, and after reading the docs for `UninitBuffer`.

As a neat side-effect of this framework, you can also get an efficient
`GenericVec` for zero-sized types, just a `usize` in size! This feature
can be on stable `no_std`.

```rust
use generic_vec::ZSVec;

struct MyType;

let mut vec = ZSVec::new();

vec.push(MyType);
vec.push(MyType);
vec.push(MyType);

assert_eq!(vec.len(), 3);
assert_eq!(std::mem::size_of_val(&vec), std::mem::size_of::<usize>());
```

### Nightly

If you enable the nightly feature then you gain access to
`ArrayVec` and `InitArrayVec`. These are just like the
slice versions, but since they own their data, they can be
freely moved around, unconstrained. You can also create
a new `ArrayVec` without passing in an existing buffer.

```rust
use generic_vec::ArrayVec;

let mut array_vec = ArrayVec::<i32, 16>::new();

array_vec.push(10);
array_vec.push(20);
array_vec.push(30);

assert_eq!(array_vec, [10, 20, 30]);
```

The distinction between `ArrayVec` and `InitArrayVec`
is identical to their slice counterparts.

Finally a `HeapVec` is just `Vec`, but built atop `GenericVec`,
meaning you get all the features of `GenericVec` for free! But this
requries either the `alloc` or `std` feature to be enabled.

Note on the documentation: if the feature exists on `Vec`, then the documentation
is either exactly the same as `Vec` or slightly adapted to better fit `GenericVec`

Current version: 0.1.1

License: MIT/Apache-2.0
