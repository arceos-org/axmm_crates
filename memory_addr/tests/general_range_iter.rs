//! General address range iteration tests.
//!
//! Both iterator variants preserve page boundaries, validation, and exhaustion.

#![feature(const_trait_impl, const_iter, const_convert)]

use memory_addr::{
    AddrRange, AddrRangeBounds, AddrRangeFrom, AddrRangeIterator, GeneralAddrRange,
    GeneralAddrRangeIter,
};

/// Checks bounded iteration during constant evaluation.
///
/// The iterator enum preserves the bounded variant and stable exhaustion.
#[test]
fn const_bounded_iteration() {
    const PAGES: [Option<usize>; 4] = {
        let range = GeneralAddrRange::from(AddrRange::new(0usize, 2));
        let mut iter = GeneralAddrRangeIter::new(range, 1).unwrap();
        assert!(matches!(&iter, GeneralAddrRangeIter::Range(_)));
        [iter.next(), iter.next(), iter.next(), iter.next()]
    };
    assert_eq!(PAGES, [Some(0), Some(1), None, None]);
}

/// Checks iteration of the last page in the address space.
///
/// The iterator enum preserves overflow termination without emitting zero.
#[test]
fn const_tail_iteration() {
    const LAST: usize = usize::MAX - 0xfff;
    const PAGES: [Option<usize>; 3] = {
        let range = GeneralAddrRange::from(AddrRangeFrom::new(LAST));
        let mut iter = range.iter(0x1000).unwrap();
        assert!(matches!(&iter, GeneralAddrRangeIter::RangeFrom(_)));
        [iter.next(), iter.next(), iter.next()]
    };
    assert_eq!(PAGES, [Some(LAST), None, None]);
}

/// Checks empty-range iteration.
///
/// An aligned empty range produces an exhausted iterator, not a failure.
#[test]
fn const_empty_iteration() {
    const PAGE: Option<usize> = GeneralAddrRange::from(AddrRange::new(0usize, 0))
        .iter(0x1000)
        .unwrap()
        .next();
    assert_eq!(PAGE, None);
}

/// Checks rejection of invalid bounded ranges.
///
/// Wrapping does not validate endpoints, but iteration still does.
#[test]
fn const_invalid_range() {
    const {
        let range = GeneralAddrRange::from(AddrRange {
            start: 2usize,
            end: 1,
        });
        assert!(!range.is_valid());
        assert!(range.iter(1).is_none());
    }
}

/// Checks invalid page sizes for both variants.
///
/// Construction must forward failure instead of creating an iterator.
#[test]
fn const_invalid_page_sizes() {
    const {
        let bounded = GeneralAddrRange::from(AddrRange::new(0usize, 0));
        let tail = GeneralAddrRange::from(AddrRangeFrom::new(0usize));
        assert!(bounded.iter(0).is_none());
        assert!(bounded.iter(3).is_none());
        assert!(tail.iter(0).is_none());
        assert!(tail.iter(3).is_none());
    }
}

/// Checks endpoint alignment validation.
///
/// Both starts and the bounded exclusive end must be page-aligned.
#[test]
fn const_unaligned_endpoints() {
    const {
        assert!(GeneralAddrRange::from(AddrRange::new(1usize, 0x2000))
            .iter(0x1000)
            .is_none());
        assert!(GeneralAddrRange::from(AddrRange::new(0usize, 0x2001))
            .iter(0x1000)
            .is_none());
        assert!(GeneralAddrRange::from(AddrRangeFrom::new(1usize))
            .iter(0x1000)
            .is_none());
    }
}
