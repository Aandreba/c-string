use core::{ffi::c_char, ops::{Deref, DerefMut, Index, IndexMut}, borrow::Borrow, fmt::{Debug, Display}};
use docfg::docfg;
#[cfg(feature = "simd")]
use {core::simd::*, crate::{SIMD_64, SIMD_512, SIMD_256}};
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

#[cfg(feature = "simd")]
impl CSubStr {
    #[allow(unused_assignments)]
    pub fn uppercase (&mut self) {
        macro_rules! uppercase {
            ($bits:literal => $target:ident) => {{
                const BYTES: usize = $bits / (8 * core::mem::size_of::<c_char>());
                const CASE_MASK: Simd<c_char, BYTES> = Simd::from_array([ASCII_CASE_MASK; BYTES]);
                const MIN: Simd<c_char, BYTES> = Simd::from_array([MIN_CHAR; BYTES]);
                const MAX: Simd<c_char, BYTES> = Simd::from_array([MAX_CHAR; BYTES]);

                let (lhs, simd, rhs) = $target.as_simd_mut::<BYTES>();
                for x in simd.iter_mut() {
                    let are_lower = x.simd_le(MAX) & x.simd_ge(MIN);
                    *x ^= CASE_MASK & are_lower.to_int().cast::<c_char>()
                }

                this_lhs = lhs;
                this_rhs = rhs;
            }};
        }

        macro_rules! uppercase_inner {
            ($fn:ident, $bits:literal) => {
                fn $fn (chars: &mut [c_char]) -> Result<(&mut [c_char], &mut [c_char]), &mut [c_char]> {
                    const BYTES: usize = $bits / (8 * core::mem::size_of::<c_char>());
                    const MIN_CHAR: c_char = b'a' as c_char;
                    const MAX_CHAR: c_char = b'z' as c_char;
            
                    if chars.len() > BYTES {
                        const CASE_MASK: Simd<c_char, BYTES> = Simd::from_array([ASCII_CASE_MASK; BYTES]);
                        const MIN: Simd<c_char, BYTES> = Simd::from_array([MIN_CHAR; BYTES]);
                        const MAX: Simd<c_char, BYTES> = Simd::from_array([MAX_CHAR; BYTES]);
            
                        let (lhs, simd, rhs) = chars.as_simd_mut::<BYTES>();
                        for x in simd.iter_mut() {
                            let are_lower = x.simd_le(MAX) & x.simd_ge(MIN);
                            *x ^= CASE_MASK & are_lower.to_int().cast::<c_char>()
                        }
            
                        return Ok((lhs, rhs))
                    }

                    return Err(chars)
                }
            };
        }

        if SIMD_512 {
            uppercase_inner! { uppercase_512, 512 }
            match uppercase_512(self.as_mut_c_chars()) {
                Ok((lhs, rhs)) => {},
                Err(e) => todo!()
            }
        }
    }
}

#[cfg(not(feature = "simd"))]
impl CSubStr {
    #[inline]
    pub fn switch_case (&mut self) {
        self.as_mut_c_chars().iter_mut().for_each(|c| *c ^= ASCII_CASE_MASK)
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