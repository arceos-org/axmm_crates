#![no_std]
#![doc = include_str!("../README.md")]
// Const features.
#![feature(const_trait_impl)]
#![feature(const_clone)]
#![feature(const_closures)]
#![feature(const_cmp)]
#![feature(const_convert)]
#![feature(const_default)]
#![feature(const_destruct)]
#![feature(const_iter)]
#![feature(const_ops)]
#![feature(const_option_ops)]
#![feature(const_range)]
#![feature(const_range_bounds)]
#![feature(derive_const)]

mod addr;
mod iter;
mod range;

pub use self::addr::{MemoryAddr, PhysAddr, VirtAddr};
pub use self::iter::{DynPageIter, PageIter};
pub use self::range::{
    AddrRange, AddrRangeBounds, AddrRangeFrom, AddrRangeFromIter, AddrRangeIter, AddrRangeIterator,
    GeneralAddrRange, GeneralAddrRangeIter, IntoAddrRange, PhysAddrRange, PhysAddrRangeFrom,
    VirtAddrRange, VirtAddrRangeFrom,
};

/// The size of a 4K page (4096 bytes).
///
/// Used by the four-kilobyte alignment helpers and [`PageIter4K`].
pub const PAGE_SIZE_4K: usize = 0x1000;

/// The size of a 2M page (2097152 bytes).
///
/// Used by [`PageIter2M`] and available to callers selecting a page size.
pub const PAGE_SIZE_2M: usize = 0x20_0000;

/// The size of a 1G page (1073741824 bytes).
///
/// Used by [`PageIter1G`] and available to callers selecting a page size.
pub const PAGE_SIZE_1G: usize = 0x4000_0000;

/// A [`PageIter`] for 4K pages.
///
/// This compatibility alias uses [`PAGE_SIZE_4K`]. Prefer [`AddrRangeIter`] for
/// new code.
pub type PageIter4K<A> = PageIter<PAGE_SIZE_4K, A>;

/// A [`PageIter`] for 2M pages.
///
/// This compatibility alias uses [`PAGE_SIZE_2M`]. Prefer [`AddrRangeIter`] for
/// new code.
pub type PageIter2M<A> = PageIter<PAGE_SIZE_2M, A>;

/// A [`PageIter`] for 1G pages.
///
/// This compatibility alias uses [`PAGE_SIZE_1G`]. Prefer [`AddrRangeIter`] for
/// new code.
pub type PageIter1G<A> = PageIter<PAGE_SIZE_1G, A>;

/// Aligns an address downwards.
///
/// Returns the greatest `x` with alignment `align` so that `x <= addr`.
///
/// The alignment must be a nonzero power of two. This requirement is not
/// checked.
#[inline]
pub const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

/// Aligns an address upwards.
///
/// Returns the smallest `x` with alignment `align` so that `x >= addr`,
/// provided the computation does not overflow.
///
/// The alignment must be a nonzero power of two. This requirement is not
/// checked.
#[inline]
pub const fn align_up(addr: usize, align: usize) -> usize {
    // TODO: Avoid intermediate overflow for representable results and define final
    // overflow behavior.
    (addr + align - 1) & !(align - 1)
}

/// Returns the offset of the address within the alignment.
///
/// Equivalent to `addr % align` for a nonzero power-of-two alignment.
/// This requirement is not checked.
#[inline]
pub const fn align_offset(addr: usize, align: usize) -> usize {
    addr & (align - 1)
}

/// Checks whether an address has the requested alignment.
///
/// Equivalent to `addr % align == 0` for a nonzero power-of-two alignment.
/// This requirement is not checked.
#[inline]
pub const fn is_aligned(addr: usize, align: usize) -> bool {
    align_offset(addr, align) == 0
}

/// Aligns an address downwards to a 4K boundary.
///
/// Equivalent to [`align_down`] with [`PAGE_SIZE_4K`].
#[inline]
pub const fn align_down_4k(addr: usize) -> usize {
    align_down(addr, PAGE_SIZE_4K)
}

/// Aligns an address upwards to a 4K boundary.
///
/// Equivalent to [`align_up`] with [`PAGE_SIZE_4K`], including its overflow
/// behavior.
#[inline]
pub const fn align_up_4k(addr: usize) -> usize {
    align_up(addr, PAGE_SIZE_4K)
}

/// Returns the offset of the address within a 4K-sized page.
///
/// Equivalent to [`align_offset`] with [`PAGE_SIZE_4K`].
#[inline]
pub const fn align_offset_4k(addr: usize) -> usize {
    align_offset(addr, PAGE_SIZE_4K)
}

/// Checks whether the address is 4K-aligned.
///
/// Equivalent to [`is_aligned`] with [`PAGE_SIZE_4K`].
#[inline]
pub const fn is_aligned_4k(addr: usize) -> bool {
    is_aligned(addr, PAGE_SIZE_4K)
}
