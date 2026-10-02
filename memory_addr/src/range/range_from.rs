//! Address ranges extending through the maximum address.
//!
//! Only the inclusive start is stored. The exclusive end is one past
//! `usize::MAX` and cannot be represented by an address.

use core::{fmt, iter::FusedIterator};

use crate::{AddrRangeBounds, AddrRangeIterator, GeneralAddrRange, MemoryAddr, PhysAddr, VirtAddr};

/// An address range extending from its start through the maximum address.
///
/// The range includes both `start` and `usize::MAX`, so it is never empty.
/// Its exclusive end is not representable. Its size is representable unless
/// `start` is zero, in which case it covers the entire address space.
///
/// This type is a mimic of the standard library's `RangeFrom` type, adapted for
/// memory addresses.
///
/// # Examples
///
/// ```
/// use memory_addr::{va, AddrRangeBounds, VirtAddrRangeFrom};
///
/// let range = VirtAddrRangeFrom::new(va!(usize::MAX));
/// assert!(range.contains(va!(usize::MAX)));
/// assert_eq!(range.checked_end(), None);
/// assert_eq!(range.size(), 1);
/// ```
#[derive(Copy)]
#[derive_const(Clone, PartialEq, Eq)]
pub struct AddrRangeFrom<A: MemoryAddr> {
    /// The inclusive lower bound.
    ///
    /// Every representable start produces a valid, nonempty range.
    pub start: A,
}

/// Constructors for ranges extending through the maximum address.
///
/// Constructing a range only stores its start and requires no const address
/// operations.
const impl<A: MemoryAddr> AddrRangeFrom<A> {
    /// Creates a range from the given start through the maximum address.
    ///
    /// This operation always succeeds, including when `start` is the maximum
    /// address.
    #[inline]
    pub fn new(start: A) -> Self {
        Self { start }
    }
}

/// Common queries for ranges extending through the maximum address.
///
/// The exclusive end is never representable. Only a range starting at zero
/// has an unrepresentable size.
const impl<A: [const] MemoryAddr> AddrRangeBounds<A> for AddrRangeFrom<A> {
    type Iterator = AddrRangeFromIter<A>;

    #[inline]
    fn start(&self) -> A {
        self.start
    }

    #[inline]
    fn checked_end(&self) -> Option<A> {
        None
    }

    #[inline]
    fn checked_size(&self) -> Option<usize> {
        (usize::MAX - self.start.into()).checked_add(1)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        false
    }

    #[inline]
    fn align_inwards(&self, alignment: usize) -> Option<Self> {
        if !alignment.is_power_of_two() {
            return None;
        }

        let start = self.start.align_up_checked(alignment)?;
        Some(Self { start })
    }

    #[inline]
    fn align_outwards(&self, alignment: usize) -> Option<Self> {
        if !alignment.is_power_of_two() {
            return None;
        }

        let start = self.start.align_down(alignment);
        Some(Self { start })
    }

    #[inline]
    fn into_general(self) -> GeneralAddrRange<A> {
        self.into()
    }
}

/// An iterator over pages through the end of the address space.
///
/// Construct it with [`AddrRangeBounds::iter`] or [`AddrRangeIterator::new`].
/// The last page is yielded exactly once, after which the iterator remains
/// exhausted. It never wraps to the zero address.
pub struct AddrRangeFromIter<A: MemoryAddr> {
    /// The next page address, or `None` after the last page.
    ///
    /// Explicit exhaustion distinguishes an initial zero from address overflow.
    next: Option<A>,
    /// The size of each page in bytes.
    ///
    /// Construction ensures this is a nonzero power of two.
    page_size: usize,
}

/// Construction of an iterator through the address-space end.
///
/// The conceptual exclusive end is aligned to every supported page size.
const impl<A: [const] MemoryAddr> AddrRangeIterator<A> for AddrRangeFromIter<A> {
    type AddrRange = AddrRangeFrom<A>;

    #[inline]
    fn new(range: AddrRangeFrom<A>, page_size: usize) -> Option<Self> {
        if !page_size.is_power_of_two() || !range.start.is_aligned(page_size) {
            None
        } else {
            Some(Self {
                next: Some(range.start),
                page_size,
            })
        }
    }
}

/// Ascending iteration through the last page of the address space.
///
/// Overflow when advancing marks exhaustion instead of wrapping or panicking.
const impl<A: [const] MemoryAddr> Iterator for AddrRangeFromIter<A> {
    // TODO: Add size_hint and direct nth while allowing the remaining count to
    // exceed usize::MAX.
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<A> {
        match self.next {
            Some(page) => {
                self.next = page.checked_add(self.page_size);
                Some(page)
            }
            None => None,
        }
    }
}

/// Stable exhaustion for iteration through the address-space end.
///
/// Once the final page is yielded, subsequent calls always return `None`.
impl<A: MemoryAddr> FusedIterator for AddrRangeFromIter<A> {}

/// Debug formatting for an open-ended address range.
///
/// The output follows `start..` syntax.
impl<A: MemoryAddr + fmt::Debug> fmt::Debug for AddrRangeFrom<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}..", self.start)
    }
}

/// Lowercase hexadecimal formatting for an open-ended address range.
///
/// The start delegates to the address type's hexadecimal formatter.
impl<A: MemoryAddr + fmt::LowerHex> fmt::LowerHex for AddrRangeFrom<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}..", self.start)
    }
}

/// Uppercase hexadecimal formatting for an open-ended address range.
///
/// The start delegates to the address type's hexadecimal formatter.
impl<A: MemoryAddr + fmt::UpperHex> fmt::UpperHex for AddrRangeFrom<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}..", self.start)
    }
}

/// A virtual address range extending through the maximum address.
///
/// Only the inclusive virtual start address is stored.
pub type VirtAddrRangeFrom = AddrRangeFrom<VirtAddr>;

/// A physical address range extending through the maximum address.
///
/// Only the inclusive physical start address is stored.
pub type PhysAddrRangeFrom = AddrRangeFrom<PhysAddr>;
