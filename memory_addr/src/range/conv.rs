//! Conversions from standard library ranges to address ranges.
//!
//! The output type preserves whether the exclusive end is representable.

use core::{
    marker::Destruct,
    ops::{Bound, RangeBounds},
};

use crate::{AddrRange, AddrRangeBounds, AddrRangeFrom, GeneralAddrRange, MemoryAddr};

/// A conversion from a standard library range to an address range.
///
/// Both [`core::range`] and [`core::ops`] range types are supported. Endpoints
/// are converted through [`Into<A>`](Into). Missing lower bounds become address
/// zero.
///
/// | Source range | Output type |
/// | --- | --- |
/// | `Range`, `RangeTo` | [`AddrRange<A>`] |
/// | `RangeFrom`, `RangeFull` | [`AddrRangeFrom<A>`] |
/// | `RangeInclusive`, `RangeToInclusive` | [`GeneralAddrRange<A>`] |
///
/// Exclusive endpoints are preserved without validation, including reversed
/// endpoints that produce an invalid `AddrRange`. Inclusive upper endpoints
/// become exclusive by adding one. An inclusive upper endpoint of `usize::MAX`
/// produces [`GeneralAddrRange::RangeFrom`], avoiding overflow.
///
/// Reversed inclusive ranges become empty bounded ranges at the converted
/// start. An exhausted [`core::ops::RangeInclusive`] also becomes an empty
/// bounded range, whose location is unspecified because its source endpoints
/// are unspecified.
///
/// # Examples
///
/// ```
/// use memory_addr::{AddrRange, AddrRangeBounds, GeneralAddrRange, IntoAddrRange};
///
/// let reversed = core::range::Range { start: 5usize, end: 2 };
/// let bounded: AddrRange<usize> = reversed.into_addr_range();
/// assert!(!bounded.is_valid());
/// let reversed = core::range::RangeInclusive { start: 5usize, last: 2 };
/// let inclusive: GeneralAddrRange<usize> = reversed.into_addr_range();
/// assert!(inclusive.is_valid());
/// assert!(inclusive.is_empty());
///
/// let mut exhausted = usize::MAX..=usize::MAX;
/// assert_eq!(exhausted.next(), Some(usize::MAX));
/// let empty: GeneralAddrRange<usize> = exhausted.into_addr_range();
/// assert!(empty.is_empty());
/// ```
pub const trait IntoAddrRange<A: [const] MemoryAddr>: Sized {
    /// The address range representation produced by this conversion.
    ///
    /// Its concrete type depends on the source range's upper bound.
    type Into: [const] AddrRangeBounds<A>;

    /// Converts the source into its corresponding address range representation.
    ///
    /// Inclusive ranges select a bounded or unbounded representation according
    /// to whether their exclusive endpoint can be represented.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{AddrRange, GeneralAddrRange, IntoAddrRange};
    ///
    /// let prefix: AddrRange<usize> = (..4usize).into_addr_range();
    /// assert_eq!(prefix, AddrRange::new(0, 4));
    /// let singleton: GeneralAddrRange<usize> = (4usize..=4).into_addr_range();
    /// assert_eq!(singleton, GeneralAddrRange::Range(AddrRange::new(4, 5)));
    /// ```
    fn into_addr_range(self) -> Self::Into;

    /// Converts the source into the general address range representation.
    ///
    /// This produces the same output type for every supported source range.
    /// It applies the same endpoint and validity rules as
    /// [`into_addr_range`](Self::into_addr_range).
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{AddrRangeBounds, GeneralAddrRange, IntoAddrRange};
    ///
    /// let prefix: GeneralAddrRange<usize> = (..4usize).into_general_addr_range();
    /// assert_eq!(prefix.checked_end(), Some(4));
    /// let full: GeneralAddrRange<usize> = (..=usize::MAX).into_general_addr_range();
    /// assert_eq!(full.start(), 0);
    /// assert_eq!(full.checked_end(), None);
    /// assert!(full.contains(usize::MAX));
    /// ```
    fn into_general_addr_range(self) -> GeneralAddrRange<A> {
        self.into_addr_range().into_general()
    }
}

/// Converts inclusive endpoints into an empty, bounded, or unbounded range.
///
/// Reversed endpoints describe an empty range at `start`. Otherwise the last
/// address determines whether the exclusive endpoint is representable.
const fn from_inclusive<A: [const] MemoryAddr>(start: A, last: A) -> GeneralAddrRange<A> {
    if start > last {
        return GeneralAddrRange::Range(AddrRange { start, end: start });
    }
    match last.checked_add(1) {
        Some(end) => GeneralAddrRange::Range(AddrRange { start, end }),
        None => GeneralAddrRange::RangeFrom(AddrRangeFrom { start }),
    }
}

/// Converts a new half-open range into a bounded address range.
///
/// Endpoint conversion does not validate their ordering.
const impl<A: [const] MemoryAddr, T: [const] Into<A> + [const] Destruct> IntoAddrRange<A>
    for core::range::Range<T>
{
    type Into = AddrRange<A>;

    fn into_addr_range(self) -> Self::Into {
        AddrRange {
            start: self.start.into(),
            end: self.end.into(),
        }
    }
}

/// Converts a new unbounded range into an address range through the maximum.
///
/// The inclusive start is preserved through endpoint conversion.
const impl<A: [const] MemoryAddr, T: [const] Into<A> + [const] Destruct> IntoAddrRange<A>
    for core::range::RangeFrom<T>
{
    type Into = AddrRangeFrom<A>;

    fn into_addr_range(self) -> Self::Into {
        AddrRangeFrom {
            start: self.start.into(),
        }
    }
}

/// Converts an exclusive prefix into a bounded address range starting at zero.
///
/// The original type path also supports nightlies predating its `core::range`
/// re-export.
const impl<A: [const] MemoryAddr, T: [const] Into<A> + [const] Destruct> IntoAddrRange<A>
    for core::ops::RangeTo<T>
{
    type Into = AddrRange<A>;

    fn into_addr_range(self) -> Self::Into {
        AddrRange {
            start: A::from(0),
            end: self.end.into(),
        }
    }
}

/// Converts a new inclusive range into a general address range.
///
/// The exclusive end is one past the last address, or unrepresentable at the
/// maximum.
const impl<A: [const] MemoryAddr, T: [const] Into<A> + [const] Destruct> IntoAddrRange<A>
    for core::range::RangeInclusive<T>
{
    type Into = GeneralAddrRange<A>;

    fn into_addr_range(self) -> Self::Into {
        from_inclusive(self.start.into(), self.last.into())
    }
}

/// Converts a new inclusive prefix into a general address range.
///
/// The start is zero. An inclusive end at the maximum covers the full address
/// space.
const impl<A: [const] MemoryAddr, T: [const] Into<A> + [const] Destruct> IntoAddrRange<A>
    for core::range::RangeToInclusive<T>
{
    type Into = GeneralAddrRange<A>;

    fn into_addr_range(self) -> Self::Into {
        from_inclusive(A::from(0), self.last.into())
    }
}

/// Converts a legacy half-open range into a bounded address range.
///
/// Current endpoints are preserved even if the range has already been iterated.
const impl<A: [const] MemoryAddr, T: [const] Into<A> + [const] Destruct> IntoAddrRange<A>
    for core::ops::Range<T>
{
    type Into = AddrRange<A>;

    fn into_addr_range(self) -> Self::Into {
        AddrRange {
            start: self.start.into(),
            end: self.end.into(),
        }
    }
}

/// Converts a legacy unbounded range into an address range through the maximum.
///
/// The current inclusive start is preserved through endpoint conversion.
const impl<A: [const] MemoryAddr, T: [const] Into<A> + [const] Destruct> IntoAddrRange<A>
    for core::ops::RangeFrom<T>
{
    type Into = AddrRangeFrom<A>;

    fn into_addr_range(self) -> Self::Into {
        AddrRangeFrom {
            start: self.start.into(),
        }
    }
}

/// Converts a legacy inclusive range while preserving its exhaustion state.
///
/// Exhaustion produces an empty range regardless of the remaining endpoint
/// values.
const impl<A: [const] MemoryAddr, T: [const] Into<A> + [const] Destruct> IntoAddrRange<A>
    for core::ops::RangeInclusive<T>
{
    type Into = GeneralAddrRange<A>;

    fn into_addr_range(self) -> Self::Into {
        // The excluded bound exposes exhaustion without requiring `T: PartialOrd`.
        let exhausted = matches!(self.end_bound(), Bound::Excluded(_));
        let (start, last) = self.into_inner();
        let start = start.into();
        let last = last.into();
        if exhausted {
            GeneralAddrRange::Range(AddrRange { start, end: start })
        } else {
            from_inclusive(start, last)
        }
    }
}

/// Converts a legacy inclusive prefix into a general address range.
///
/// The start is zero. An inclusive end at the maximum covers the full address
/// space.
const impl<A: [const] MemoryAddr, T: [const] Into<A> + [const] Destruct> IntoAddrRange<A>
    for core::ops::RangeToInclusive<T>
{
    type Into = GeneralAddrRange<A>;

    fn into_addr_range(self) -> Self::Into {
        from_inclusive(A::from(0), self.end.into())
    }
}

/// Converts the full range into an address range covering every address.
///
/// The inclusive start is zero and the exclusive end is unrepresentable.
const impl<A: [const] MemoryAddr> IntoAddrRange<A> for core::ops::RangeFull {
    type Into = AddrRangeFrom<A>;

    fn into_addr_range(self) -> Self::Into {
        AddrRangeFrom { start: A::from(0) }
    }
}

/// Converts a standard library range into an address range.
///
/// The address type is inferred from context. The source's upper bound
/// determines the output type as described by [`IntoAddrRange`]. Exclusive
/// endpoints are preserved without validation, so reversed exclusive ranges do
/// not panic.
///
/// # Example
///
/// ```
/// use memory_addr::{addr_range, AddrRange};
///
/// let range: AddrRange<usize> = addr_range!(0x1000usize..0x2000);
/// assert_eq!(range.start, 0x1000usize);
/// assert_eq!(range.end, 0x2000usize);
/// ```
///
/// Inclusive ranges can include the maximum address:
///
/// ```
/// use memory_addr::{addr_range, AddrRangeBounds, GeneralAddrRange};
///
/// let range: GeneralAddrRange<usize> = addr_range!(usize::MAX..=usize::MAX);
/// assert_eq!(range.size(), 1);
/// assert!(range.contains(usize::MAX));
/// ```
#[macro_export]
macro_rules! addr_range {
    ($range:expr) => {
        $crate::IntoAddrRange::into_addr_range($range)
    };
}

/// Converts a standard library range into a virtual address range.
///
/// The output is [`AddrRange`], [`AddrRangeFrom`], or [`GeneralAddrRange`] with
/// [`VirtAddr`](crate::VirtAddr) endpoints, following [`IntoAddrRange`].
/// Reversed exclusive endpoints are preserved without validation.
///
/// # Example
///
/// ```
/// use memory_addr::{va_range, AddrRangeBounds};
///
/// let range = va_range!(0x1000..0x2000);
/// assert_eq!(range.start, 0x1000.into());
/// assert_eq!(range.end, 0x2000.into());
/// assert!(va_range!(..=usize::MAX).contains(memory_addr::va!(usize::MAX)));
/// ```
#[macro_export]
macro_rules! va_range {
    ($range:expr) => {
        $crate::IntoAddrRange::<$crate::VirtAddr>::into_addr_range($range)
    };
}

/// Converts a standard library range into a physical address range.
///
/// The output is [`AddrRange`], [`AddrRangeFrom`], or [`GeneralAddrRange`] with
/// [`PhysAddr`](crate::PhysAddr) endpoints, following [`IntoAddrRange`].
/// Reversed exclusive endpoints are preserved without validation.
///
/// # Example
///
/// ```
/// use memory_addr::{pa_range, AddrRangeBounds};
///
/// let range = pa_range!(0x1000..0x2000);
/// assert_eq!(range.start, 0x1000.into());
/// assert_eq!(range.end, 0x2000.into());
/// assert_eq!(pa_range!(usize::MAX..).size(), 1);
/// ```
#[macro_export]
macro_rules! pa_range {
    ($range:expr) => {
        $crate::IntoAddrRange::<$crate::PhysAddr>::into_addr_range($range)
    };
}
