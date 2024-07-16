#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../README.md")]

mod addr;

pub use self::addr::{PhysAddr, VirtAddr};

/// The size of a 4K page (4096 bytes).
pub const PAGE_SIZE_4K: usize = 0x1000;

/// Align address downwards.
///
/// Returns the greatest `x` with alignment `align` so that `x <= addr`.
///
/// The alignment must be a power of two.
#[inline]
pub const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

/// Align address upwards.
///
/// Returns the smallest `x` with alignment `align` so that `x >= addr`.
///
/// The alignment must be a power of two.
#[inline]
pub const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Returns the offset of the address within the alignment.
///
/// Equivalent to `addr % align`, but the alignment must be a power of two.
#[inline]
pub const fn align_offset(addr: usize, align: usize) -> usize {
    addr & (align - 1)
}

/// Checks whether the address has the demanded alignment.
///
/// Equivalent to `addr % align == 0`, but the alignment must be a power of two.
#[inline]
pub const fn is_aligned(addr: usize, align: usize) -> bool {
    align_offset(addr, align) == 0
}

/// Align address downwards to 4096 (bytes).
#[inline]
pub const fn align_down_4k(addr: usize) -> usize {
    align_down(addr, PAGE_SIZE_4K)
}

/// Align address upwards to 4096 (bytes).
#[inline]
pub const fn align_up_4k(addr: usize) -> usize {
    align_up(addr, PAGE_SIZE_4K)
}

/// Returns the offset of the address within a 4K-sized page.
#[inline]
pub const fn align_offset_4k(addr: usize) -> usize {
    align_offset(addr, PAGE_SIZE_4K)
}

/// Checks whether the address is 4K-aligned.
#[inline]
pub const fn is_aligned_4k(addr: usize) -> bool {
    is_aligned(addr, PAGE_SIZE_4K)
}

#[cfg(test)]
mod tests {
    use crate::va;

    #[test]
    fn test_addr() {
        let addr = va!(0x2000);
        assert!(addr.is_aligned_4k());
        assert!(!addr.is_aligned(0x10000usize));
        assert_eq!(addr.align_offset_4k(), 0);
        assert_eq!(addr.align_down_4k(), va!(0x2000));
        assert_eq!(addr.align_up_4k(), va!(0x2000));

        let addr = va!(0x2fff);
        assert!(!addr.is_aligned_4k());
        assert_eq!(addr.align_offset_4k(), 0xfff);
        assert_eq!(addr.align_down_4k(), va!(0x2000));
        assert_eq!(addr.align_up_4k(), va!(0x3000));

        let align = 0x100000;
        let addr = va!(align * 5) + 0x2000;
        assert!(addr.is_aligned_4k());
        assert!(!addr.is_aligned(align));
        assert_eq!(addr.align_offset(align), 0x2000);
        assert_eq!(addr.align_down(align), va!(align * 5));
        assert_eq!(addr.align_up(align), va!(align * 6));
    }
}
