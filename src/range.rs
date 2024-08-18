use core::{fmt, ops::Range};

use crate::{MemoryAddr, PhysAddr, VirtAddr};

macro_rules! usize {
    ($addr:expr) => {
        Into::<usize>::into($addr)
    };
}

/// A range of a given memory address type `A`.
///
/// The range is inclusive on the start and exclusive on the end.
/// It is empty if `start >= end`.
///
/// ## Example
///
/// ```
/// use memory_addr::{AddrRange, VirtAddr};
///
/// let range = AddrRange::<VirtAddr>::new(0x1000.into(), 0x2000.into());
/// assert_eq!(range.start, 0x1000.into());
/// assert_eq!(range.end, 0x2000.into());
/// ```
#[derive(Clone, Copy)]
pub struct AddrRange<A: MemoryAddr> {
    /// The lower bound of the range (inclusive).
    pub start: A,
    /// The upper bound of the range (exclusive).
    pub end: A,
}

/// Methods for [`AddrRange`].
impl<A> AddrRange<A>
where
    A: MemoryAddr,
{
    /// Creates a new address range.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::VirtAddrRange;
    ///
    /// let range = VirtAddrRange::new(0x1000.into(), 0x2000.into());
    /// assert_eq!(range.start, 0x1000.into());
    /// assert_eq!(range.end, 0x2000.into());
    /// ```
    #[inline]
    pub const fn new(start: A, end: A) -> Self {
        Self { start, end }
    }

    /// Creates a new address range from the start address and the size.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::VirtAddrRange;
    ///
    /// let range = VirtAddrRange::from_start_size(0x1000.into(), 0x1000);
    /// assert_eq!(range.start, 0x1000.into());
    /// assert_eq!(range.end, 0x2000.into());
    /// ```
    #[inline]
    pub fn from_start_size(start: A, size: usize) -> Self {
        Self {
            start,
            end: A::from(usize!(start) + size),
        }
    }

    /// Returns `true` if the range is empty (`start >= end`).
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::va_range;
    ///
    /// assert!(va_range!(0x1000..0x1000).is_empty());
    /// assert!(va_range!(0x1000..0xfff).is_empty());
    /// assert!(!va_range!(0x1000..0x2000).is_empty());
    /// ```
    #[inline]
    pub fn is_empty(self) -> bool {
        usize!(self.start) >= usize!(self.end)
    }

    /// Returns the size of the range.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::va_range;
    ///
    /// let range = va_range!(0x1000..0x2000);
    /// assert_eq!(range.size(), 0x1000);
    /// ```
    #[inline]
    pub fn size(self) -> usize {
        usize!(self.end) - usize!(self.start)
    }

    /// Checks if the range contains the given address.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::va_range;
    ///
    /// let range = va_range!(0x1000..0x2000);
    /// assert!(!range.contains(0xfff.into()));
    /// assert!(range.contains(0x1000.into()));
    /// assert!(range.contains(0x1fff.into()));
    /// assert!(!range.contains(0x2000.into()));
    /// ```
    #[inline]
    pub fn contains(self, addr: A) -> bool {
        usize!(self.start) <= usize!(addr) && usize!(addr) < usize!(self.end)
    }

    /// Checks if the range contains the given address range.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::va_range;
    ///
    /// let range = va_range!(0x1000..0x2000);
    /// assert!(!range.contains_range(va_range!(0x0..0xfff)));
    /// assert!(!range.contains_range(va_range!(0xfff..0x1fff)));
    /// assert!(range.contains_range(va_range!(0x1001..0x1fff)));
    /// assert!(range.contains_range(va_range!(0x1000..0x2000)));
    /// assert!(!range.contains_range(va_range!(0x1001..0x2001)));
    /// assert!(!range.contains_range(va_range!(0x2001..0x3001)));
    /// ```
    #[inline]
    pub fn contains_range(self, other: Self) -> bool {
        usize!(self.start) <= usize!(other.start) && usize!(other.end) <= usize!(self.end)
    }

    /// Checks if the range is contained in the given address range.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::va_range;
    ///
    /// let range = va_range!(0x1000..0x2000);
    /// assert!(!range.contained_in(va_range!(0xfff..0x1fff)));
    /// assert!(!range.contained_in(va_range!(0x1001..0x2001)));
    /// assert!(range.contained_in(va_range!(0xfff..0x2001)));
    /// assert!(range.contained_in(va_range!(0x1000..0x2000)));
    /// ```
    #[inline]
    pub fn contained_in(self, other: Self) -> bool {
        other.contains_range(self)
    }

    /// Checks if the range overlaps with the given address range.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::va_range;
    ///
    /// let range = va_range!(0x1000..0x2000);
    /// assert!(!range.overlaps(va_range!(0xfff..0xfff)));
    /// assert!(!range.overlaps(va_range!(0x2000..0x2000)));
    /// assert!(!range.overlaps(va_range!(0xfff..0x1000)));
    /// assert!(range.overlaps(va_range!(0xfff..0x1001)));
    /// assert!(range.overlaps(va_range!(0x1fff..0x2001)));
    /// assert!(range.overlaps(va_range!(0xfff..0x2001)));
    /// ```
    #[inline]
    pub fn overlaps(self, other: Self) -> bool {
        usize!(self.start) < usize!(other.end) && usize!(other.start) < usize!(self.end)
    }
}

/// Implementations of [`Default`] for [`AddrRange`].
///
/// The default value is an empty range `0..0`.
impl<A> Default for AddrRange<A>
where
    A: MemoryAddr,
{
    #[inline]
    fn default() -> Self {
        Self::new(0.into(), 0.into())
    }
}

/// Implementations of [`PartialEq`] for [`AddrRange`].
///
/// Two ranges are equal iff their start and end addresses are equal.
impl<A> PartialEq for AddrRange<A>
where
    A: MemoryAddr,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        usize!(self.start) == usize!(other.start) && usize!(self.end) == usize!(other.end)
    }
}

/// Implementations of [`Eq`] for [`AddrRange`].
impl<A> Eq for AddrRange<A> where A: MemoryAddr {}

/// Implementations of [`From`] for [`AddrRange`] and [`Range`].
///
/// Converts a range into an address range.
impl<A, T> From<Range<T>> for AddrRange<A>
where
    A: MemoryAddr + From<T>,
{
    #[inline]
    fn from(range: Range<T>) -> Self {
        Self::new(range.start.into(), range.end.into())
    }
}

/// Implementations of [`fmt::Debug`] for [`AddrRange`].
impl<A> fmt::Debug for AddrRange<A>
where
    A: MemoryAddr,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // todo: maybe use <A as fmt::Debug>::fmt?
        write!(f, "{:#x?}..{:#x?}", usize!(self.start), usize!(self.end))
    }
}

/// A range of virtual addresses.
pub type VirtAddrRange = AddrRange<VirtAddr>;
/// A range of physical addresses.
pub type PhysAddrRange = AddrRange<PhysAddr>;

/// Converts the given range expression into [`VirtAddrRange`].
///
/// # Example
///
/// ```
/// use memory_addr::va_range;
///
/// let range = va_range!(0x1000..0x2000);
/// assert_eq!(range.start, 0x1000.into());
/// assert_eq!(range.end, 0x2000.into());
/// ```
#[macro_export]
macro_rules! va_range {
    ($range:expr) => {
        $crate::VirtAddrRange::from($range)
    };
}

/// Converts the given range expression into [`PhysAddrRange`].
///
/// # Example
///
/// ```
/// use memory_addr::pa_range;
///
/// let range = pa_range!(0x1000..0x2000);
/// assert_eq!(range.start, 0x1000.into());
/// assert_eq!(range.end, 0x2000.into());
/// ```
#[macro_export]
macro_rules! pa_range {
    ($range:expr) => {
        $crate::PhysAddrRange::from($range)
    };
}
