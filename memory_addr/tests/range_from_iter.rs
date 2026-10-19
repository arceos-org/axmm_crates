//! Open-ended address range iteration tests.
//!
//! The maximum address is included without wrapping and exhaustion remains
//! permanent.

#![feature(const_trait_impl, const_iter)]

use memory_addr::{
    pa, AddrRangeBounds, AddrRangeFrom, AddrRangeFromIter, AddrRangeIterator, PhysAddrRangeFrom,
};

memory_addr::def_usize_addr! {
    /// A runtime-only address for compatibility checks.
    ///
    /// Public range APIs must not require const address traits at runtime.
    #[derive(Debug)]
    type RuntimeAddr;
}

/// Checks const iteration over the final two pages.
///
/// The last page is returned before overflow marks permanent exhaustion.
#[test]
fn const_final_pages() {
    const START: usize = usize::MAX - 0x1fff;
    const PAGES: [Option<memory_addr::PhysAddr>; 4] = {
        let mut iter = PhysAddrRangeFrom::new(pa!(START)).iter(0x1000).unwrap();
        [iter.next(), iter.next(), iter.next(), iter.next()]
    };
    assert_eq!(
        PAGES,
        [Some(pa!(START)), Some(pa!(usize::MAX - 0xfff)), None, None]
    );
}

/// Checks iteration of the final individual address.
///
/// A page size of one includes the maximum address exactly once.
#[test]
fn const_maximum_address_page() {
    const PAGES: [Option<usize>; 3] = {
        let mut iter = AddrRangeFrom::new(usize::MAX).iter(1).unwrap();
        [iter.next(), iter.next(), iter.next()]
    };
    assert_eq!(PAGES, [Some(usize::MAX), None, None]);
}

/// Checks a complete address space in two pages.
///
/// Neither construction nor iteration requires a representable total byte
/// size.
#[test]
fn const_full_space_pages() {
    const PAGE_SIZE: usize = 1usize << (usize::BITS - 1);
    const PAGES: [Option<usize>; 4] = {
        let mut iter = AddrRangeFrom::new(0usize).iter(PAGE_SIZE).unwrap();
        [iter.next(), iter.next(), iter.next(), iter.next()]
    };
    assert_eq!(PAGES, [Some(0), Some(PAGE_SIZE), None, None]);
}

/// Checks initial iteration of the full byte-addressed space.
///
/// The unrepresentable total page count must not cause premature
/// exhaustion.
#[test]
fn const_full_space_byte_prefix() {
    const PAGES: [Option<usize>; 3] = {
        let mut iter = AddrRangeFrom::new(0usize).iter(1).unwrap();
        [iter.next(), iter.next(), iter.next()]
    };
    assert_eq!(PAGES, [Some(0), Some(1), Some(2)]);
}

/// Checks direct construction with runtime-only addresses.
///
/// The final address is yielded without requiring const endpoint
/// conversions.
#[test]
fn runtime_final_page() {
    let range = AddrRangeFrom::new(RuntimeAddr::from_usize(usize::MAX));
    let pages: Vec<_> = AddrRangeFromIter::new(range, 1)
        .unwrap()
        .map(RuntimeAddr::as_usize)
        .collect();
    assert_eq!(pages, [usize::MAX]);
}

/// Checks rejection of a zero page size.
///
/// A non-advancing page iterator must not be constructed.
#[test]
fn iterator_rejects_zero_page_size() {
    const { assert!(AddrRangeFrom::new(0usize).iter(0).is_none()) };
}

/// Checks rejection of a non-power-of-two page size.
///
/// The conceptual exclusive end must be page-aligned.
#[test]
fn iterator_rejects_non_power_of_two() {
    const { assert!(AddrRangeFrom::new(0usize).iter(3).is_none()) };
}

/// Checks rejection of an unaligned start.
///
/// Iteration never rounds the start to an earlier or later page.
#[test]
fn iterator_rejects_unaligned_start() {
    const { assert!(AddrRangeFrom::new(usize::MAX).iter(0x1000).is_none()) };
}
