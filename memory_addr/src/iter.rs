//! Page iterators with fixed or runtime-selected page sizes.
//!
//! Both interfaces delegate validation and iteration to the bounded range
//! iterator.

use crate::{AddrRange, AddrRangeIter, AddrRangeIterator, MemoryAddr};

/// A page-by-page iterator.
///
/// This type is retained for compatibility with existing code. Prefer
/// [`AddrRangeIter`] for new code, typically constructed through
/// [`AddrRangeBounds::iter`](crate::AddrRangeBounds::iter).
///
/// The page size is specified by the generic parameter `PAGE_SIZE`, which must
/// be a nonzero power of two.
///
/// The address type is specified by the type parameter `A`.
/// Validation and iteration use [`AddrRangeIter`], which also stores the page
/// size.
///
/// # Examples
///
/// ```
/// use memory_addr::PageIter;
///
/// let mut iter = PageIter::<0x1000, usize>::new(0x1000, 0x3000).unwrap();
/// assert_eq!(iter.next(), Some(0x1000));
/// assert_eq!(iter.next(), Some(0x2000));
/// assert_eq!(iter.next(), None);
///
/// assert!(PageIter::<0x1000, usize>::new(0x1000, 0x3001).is_none());
/// assert!(PageIter::<0x1000, usize>::new(0x3000, 0x1000).is_none());
/// ```
pub struct PageIter<const PAGE_SIZE: usize, A>
where
    A: MemoryAddr,
{
    /// The underlying bounded page iterator.
    ///
    /// Its page size is initialized from `PAGE_SIZE`.
    inner: AddrRangeIter<A>,
}

/// Construction with a compile-time page size.
///
/// The underlying iterator validates the endpoints and page size.
const impl<A, const PAGE_SIZE: usize> PageIter<PAGE_SIZE, A>
where
    A: [const] MemoryAddr,
{
    /// Creates a new [`PageIter`].
    ///
    /// Returns `None` if `start > end`, `PAGE_SIZE` is not a nonzero power of
    /// two, or either endpoint is not page-aligned. Equal aligned endpoints
    /// produce an empty iterator.
    #[inline]
    pub fn new(start: A, end: A) -> Option<Self> {
        match AddrRangeIter::new(AddrRange { start, end }, PAGE_SIZE) {
            Some(inner) => Some(Self { inner }),
            None => None,
        }
    }
}

/// Iteration with a compile-time page size.
///
/// Page advancement and exhaustion are delegated to the underlying iterator.
const impl<A, const PAGE_SIZE: usize> Iterator for PageIter<PAGE_SIZE, A>
where
    A: [const] MemoryAddr,
{
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// A page-by-page iterator with dynamic page size.
///
/// This type is retained for compatibility with existing code. Prefer
/// [`AddrRangeIter`] for new code, typically constructed through
/// [`AddrRangeBounds::iter`](crate::AddrRangeBounds::iter).
///
/// The address type is specified by the type parameter `A`.
/// Validation and iteration use [`AddrRangeIter`].
///
/// # Examples
///
/// ```
/// use memory_addr::DynPageIter;
///
/// let mut iter = DynPageIter::<usize>::new(0x1000, 0x3000, 0x1000).unwrap();
/// assert_eq!(iter.next(), Some(0x1000));
/// assert_eq!(iter.next(), Some(0x2000));
/// assert_eq!(iter.next(), None);
///
/// assert!(DynPageIter::<usize>::new(0x1000, 0x3001, 0x1000).is_none());
/// assert!(DynPageIter::<usize>::new(0x1000, 0x3000, 0x1001).is_none());
/// assert!(DynPageIter::<usize>::new(0x3000, 0x1000, 0x1000).is_none());
/// ```
pub struct DynPageIter<A>
where
    A: MemoryAddr,
{
    /// The underlying bounded page iterator.
    ///
    /// It stores the runtime-selected page size and the remaining endpoints.
    inner: AddrRangeIter<A>,
}

/// Construction with a runtime-selected page size.
///
/// The underlying iterator validates the endpoints and page size.
const impl<A> DynPageIter<A>
where
    A: [const] MemoryAddr,
{
    /// Creates a new [`DynPageIter`].
    ///
    /// Returns `None` if `start > end`, `page_size` is not a nonzero power of
    /// two, or either endpoint is not page-aligned. Equal aligned endpoints
    /// produce an empty iterator.
    #[inline]
    pub fn new(start: A, end: A, page_size: usize) -> Option<Self> {
        match AddrRangeIter::new(AddrRange { start, end }, page_size) {
            Some(inner) => Some(Self { inner }),
            None => None,
        }
    }
}

/// Iteration with a runtime-selected page size.
///
/// Page advancement and exhaustion are delegated to the underlying iterator.
const impl<A> Iterator for DynPageIter<A>
where
    A: [const] MemoryAddr,
{
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
