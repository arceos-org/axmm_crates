use core::{fmt, ops::Range};

use crate::{MemoryAddr, PhysAddr, VirtAddr};

/// A range of a given memory address type `A`.
///
/// The range is inclusive on the start and exclusive on the end. A range is
/// considered empty if `start >= end`. An empty range has a size of `0`,
/// contains no addresses or ranges, overlaps with no ranges, and is contained
/// in no ranges.
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
#[derive(Clone, Copy, PartialEq, Eq)]
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
    /// Creates a new address range from the start and end addresses.
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
    #[inline]
    pub const fn new(start: A, end: A) -> Self {
        Self { start, end }
    }

    /// Creates a new address range from the start address and the size.
    ///
    /// This is equivalent to `AddrRange::new(start, start.wrapping_add(size))`,
    /// which means, if `start.wrapping_add(size)` overflows, the range will be
    /// empty.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::AddrRange;
    ///
    /// let range = AddrRange::from_start_size(0x1000usize, 0x1000);
    /// assert_eq!(range.start, 0x1000);
    /// assert_eq!(range.end, 0x2000);
    ///
    /// let empty_range = AddrRange::from_start_size(usize::MAX, 0x1);
    /// assert_eq!(empty_range.start, usize::MAX);
    /// assert_eq!(empty_range.end, 0);
    /// ```
    #[inline]
    pub fn from_start_size(start: A, size: usize) -> Self {
        Self {
            start,
            end: start.wrapping_add(size),
        }
    }

    /// Returns `true` if the range is empty (`start >= end`).
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::AddrRange;
    ///
    /// assert!(AddrRange::new(0x1000usize, 0x1000).is_empty());
    /// assert!(AddrRange::new(0x1000usize, 0x0fff).is_empty());
    /// assert!(!AddrRange::new(0x1000usize, 0x2000).is_empty());
    /// ```
    #[inline]
    pub fn is_empty(self) -> bool {
        self.start >= self.end
    }

    /// Returns the size of the range.
    ///
    /// Returns `0` if the range is empty, even if `start > end`.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::AddrRange;
    ///
    /// assert_eq!(AddrRange::new(0x1000usize, 0x1000).size(), 0);
    /// assert_eq!(AddrRange::new(0x1000usize, 0x0fff).size(), 0);
    /// assert_eq!(AddrRange::new(0x1000usize, 0x2000).size(), 0x1000);
    /// ```
    #[inline]
    pub fn size(self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.end.offset_from(self.start) as usize
        }
    }

    /// Checks if the range contains the given address.
    ///
    /// Always returns `false` if the range is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::AddrRange;
    ///
    /// let range = AddrRange::new(0x1000usize, 0x2000);
    /// assert!(!range.contains(0x0fff));
    /// assert!(range.contains(0x1000));
    /// assert!(range.contains(0x1fff));
    /// assert!(!range.contains(0x2000));
    ///
    /// let empty_range = AddrRange::new(0x2000usize, 0x1000);
    /// assert!(!empty_range.contains(0x0fff));
    /// assert!(!empty_range.contains(0x1000));
    /// assert!(!empty_range.contains(0x1fff));
    /// assert!(!empty_range.contains(0x2000));
    /// ```
    #[inline]
    pub fn contains(self, addr: A) -> bool {
        // this will always be `false` if the range is empty
        self.start <= addr && addr < self.end
    }

    /// Checks if the range contains the given address range.
    ///
    /// Always returns `false` if either range is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::AddrRange;
    ///
    /// let range = AddrRange::new(0x1000usize, 0x2000);
    /// assert!(!range.contains_range(AddrRange::from(0x0usize..0xfff)));
    /// assert!(!range.contains_range(AddrRange::from(0x0fffusize..0x1fff)));
    /// assert!(range.contains_range(AddrRange::from(0x1001usize..0x1fff)));
    /// assert!(range.contains_range(AddrRange::from(0x1000usize..0x2000)));
    /// assert!(!range.contains_range(AddrRange::from(0x1001usize..0x2001)));
    /// assert!(!range.contains_range(AddrRange::from(0x2001usize..0x3001)));
    ///
    /// let empty_range = AddrRange::new(0x2000usize, 0x1000);
    /// assert!(!empty_range.contains_range(range));
    /// assert!(!range.contains_range(empty_range));
    /// ```
    #[inline]
    pub fn contains_range(self, other: Self) -> bool {
        !self.is_empty() && !other.is_empty() && self.start <= other.start && other.end <= self.end
    }

    /// Checks if the range is contained in the given address range.
    ///
    /// Always returns `false` if either range is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::AddrRange;
    ///
    /// let range = AddrRange::new(0x1000usize, 0x2000);
    /// assert!(!range.contained_in(AddrRange::from(0xfffusize..0x1fff)));
    /// assert!(!range.contained_in(AddrRange::from(0x1001usize..0x2001)));
    /// assert!(range.contained_in(AddrRange::from(0xfffusize..0x2001)));
    /// assert!(range.contained_in(AddrRange::from(0x1000usize..0x2000)));
    ///
    /// let empty_range = AddrRange::new(0x2000usize, 0x1000);
    /// assert!(!empty_range.contained_in(range));
    /// assert!(!range.contained_in(empty_range));
    /// ```
    #[inline]
    pub fn contained_in(self, other: Self) -> bool {
        other.contains_range(self)
    }

    /// Checks if the range overlaps with the given address range.
    ///
    /// Always returns `false` if either range is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_addr::AddrRange;
    ///
    /// let range = AddrRange::new(0x1000usize, 0x2000usize);
    /// assert!(!range.overlaps(AddrRange::from(0xfffusize..0xfff)));
    /// assert!(!range.overlaps(AddrRange::from(0x2000usize..0x2000)));
    /// assert!(!range.overlaps(AddrRange::from(0xfffusize..0x1000)));
    /// assert!(range.overlaps(AddrRange::from(0xfffusize..0x1001)));
    /// assert!(range.overlaps(AddrRange::from(0x1fffusize..0x2001)));
    /// assert!(range.overlaps(AddrRange::from(0xfffusize..0x2001)));
    ///
    /// let empty_range = AddrRange::new(0x2000usize, 0x1000);
    /// assert!(!empty_range.overlaps(range));
    /// assert!(!range.overlaps(empty_range));
    /// ```
    #[inline]
    pub fn overlaps(self, other: Self) -> bool {
        !self.is_empty() && !other.is_empty() && self.start < other.end && other.start < self.end
    }
}

/// Conversion from [`Range`] to [`AddrRange`], provided that the type of the
/// endpoints can be converted to the address type `A`.
impl<A, T> From<Range<T>> for AddrRange<A>
where
    A: MemoryAddr + From<T>,
{
    #[inline]
    fn from(range: Range<T>) -> Self {
        Self::new(range.start.into(), range.end.into())
    }
}

/// Implementations of [`Default`] for [`AddrRange`].
///
/// The default value is an empty range `Range { start: 0, end: 0 }`.
impl<A> Default for AddrRange<A>
where
    A: MemoryAddr,
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
impl<A> fmt::Debug for AddrRange<A>
where
    A: MemoryAddr + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}..{:?}", self.start, self.end)
    }
}

/// Implementations of [`LowerHex`](fmt::LowerHex) for [`AddrRange`].
impl<A> fmt::LowerHex for AddrRange<A>
where
    A: MemoryAddr + fmt::LowerHex,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}..{:x}", self.start, self.end)
    }
}

/// Implementations of [`UpperHex`](fmt::UpperHex) for [`AddrRange`].
impl<A> fmt::UpperHex for AddrRange<A>
where
    A: MemoryAddr + fmt::UpperHex,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}..{:X}", self.start, self.end)
    }
}

/// A range of virtual addresses [`VirtAddr`].
pub type VirtAddrRange = AddrRange<VirtAddr>;
/// A range of physical addresses [`PhysAddr`].
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
