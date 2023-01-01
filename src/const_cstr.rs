use core::{ptr::NonNull, ffi::c_char, marker::PhantomData, fmt::{Debug, Display}, ops::{Deref}, hash::Hash, borrow::Borrow};
use crate::{CSubStr, strlen};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct CStr<'a> {
    pub(crate) inner: NonNull<c_char>,
    pub(crate) _phtm: PhantomData<&'a [c_char]>
}

impl<'a> CStr<'a> {
    #[inline]
    pub const fn from_rust (c: &'a core::ffi::CStr) -> Self {
        return Self {
            inner: unsafe { NonNull::new_unchecked(c.as_ptr().cast_mut()) },
            _phtm: PhantomData
        }
    }

    #[inline]
    pub const unsafe fn from_chars_unchecked (chars: &'a [c_char]) -> Self {
        return Self {
            inner: unsafe { NonNull::new_unchecked(chars.as_ptr().cast_mut()) },
            _phtm: PhantomData
        }
    }

    #[inline]
    pub fn to_ptr (self) -> *const c_char {
        self.inner.as_ptr()
    }

    #[inline]
    pub fn len (self) -> usize {
        return unsafe { strlen(self.as_ptr()) }
    }

    #[inline]
    pub fn to_bytes (self) -> &'a [u8] {
        return unsafe { &*(self.to_c_chars() as *const [c_char] as *const [_]) }
    }

    #[inline]
    pub fn to_bytes_with_nul (self) -> &'a [u8] {
        return unsafe { &*(self.to_c_chars_with_nul() as *const [c_char] as *const [_]) }
    }
    
    #[inline]
    pub fn to_c_chars (self) -> &'a [c_char] {
        return unsafe {
            core::slice::from_raw_parts(self.to_ptr(), self.len())
        };
    }

    #[inline]
    pub fn to_c_chars_with_nul (self) -> &'a [c_char] {
        return unsafe {
            core::slice::from_raw_parts(self.to_ptr(), self.len() + core::mem::size_of::<c_char>())
        };
    }
}

impl Deref for CStr<'_> {
    type Target = CSubStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { CSubStr::from_chars_unchecked(self.to_c_chars()) }
    }
}

impl Borrow<CSubStr> for CStr<'_> {
    #[inline]
    fn borrow(&self) -> &CSubStr {
        self.deref()
    }
}

impl Hash for CStr<'_> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_c_chars().hash(state)
    }
}

impl<T: Borrow<CSubStr>> PartialEq<T> for CStr<'_> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.deref() == other.borrow()
    }
}

impl<T: Borrow<CSubStr>> PartialOrd<T> for CStr<'_> {
    #[inline]
    fn partial_cmp(&self, other: &T) -> Option<core::cmp::Ordering> {
        self.deref().partial_cmp(other.borrow())
    }
}

impl Ord for CStr<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.deref().cmp(other.deref())
    }
}

impl Debug for CStr<'_> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "\"{}\"", self.as_bytes().escape_ascii())
    }
}

impl Display for CStr<'_> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.as_bytes().escape_ascii(), f)
    }
}

impl Eq for CStr<'_> {}
unsafe impl Send for CStr<'_> {}
unsafe impl Sync for CStr<'_> {}