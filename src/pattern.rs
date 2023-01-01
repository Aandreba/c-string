use core::{str::pattern::SearchStep, ffi::c_char};
use crate::{CStr, CSubStr};

pub trait CPattern<'a>: Sized {
    type Searcher: CSearcher<'a>;

    fn into_searcher (self, haystack: &'a CSubStr) -> Self::Searcher;
}

impl<'a, 'b> CPattern<'a> for &'b CSubStr {
    type Searcher = CSubStrSearcher<'a, 'b>;

    #[inline]
    fn into_searcher(self, haystack: &'a CSubStr) -> Self::Searcher {
        return CSubStrSearcher {
            haystack,
            needle: self,
            offset: 0,
        }
    }
}

impl<'a> CPattern<'a> for c_char {
    type Searcher = CCharSearcher<'a>;

    #[inline]
    fn into_searcher(self, haystack: &'a CSubStr) -> Self::Searcher {
        return CCharSearcher {
            haystack,
            needle: self,
            offset: 0,
        }
    }
}

/// A searcher for a C string pattern.
///
/// This trait provides methods for searching for non-overlapping
/// matches of a pattern starting from the front (left) of a C string.
///
/// It will be implemented by associated `CSearcher`
/// types of the [`CPattern`] trait.
///
/// The trait is marked unsafe because the indices returned by the
/// [`next()`][CSearcher::next] methods are required to lie on valid utf8
/// boundaries in the haystack. This enables consumers of this trait to
/// slice the haystack without additional runtime checks.
pub unsafe trait CSearcher<'a> {
    /// Getter for the underlying C string to be searched in
    ///
    /// Will always return the same [`CStr`].
    fn haystack(&self) -> CStr<'a>;

    /// Performs the next search step starting from the front.
    ///
    /// - Returns [`Match(a, b)`][SearchStep::Match] if `haystack[a..b]` matches
    ///   the pattern.
    /// - Returns [`Reject(a, b)`][SearchStep::Reject] if `haystack[a..b]` can
    ///   not match the pattern, even partially.
    /// - Returns [`Done`][SearchStep::Done] if every byte of the haystack has
    ///   been visited.
    ///
    /// The stream of [`Match`][SearchStep::Match] and
    /// [`Reject`][SearchStep::Reject] values up to a [`Done`][SearchStep::Done]
    /// will contain index ranges that are adjacent, non-overlapping,
    /// covering the whole haystack, and laying on utf8 boundaries.
    ///
    /// A [`Match`][SearchStep::Match] result needs to contain the whole matched
    /// pattern, however [`Reject`][SearchStep::Reject] results may be split up
    /// into arbitrary many adjacent fragments. Both ranges may have zero length.
    ///
    /// As an example, the pattern `"aaa"` and the haystack `"cbaaaaab"`
    /// might produce the stream
    /// `[Reject(0, 1), Reject(1, 2), Match(2, 5), Reject(5, 8)]`
    fn next(&mut self) -> SearchStep;

    #[inline]
    fn next_match(&mut self) -> Option<(usize, usize)> {
        loop {
            match self.next() {
                SearchStep::Match(a, b) => return Some((a, b)),
                SearchStep::Done => return None,
                _ => continue,
            }
        }
    }

    #[inline]
    fn next_reject(&mut self) -> Option<(usize, usize)> {
        loop {
            match self.next() {
                SearchStep::Reject(a, b) => return Some((a, b)),
                SearchStep::Done => return None,
                _ => continue,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CSubStrSearcher<'a, 'b> {
    haystack: &'a CSubStr, // search in here
    needle: &'b CSubStr, // search for this
    offset: usize
}

unsafe impl<'a, 'b> CSearcher<'a> for CSubStrSearcher<'a, 'b> {
    #[inline]
    fn haystack(&self) -> CStr<'a> {
        unsafe { CStr::from_chars_unchecked(self.haystack) }
    }

    #[inline]
    fn next(&mut self) -> SearchStep {
        let start = self.offset;
        let end = self.offset + self.needle.len();
        if end >= self.haystack.len() { return SearchStep::Done }

        let region = &self.haystack[start..end];
        if region == self.needle {
            self.offset = end;
            return SearchStep::Match(start, end)
        }

        self.offset += 1;
        return SearchStep::Reject(start, end)
    }
}

#[derive(Debug, Clone)]
pub struct CCharSearcher<'a> {
    haystack: &'a CSubStr, // search in here
    needle: c_char, // search for this
    offset: usize
}

unsafe impl<'a> CSearcher<'a> for CCharSearcher<'a> {
    #[inline]
    fn haystack(&self) -> CStr<'a> {
        unsafe { CStr::from_chars_unchecked(self.haystack) }
    }

    #[inline]
    fn next(&mut self) -> SearchStep {
        if self.offset >= self.haystack.len() {
            return SearchStep::Done
        }

        let next = self.offset + 1;
        let result = match self.haystack[self.offset] == self.needle {
            true => SearchStep::Match(self.offset, next),
            false => SearchStep::Reject(self.offset, next)
        };

        self.offset =  next;
        return result
    }
}