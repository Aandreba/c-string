#[cfg(feature = "alloc_api")]
use alloc::alloc::*;
use alloc::ffi::FromVecWithNulError;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use docfg::docfg;
use core::{ffi::{c_char}, fmt::{Debug}};

#[derive(Debug, Clone)]
#[docfg(feature = "alloc")]
pub struct NulError<#[cfg(feature = "alloc_api")] A: Allocator = Global> {
    idx: Option<usize>,
    #[cfg(feature = "alloc_api")]
    chars: Vec<u8, A>,
    #[cfg(not(feature = "alloc_api"))]
    chars: Vec<u8>,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc_api")] {
        impl<A: Allocator> NulError<A> {
            #[inline]
            pub fn new (idx: Option<usize>, chars: Vec<u8, A>) -> Self {
                debug_assert!(idx.is_none() || chars.get(idx.unwrap()).copied() == Some(0));
                return Self {
                    idx,
                    chars
                }
            }

            #[inline]
            pub fn nul_position (&self) -> Option<usize> {
                return self.idx
            }

            #[inline]
            pub fn into_bytes (self) -> Vec<u8, A> {
                return self.chars
            }

            #[inline]
            pub fn into_chars (self) -> Vec<c_char, A> {
                let (ptr, len, cap, alloc) = self.into_bytes().into_raw_parts_with_alloc();
                return unsafe { Vec::from_raw_parts_in(ptr.cast(), len, cap, alloc) }
            }
        }
    } else if #[cfg(feature = "alloc")] {
        impl NulError {
            #[inline]
            pub fn new (idx: Option<usize>, chars: Vec<u8>) -> Self {
                debug_assert!(idx.is_none() || chars.get(idx.unwrap()).copied() == Some(0));
                return Self {
                    idx,
                    chars
                }
            }

            #[inline]
            pub fn nul_position (&self) -> Option<usize> {
                return self.idx
            }

            #[inline]
            pub fn into_bytes (self) -> Vec<u8> {
                return self.chars
            }

            #[inline]
            pub fn into_chars (self) -> Vec<c_char> {
                let (ptr, len, cap) = crate::vec_into_raw_parts(self.into_bytes());
                return unsafe { Vec::from_raw_parts(ptr.cast(), len, cap) }
            }
        }
    }
}

#[docfg(feature = "std")]
impl From<std::ffi::NulError> for NulError {
    #[inline]
    fn from(value: std::ffi::NulError) -> Self {
        return Self {
            idx: Some(value.nul_position()),
            chars: value.into_vec()
        }
    }
}

impl From<FromVecWithNulError> for NulError {
    #[inline]
    fn from(value: FromVecWithNulError) -> Self {
        return Self {
            idx: None,
            chars: value.into_bytes()
        }
    }
}