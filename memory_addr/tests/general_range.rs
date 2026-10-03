//! General address range representation tests.
//!
//! Both enum variants retain their endpoints, equality, and formatting.

#![feature(const_trait_impl, const_clone, const_cmp, const_convert)]

use memory_addr::{AddrRange, AddrRangeBounds, AddrRangeFrom, GeneralAddrRange};

/// Checks the unrepresentable full-space size.
///
/// The enum preserves the distinction between a missing end and an empty
/// range.
#[test]
fn const_full_space() {
    const {
        let range = GeneralAddrRange::from(AddrRangeFrom::new(0usize));
        assert!(range.checked_end().is_none());
        assert!(range.checked_size().is_none());
        assert!(range.is_valid());
        assert!(!range.is_empty());
    }
}

/// Checks const cloning and structural equality.
///
/// Variant and endpoint differences remain observable after wrapping.
#[test]
#[allow(clippy::clone_on_copy)]
fn const_clone_and_equality() {
    const {
        let bounded = GeneralAddrRange::from(AddrRange::new(0usize, usize::MAX));
        let tail = GeneralAddrRange::from(AddrRangeFrom::new(0usize));
        assert!(bounded.clone() == bounded);
        assert!(tail.clone() == tail);
        assert!(bounded != tail);
        assert!(tail != GeneralAddrRange::from(AddrRangeFrom::new(1usize)));
    }
}

/// Checks formatting of both representations.
///
/// Debug exposes the variant, while hexadecimal output follows the inner
/// range.
#[test]
fn formatting() {
    let bounded = GeneralAddrRange::from(AddrRange::new(0xabusize, 0xcd));
    let tail = GeneralAddrRange::from(AddrRangeFrom::new(0xabusize));
    assert_eq!(format!("{bounded:?}"), "Range(171..205)");
    assert_eq!(format!("{tail:?}"), "RangeFrom(171..)");
    assert_eq!(format!("{bounded:x}"), "ab..cd");
    assert_eq!(format!("{bounded:X}"), "AB..CD");
    assert_eq!(format!("{tail:x}"), "ab..");
    assert_eq!(format!("{tail:X}"), "AB..");
}

/// Delegates alignment operations to the represented range variant.
#[test]
fn general_range_alignment_delegates() {
    let bounded = GeneralAddrRange::from(AddrRange::new(0x1234usize, 0x6789));
    let open = GeneralAddrRange::from(AddrRangeFrom::new(0x2345usize));

    assert_eq!(
        bounded.align_inwards(0x1000),
        Some(GeneralAddrRange::from(AddrRange::new(0x2000, 0x6000)))
    );
    assert_eq!(
        bounded.align_outwards(0x1000),
        Some(GeneralAddrRange::from(AddrRange::new(0x1000, 0x7000)))
    );
    assert_eq!(
        open.align_inwards(0x1000),
        Some(GeneralAddrRange::from(AddrRangeFrom::new(0x3000)))
    );
    assert_eq!(
        open.align_outwards(0x1000),
        Some(GeneralAddrRange::from(AddrRangeFrom::new(0x2000)))
    );
}
