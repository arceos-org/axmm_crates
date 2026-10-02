//! Address ranges with a runtime-selected endpoint representation.
//!
//! The range and iterator enums delegate to the corresponding bounded or
//! open-ended implementations.

use core::{fmt, iter::FusedIterator};

use crate::{
    AddrRange, AddrRangeBounds, AddrRangeFrom, AddrRangeFromIter, AddrRangeIter, AddrRangeIterator,
    MemoryAddr,
};

/// An address range with either a bounded or an open-ended representation.
///
/// Use this type to store or return either kind of range as a single type.
/// Queries use the same semantics as the contained range. Prefer the concrete
/// range types when the endpoint representation is known in advance to avoid
/// storing an enum discriminant.
///
/// Equality compares the variant and its stored endpoints. Empty ranges with
/// different endpoints compare unequal even though they contain each other.
///
/// # Examples
///
/// ```
/// use memory_addr::{va, va_range, AddrRangeBounds, GeneralAddrRange, VirtAddrRangeFrom};
///
/// let ranges = [
///     GeneralAddrRange::from(va_range!(0x1000..0x2000)),
///     GeneralAddrRange::from(VirtAddrRangeFrom::new(va!(usize::MAX))),
/// ];
/// assert_eq!(ranges[0].checked_end(), Some(va!(0x2000)));
/// assert_eq!(ranges[1].checked_end(), None);
/// assert_eq!(ranges[1].size(), 1);
/// match ranges[0] {
///     GeneralAddrRange::Range(range) => assert_eq!(range.start, va!(0x1000)),
///     GeneralAddrRange::RangeFrom(_) => unreachable!(),
/// }
/// ```
#[derive(Debug, Copy)]
#[derive_const(Clone, PartialEq, Eq)]
pub enum GeneralAddrRange<A: MemoryAddr> {
    /// A range with a representable exclusive end.
    ///
    /// The contained range must satisfy the validity requirements of
    /// `AddrRange`.
    Range(AddrRange<A>),
    /// A range extending through the maximum address.
    ///
    /// The conceptual exclusive end is one past `usize::MAX`.
    RangeFrom(AddrRangeFrom<A>),
}

/// Conversion from a bounded range.
///
/// Wrapping preserves the endpoints without additional validation.
const impl<A: MemoryAddr> From<AddrRange<A>> for GeneralAddrRange<A> {
    #[inline]
    fn from(range: AddrRange<A>) -> Self {
        Self::Range(range)
    }
}

/// Conversion from an open-ended range.
///
/// Wrapping preserves the inclusive start and implicit exclusive end.
const impl<A: MemoryAddr> From<AddrRangeFrom<A>> for GeneralAddrRange<A> {
    #[inline]
    fn from(range: AddrRangeFrom<A>) -> Self {
        Self::RangeFrom(range)
    }
}

/// Common queries for either endpoint representation.
///
/// Primitive queries delegate to the contained range. Relationship queries use
/// the shared trait's defaults and accept all supported range types.
const impl<A: [const] MemoryAddr> AddrRangeBounds<A> for GeneralAddrRange<A> {
    type Iterator = GeneralAddrRangeIter<A>;

    #[inline]
    fn start(&self) -> A {
        match self {
            Self::Range(range) => range.start(),
            Self::RangeFrom(range) => range.start(),
        }
    }

    #[inline]
    fn checked_end(&self) -> Option<A> {
        match self {
            Self::Range(range) => range.checked_end(),
            Self::RangeFrom(range) => range.checked_end(),
        }
    }

    #[inline]
    fn checked_size(&self) -> Option<usize> {
        match self {
            Self::Range(range) => range.checked_size(),
            Self::RangeFrom(range) => range.checked_size(),
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        match self {
            Self::Range(range) => range.is_empty(),
            Self::RangeFrom(range) => range.is_empty(),
        }
    }

    #[inline]
    fn align_inwards(&self, alignment: usize) -> Option<Self> {
        match self {
            Self::Range(range) => range.align_inwards(alignment).map(Self::Range),
            Self::RangeFrom(range) => range.align_inwards(alignment).map(Self::RangeFrom),
        }
    }

    #[inline]
    fn align_outwards(&self, alignment: usize) -> Option<Self> {
        match self {
            Self::Range(range) => range.align_outwards(alignment).map(Self::Range),
            Self::RangeFrom(range) => range.align_outwards(alignment).map(Self::RangeFrom),
        }
    }

    #[inline]
    fn into_general(self) -> GeneralAddrRange<A> {
        self
    }
}

/// Lowercase hexadecimal formatting for either range representation.
///
/// Formatting delegates to the contained range without an enum variant prefix.
impl<A: MemoryAddr + fmt::LowerHex> fmt::LowerHex for GeneralAddrRange<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Range(range) => fmt::LowerHex::fmt(range, f),
            Self::RangeFrom(range) => fmt::LowerHex::fmt(range, f),
        }
    }
}

/// Uppercase hexadecimal formatting for either range representation.
///
/// Formatting delegates to the contained range without an enum variant prefix.
impl<A: MemoryAddr + fmt::UpperHex> fmt::UpperHex for GeneralAddrRange<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Range(range) => fmt::UpperHex::fmt(range, f),
            Self::RangeFrom(range) => fmt::UpperHex::fmt(range, f),
        }
    }
}

/// A page iterator over either kind of address range.
///
/// Construct it with [`AddrRangeBounds::iter`] or [`AddrRangeIterator::new`].
/// Iteration delegates to the contained iterator, preserving the exclusive
/// end and permanent exhaustion after the final page.
///
/// # Examples
///
/// ```
/// use memory_addr::{va, va_range, AddrRangeBounds, GeneralAddrRange, VirtAddrRangeFrom};
///
/// let last_page = va!(usize::MAX - 0xfff);
/// let ranges = [
///     GeneralAddrRange::from(va_range!(0x1000..0x2000)),
///     GeneralAddrRange::from(VirtAddrRangeFrom::new(last_page)),
/// ];
/// let pages: Vec<_> = ranges.into_iter()
///     .flat_map(|range| range.iter(0x1000).unwrap())
///     .collect();
/// assert_eq!(pages, [va!(0x1000), last_page]);
/// ```
pub enum GeneralAddrRangeIter<A: MemoryAddr> {
    /// An iterator with a representable exclusive end.
    ///
    /// The contained iterator owns its bounded range state.
    Range(AddrRangeIter<A>),
    /// An iterator extending through the final page of the address space.
    ///
    /// The contained iterator terminates without wrapping to zero.
    RangeFrom(AddrRangeFromIter<A>),
}

/// Construction of a page iterator for either range variant.
///
/// Validation delegates to the corresponding concrete iterator. Invalid
/// ranges, invalid page sizes, and unaligned endpoints return `None`.
const impl<A: [const] MemoryAddr> AddrRangeIterator<A> for GeneralAddrRangeIter<A> {
    type AddrRange = GeneralAddrRange<A>;

    #[inline]
    fn new(range: GeneralAddrRange<A>, page_size: usize) -> Option<Self> {
        match range {
            GeneralAddrRange::Range(range) => match AddrRangeIter::new(range, page_size) {
                Some(iter) => Some(Self::Range(iter)),
                None => None,
            },
            GeneralAddrRange::RangeFrom(range) => match AddrRangeFromIter::new(range, page_size) {
                Some(iter) => Some(Self::RangeFrom(iter)),
                None => None,
            },
        }
    }
}

/// Ascending page iteration for either range variant.
///
/// Each call advances only the contained iterator.
const impl<A: [const] MemoryAddr> Iterator for GeneralAddrRangeIter<A> {
    // TODO: Delegate size_hint and nth once the concrete iterators provide them.
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<A> {
        match self {
            Self::Range(iter) => iter.next(),
            Self::RangeFrom(iter) => iter.next(),
        }
    }
}

/// Permanent exhaustion for either iterator variant.
///
/// Both contained iterator types are fused.
impl<A: MemoryAddr> FusedIterator for GeneralAddrRangeIter<A> {}
