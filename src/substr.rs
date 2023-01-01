use core::{ffi::c_char, ops::{Deref, DerefMut, Index, IndexMut}, borrow::Borrow, fmt::{Debug, Display}};
use docfg::docfg;
#[cfg(feature = "simd")]
use {core::simd::*, crate::{SIMD_64, SIMD_128, SIMD_512, SIMD_256}};
use crate::CString;
#[cfg(feature = "nightly")]
use crate::pattern::*;

const ASCII_CASE_MASK: c_char = 0b0010_0000;

#[repr(transparent)]
pub struct CSubStr {
    pub(crate) inner: [c_char]
}

impl CSubStr {
    #[inline]
    pub const unsafe fn from_str_unchecked<'a> (s: &'a str) -> &'a Self {
        return Self::from_bytes_unchecked(s.as_bytes())
    }

    #[inline]
    pub unsafe fn from_mut_str_unchecked<'a> (s: &'a mut str) -> &'a mut Self {
        return Self::from_mut_bytes_unchecked(s.as_bytes_mut())
    }

    #[inline]
    pub const unsafe fn from_bytes_unchecked<'a> (chars: &'a [u8]) -> &'a Self {
        return Self::from_chars_unchecked(&*(chars as *const [_] as *const [c_char]))
    }

    #[inline]
    pub unsafe fn from_mut_bytes_unchecked<'a> (chars: &'a mut [u8]) -> &'a mut Self {
        return Self::from_mut_chars_unchecked(&mut *(chars as *mut [_] as *mut [c_char]))
    }
    
    #[inline]
    pub const unsafe fn from_chars_unchecked<'a> (chars: &'a [c_char]) -> &'a Self {
        return unsafe { core::mem::transmute(chars) }
    }

    #[inline]
    pub unsafe fn from_mut_chars_unchecked<'a> (chars: &'a mut [c_char]) -> &'a mut Self {
        return unsafe { core::mem::transmute(chars) }
    }

    #[inline]
    pub fn len (&self) -> usize {
        #[cfg(feature = "nightly")]
        return core::ptr::metadata(self);
        #[cfg(not(feature = "nightly"))]
        return self.inner.len();
    }

    #[inline]
    pub fn as_ptr (&self) -> *const c_char {
        self.inner.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr (&mut self) -> *mut c_char {
        self.inner.as_mut_ptr()
    }

    #[inline]
    pub fn as_bytes (&self) -> &[u8] {
        return unsafe { &*(self.as_c_chars() as *const [c_char] as *const [_]) }
    }

    #[inline]
    pub fn as_mut_bytes (&mut self) -> &mut [u8] {
        return unsafe { &mut *(self.as_mut_c_chars() as *mut [c_char] as *mut [_]) }
    }
    
    #[inline]
    pub fn as_c_chars (&self) -> &[c_char] {
        return &self.inner
    }

    #[inline]
    pub fn as_mut_c_chars (&mut self) -> &mut [c_char] {
        return &mut self.inner
    }

    #[docfg(feature = "nightly")]
    #[inline]
    pub fn contains<'a, P: CPattern<'a>> (&'a self, pat: P) -> bool {
        let mut searcher = pat.into_searcher(self);
        return crate::pattern::CSearcher::next_match(&mut searcher).is_some();
    }
}

impl CSubStr {
    #[docfg(feature = "alloc_api")]
    #[inline]
    pub fn to_cstring_in<A: core::alloc::Allocator> (&self, alloc: A) -> CString<A> {
        return CString::from_substr_in(self, alloc)
    }

    #[inline]
    pub fn to_cstring (&self) -> CString {
        return CString::from_substr(self)
    }

    #[inline]
    pub fn to_uppercase (&self) -> CString {
        let mut result = self.to_cstring();
        result.uppercase();
        return result
    }

    #[inline]
    pub fn to_lowercase (&self) -> CString {
        let mut result = self.to_cstring();
        result.lowercase();
        return result
    }
}

#[cfg(feature = "simd")]
impl CSubStr {
    #[inline]
    pub fn uppercase (&mut self) {
        Self::uppercase_inner(self.as_mut_c_chars())
    }

    #[inline]
    pub fn lowercase (&mut self) {
        Self::lowercase_inner(self.as_mut_c_chars())
    }

    fn uppercase_inner (mut chars: &mut [c_char]) {
        const MIN_CHAR: c_char = b'a' as c_char;
        const MAX_CHAR: c_char = b'z' as c_char;

        macro_rules! inner {
            ($bits:literal, $has:expr) => {
                if $has {
                    match uppercase_inner! { $bits, chars } {
                        Ok((lhs, rhs)) => {
                            chars = lhs;
                            Self::uppercase_inner(rhs)
                        },
                        Err(this) => chars = this
                    }
                }
            };
        }

        macro_rules! uppercase_inner {
            ($bits:literal, $chars:expr) => {{
                const BYTES: usize = $bits / (8 * core::mem::size_of::<c_char>());
        
                if chars.len() > BYTES {
                    const CASE_MASK: Simd<c_char, BYTES> = Simd::from_array([ASCII_CASE_MASK; BYTES]);
                    const MIN: Simd<c_char, BYTES> = Simd::from_array([MIN_CHAR; BYTES]);
                    const MAX: Simd<c_char, BYTES> = Simd::from_array([MAX_CHAR; BYTES]);
        
                    let (lhs, simd, rhs) = chars.as_simd_mut::<BYTES>();
                    for x in simd.iter_mut() {
                        let are_lower = x.simd_le(MAX) & x.simd_ge(MIN);
                        *x ^= CASE_MASK & are_lower.to_int().cast::<c_char>()
                    }
        
                    Ok((lhs, rhs))
                } else {
                    Err(chars)
                }
            }};
        }

        inner! { 512, SIMD_512 }
        inner! { 256, SIMD_256 }
        inner! { 128, SIMD_128 }
        inner! { 64, SIMD_64 }

        for c in chars {
            if matches!(*c, MIN_CHAR..=MAX_CHAR) {
                *c ^= ASCII_CASE_MASK;
            }
        }
    }

    fn lowercase_inner (mut chars: &mut [c_char]) {
        const MIN_CHAR: c_char = b'A' as c_char;
        const MAX_CHAR: c_char = b'Z' as c_char;

        macro_rules! inner {
            ($bits:literal, $has:expr) => {
                if $has {
                    match lowercase_inner! { $bits, chars } {
                        Ok((lhs, rhs)) => {
                            chars = lhs;
                            Self::lowercase_inner(rhs)
                        },
                        Err(this) => chars = this
                    }
                }
            };
        }

        macro_rules! lowercase_inner {
            ($bits:literal, $chars:expr) => {{
                const BYTES: usize = $bits / (8 * core::mem::size_of::<c_char>());
        
                if chars.len() > BYTES {
                    const CASE_MASK: Simd<c_char, BYTES> = Simd::from_array([ASCII_CASE_MASK; BYTES]);
                    const MIN: Simd<c_char, BYTES> = Simd::from_array([MIN_CHAR; BYTES]);
                    const MAX: Simd<c_char, BYTES> = Simd::from_array([MAX_CHAR; BYTES]);
        
                    let (lhs, simd, rhs) = chars.as_simd_mut::<BYTES>();
                    for x in simd.iter_mut() {
                        let are_lower = x.simd_le(MAX) & x.simd_ge(MIN);
                        *x ^= CASE_MASK & are_lower.to_int().cast::<c_char>()
                    }
        
                    Ok((lhs, rhs))
                } else {
                    Err(chars)
                }
            }};
        }

        inner! { 512, SIMD_512 }
        inner! { 256, SIMD_256 }
        inner! { 128, SIMD_128 }
        inner! { 64, SIMD_64 }

        for c in chars {
            if matches!(*c, MIN_CHAR..=MAX_CHAR) {
                *c ^= ASCII_CASE_MASK;
            }
        }
    }
}

#[cfg(not(feature = "simd"))]
impl CSubStr {
    #[inline]
    pub fn uppercase (&mut self) {
        const MIN_CHAR: c_char = b'a' as c_char;
        const MAX_CHAR: c_char = b'z' as c_char;

        for c in self.as_mut_c_chars() {
            if matches!(*c, MIN_CHAR..=MAX_CHAR) {
                *c ^= ASCII_CASE_MASK;
            }
        }
    }

    #[inline]
    pub fn lowercase (&mut self) {
        const MIN_CHAR: c_char = b'A' as c_char;
        const MAX_CHAR: c_char = b'Z' as c_char;

        for c in self.as_mut_c_chars() {
            if matches!(*c, MIN_CHAR..=MAX_CHAR) {
                *c ^= ASCII_CASE_MASK;
            }
        }
    }
}

impl<T> Index<T> for CSubStr where [c_char]: Index<T> {
    type Output = <[c_char] as Index<T>>::Output;

    #[inline]
    fn index(&self, index: T) -> &Self::Output {
        self.as_c_chars().index(index)
    }
}

impl<T> IndexMut<T> for CSubStr where [c_char]: IndexMut<T> {
    #[inline]
    fn index_mut (&mut self, index: T) -> &mut Self::Output {
        self.as_mut_c_chars().index_mut(index)
    }
}

// CHARS COMPARISON
impl PartialEq<[c_char]> for CSubStr {
    #[inline]
    fn eq(&self, other: &[c_char]) -> bool {
        self.as_c_chars().eq(other)
    }
}

impl PartialEq<CSubStr> for [c_char] {
    #[inline]
    fn eq(&self, other: &CSubStr) -> bool {
        self.eq(other.as_c_chars())
    }
}

impl PartialOrd<[c_char]> for CSubStr {
    #[inline]
    fn partial_cmp(&self, other: &[c_char]) -> Option<core::cmp::Ordering> {
        Some(self.as_c_chars().cmp(other))
    }
}

impl PartialOrd<CSubStr> for [c_char] {
    #[inline]
    fn partial_cmp(&self, other: &CSubStr) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other.as_c_chars()))
    }
}

// SLICE COMPARISON
impl<T: ?Sized + Borrow<CSubStr>> PartialEq<T> for CSubStr {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.as_c_chars().eq(other.borrow().as_c_chars())
    }
}

impl<T: ?Sized + Borrow<CSubStr>> PartialOrd<T> for CSubStr {
    #[inline]
    fn partial_cmp(&self, other: &T) -> Option<core::cmp::Ordering> {
        Some(self.as_c_chars().cmp(other.borrow().as_c_chars()))
    }
}

impl Ord for CSubStr {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_c_chars().cmp(other.as_c_chars())
    }
}

impl Deref for CSubStr {
    type Target = [c_char];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_c_chars()
    }
}

impl DerefMut for CSubStr {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_c_chars()
    }
}

impl Debug for CSubStr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "\"{}\"", self.as_bytes().escape_ascii())
    }
}

impl Display for CSubStr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.as_bytes().escape_ascii(), f)
    }
}

impl Eq for CSubStr {}