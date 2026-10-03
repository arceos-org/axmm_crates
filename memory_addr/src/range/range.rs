//! Address ranges with a representable exclusive end.
//!
//! This module provides bounded ranges, their constructors, and page iterators.

use core::{fmt, iter::FusedIterator};

use crate::{AddrRangeBounds, AddrRangeIterator, GeneralAddrRange, MemoryAddr, PhysAddr, VirtAddr};

/// A range of a given memory address type `A`.
///
/// The range is inclusive on the start and exclusive on the end. A range is
/// considered **empty** if and only if `start == end`, and **invalid** if and
/// only if `start > end`. All methods, unless explicitly stated otherwise, do
/// not promise a correct or reasonable result or behavior when applied to
/// invalid ranges.
///
/// Safe constructors reject invalid ranges. When modifying the public fields
/// directly, callers must preserve `start <= end`.
///
/// Equality compares stored endpoints. Empty ranges at different addresses
/// compare unequal, even though each contains the other according to
/// [`AddrRangeBounds::contains_range`].
///
/// This type is a mimic of the standard library's `Range` type, adapted for
/// memory addresses.
///
/// # Example
///
/// ```
/// use memory_addr::AddrRange;
///
/// let range = AddrRange::<usize>::new(0x1000, 0x2000);
/// assert_eq!(range.start, 0x1000);
/// assert_eq!(range.end, 0x2000);
/// ```
#[derive(Copy)]
#[derive_const(Clone, PartialEq, Eq)]
pub struct AddrRange<A: MemoryAddr> {
    /// The lower bound of the range (inclusive).
    ///
    /// This address must not exceed the exclusive end.
    pub start: A,
    /// The upper bound of the range (exclusive).
    ///
    /// This address must not precede the inclusive start.
    pub end: A,
}

/// Always-const implementations of [`AddrRange`].
///
/// Storing endpoints does not require const address operations.
const impl<A> AddrRange<A>
where
    A: MemoryAddr,
{
    /// Creates an address range without checking its endpoints.
    ///
    /// Stores the endpoints without comparing them.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `start <= end`.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::AddrRange;
    ///
    /// let range = unsafe { AddrRange::new_unchecked(0x1000usize, 0x2000) };
    /// assert_eq!(range.start, 0x1000);
    /// assert_eq!(range.end, 0x2000);
    /// ```
    #[inline]
    pub unsafe fn new_unchecked(start: A, end: A) -> Self {
        // TODO: Reassess unsafe constructors since public fields also permit invalid
        // ranges.
        Self { start, end }
    }
}

/// Methods for [`AddrRange`].
///
/// Checked constructors enforce ordered endpoints and a representable end.
/// The unchecked constructor relies on its caller to prevent overflow.
const impl<A> AddrRange<A>
where
    A: [const] MemoryAddr,
{
    /// Creates a new address range from the start and end addresses.
    ///
    /// Equal endpoints produce an empty range.
    ///
    /// # Panics
    ///
    /// Panics if `start > end`.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::AddrRange;
    ///
    /// let range = AddrRange::new(0x1000usize, 0x2000);
    /// assert_eq!(range.start, 0x1000);
    /// assert_eq!(range.end, 0x2000);
    /// ```
    ///
    /// And this will panic:
    ///
    /// ```should_panic
    /// # use memory_addr::AddrRange;
    /// let _ = AddrRange::new(0x2000usize, 0x1000);
    /// ```
    #[inline]
    pub fn new(start: A, end: A) -> Self {
        assert!(
            start <= end,
            "invalid `AddrRange`: `start` must be less than or equal to `end`",
        );
        Self { start, end }
    }

    /// Creates an address range if the start does not exceed the end.
    ///
    /// Returns `None` if `start > end`.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::AddrRange;
    ///
    /// let range = AddrRange::try_new(0x1000usize, 0x2000).unwrap();
    /// assert_eq!(range.start, 0x1000);
    /// assert_eq!(range.end, 0x2000);
    /// assert!(AddrRange::try_new(0x2000usize, 0x1000).is_none());
    /// ```
    #[inline]
    pub fn try_new(start: A, end: A) -> Option<Self> {
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    /// Creates a new address range from the start address and the size.
    ///
    /// The size is measured in bytes. A zero size produces an empty range.
    ///
    /// # Panics
    ///
    /// Panics if `start + size` exceeds the maximum address.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::AddrRange;
    ///
    /// let range = AddrRange::from_start_size(0x1000usize, 0x1000);
    /// assert_eq!(range.start, 0x1000);
    /// assert_eq!(range.end, 0x2000);
    /// ```
    ///
    /// And this will panic:
    ///
    /// ```should_panic
    /// # use memory_addr::AddrRange;
    /// let _ = AddrRange::from_start_size(0x1000usize, usize::MAX);
    /// ```
    #[inline]
    pub fn from_start_size(start: A, size: usize) -> Self {
        if let Some(end) = start.checked_add(size) {
            Self { start, end }
        } else {
            panic!("invalid `AddrRange`: `start` + `size` overflows",);
        }
    }

    /// Creates a new address range from the start address and the size.
    ///
    /// The size is measured in bytes. Returns `None` if `start + size` exceeds
    /// the maximum address.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::AddrRange;
    ///
    /// let range = AddrRange::try_from_start_size(0x1000usize, 0x1000).unwrap();
    /// assert_eq!(range.start, 0x1000);
    /// assert_eq!(range.end, 0x2000);
    /// assert!(AddrRange::try_from_start_size(0x1000usize, usize::MAX).is_none());
    /// ```
    #[inline]
    pub fn try_from_start_size(start: A, size: usize) -> Option<Self> {
        start.checked_add(size).map(const |end| Self { start, end })
    }

    /// Creates a range from a start and size without checking overflow.
    ///
    /// The size is measured in bytes. The caller must ensure the end is
    /// representable.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `start + size` does not overflow.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::AddrRange;
    ///
    /// let range = unsafe { AddrRange::from_start_size_unchecked(0x1000usize, 0x1000) };
    /// assert_eq!(range.start, 0x1000);
    /// assert_eq!(range.end, 0x2000);
    /// ```
    #[inline]
    pub unsafe fn from_start_size_unchecked(start: A, size: usize) -> Self {
        Self {
            start,
            end: start.wrapping_add(size),
        }
    }
}

/// Common queries for bounded address ranges.
///
/// A valid bounded range always has a representable end and size.
const impl<A: [const] MemoryAddr> AddrRangeBounds<A> for AddrRange<A> {
    type Iterator = AddrRangeIter<A>;

    #[inline]
    fn start(&self) -> A {
        self.start
    }

    #[inline]
    fn checked_end(&self) -> Option<A> {
        Some(self.end)
    }

    #[inline]
    fn checked_size(&self) -> Option<usize> {
        Some(self.end.wrapping_sub_addr(self.start))
    }

    #[inline]
    fn subtract<R: [const] AddrRangeBounds<A>>(
        &self,
        other: R,
    ) -> (Option<AddrRange<A>>, Option<Self>) {
        if other.is_empty() {
            return (None, Some(*self));
        }

        let other_start = other.start();
        let other_end = other.checked_end();

        if other_start <= self.start {
            // The other range starts before this range.
            match other_end {
                // The other range ends before or inside this range, leaving the portion after it as
                // a bounded range.
                Some(other_end) if other_end < self.end => (
                    None,
                    Some(Self {
                        start: self.start.max(other_end),
                        end: self.end,
                    }),
                ),
                // The other range extends to the end of the address space, leaving nothing.
                _ => (None, None),
            }
        } else if other_start < self.end {
            // The other range starts inside this range, leaving the portion before it as a
            // bounded range.
            let before = Self {
                start: self.start,
                end: other_start,
            };

            match other_end {
                // The other range ends before the end of this range, leaving the portion after it
                // as a bounded range.
                Some(other_end) if other_end < self.end => (
                    Some(before),
                    Some(Self {
                        start: other_end,
                        end: self.end,
                    }),
                ),
                // The other range extends to or beyond the end of this range, leaving nothing after
                // it.
                _ => (Some(before), None),
            }
        } else {
            // The other range starts after this range, leaving the original range intact.
            (Some(*self), None)
        }
    }

    #[inline]
    fn align_inwards(&self, alignment: usize) -> Option<Self> {
        if !alignment.is_power_of_two() {
            return None;
        }

        let start = self.start.align_up_checked(alignment)?;
        let end = self.end.align_down(alignment);
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    #[inline]
    fn align_outwards(&self, alignment: usize) -> Option<Self> {
        if !alignment.is_power_of_two() {
            return None;
        }

        let start = self.start.align_down(alignment);
        let end = self.end.align_up_checked(alignment)?;
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    #[inline]
    fn into_general(self) -> GeneralAddrRange<A> {
        self.into()
    }
}

/// An iterator over the pages in a bounded address range.
///
/// Construct it with [`AddrRangeBounds::iter`] or [`AddrRangeIterator::new`].
/// The exclusive end is never yielded. Once exhausted, the iterator remains
/// exhausted.
pub struct AddrRangeIter<A: MemoryAddr> {
    /// The next page address.
    ///
    /// Equality with `end` indicates exhaustion.
    next: A,
    /// The exclusive end address.
    ///
    /// Both endpoints are aligned to the page size.
    end: A,
    /// The size of each page in bytes.
    ///
    /// Construction ensures this is a nonzero power of two.
    page_size: usize,
}

/// Construction of a bounded page iterator.
///
/// Validation ensures each remaining step fits below or at the exclusive end.
const impl<A: [const] MemoryAddr> AddrRangeIterator<A> for AddrRangeIter<A> {
    type AddrRange = AddrRange<A>;

    #[inline]
    fn new(range: AddrRange<A>, page_size: usize) -> Option<Self> {
        if !page_size.is_power_of_two()
            || !range.is_valid()
            || !range.start.is_aligned(page_size)
            || !range.end.is_aligned(page_size)
        {
            None
        } else {
            Some(Self {
                next: range.start,
                end: range.end,
                page_size,
            })
        }
    }
}

/// Ascending iteration over a bounded range's page starts.
///
/// Aligned endpoints ensure that advancing never crosses the exclusive end.
const impl<A: [const] MemoryAddr> Iterator for AddrRangeIter<A> {
    // TODO: Add size_hint and direct nth, and consider ExactSizeIterator for
    // bounded ranges.
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<A> {
        if self.next < self.end {
            let page = self.next;
            self.next = page.add(self.page_size);
            Some(page)
        } else {
            None
        }
    }
}

/// Stable exhaustion for bounded page iteration.
///
/// Once the exclusive end is reached, all subsequent calls return `None`.
impl<A: MemoryAddr> FusedIterator for AddrRangeIter<A> {}

/// Default initialization of a bounded address range.
///
/// The default value is an empty range `AddrRange { start: 0, end: 0 }`.
const impl<A> Default for AddrRange<A>
where
    A: [const] MemoryAddr,
{
    #[inline]
    fn default() -> Self {
        Self {
            start: 0.into(),
            end: 0.into(),
        }
    }
}

/// Implementations of [`Debug`](fmt::Debug) for [`AddrRange`].
///
/// Both endpoints use their address type's debug representation.
impl<A> fmt::Debug for AddrRange<A>
where
    A: MemoryAddr + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}..{:?}", self.start, self.end)
    }
}

/// Implementations of [`LowerHex`](fmt::LowerHex) for [`AddrRange`].
///
/// Both endpoints use their address type's lowercase hexadecimal
/// representation.
impl<A> fmt::LowerHex for AddrRange<A>
where
    A: MemoryAddr + fmt::LowerHex,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}..{:x}", self.start, self.end)
    }
}

/// Implementations of [`UpperHex`](fmt::UpperHex) for [`AddrRange`].
///
/// Both endpoints use their address type's uppercase hexadecimal
/// representation.
impl<A> fmt::UpperHex for AddrRange<A>
where
    A: MemoryAddr + fmt::UpperHex,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}..{:X}", self.start, self.end)
    }
}

/// A range of virtual addresses [`VirtAddr`].
///
/// Both endpoints are representable virtual addresses, with an exclusive end.
pub type VirtAddrRange = AddrRange<VirtAddr>;

/// A range of physical addresses [`PhysAddr`].
///
/// Both endpoints are representable physical addresses, with an exclusive end.
pub type PhysAddrRange = AddrRange<PhysAddr>;
