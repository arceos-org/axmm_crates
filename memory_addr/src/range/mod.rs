//! Address range types and common queries.
//!
//! Ranges have an inclusive start and either a representable exclusive end or
//! an exclusive end one past the maximum address.

use crate::MemoryAddr;

mod conv;
mod general;
// This private module mirrors the `AddrRange` type name.
#[allow(clippy::module_inception)]
mod range;
mod range_from;
mod sealed;

pub use conv::IntoAddrRange;
pub use general::{GeneralAddrRange, GeneralAddrRangeIter};
pub use range::{AddrRange, AddrRangeIter, PhysAddrRange, VirtAddrRange};
pub use range_from::{AddrRangeFrom, AddrRangeFromIter, PhysAddrRangeFrom, VirtAddrRangeFrom};

/// A page iterator constructed from an address range.
///
/// Each item is the start address of a complete page in the range. Iteration
/// proceeds in ascending address order and stops at the exclusive end,
/// including when that end is one past the maximum address.
///
/// This trait is sealed. Only the iterator types provided by this crate can
/// implement it.
pub const trait AddrRangeIterator<A: [const] MemoryAddr>:
    [const] Iterator<Item = A> + Sized + sealed::RangeIterator
{
    /// The range type accepted by this iterator.
    ///
    /// The range determines the representation of the exclusive end.
    type AddrRange: [const] AddrRangeBounds<A>;

    /// Creates an iterator over the pages in the given range.
    ///
    /// The iterator owns its state and does not borrow the original range.
    ///
    /// Returns `None` if the range is invalid, the page size is not a non-zero
    /// power of two, or a representable endpoint is not page-aligned.
    fn new(range: Self::AddrRange, page_size: usize) -> Option<Self>;
}

/// A trait for address range types.
///
/// Implementations expose an inclusive start and an exclusive end.
/// An unrepresentable end denotes the address immediately past `usize::MAX`.
/// The end and size must describe the same range so that queries can compare
/// different implementations consistently.
///
/// Empty ranges contain no addresses, are contained in every valid range,
/// and never overlap any range. These rules also apply between two empty
/// ranges, regardless of their endpoints.
///
/// Some implementations allow invalid ranges with `start > end`. Unless
/// otherwise documented, queries on invalid ranges have no guaranteed result.
/// [`is_valid`](Self::is_valid) detects them and [`iter`](Self::iter) rejects
/// them.
///
/// This trait is a mimic of the standard library's `RangeBounds` trait, adapted
/// for address ranges.
///
/// This trait is sealed and implemented by [`AddrRange`], [`AddrRangeFrom`],
/// and [`GeneralAddrRange`]. These types still accept external address types
/// implementing [`MemoryAddr`].
///
/// # Examples
///
/// ```
/// use memory_addr::{AddrRange, AddrRangeBounds, AddrRangeFrom};
///
/// let bounded = AddrRange::new(10usize, 20);
/// let tail = AddrRangeFrom::new(15usize);
/// assert_eq!(bounded.end(), 20);
/// assert_eq!(bounded.size(), 10);
/// assert!(bounded.contains(10));
/// assert!(!bounded.contains(20));
/// assert!(bounded.overlaps(tail));
/// assert!(tail.contains_range(AddrRange::new(15, 20)));
/// assert!(AddrRange::new(15, 20).contained_in(tail));
/// ```
pub const trait AddrRangeBounds<A: [const] MemoryAddr>: Copy + sealed::Range {
    /// Returns the inclusive start address of the range.
    ///
    /// The start is representable for both bounded and open-ended ranges.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{va, AddrRangeBounds, VirtAddrRange, VirtAddrRangeFrom};
    ///
    /// let bounded = VirtAddrRange::new(va!(0x1000), va!(0x2000));
    /// let tail = VirtAddrRangeFrom::new(va!(0x1000));
    /// assert_eq!(bounded.start(), va!(0x1000));
    /// assert_eq!(tail.start(), va!(0x1000));
    /// ```
    fn start(&self) -> A;

    /// Returns the exclusive end address when it is representable.
    ///
    /// Returns `None` when the exclusive end is one past `usize::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{va, AddrRangeBounds, VirtAddrRange, VirtAddrRangeFrom};
    ///
    /// let bounded = VirtAddrRange::new(va!(0), va!(usize::MAX));
    /// let tail = VirtAddrRangeFrom::new(va!(usize::MAX));
    /// assert_eq!(bounded.checked_end(), Some(va!(usize::MAX)));
    /// assert_eq!(tail.checked_end(), None);
    /// ```
    fn checked_end(&self) -> Option<A>;

    /// Returns the exclusive end address of the range.
    ///
    /// # Panics
    ///
    /// Panics if the end address exceeds `usize::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{va, AddrRangeBounds, VirtAddrRange};
    ///
    /// let range = VirtAddrRange::new(va!(0x1000), va!(0x2000));
    /// assert_eq!(range.end(), va!(0x2000));
    /// ```
    ///
    /// An open-ended range has no representable exclusive end, even when it
    /// contains only one address:
    ///
    /// ```should_panic
    /// use memory_addr::{va, AddrRangeBounds, VirtAddrRangeFrom};
    ///
    /// VirtAddrRangeFrom::new(va!(usize::MAX)).end();
    /// ```
    fn end(&self) -> A {
        self.checked_end()
            .unwrap_or_else(const || panic!("the `end` of `AddrRangeBounds` exceeds `usize::MAX`"))
    }

    /// Returns the size of the range when it is representable.
    ///
    /// Returns `None` if the range covers the entire `[0, usize::MAX]` range,
    /// whose size exceeds `usize::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{va, AddrRangeBounds, VirtAddrRange, VirtAddrRangeFrom};
    ///
    /// let bounded = VirtAddrRange::new(va!(0x1000), va!(0x2000));
    /// assert_eq!(bounded.checked_size(), Some(0x1000));
    /// assert_eq!(VirtAddrRangeFrom::new(va!(usize::MAX)).checked_size(), Some(1));
    /// assert_eq!(VirtAddrRangeFrom::new(va!(1)).checked_size(), Some(usize::MAX));
    /// assert_eq!(VirtAddrRangeFrom::new(va!(0)).checked_size(), None);
    /// ```
    fn checked_size(&self) -> Option<usize>;

    /// Returns the size of the range.
    ///
    /// # Panics
    ///
    /// Panics if the size exceeds `usize::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{va, AddrRangeBounds, VirtAddrRange, VirtAddrRangeFrom};
    ///
    /// assert_eq!(VirtAddrRange::new(va!(0x1000), va!(0x1000)).size(), 0);
    /// assert_eq!(VirtAddrRange::new(va!(0x1000), va!(0x2000)).size(), 0x1000);
    /// assert_eq!(VirtAddrRangeFrom::new(va!(usize::MAX)).size(), 1);
    /// ```
    ///
    /// The entire address space has more addresses than `usize` can count:
    ///
    /// ```should_panic
    /// use memory_addr::{va, AddrRangeBounds, VirtAddrRangeFrom};
    ///
    /// VirtAddrRangeFrom::new(va!(0)).size();
    /// ```
    fn size(&self) -> usize {
        self.checked_size()
            .unwrap_or_else(const || panic!("the `size` of `AddrRangeBounds` exceeds `usize::MAX`"))
    }

    /// Returns `true` if the range is empty.
    ///
    /// A valid range is empty exactly when its start equals its exclusive end.
    /// Open-ended ranges are never empty.
    /// This method does not check validity. Use [`is_valid`](Self::is_valid)
    /// to detect reversed endpoints.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{va, AddrRangeBounds, VirtAddrRange, VirtAddrRangeFrom};
    ///
    /// assert!(VirtAddrRange::new(va!(0x1000), va!(0x1000)).is_empty());
    /// assert!(!VirtAddrRange::new(va!(0x1000), va!(0x2000)).is_empty());
    /// assert!(!VirtAddrRangeFrom::new(va!(usize::MAX)).is_empty());
    /// ```
    fn is_empty(&self) -> bool {
        Some(self.start()) == self.checked_end()
    }

    /// Checks if the range is valid.
    ///
    /// A bounded range is valid if and only if `start <= end`. Open-ended
    /// ranges are always valid. This query also accepts invalid ranges.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::{AddrRange, AddrRangeBounds, AddrRangeFrom};
    ///
    /// let range = AddrRange::new(0x1000usize, 0x2000);
    /// assert!(range.is_valid());
    /// let invalid_range = AddrRange { start: 0x2000usize, end: 0x1000 };
    /// assert!(!invalid_range.is_valid());
    /// assert!(AddrRange::new(0x1000usize, 0x1000).is_valid());
    /// assert!(AddrRangeFrom::new(usize::MAX).is_valid());
    /// ```
    #[inline]
    fn is_valid(&self) -> bool {
        self.checked_end()
            .is_none_or(const |end| end >= self.start())
    }

    /// Checks if the range contains the given address.
    ///
    /// The start is included and a representable exclusive end is excluded.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{va, AddrRangeBounds, VirtAddrRange, VirtAddrRangeFrom};
    ///
    /// let bounded = VirtAddrRange::new(va!(0x1000), va!(0x2000));
    /// assert!(!bounded.contains(va!(0x0fff)));
    /// assert!(bounded.contains(va!(0x1000)));
    /// assert!(bounded.contains(va!(0x1fff)));
    /// assert!(!bounded.contains(va!(0x2000)));
    ///
    /// let tail = VirtAddrRangeFrom::new(va!(0x1000));
    /// assert!(!tail.contains(va!(0x0fff)));
    /// assert!(tail.contains(va!(0x1000)));
    /// assert!(tail.contains(va!(usize::MAX)));
    /// ```
    ///
    /// An empty range contains no address, including its own start:
    ///
    /// ```
    /// use memory_addr::{va, va_range, AddrRangeBounds};
    ///
    /// let empty = va_range!(0x1000..0x1000);
    /// assert!(!empty.contains(va!(0x1000)));
    /// ```
    fn contains(&self, addr: A) -> bool {
        // It's self.start <= addr && addr < self.end
        addr >= self.start() && self.checked_end().is_none_or(const |end| addr < end)
    }

    /// Checks if the range contains the given address range.
    ///
    /// An empty range is contained in every valid range, including another
    /// empty range, regardless of its endpoints. An empty range cannot contain
    /// a nonempty range.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{va, va_range, AddrRangeBounds, VirtAddrRangeFrom};
    ///
    /// let bounded = va_range!(0x1000..0x2000);
    /// assert!(bounded.contains_range(va_range!(0x1000..0x2000)));
    /// assert!(bounded.contains_range(va_range!(0x1001..0x1fff)));
    /// assert!(!bounded.contains_range(va_range!(0x0fff..0x2000)));
    /// assert!(!bounded.contains_range(va_range!(0x1000..0x2001)));
    ///
    /// let tail = VirtAddrRangeFrom::new(va!(0x1000));
    /// assert!(tail.contains_range(bounded));
    /// assert!(!bounded.contains_range(tail));
    /// assert!(tail.contains_range(VirtAddrRangeFrom::new(va!(0x2000))));
    /// ```
    ///
    /// Empty ranges need not lie within the containing range's bounds:
    ///
    /// ```
    /// use memory_addr::{va, va_range, AddrRangeBounds, VirtAddrRangeFrom};
    ///
    /// let empty = va_range!(0x3000..0x3000);
    /// let other_empty = va_range!(0x1000..0x1000);
    /// let bounded = va_range!(0x1000..0x2000);
    /// assert!(bounded.contains_range(empty));
    /// assert!(empty.contains_range(other_empty));
    /// assert!(other_empty.contains_range(empty));
    /// assert!(VirtAddrRangeFrom::new(va!(0x4000)).contains_range(empty));
    /// assert!(!empty.contains_range(bounded));
    /// ```
    fn contains_range<R: [const] AddrRangeBounds<A>>(&self, other: R) -> bool {
        // Empty ranges are subsets regardless of their endpoints.
        // Otherwise compare both bounds, treating an absent end as one past MAX.
        other.is_empty()
            || self.start() <= other.start()
                && self.checked_end().is_none_or(const |end| {
                    other
                        .checked_end()
                        .is_some_and(const |other_end| other_end <= end)
                })
    }

    /// Checks if the range is contained in the given address range.
    ///
    /// An empty range is contained in every valid range, including another
    /// empty range at a different address. This is the reverse of
    /// [`contains_range`](Self::contains_range).
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{va, va_range, AddrRangeBounds, VirtAddrRangeFrom};
    ///
    /// let bounded = va_range!(0x1000..0x2000);
    /// assert!(bounded.contained_in(va_range!(0x0fff..0x2000)));
    /// assert!(!bounded.contained_in(va_range!(0x1001..0x2000)));
    ///
    /// let tail = VirtAddrRangeFrom::new(va!(0x1000));
    /// assert!(bounded.contained_in(tail));
    /// assert!(!tail.contained_in(bounded));
    /// assert!(va_range!(0x2000..0x2000).contained_in(bounded));
    /// ```
    ///
    /// ```
    /// use memory_addr::{va, va_range, AddrRangeBounds, VirtAddrRangeFrom};
    ///
    /// let empty = va_range!(0..0);
    /// assert!(empty.contained_in(va_range!(0x1000..0x2000)));
    /// assert!(empty.contained_in(va_range!(0x2000..0x2000)));
    /// assert!(empty.contained_in(VirtAddrRangeFrom::new(va!(usize::MAX))));
    /// assert!(!va_range!(0x1000..0x2000).contained_in(empty));
    /// ```
    fn contained_in<R: [const] AddrRangeBounds<A>>(&self, other: R) -> bool {
        other.contains_range(*self)
    }

    /// Checks if the range overlaps with the given address range.
    ///
    /// Two ranges overlap exactly when they share at least one address.
    /// An empty range never overlaps with any range, including itself or
    /// another empty range.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{va, va_range, AddrRangeBounds, VirtAddrRangeFrom};
    ///
    /// let bounded = va_range!(0x1000..0x2000);
    /// assert!(!bounded.overlaps(va_range!(0x0fff..0x1000)));
    /// assert!(bounded.overlaps(va_range!(0x0fff..0x1001)));
    /// assert!(bounded.overlaps(va_range!(0x1fff..0x2001)));
    /// assert!(!bounded.overlaps(va_range!(0x2000..0x2001)));
    ///
    /// let tail = VirtAddrRangeFrom::new(va!(0x1fff));
    /// assert!(bounded.overlaps(tail));
    /// assert!(tail.overlaps(bounded));
    /// assert!(!bounded.overlaps(VirtAddrRangeFrom::new(va!(0x2000))));
    /// assert!(tail.overlaps(VirtAddrRangeFrom::new(va!(usize::MAX))));
    /// ```
    ///
    /// Even an empty range positioned inside a nonempty range does not overlap:
    ///
    /// ```
    /// use memory_addr::{va, va_range, AddrRangeBounds, VirtAddrRangeFrom};
    ///
    /// let bounded = va_range!(0x1000..0x2000);
    /// let empty = va_range!(0x1800..0x1800);
    /// assert!(!bounded.overlaps(empty));
    /// assert!(!empty.overlaps(bounded));
    /// assert!(!empty.overlaps(empty));
    /// assert!(!empty.overlaps(va_range!(0..0)));
    /// assert!(!empty.overlaps(VirtAddrRangeFrom::new(va!(0))));
    /// ```
    fn overlaps<R: [const] AddrRangeBounds<A>>(&self, other: R) -> bool {
        // Strict bound comparisons detect a shared address only for nonempty ranges.
        !self.is_empty()
            && !other.is_empty()
            && other
                .checked_end()
                .is_none_or(const |other_end| self.start() < other_end)
            && self
                .checked_end()
                .is_none_or(const |end| other.start() < end)
    }

    /// The page iterator for this range type.
    ///
    /// Its items use the same semantic address type as the range endpoints.
    type Iterator: [const] AddrRangeIterator<A, AddrRange = Self>;

    /// Creates an iterator over complete pages in the range.
    ///
    /// Each item is a page's start address. Empty aligned ranges yield no
    /// pages. Open-ended ranges include the last page in the address space
    /// and terminate without wrapping to zero.
    ///
    /// Returns `None` if the range is invalid, `page_size` is not a non-zero
    /// power of two, or a representable endpoint is not page-aligned.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{va, va_range, AddrRangeBounds, VirtAddrRangeFrom};
    ///
    /// let pages: Vec<_> = va_range!(0x1000..0x3000).iter(0x1000).unwrap().collect();
    /// assert_eq!(pages, [va!(0x1000), va!(0x2000)]);
    ///
    /// let last_page = va!(usize::MAX - 0xfff);
    /// let mut pages = VirtAddrRangeFrom::new(last_page).iter(0x1000).unwrap();
    /// assert_eq!(pages.next(), Some(last_page));
    /// assert_eq!(pages.next(), None);
    /// assert_eq!(pages.next(), None);
    /// ```
    ///
    /// Unaligned endpoints are rejected instead of silently rounding them:
    ///
    /// ```
    /// use memory_addr::{va_range, AddrRangeBounds};
    ///
    /// assert!(va_range!(0x1000..0x2001).iter(0x1000).is_none());
    /// assert!(va_range!(0..0).iter(0).is_none());
    /// assert!(va_range!(0..0).iter(0x1000).is_some());
    /// ```
    fn iter(&self, page_size: usize) -> Option<Self::Iterator> {
        Self::Iterator::new(*self, page_size)
    }

    /// Wraps the range in the general address range representation.
    ///
    /// The endpoints and validity are preserved without additional checks.
    /// An existing general range is copied without changing its variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{AddrRange, AddrRangeBounds, AddrRangeFrom, GeneralAddrRange};
    ///
    /// let bounded = AddrRange::new(1usize, 2).into_general();
    /// let tail = AddrRangeFrom::new(usize::MAX).into_general();
    /// assert!(matches!(bounded, GeneralAddrRange::Range(_)));
    /// assert!(matches!(tail, GeneralAddrRange::RangeFrom(_)));
    /// assert_eq!(tail.into_general(), tail);
    /// ```
    fn into_general(self) -> GeneralAddrRange<A>;
}
