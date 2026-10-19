//! Bounded address range construction and representation tests.
//!
//! Constructors, size boundaries, formatting, and derives use representable
//! exclusive ends.

#![feature(const_trait_impl, const_clone, const_cmp, const_default)]

use core::mem::size_of;

use memory_addr::{va, va_range, AddrRange, AddrRangeBounds, PhysAddrRange, VirtAddrRange};

memory_addr::def_usize_addr! {
    /// A runtime-only address for compatibility checks.
    ///
    /// Public range APIs must not require const address traits at runtime.
    #[derive(Debug)]
    type RuntimeAddr;
}

/// Requires const equality for a range type.
///
/// `Eq` has no methods, so its const implementation is checked through a
/// bound.
const fn require_const_eq<T: [const] Eq>() {}

/// Checks fallible construction of a nonempty range during const evaluation.
///
/// Ordered endpoints are preserved by the successful result.
#[test]
fn const_try_new_nonempty() {
    const RANGE: Option<VirtAddrRange> = VirtAddrRange::try_new(va!(0x1000), va!(0x2000));
    assert_eq!(RANGE, Some(VirtAddrRange::new(va!(0x1000), va!(0x2000))));
}

/// Checks fallible construction of an empty range during const evaluation.
///
/// Equal endpoints remain valid even at the maximum address.
#[test]
fn const_try_new_empty() {
    const RANGE: Option<VirtAddrRange> = VirtAddrRange::try_new(va!(usize::MAX), va!(usize::MAX));
    assert_eq!(
        RANGE,
        Some(VirtAddrRange::new(va!(usize::MAX), va!(usize::MAX)))
    );
}

/// Checks rejection of reversed endpoints during const evaluation.
///
/// The fallible constructor returns `None` without panicking.
#[test]
fn const_try_new_reversed() {
    const RANGE: Option<VirtAddrRange> = VirtAddrRange::try_new(va!(0x2000), va!(0x1000));
    assert_eq!(RANGE, None);
}

/// Checks unchecked endpoint construction with a runtime-only address type.
///
/// Storing valid endpoints must remain possible during const evaluation.
#[test]
fn const_new_unchecked_runtime_endpoints() {
    // SAFETY: The start address precedes the end address.
    const RANGE: AddrRange<RuntimeAddr> = unsafe {
        AddrRange::new_unchecked(
            RuntimeAddr::from_usize(0x1000),
            RuntimeAddr::from_usize(0x2000),
        )
    };
    assert_eq!(RANGE.start.as_usize(), 0x1000);
    assert_eq!(RANGE.end.as_usize(), 0x2000);
}

/// Checks unchecked construction of an empty range at the maximum address.
///
/// Equal endpoints satisfy the unchecked constructor's ordering requirement.
#[test]
fn const_new_unchecked_empty() {
    // SAFETY: Equal endpoints satisfy `start <= end`.
    const RANGE: VirtAddrRange =
        unsafe { VirtAddrRange::new_unchecked(va!(usize::MAX), va!(usize::MAX)) };
    assert_eq!(RANGE.start, va!(usize::MAX));
    assert_eq!(RANGE.end, RANGE.start);
}

/// Checks unchecked size construction ending at the maximum address.
///
/// A representable exclusive end must also be computed correctly in const code.
#[test]
fn const_from_start_size_unchecked_boundary() {
    // SAFETY: Adding one to `usize::MAX - 1` does not overflow.
    const RANGE: VirtAddrRange =
        unsafe { VirtAddrRange::from_start_size_unchecked(va!(usize::MAX - 1), 1) };
    assert_eq!(RANGE.start, va!(usize::MAX - 1));
    assert_eq!(RANGE.end, va!(usize::MAX));
}

/// Checks unchecked zero-size construction at the maximum address.
///
/// A zero size preserves the start and does not overflow.
#[test]
fn const_from_start_size_unchecked_empty() {
    // SAFETY: Adding zero to `usize::MAX` does not overflow.
    const RANGE: VirtAddrRange =
        unsafe { VirtAddrRange::from_start_size_unchecked(va!(usize::MAX), 0) };
    assert_eq!(RANGE.start, va!(usize::MAX));
    assert_eq!(RANGE.end, RANGE.start);
}

/// Checks const range cloning with semantic endpoints.
///
/// Explicit cloning verifies the trait implementation rather than copying.
#[test]
#[allow(clippy::clone_on_copy)]
fn const_range_clone() {
    const RANGE: VirtAddrRange = VirtAddrRange::new(va!(0x1000), va!(0x2000)).clone();
    assert_eq!(RANGE.start, va!(0x1000));
    assert_eq!(RANGE.end, va!(0x2000));
}

/// Checks const range equality and inequality.
///
/// Both comparison traits remain available during constant evaluation.
#[test]
fn const_range_equality() {
    const EQUAL: bool = {
        require_const_eq::<VirtAddrRange>();
        VirtAddrRange::new(va!(1), va!(2)) == VirtAddrRange::new(va!(1), va!(2))
    };
    const DIFFERENT: bool =
        VirtAddrRange::new(va!(1), va!(2)) != VirtAddrRange::new(va!(1), va!(3));
    assert_eq!((EQUAL, DIFFERENT), (true, true));
}

/// Checks runtime ranges with ordinary endpoint implementations.
///
/// Const derives must preserve runtime cloning and equality support.
#[test]
#[allow(clippy::clone_on_copy)]
fn runtime_range_traits() {
    let range = AddrRange::new(RuntimeAddr::from_usize(1), RuntimeAddr::from_usize(2));
    assert!(range.clone() == range);
    assert_eq!(range.start.as_usize(), 1);
}

/// Checks range construction ending exactly at the maximum address.
///
/// Both checked constructors accept a representable exclusive end.
#[test]
fn const_range_size_boundary() {
    const RANGE: VirtAddrRange = VirtAddrRange::from_start_size(va!(usize::MAX - 1), 1);
    const CHECKED: Option<VirtAddrRange> =
        VirtAddrRange::try_from_start_size(va!(usize::MAX - 1), 1);
    assert_eq!(RANGE.end, va!(usize::MAX));
    assert_eq!(CHECKED, Some(RANGE));
}

/// Checks rejection of an overflowing range size during const evaluation.
///
/// `AddrRange` cannot represent the exclusive end of a one-byte range
/// starting at the maximum address. Such a range requires `AddrRangeFrom`.
#[test]
fn const_range_size_overflow() {
    const RANGE: Option<VirtAddrRange> = VirtAddrRange::try_from_start_size(va!(usize::MAX), 1);
    assert_eq!(RANGE, None);
}

/// Checks overflow rejection by the panicking range constructor.
///
/// The constructor must reject a wrapped end in both debug and release
/// builds.
#[test]
#[should_panic(expected = "overflows")]
fn range_size_overflow_panics() {
    let _ = VirtAddrRange::from_start_size(va!(usize::MAX), 1);
}

/// Rounds bounded ranges to page boundaries in both directions.
#[test]
fn range_alignment_rounds_endpoints() {
    let range = VirtAddrRange::new(va!(0x1234), va!(0x6789));

    assert_eq!(
        range.align_inwards(0x1000),
        Some(VirtAddrRange::new(va!(0x2000), va!(0x6000)))
    );
    assert_eq!(
        range.align_outwards(0x1000),
        Some(VirtAddrRange::new(va!(0x1000), va!(0x7000)))
    );
}

/// Rejects invalid alignment values and ranges with no aligned interior.
#[test]
fn range_alignment_rejects_invalid_or_empty_results() {
    let range = VirtAddrRange::new(va!(0x1001), va!(0x1fff));
    let empty = VirtAddrRange::new(va!(0x2000), va!(0x2001));

    assert_eq!(range.align_inwards(0), None);
    assert_eq!(range.align_inwards(3), None);
    assert_eq!(range.align_inwards(0x1000), None);
    assert_eq!(
        empty.align_inwards(0x1000),
        Some(VirtAddrRange::new(va!(0x2000), va!(0x2000)))
    );
}

/// Checked endpoint rounding reports overflow instead of producing a wrapped
/// range.
#[test]
fn range_alignment_reports_endpoint_overflow() {
    let range = VirtAddrRange::new(va!(usize::MAX - 1), va!(usize::MAX));

    assert_eq!(range.align_inwards(4), None);
    assert_eq!(range.align_outwards(4), None);
}

/// Checks formatting of bounded virtual address ranges.
///
/// Both endpoints preserve their semantic address prefixes.
#[test]
fn test_range_format() {
    let range = va_range!(0xfec000..0xfff000usize);

    assert_eq!(format!("{:?}", range), "VA:0xfec000..VA:0xfff000");
    assert_eq!(format!("{:x}", range), "VA:0xfec000..VA:0xfff000");
    assert_eq!(format!("{:X}", range), "VA:0xFEC000..VA:0xFFF000");
}

/// Checks endpoints and length of a nonempty range.
///
/// Construction through the macro must preserve the requested bounds.
#[test]
fn test_range_endpoints_and_size() {
    let start = va!(0x1000);
    let end = va!(0x2000);
    let range = va_range!(start..end);

    assert!(!range.is_empty());

    assert_eq!(range.start, start);
    assert_eq!(range.end, end);
    assert_eq!(range.size(), 0x1000);
}

/// Checks default range initialization during const evaluation.
///
/// Both endpoints and the length are zero.
#[test]
fn const_range_default() {
    const DEFAULT: VirtAddrRange = VirtAddrRange::default();
    let default_range = DEFAULT;
    assert!(default_range.is_empty());
    assert_eq!(default_range.size(), 0);
    assert_eq!(default_range.start, va!(0));
    assert_eq!(default_range.end, va!(0));
}

/// Checks the storage cost of bounded semantic address ranges.
///
/// Two endpoints occupy two machine words without an additional tag.
#[test]
fn bounded_range_size() {
    assert_eq!(size_of::<VirtAddrRange>(), 2 * size_of::<usize>());
    assert_eq!(size_of::<PhysAddrRange>(), 2 * size_of::<usize>());
}
