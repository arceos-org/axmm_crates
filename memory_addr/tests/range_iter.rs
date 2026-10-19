//! Bounded address range iteration tests.
//!
//! Complete pages exclude the upper bound and reject invalid parameters.

#![feature(const_trait_impl, const_iter)]

use memory_addr::{
    va, AddrRange, AddrRangeBounds, AddrRangeIter, AddrRangeIterator, VirtAddrRange,
};

memory_addr::def_usize_addr! {
    /// A runtime-only address for compatibility checks.
    ///
    /// Public range APIs must not require const address traits at runtime.
    #[derive(Debug)]
    type RuntimeAddr;
}

/// Checks const iteration over a bounded range.
///
/// The exclusive end is omitted and exhaustion is stable.
#[test]
fn const_page_sequence() {
    const PAGES: [Option<memory_addr::VirtAddr>; 4] = {
        let mut iter = VirtAddrRange::new(va!(0x1000), va!(0x3000))
            .iter(0x1000)
            .unwrap();
        [iter.next(), iter.next(), iter.next(), iter.next()]
    };
    assert_eq!(PAGES, [Some(va!(0x1000)), Some(va!(0x2000)), None, None]);
}

/// Checks direct construction through the iterator trait.
///
/// Runtime-only endpoint traits remain supported.
#[test]
fn runtime_page_sequence() {
    let range = AddrRange::new(RuntimeAddr::from_usize(0), RuntimeAddr::from_usize(2));
    let pages: Vec<_> = AddrRangeIter::new(range, 1)
        .unwrap()
        .map(RuntimeAddr::as_usize)
        .collect();
    assert_eq!(pages, [0, 1]);
}

/// Checks an empty aligned range during constant evaluation.
///
/// Neither the start nor the end is emitted.
#[test]
fn const_empty_page_sequence() {
    const PAGES: [Option<usize>; 2] = {
        let mut iter = AddrRange::new(0x1000usize, 0x1000).iter(0x1000).unwrap();
        [iter.next(), iter.next()]
    };
    assert_eq!(PAGES, [None, None]);
}

/// Checks the last representable bounded endpoint.
///
/// Advancing to the end does not wrap or yield the end itself.
#[test]
fn const_page_sequence_at_maximum() {
    const PAGES: [Option<usize>; 3] = {
        let mut iter = AddrRange::new(usize::MAX - 1, usize::MAX).iter(1).unwrap();
        [iter.next(), iter.next(), iter.next()]
    };
    assert_eq!(PAGES, [Some(usize::MAX - 1), None, None]);
}

/// Checks rejection of a zero page size.
///
/// Empty ranges still validate iterator parameters.
#[test]
fn iterator_rejects_zero_page_size() {
    const { assert!(AddrRange::new(0usize, 0).iter(0).is_none()) };
}

/// Checks rejection of a non-power-of-two page size.
///
/// Otherwise-aligned endpoints do not make the page size valid.
#[test]
fn iterator_rejects_non_power_of_two() {
    const { assert!(AddrRange::new(0usize, 0).iter(3).is_none()) };
}

/// Checks rejection of an unaligned start.
///
/// Iteration must not silently round down to include preceding addresses.
#[test]
fn iterator_rejects_unaligned_start() {
    const { assert!(AddrRange::new(1usize, 0x2000).iter(0x1000).is_none()) };
}

/// Checks rejection of an unaligned end.
///
/// Iteration must not include a page that extends past the range.
#[test]
fn iterator_rejects_unaligned_end() {
    const { assert!(AddrRange::new(0usize, 0x2001).iter(0x1000).is_none()) };
}

/// Checks rejection of reversed endpoints.
///
/// Iterator construction diagnoses invalid ranges instead of treating them
/// as empty.
#[test]
fn iterator_rejects_invalid_range() {
    const {
        assert!(AddrRange {
            start: 0x2000usize,
            end: 0x1000,
        }
        .iter(0x1000)
        .is_none())
    };
}
