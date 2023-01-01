use core::{marker::PhantomData, ptr::NonNull, ffi::c_char, fmt::{Debug, Display}, ops::{Deref, DerefMut}, hash::Hash, borrow::Borrow};
use crate::{CStr, CSubStr, strlen};

#[repr(transparent)]
pub struct CMutStr<'a> {
    pub(crate) inner: NonNull<c_char>,
    pub(crate) _phtm: PhantomData<&'a mut [c_char]>
}

impl<'a> CMutStr<'a> {
    #[inline]
    pub fn from_rust (c: &'a mut core::ffi::CStr) -> Self {
        return Self {
            inner: unsafe { NonNull::new_unchecked(c.as_ptr().cast_mut()) },
            _phtm: PhantomData
        }
    }

    #[inline]
    pub unsafe fn from_chars_unchecked (chars: &'a mut [c_char]) -> Self {
        return Self {
            inner: unsafe { NonNull::new_unchecked(chars.as_ptr().cast_mut()) },
            _phtm: PhantomData
        }
    }
    
    #[inline]
    pub fn len (&self) -> usize {
        return unsafe { strlen(self.as_ptr()) }
    }

    #[inline]
    pub fn into_cstr (self) -> CStr<'a> {
        return CStr { inner: self.inner, _phtm: PhantomData }
    }

    #[inline]
    pub fn as_ptr (&self) -> *const c_char {
        self.inner.as_ptr()
    }
    
    #[inline]
    pub fn as_mut_ptr (&mut self) -> *mut c_char {
        self.inner.as_ptr()
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
    pub unsafe fn as_bytes_with_nul (&self) -> &[u8] {
        return unsafe { &*(self.as_c_chars_with_nul() as *const [c_char] as *const [_]) }
    }
    
    #[inline]
    pub unsafe fn as_mut_bytes_with_nul (&mut self) -> &mut [u8] {
        return unsafe { &mut *(self.as_mut_c_chars_with_nul() as *mut [c_char] as *mut [_]) }
    }
    
    #[inline]
    pub fn as_c_chars (&self) -> &[c_char] {
        return unsafe {
            core::slice::from_raw_parts(self.as_ptr(), self.len())
        };
    }

    #[inline]
    pub fn as_mut_c_chars (&mut self) -> &mut [c_char] {
        return unsafe {
            core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len())
        };
    }

    #[inline]
    pub fn as_c_chars_with_nul (&self) -> &[c_char] {
        return unsafe {
            core::slice::from_raw_parts(self.as_ptr(), self.len() + core::mem::size_of::<c_char>())
        };
    }

    #[inline]
    pub unsafe fn as_mut_c_chars_with_nul (&mut self) -> &mut [c_char] {
        return unsafe {
            core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len() + core::mem::size_of::<c_char>())
        };
    }
}

impl Deref for CMutStr<'_> {
    type Target = CSubStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { CSubStr::from_chars_unchecked(self.as_c_chars()) }
    }
}

impl DerefMut for CMutStr<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { CSubStr::from_mut_chars_unchecked(self.as_mut_c_chars()) }
    }
}

impl Borrow<CSubStr> for CMutStr<'_> {
    #[inline]
    fn borrow(&self) -> &CSubStr {
        self.deref()
    }
}

impl<'a> Borrow<CStr<'a>> for CMutStr<'a> {
    #[inline]
    fn borrow(&self) -> &CStr<'a> {
        return unsafe { &*(self as *const _ as *const _) }
    }
}

impl Hash for CMutStr<'_> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_c_chars().hash(state)
    }
}

impl<T: Borrow<CSubStr>> PartialEq<T> for CMutStr<'_> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.deref() == other.borrow()
    }
}

impl<T: Borrow<CSubStr>> PartialOrd<T> for CMutStr<'_> {
    #[inline]
    fn partial_cmp(&self, other: &T) -> Option<core::cmp::Ordering> {
        self.deref().partial_cmp(other.borrow())
    }
}

impl Ord for CMutStr<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.deref().cmp(other.deref())
    }
}

impl Debug for CMutStr<'_> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "\"{}\"", self.as_bytes().escape_ascii())
    }
}

impl Display for CMutStr<'_> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.as_bytes().escape_ascii(), f)
    }
}

impl Eq for CMutStr<'_> {}
unsafe impl Sync for CMutStr<'_> {}