#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(pattern, ptr_metadata, raw_os_nonzero, vec_into_raw_parts))]
#![cfg_attr(feature = "simd", feature(portable_simd))]
#![cfg_attr(feature = "alloc_api", feature(allocator_api))]
#![cfg_attr(docsrs, feature(doc_cfg))]

static_assertions::assert_eq_size!(c_char, u8);
static_assertions::assert_eq_align!(c_char, u8);

pub(crate) const NUL_CHAR: &c_char = &0;
pub(crate) const NUL_CHAR_PTR: *const c_char = NUL_CHAR;

cfg_if::cfg_if! {
    if #[cfg(feature = "nightly")] {
        pub(crate) const SIMD_64: bool = cfg!(all(
            any(
                target_arch = "arm",
                target_arch = "aarm64"
            ),
            target_feature = "neon"
        ));

        pub(crate) const SIMD_256: bool = cfg!(
            all(
                any(
                    target_arch = "x86",
                    target_arch = "x86_64"
                ),
                target_feature = "avx"
            )
        );

        pub(crate) const SIMD_512: bool = cfg!(
            all(
                any(
                    target_arch = "x86",
                    target_arch = "x86_64"
                ),
                target_feature = "avx512f"
            )
        );
    }
}

#[cfg(feature = "alloc")]
pub(crate) extern crate alloc;

mod substr;
pub use substr::*;

mod mut_cstr;
pub use mut_cstr::*;

mod const_cstr;
pub use const_cstr::*;

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
pub mod string;

#[docfg::docfg(feature = "alloc")]
pub use string::CString;

pub mod error;

#[cfg_attr(docsrs, doc(cfg(feature = "nightly")))]
#[cfg(feature = "nightly")]
pub mod pattern;

use core::{ffi::{c_char, c_int}};

extern "C" {
    pub(crate) fn strlen (p: *const c_char) -> usize;
    pub(crate) fn strcmp (lhs: *const c_char, rhs: *const c_char) -> c_int;
}

#[allow(dead_code)]
#[inline]
pub(crate) fn vec_into_raw_parts<T> (v: Vec<T>) -> (*mut T, usize, usize) {
    #[cfg(feature = "nightly")]
    return Vec::into_raw_parts(v);
    #[cfg(not(feature = "nightly"))]
    {
        let mut me = core::mem::ManuallyDrop::new(v);
        (me.as_mut_ptr(), me.len(), me.capacity())
    }
}