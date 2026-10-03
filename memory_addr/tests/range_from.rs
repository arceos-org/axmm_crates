//! Open-ended address range construction and representation tests.
//!
//! Ranges through the maximum address distinguish unrepresentable ends and
//! sizes.

#![feature(const_trait_impl, const_clone, const_cmp)]

use core::mem::size_of;

use memory_addr::{va, AddrRangeBounds, AddrRangeFrom, PhysAddrRangeFrom, VirtAddrRangeFrom};

memory_addr::def_usize_addr! {
    /// A runtime-only address for compatibility checks.
    ///
    /// Public range APIs must not require const address traits at runtime.
    #[derive(Debug)]
    type RuntimeAddr;
}

/// Checks construction without const address operations.
///
/// Storing an endpoint is const even for an otherwise runtime-only type.
#[test]
fn const_constructor_with_runtime_address() {
    const RANGE: AddrRangeFrom<RuntimeAddr> = AddrRangeFrom::new(RuntimeAddr::from_usize(1));
    assert_eq!(RANGE.start().as_usize(), 1);
    assert_eq!(RANGE.size(), usize::MAX);
    assert!(RANGE.contains(RuntimeAddr::from_usize(usize::MAX)));
}

/// Checks the single-address range at the upper boundary.
///
/// Its exclusive end is unrepresentable even though its size is one.
#[test]
fn const_maximum_address() {
    const RANGE: VirtAddrRangeFrom = VirtAddrRangeFrom::new(va!(usize::MAX));
    const SIZE: usize = RANGE.size();
    const END: Option<memory_addr::VirtAddr> = RANGE.checked_end();
    const CONTAINS_MAX: bool = RANGE.contains(va!(usize::MAX));
    const CONTAINS_BEFORE: bool = RANGE.contains(va!(usize::MAX - 1));
    assert_eq!(SIZE, 1);
    assert_eq!(END, None);
    const { assert!(CONTAINS_MAX) };
    const { assert!(!CONTAINS_BEFORE) };
}

/// Checks the full address space without overflowing.
///
/// A range starting at zero is nonempty but has no representable size.
#[test]
fn const_full_address_space() {
    const RANGE: AddrRangeFrom<usize> = AddrRangeFrom::new(0);
    const SIZE: Option<usize> = RANGE.checked_size();
    const EMPTY: bool = RANGE.is_empty();
    assert_eq!(SIZE, None);
    const { assert!(!EMPTY) };
    assert!(RANGE.contains(0));
    assert!(RANGE.contains(usize::MAX));
}

/// Checks the largest representable range size.
///
/// Starting at one leaves exactly `usize::MAX` addresses.
#[test]
fn const_maximum_size() {
    const SIZE: Option<usize> = AddrRangeFrom::new(1usize).checked_size();
    assert_eq!(SIZE, Some(usize::MAX));
}

/// Checks failure when requesting an unrepresentable end.
///
/// The panicking query must not wrap the end to zero.
#[test]
#[should_panic(expected = "end")]
fn end_panics() {
    AddrRangeFrom::new(usize::MAX).end();
}

/// Checks failure when requesting the full address space size.
///
/// The panicking query must not report a zero-sized range.
#[test]
#[should_panic(expected = "size")]
fn full_size_panics() {
    AddrRangeFrom::new(0usize).size();
}

/// Checks const cloning and equality.
///
/// Derives retain const support on the new range type.
#[test]
#[allow(clippy::clone_on_copy)]
fn const_clone_and_equality() {
    const RANGE: VirtAddrRangeFrom = VirtAddrRangeFrom::new(va!(1)).clone();
    const EQUAL: bool = RANGE == VirtAddrRangeFrom::new(va!(1));
    const DIFFERENT: bool = RANGE != VirtAddrRangeFrom::new(va!(2));
    const { assert!(EQUAL) };
    const { assert!(DIFFERENT) };
}

/// Checks open-ended range formatting.
///
/// Each formatter preserves the address type's representation.
#[test]
fn formatting() {
    let range = VirtAddrRangeFrom::new(va!(0xabc));
    assert_eq!(format!("{range:?}"), "VA:0xabc..");
    assert_eq!(format!("{range:x}"), "VA:0xabc..");
    assert_eq!(format!("{range:X}"), "VA:0xABC..");
}

/// Checks the storage cost of open-ended semantic address ranges.
///
/// The implicit exclusive end requires no storage beyond the start.
#[test]
fn open_ended_range_size() {
    assert_eq!(size_of::<VirtAddrRangeFrom>(), size_of::<usize>());
    assert_eq!(size_of::<PhysAddrRangeFrom>(), size_of::<usize>());
}

/// Rounds the inclusive start of an open-ended range in both directions.
#[test]
fn open_ended_alignment_rounds_start() {
    let range = VirtAddrRangeFrom::new(va!(0x2345));

    assert_eq!(
        range.align_inwards(0x1000),
        Some(VirtAddrRangeFrom::new(va!(0x3000)))
    );
    assert_eq!(
        range.align_outwards(0x1000),
        Some(VirtAddrRangeFrom::new(va!(0x2000)))
    );
}

/// Open-ended alignment rejects invalid values and upward overflow.
#[test]
fn open_ended_alignment_rejects_invalid_or_overflowing_inputs() {
    let range = VirtAddrRangeFrom::new(va!(0x2345));

    assert_eq!(range.align_inwards(0), None);
    assert_eq!(range.align_outwards(3), None);
    assert_eq!(
        VirtAddrRangeFrom::new(va!(usize::MAX - 1)).align_inwards(4),
        None
    );
}
