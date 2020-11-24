#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc as std;

#[cfg(feature = "alloc")]
use mockalloc::Mockalloc;
#[cfg(feature = "std")]
use std::alloc::System;

#[global_allocator]
#[cfg(feature = "std")]
static ALLOC: Mockalloc<System> = Mockalloc(System);

#[global_allocator]
#[cfg(all(feature = "alloc", not(feature = "std")))]
static ALLOC: Mockalloc<static_alloc::Bump<[u8; 1 << 16]>> =
    Mockalloc(static_alloc::Bump::new([0; 1 << 16]));

macro_rules! imp_make_tests_files {
    ($($ident:ident),* $(,)?) => {$(
        mod $ident {
            include!(concat!("template/", stringify!($ident), ".rs"));
        }
    )*};
}

macro_rules! make_tests_files {
    () => {
        imp_make_tests_files! { drain, simple, splice }
    };
}

macro_rules! leak {
    ($ident:ident) => {
        0
    };
}

#[cfg(feature = "nightly")]
mod array_vec {
    macro_rules! new_vec {
        ($vec:pat, max($len:expr)) => {
            let $vec = generic_vec::ArrayVec::<_, $len>::new();
        };
    }

    make_tests_files!();
}

mod slice_vec {
    macro_rules! new_vec {
        ($vec:pat, max($len:expr)) => {
            let mut buf = generic_vec::uninit_array!($len);
            let $vec = generic_vec::SliceVec::new(&mut buf);
        };
    }

    make_tests_files!();
}

#[cfg(feature = "nightly")]
mod init_array_vec {
    macro_rules! new_vec {
        ($vec:pat, max($len:expr)) => {
            let mut vec = generic_vec::InitArrayVec::new([Default::default(); $len]);
            vec.set_len(0);
            let $vec = vec;
        };
    }

    make_tests_files!();
}

#[cfg(feature = "alloc")]
mod heap_vec {
    macro_rules! new_vec {
        ($vec:pat, max($len:expr)) => {
            let $vec = generic_vec::Vec::new();
        };
    }

    make_tests_files!();
}

mod init_slice_vec {
    macro_rules! new_vec {
        ($vec:pat, max($len:expr)) => {
            let mut buf = [Default::default(); $len];
            let mut vec = generic_vec::InitSliceVec::new(&mut buf);
            vec.set_len(0);
            let $vec = vec;
        };
    }

    make_tests_files!();
}
