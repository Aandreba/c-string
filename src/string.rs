use core::{ffi::c_char, ptr::{NonNull}, marker::PhantomData, ops::{Deref, DerefMut}, hash::Hash, borrow::{Borrow, BorrowMut}, mem::ManuallyDrop};
use docfg::docfg;
use memchr::memchr;
use crate::{CStr, NUL_CHAR_PTR, error::NulError, CMutStr, CSubStr, strlen};

use core::alloc::Layout;
#[cfg(feature = "alloc_api")]
use {core::alloc::*, alloc::alloc::Global};

macro_rules! impl_all {
    (unsafe trait $trait:path { $($t:tt)* }) => {
        #[cfg(feature = "alloc_api")]
        unsafe impl<A: Allocator> $trait for CString<A> {
            $($t)*
        }

        #[cfg(not(feature = "alloc_api"))]
        unsafe impl $trait for CString {
            $($t)*
        }
    };
    
    (trait $trait:path { $($t:tt)* }) => {
        #[cfg(feature = "alloc_api")]
        impl<A: Allocator> $trait for CString<A> {
            $($t)*
        }

        #[cfg(not(feature = "alloc_api"))]
        impl $trait for CString {
            $($t)*
        }
    };
    
    ($($t:tt)*) => {
        #[cfg(feature = "alloc_api")]
        impl<A: Allocator> CString<A> {
            $($t)*
        }

        #[cfg(not(feature = "alloc_api"))]
        impl CString {
            $($t)*
        }
    };
}

pub struct CString<#[cfg(feature = "alloc_api")] A: Allocator = Global> {
    inner: NonNull<c_char>,
    capacity: usize, // nul excluded
    #[cfg(feature = "alloc_api")]
    alloc: A
}

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc_api")] {
        impl CString {
            #[inline]
            pub const fn new () -> Self {
                return Self::new_in(Global)
            }

            #[inline]
            pub fn from_substr (sub: &CSubStr) -> Self {
                return Self::from_substr_in(sub, Global)
            }
            
            #[inline]
            pub fn with_capacity (capacity: usize) -> Self {
                return Self::try_with_capacity_in(capacity, Global).unwrap()
            }
        }
    } else {
        impl CString {
            pub const fn new () -> Self {
                return Self {
                    inner: unsafe { NonNull::new_unchecked(NUL_CHAR_PTR.cast_mut()) },
                    capacity: 0
                }
            }

            #[inline]
            pub fn from_substr (sub: &CSubStr) -> Self {
                let mut this = Self::with_capacity(sub.len());
                this.append(sub);
                return this
            }
        
            pub fn with_capacity (capacity: usize) -> Self {
                let nul_capacity = capacity.checked_add(1).unwrap();
                let layout = Layout::array::<c_char>(nul_capacity).unwrap();
        
                let inner = NonNull::new(unsafe { alloc::alloc::alloc(layout) }).unwrap().cast::<c_char>();
                unsafe {
                    inner.as_ptr().write(0);
                    return Self { inner, capacity }
                };
            }
        }
    }
}

#[docfg(feature = "alloc_api")]
impl<A: Allocator> CString<A> {
    #[inline]
    pub const fn new_in (alloc: A) -> Self {
        return Self {
            inner: unsafe { NonNull::new_unchecked(NUL_CHAR_PTR.cast_mut()) },
            capacity: 0,
            alloc
        }
    }

    #[inline]
    pub fn from_substr_in (sub: &CSubStr, alloc: A) -> Self {
        let mut this = Self::with_capacity_in(sub.len(), alloc);
        this.append(sub);
        return this
    }
    
    /// # Panics
    /// This method panics in whatecer case the `try_with_capacity_in` method panics or fails
    #[inline]
    pub fn with_capacity_in (capacity: usize, alloc: A) -> Self {
        return Self::try_with_capacity_in(capacity, alloc).unwrap()
    }

    pub fn try_with_capacity_in (capacity: usize, alloc: A) -> Result<Self, AllocError> {
        if let Some(nul_capacity) = capacity.checked_add(1) {
            let layout = Layout::array::<c_char>(nul_capacity).map_err(|_| AllocError)?;
            let inner = alloc.allocate(layout)?.cast::<c_char>();
            unsafe { inner.as_ptr().write(0) };
            return Ok(Self { inner, capacity, alloc })
        }
        return Err(AllocError)
    }
}

impl CString {    
    #[inline]
    pub fn from_string (string: String) -> Result<Self, NulError> {
        return Self::from_bytes(string.into_bytes())
    }

    #[inline]
    pub fn from_string_with_nul (string: String) -> Result<Self, NulError> {
        return Self::from_bytes_with_nul(string.into_bytes())
    }
}

impl_all! {
    #[inline]
    pub fn push_char (&mut self, c: char) {
        assert!(c.is_ascii());
        return self.push(c as c_char);
    }

    #[docfg(feature = "nightly")]
    #[inline]
    pub fn push_nonzero (&mut self, c: core::ffi::NonZero_c_char) {
        return unsafe { self.push_unchecked(c.get()) }
    }
    
    #[inline]
    pub fn push (&mut self, c: c_char) {
        assert_ne!(c, 0);
        return unsafe { self.push_unchecked(c) }
    }

    #[inline]
    pub fn append (&mut self, chars: &CSubStr) {
        self.reserve(chars.len());

        unsafe {
            let ptr = self.as_mut_ptr().add(self.len());
            core::ptr::copy_nonoverlapping(
                chars.as_ptr(),
                ptr,
                chars.len()
            );

            ptr.add(chars.len()).add(1).write(0);
        }
    }

    #[inline]
    pub fn clear (&mut self) {
        unsafe { self.as_mut_ptr().write(0) };
    }

    #[inline]
    pub fn leak<'a> (self) -> &'a mut CSubStr {
        let mut this = ManuallyDrop::new(self);
        unsafe {
            let slice = core::slice::from_raw_parts_mut(this.as_mut_ptr(), this.len());
            return CSubStr::from_mut_chars_unchecked(slice)
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc_api")] {
        impl<A: Allocator> CString<A> {    
            #[inline]
            pub fn from_chars (chars: Vec<c_char, A>) -> Result<Self, NulError<A>> {
                let (ptr, len, cap, alloc) = Vec::into_raw_parts_with_alloc(chars);
                return unsafe {
                    Self::from_bytes(Vec::from_raw_parts_in(ptr.cast(), len, cap, alloc))
                };
            }

            #[inline]
            pub fn from_bytes (mut chars: Vec<u8, A>) -> Result<Self, NulError<A>> {
                chars.push(0);
                return Self::from_bytes_with_nul(chars)
            }

            #[inline]
            pub fn from_chars_with_nul (chars: Vec<c_char, A>) -> Result<Self, NulError<A>> {
                let (ptr, len, cap, alloc) = Vec::into_raw_parts_with_alloc(chars);
                return unsafe {
                    Self::from_bytes_with_nul(Vec::from_raw_parts_in(ptr.cast(), len, cap, alloc))
                };
            }
            
            pub fn from_bytes_with_nul (chars: Vec<u8, A>) -> Result<Self, NulError<A>> {
                match (memchr(0, &chars), chars.len()) {
                    (Some(pos), len) if pos + 1 == len => {},
                    (pos, _) => return Err(NulError::new(pos, chars))
                }

                let (ptr, _, cap, alloc) = Vec::into_raw_parts_with_alloc(chars);
                return Ok(Self {
                    inner: unsafe { NonNull::new_unchecked(ptr.cast()) },
                    capacity: cap,
                    alloc
                })
            }

            #[inline]
            pub unsafe fn push_unchecked (&mut self, c: c_char) {
                self.try_push_unchecked(c).unwrap()
            }
        
            #[inline]
            pub fn reserve (&mut self, cap: usize) {
                self.try_reserve(cap).unwrap()
            }
        
            #[inline]
            pub fn reserve_exact (&mut self, cap: usize) {
                self.try_reserve_exact(cap).unwrap()
            }
        }
    } else {
        impl CString {
            #[inline]
            pub fn from_chars (chars: Vec<c_char>) -> Result<Self, NulError> {
                let (ptr, len, cap) = crate::vec_into_raw_parts(chars);
                return unsafe {
                    Self::from_bytes(Vec::from_raw_parts(ptr.cast(), len, cap))
                };
            }

            #[inline]
            pub fn from_bytes (mut chars: Vec<u8>) -> Result<Self, NulError> {
                chars.push(0);
                return Self::from_bytes_with_nul(chars)
            }

            #[inline]
            pub fn from_chars_with_nul (chars: Vec<c_char>) -> Result<Self, NulError> {
                let (ptr, len, cap) = crate::vec_into_raw_parts(chars);
                return unsafe {
                    Self::from_bytes_with_nul(Vec::from_raw_parts(ptr.cast(), len, cap))
                };
            }

            pub fn from_bytes_with_nul (chars: Vec<u8>) -> Result<Self, NulError> {
                match (memchr(0, &chars), chars.len()) {
                    (Some(pos), len) if pos + 1 == len => {},
                    (pos, _) => return Err(NulError::new(pos, chars))
                }

                let (ptr, _, cap) = crate::vec_into_raw_parts(chars);
                return Ok(Self {
                    inner: unsafe { NonNull::new_unchecked(ptr.cast()) },
                    capacity: cap
                })
            }

            #[inline]
            pub unsafe fn push_unchecked (&mut self, c: c_char) {
                let len = self.len();
                self.reserve_inner(len, 1);

                let ptr = self.inner.as_ptr().add(len);
                ptr.add(1).write(0);
                ptr.write(c);
            }
        
            #[inline]
            pub fn reserve (&mut self, cap: usize) {
                self.reserve_inner(self.len(), cap)
            }

            #[inline]
            pub fn reserve_exact (&mut self, cap: usize) {
                self.reserve_exact_inner(self.len(), cap)
            }
        
            fn reserve_inner (&mut self, len: usize, cap: usize) {
                let delta = len.checked_add(cap).unwrap();
                return match delta.overflowing_sub(self.capacity()) {
                    // capacity >= len + extra
                    (_, true) | (0, _) => {},
                    // capacity < len + extra
                    (cap, false) => self.reserve_exact_inner(len, cap)
                }
            }
            
            fn reserve_exact_inner (&mut self, len: usize, cap: usize) {
                unsafe {
                    let prev_nul_capacity = self.capacity()
                        .checked_add(1)
                        .unwrap();
        
                    let new_nul_capacity = prev_nul_capacity
                        .checked_add(cap)
                        .unwrap();
        
                    let prev_layout = Layout::array::<c_char>(prev_nul_capacity).unwrap();
                    let new_layout = Layout::array::<c_char>(new_nul_capacity).unwrap();
        
                    let new_ptr = NonNull::new(alloc::alloc::alloc(new_layout)).unwrap().cast::<c_char>();
                    core::ptr::copy_nonoverlapping(self.as_ptr(), new_ptr.as_ptr(), len + 1);
                    
                    let prev_ptr = core::mem::replace(&mut self.inner, new_ptr);
                    if !core::ptr::eq(prev_ptr.as_ptr(), NUL_CHAR_PTR) {
                        alloc::alloc::dealloc(prev_ptr.as_ptr().cast(), prev_layout);
                    }
                }
            }
        }
    }
}

#[docfg(feature = "alloc_api")]
impl<A: Allocator> CString<A> {
    #[inline]
    pub unsafe fn try_push_unchecked (&mut self, c: c_char) -> Result<(), AllocError> {
        let len = self.len();
        self.try_reserve_inner(len, 1)?;

        let ptr = self.inner.as_ptr().add(len);
        ptr.add(1).write(0);
        ptr.write(c);
        return Ok(())
    }

    #[inline]
    pub fn try_reserve (&mut self, cap: usize) -> Result<(), AllocError> {
        self.try_reserve_inner(self.len(), cap)
    }

    #[inline]
    pub fn try_reserve_exact (&mut self, cap: usize) -> Result<(), AllocError> {
        self.try_reserve_exact_inner(self.len(), cap)
    }

    fn try_reserve_inner (&mut self, len: usize, cap: usize) -> Result<(), AllocError> {
        let delta = len.checked_add(cap).ok_or(AllocError)?;
        return match delta.overflowing_sub(self.capacity()) {
            // capacity >= len + extra
            (_, true) | (0, _) => Ok(()),
            // capacity < len + extra
            (cap, false) => self.try_reserve_exact_inner(len, cap)
        }
    }

    fn try_reserve_exact_inner (&mut self, len: usize, cap: usize) -> Result<(), AllocError> {
        unsafe {
            let prev_nul_capacity = self.capacity()
                .checked_add(1)
                .ok_or(AllocError)?;

            let new_nul_capacity = prev_nul_capacity
                .checked_add(cap)
                .ok_or(AllocError)?;

            let prev_layout = Layout::array::<c_char>(prev_nul_capacity).map_err(|_| AllocError)?;
            let new_layout = Layout::array::<c_char>(new_nul_capacity).map_err(|_| AllocError)?;

            let new_ptr = self.alloc.allocate(new_layout)?.cast::<c_char>();
            core::ptr::copy_nonoverlapping(self.as_ptr(), new_ptr.as_ptr(), len + 1);
            
            let prev_ptr = core::mem::replace(&mut self.inner, new_ptr);
            if !core::ptr::eq(prev_ptr.as_ptr(), NUL_CHAR_PTR) {
                self.alloc.deallocate(prev_ptr.cast(), prev_layout);
            }
            return Ok(())
        }
    }
}

impl_all! {
    #[inline]
    pub fn as_cstr (&self) -> CStr<'_> {
        return CStr { inner: self.inner, _phtm: PhantomData }
    }

    #[inline]
    pub fn as_mut_cstr (&mut self) -> CMutStr<'_> {
        return CMutStr { inner: self.inner, _phtm: PhantomData }
    }

    #[inline]
    pub fn len (&self) -> usize {
        return unsafe { strlen(self.as_ptr()) }
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

    #[inline]
    pub fn capacity (&self) -> usize {
        return self.capacity
    }
}

impl_all! {
    trait Deref {
        type Target = CSubStr;

        #[inline]
        fn deref(&self) -> &Self::Target {
            unsafe { CSubStr::from_chars_unchecked(self.as_c_chars()) }
        }
    }
}

impl_all! {
    trait DerefMut {
        #[inline]
        fn deref_mut (&mut self) -> &mut Self::Target {
            unsafe { CSubStr::from_mut_chars_unchecked(self.as_mut_c_chars()) }
        }
    }
}

impl_all! {
    trait Borrow<CSubStr> {
        #[inline]
        fn borrow(&self) -> &CSubStr {
            self.deref()
        }
    }
}

impl_all! {
    trait BorrowMut<CSubStr> {
        #[inline]
        fn borrow_mut(&mut self) -> &mut CSubStr {
            self.deref_mut()
        }
    }
}

impl_all! {
    trait Hash {
        #[inline]
        fn hash<H: core::hash::Hasher> (&self, state: &mut H) {
            self.as_c_chars().hash(state);
        }
    }
}

#[docfg(feature = "nightly")]
impl_all! {
    trait Extend<core::ffi::NonZero_c_char> {
        #[inline]
        fn extend<T: IntoIterator<Item = core::ffi::NonZero_c_char>>(&mut self, iter: T) {
            let iter = iter.into_iter();
            self.reserve(match iter.size_hint() {
                (_, Some(len)) => len,
                (len, _) => len
            });

            for c in iter {
                self.push_nonzero(c)
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc_api")] {
        impl<T: ?Sized + Borrow<CSubStr>, A: Allocator> PartialEq<T> for CString<A> {
            #[inline]
            fn eq(&self, other: &T) -> bool {
                self.deref() == other.borrow()
            }
        }
        
        impl<T: ?Sized + Borrow<CSubStr>, A: Allocator> PartialOrd<T> for CString<A> {
            #[inline]
            fn partial_cmp(&self, other: &T) -> Option<core::cmp::Ordering> {
                self.as_c_chars().partial_cmp(other.borrow().as_c_chars())
            }
        }
    } else {
        impl<T: ?Sized + Borrow<CSubStr>> PartialEq<T> for CString {
            #[inline]
            fn eq(&self, other: &T) -> bool {
                self.deref() == other.borrow()
            }
        }
        
        impl<T: ?Sized + Borrow<CSubStr>> PartialOrd<T> for CString {
            #[inline]
            fn partial_cmp(&self, other: &T) -> Option<core::cmp::Ordering> {
                self.as_c_chars().partial_cmp(other.borrow().as_c_chars())
            }
        }
    }
}

impl_all! {
    trait Ord {
        #[inline]
        fn cmp(&self, other: &Self) -> core::cmp::Ordering {
            self.as_c_chars().cmp(other.as_c_chars())
        }
    }
}

impl_all! {
    trait core::fmt::Debug {
        #[inline]
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "\"{}\"", self.as_bytes().escape_ascii())
        }
    }
}

impl_all! {
    trait core::fmt::Display {
        #[inline]
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            core::fmt::Display::fmt(&self.as_bytes().escape_ascii(), f)
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc_api")] {        
        impl<A: Allocator> Drop for CString<A> {
            #[inline]
            fn drop(&mut self) {
                unsafe {
                    let layout = Layout::array::<c_char>(self.capacity() + 1).unwrap();
                    self.alloc.deallocate(self.inner.cast(), layout);
                }
            }
        }

        unsafe impl<A: Send + Allocator> Send for CString<A> {}
        unsafe impl<A: Sync + Allocator> Sync for CString<A> {}
    } else {
        impl Drop for CString {
            #[inline]
            fn drop(&mut self) {
                unsafe {
                    let layout = Layout::array::<c_char>(self.capacity() + 1).unwrap();
                    alloc::alloc::dealloc(self.as_mut_ptr().cast(), layout);
                }
            }
        }

        unsafe impl Send for CString {}
        unsafe impl Sync for CString {}
    }
}

impl_all! {
    trait Eq {}
}