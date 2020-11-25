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
static ALLOC: Mockalloc<static_alloc::Bump<[u8; 1 << 22]>> = Mockalloc(static_alloc::Bump::new([0; 1 << 22]));

#[cfg(feature = "alloc")]
macro_rules! S {
    ([$($e:expr),* $(,)?]) => {
        [$({
            let x = $e;
            crate::to_string::assert(&x);
            crate::to_string::TestToString::to_string(&x)
        }),*]
    };
    ($l:expr) => {
        {
            let x = $l;
            crate::to_string::assert(&x);
            crate::to_string::TestToString::to_string(&x)
        }
    };
}

#[cfg(feature = "alloc")]
mod to_string {
    pub trait TestToString: std::string::ToString {}
    pub fn assert<T: TestToString>(_: &T) {}

    impl TestToString for i32 {}
    impl TestToString for &i32 {}
    impl TestToString for &str {}
    impl TestToString for &&str {}
}

macro_rules! imp_make_tests_files {
    ($(#[$meta:meta])*mod $mod:ident {
        $($ident:ident),* $(,)?
    }) => {
        $(#[$meta])*
        mod $mod {
            $(
                mod $ident {
                    include!(concat!("template/", stringify!($mod), "/", stringify!($ident), ".rs"));
                }
            )*
        }
    };
}

macro_rules! make_tests_files {
    () => {
        make_tests_files! { copy_only }
        imp_make_tests_files! {
            #[cfg(feature = "alloc")]
            mod owned { simple, into_iter, drain, splice, vec_ops }
        }
    };
    (copy_only) => {
        imp_make_tests_files! {
            mod copy { simple, into_iter, drain, splice, vec_ops }
        }
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

    make_tests_files!(copy_only);
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

    make_tests_files!(copy_only);
}
