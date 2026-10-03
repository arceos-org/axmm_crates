//! Standard and semantic address range conversion tests.
//!
//! All supported source types, endpoint boundaries, iterator states, and macros
//! preserve conversion semantics.

#![feature(const_trait_impl, const_convert, const_cmp)]

use memory_addr::{
    addr_range, pa, pa_range, va, va_range, AddrRange, AddrRangeBounds, AddrRangeFrom,
    GeneralAddrRange, IntoAddrRange, PhysAddrRange, PhysAddrRangeFrom, VirtAddr, VirtAddrRange,
    VirtAddrRangeFrom,
};

memory_addr::def_usize_addr! {
    /// A runtime-only address for compatibility checks.
    ///
    /// Public range APIs must not require const address traits at runtime.
    #[derive(Debug)]
    type RuntimeAddr;
}

/// A non-copy endpoint supporting only runtime conversion.
///
/// No ordering or cloning requirement should be imposed on source
/// endpoints.
struct Endpoint(usize);

/// Converts a test endpoint into its stored integer value.
///
/// This conversion consumes the source without requiring it to be copyable.
impl From<Endpoint> for usize {
    fn from(endpoint: Endpoint) -> Self {
        endpoint.0
    }
}

/// Checks const conversion from integer endpoints.
///
/// Successful conversion preserves both semantic endpoints.
#[test]
fn const_range_conversion() {
    const RANGE: VirtAddrRange = (0x1000usize..0x2000).into_addr_range();
    assert_eq!(RANGE, VirtAddrRange::new(va!(0x1000), va!(0x2000)));
}

/// Checks preservation of reversed endpoints during const conversion.
///
/// Conversion preserves endpoints while validity is checked separately.
#[test]
#[allow(clippy::reversed_empty_ranges)]
fn const_range_conversion_preserves_reversed_endpoints() {
    const RANGE: VirtAddrRange = (2usize..1).into_addr_range();
    assert_eq!(RANGE.start, va!(2));
    assert_eq!(RANGE.end, va!(1));
    assert!(!RANGE.is_valid());
}

/// Checks an empty range conversion at the maximum address.
///
/// Equal endpoints are accepted even when no larger address is
/// representable.
#[test]
fn const_range_conversion_at_maximum() {
    const RANGE: VirtAddrRange = (usize::MAX..usize::MAX).into_addr_range();
    assert_eq!(RANGE.start, va!(usize::MAX));
    assert_eq!(RANGE.end, va!(usize::MAX));
}

/// Checks conversion from a Rust range during constant evaluation.
///
/// Both physical and virtual aliases preserve their semantic start.
#[test]
fn const_conversion() {
    const VIRTUAL: VirtAddrRangeFrom = (1usize..).into_addr_range();
    const PHYSICAL: PhysAddrRangeFrom = (1usize..).into_addr_range();
    assert_eq!(VIRTUAL.start(), va!(1));
    assert_eq!(PHYSICAL.start(), pa!(1));
}

/// Checks const conversion and bounded queries.
///
/// The enum preserves the concrete variant, endpoints, and size.
#[test]
fn const_bounded_conversion() {
    const RANGE: GeneralAddrRange<VirtAddr> =
        GeneralAddrRange::from(VirtAddrRange::new(va!(10), va!(20)));
    const {
        assert!(matches!(RANGE, GeneralAddrRange::Range(_)));
        assert!(RANGE.start() == va!(10));
        assert!(RANGE.checked_end() == Some(va!(20)));
        assert!(RANGE.checked_size() == Some(10));
        assert!(RANGE.end() == va!(20));
        assert!(RANGE.size() == 10);
    }
}

/// Checks const conversion of the final single-address range.
///
/// The unrepresentable end does not prevent a representable size.
#[test]
fn const_tail_conversion() {
    const RANGE: GeneralAddrRange<VirtAddr> =
        GeneralAddrRange::from(VirtAddrRangeFrom::new(va!(usize::MAX)));
    const {
        assert!(matches!(RANGE, GeneralAddrRange::RangeFrom(_)));
        assert!(RANGE.start() == va!(usize::MAX));
        assert!(RANGE.checked_end().is_none());
        assert!(RANGE.size() == 1);
        assert!(RANGE.contains(va!(usize::MAX)));
        assert!(!RANGE.is_empty());
    }
}

/// Checks the new half-open range conversion in const evaluation.
///
/// Both endpoints retain their original values.
#[test]
fn const_range() {
    const RANGE: VirtAddrRange = core::range::Range {
        start: 2usize,
        end: 5,
    }
    .into_addr_range();
    assert_eq!(RANGE, AddrRange::new(va!(2), va!(5)));
}

/// Checks the legacy half-open range conversion in const evaluation.
///
/// The output remains bounded even at the maximum representable endpoint.
#[test]
fn const_legacy_range() {
    const RANGE: PhysAddrRange = (0usize..usize::MAX).into_addr_range();
    assert_eq!(RANGE, AddrRange::new(pa!(0), pa!(usize::MAX)));
    assert!(!RANGE.contains(pa!(usize::MAX)));
}

/// Checks the new unbounded range conversion in const evaluation.
///
/// A start at the maximum address still contains one byte.
#[test]
fn const_range_from() {
    const RANGE: VirtAddrRangeFrom = core::range::RangeFrom { start: usize::MAX }.into_addr_range();
    assert_eq!(RANGE, AddrRangeFrom::new(va!(usize::MAX)));
    assert_eq!(RANGE.size(), 1);
}

/// Checks the legacy unbounded range conversion in const evaluation.
///
/// The converted range retains its inclusive start.
#[test]
fn const_legacy_range_from() {
    const RANGE: AddrRangeFrom<usize> = (5usize..).into_addr_range();
    assert_eq!(RANGE, AddrRangeFrom::new(5));
}

/// Checks the exclusive prefix conversion shared by both standard modules.
///
/// Zero remains a valid empty prefix and the upper endpoint is not
/// decremented.
#[test]
fn const_range_to() {
    const EMPTY: AddrRange<usize> = (..0usize).into_addr_range();
    const PREFIX: AddrRange<usize> = core::ops::RangeTo { end: usize::MAX }.into_addr_range();
    assert_eq!(EMPTY, AddrRange::new(0, 0));
    assert_eq!(PREFIX, AddrRange::new(0, usize::MAX));
}

/// Checks a new inclusive range with a representable exclusive end.
///
/// The final included byte advances the exclusive endpoint by one.
#[test]
fn const_range_inclusive() {
    const RANGE: GeneralAddrRange<usize> = core::range::RangeInclusive {
        start: 2usize,
        last: 5,
    }
    .into_addr_range();
    assert_eq!(RANGE, GeneralAddrRange::Range(AddrRange::new(2, 6)));
}

/// Checks a legacy inclusive range with a representable exclusive end.
///
/// Equal endpoints describe a singleton rather than an empty range.
#[test]
fn const_legacy_range_inclusive() {
    const RANGE: GeneralAddrRange<usize> = (5usize..=5).into_addr_range();
    assert_eq!(RANGE, GeneralAddrRange::Range(AddrRange::new(5, 6)));
}

/// Checks a new inclusive prefix conversion at zero.
///
/// The zero address is included, yielding a one-byte range.
#[test]
fn const_range_to_inclusive() {
    const RANGE: GeneralAddrRange<usize> =
        core::range::RangeToInclusive { last: 0usize }.into_addr_range();
    assert_eq!(RANGE, GeneralAddrRange::Range(AddrRange::new(0, 1)));
}

/// Checks a legacy inclusive prefix conversion below the maximum address.
///
/// An exclusive end at the maximum address is still representable.
#[test]
fn const_legacy_range_to_inclusive() {
    const RANGE: GeneralAddrRange<usize> = (..=usize::MAX - 1).into_addr_range();
    assert_eq!(
        RANGE,
        GeneralAddrRange::Range(AddrRange::new(0, usize::MAX))
    );
}

/// Checks full-range conversion through its type name and range syntax.
///
/// Both forms include every address without a representable exclusive end.
#[test]
fn const_range_full() {
    const NAMED: AddrRangeFrom<usize> = core::ops::RangeFull.into_addr_range();
    const SYNTAX: AddrRangeFrom<usize> = (..).into_addr_range();
    assert_eq!(NAMED, AddrRangeFrom::new(0));
    assert_eq!(NAMED, SYNTAX);
    assert_eq!(NAMED.checked_size(), None);
}

/// Checks inclusive upper bounds at the maximum address.
///
/// Both inclusive range types select the unbounded variant without
/// wrapping.
#[test]
fn const_maximum_inclusive_endpoint() {
    const NEW: GeneralAddrRange<usize> = core::range::RangeInclusive {
        start: usize::MAX,
        last: usize::MAX,
    }
    .into_addr_range();
    const LEGACY: GeneralAddrRange<usize> = (usize::MAX..=usize::MAX).into_addr_range();
    assert_eq!(
        NEW,
        GeneralAddrRange::RangeFrom(AddrRangeFrom::new(usize::MAX))
    );
    assert_eq!(NEW, LEGACY);
    assert_eq!(NEW.size(), 1);
}

/// Checks inclusive prefixes covering the full address space.
///
/// Both prefix types preserve the unrepresentable exclusive end and size.
#[test]
fn const_maximum_inclusive_prefix() {
    const NEW: GeneralAddrRange<usize> =
        core::range::RangeToInclusive { last: usize::MAX }.into_addr_range();
    const LEGACY: GeneralAddrRange<usize> = (..=usize::MAX).into_addr_range();
    assert_eq!(NEW, GeneralAddrRange::RangeFrom(AddrRangeFrom::new(0)));
    assert_eq!(NEW, LEGACY);
    assert_eq!(NEW.checked_size(), None);
}

/// Checks reversed exclusive endpoints without implicit validation.
///
/// Both source types preserve invalid bounds for explicit validation by the
/// caller.
#[test]
#[allow(clippy::reversed_empty_ranges)]
fn const_reversed_exclusive_range() {
    const NEW: AddrRange<usize> = core::range::Range {
        start: 5usize,
        end: 2,
    }
    .into_addr_range();
    const LEGACY: AddrRange<usize> = (5usize..2).into_addr_range();
    assert_eq!(NEW, AddrRange { start: 5, end: 2 });
    assert_eq!(NEW, LEGACY);
    assert!(!NEW.is_valid());
}

/// Checks reversed inclusive endpoints as empty sets.
///
/// Both source types produce a valid empty range at the converted start.
#[test]
#[allow(clippy::reversed_empty_ranges)]
fn const_reversed_inclusive_range() {
    const NEW: GeneralAddrRange<usize> = core::range::RangeInclusive {
        start: 5usize,
        last: 2,
    }
    .into_addr_range();
    const LEGACY: GeneralAddrRange<usize> = (5usize..=2).into_addr_range();
    assert_eq!(NEW, GeneralAddrRange::Range(AddrRange::new(5, 5)));
    assert_eq!(NEW, LEGACY);
}

/// Checks a legacy inclusive iterator after exhaustion.
///
/// Neither an ordinary singleton nor the maximum-address singleton may
/// reappear.
#[test]
fn exhausted_inclusive_range() {
    for address in [0usize, usize::MAX] {
        let mut source = address..=address;
        assert_eq!(source.next(), Some(address));
        let converted: GeneralAddrRange<usize> = source.into_addr_range();
        assert!(converted.is_valid());
        assert!(converted.is_empty());
    }
}

/// Checks a legacy inclusive iterator with items remaining.
///
/// Conversion respects consumption from both ends of the iterator.
#[test]
fn partially_consumed_inclusive_range() {
    let mut source = 2usize..=5;
    assert_eq!(source.next(), Some(2));
    assert_eq!(source.next_back(), Some(5));
    let converted: GeneralAddrRange<usize> = source.into_addr_range();
    assert_eq!(converted, GeneralAddrRange::Range(AddrRange::new(3, 5)));
}

/// Checks ownership transfer from non-copy source endpoints.
///
/// Both inclusive range implementations consume endpoints through `Into`.
#[test]
fn runtime_non_copy_endpoints() {
    let new: GeneralAddrRange<usize> = core::range::RangeInclusive {
        start: Endpoint(2),
        last: Endpoint(5),
    }
    .into_addr_range();
    let legacy: GeneralAddrRange<usize> = (Endpoint(2)..=Endpoint(5)).into_addr_range();
    assert_eq!(new, GeneralAddrRange::Range(AddrRange::new(2, 6)));
    assert_eq!(new, legacy);
}

/// Checks conversions targeting runtime-only address types.
///
/// Both bounded and unbounded outputs use the same trait implementations.
#[test]
fn runtime_address_type() {
    let bounded: AddrRange<RuntimeAddr> = (2usize..5).into_addr_range();
    let tail: GeneralAddrRange<RuntimeAddr> = (2usize..=usize::MAX).into_addr_range();
    assert_eq!(bounded.start, RuntimeAddr::from_usize(2));
    assert_eq!(bounded.end.as_usize(), 5);
    assert_eq!(tail.start(), RuntimeAddr::from(2));
    assert_eq!(tail.checked_end(), None);
}

/// Checks the common-output conversion method for every output
/// representation.
///
/// The default method preserves bounded, unbounded, and inclusive results
/// in const code.
#[test]
fn const_general_conversion() {
    const BOUNDED: GeneralAddrRange<usize> = (2usize..5).into_general_addr_range();
    const TAIL: GeneralAddrRange<usize> = (2usize..).into_general_addr_range();
    const INCLUSIVE: GeneralAddrRange<usize> = (2usize..=usize::MAX).into_general_addr_range();
    assert_eq!(BOUNDED, GeneralAddrRange::Range(AddrRange::new(2, 5)));
    assert_eq!(TAIL, GeneralAddrRange::RangeFrom(AddrRangeFrom::new(2)));
    assert_eq!(INCLUSIVE, TAIL);
}

/// Checks wrapping existing address ranges through their common trait.
///
/// Invalid bounds remain unchanged and general ranges are copied without
/// alteration.
#[test]
fn const_into_general() {
    const BOUNDED: GeneralAddrRange<usize> = AddrRange {
        start: 5usize,
        end: 2,
    }
    .into_general();
    const TAIL: GeneralAddrRange<usize> = AddrRangeFrom::new(usize::MAX).into_general();
    const COPY: GeneralAddrRange<usize> = TAIL.into_general();
    assert_eq!(
        BOUNDED,
        GeneralAddrRange::Range(AddrRange { start: 5, end: 2 })
    );
    assert_eq!(
        TAIL,
        GeneralAddrRange::RangeFrom(AddrRangeFrom::new(usize::MAX))
    );
    assert_eq!(COPY, TAIL);
}

/// Checks inferred address types across all macro output representations.
///
/// Each macro invocation remains usable during const evaluation.
#[test]
fn const_inferred_macro() {
    const BOUNDED: AddrRange<usize> = addr_range!(2usize..5);
    const TAIL: AddrRangeFrom<usize> = addr_range!(2usize..);
    const INCLUSIVE: GeneralAddrRange<usize> = addr_range!(2usize..=5);
    assert_eq!(BOUNDED, AddrRange::new(2, 5));
    assert_eq!(TAIL, AddrRangeFrom::new(2));
    assert_eq!(INCLUSIVE, GeneralAddrRange::Range(AddrRange::new(2, 6)));
}

/// Checks virtual-address macros with raw and typed endpoints.
///
/// Bounded, unbounded, inclusive, and full ranges select their documented
/// output types.
#[test]
fn const_virtual_macro() {
    const BOUNDED: VirtAddrRange = va_range!(va!(2)..va!(5));
    const TAIL: VirtAddrRangeFrom = va_range!(2usize..);
    const INCLUSIVE: GeneralAddrRange<memory_addr::VirtAddr> = va_range!(..=usize::MAX);
    const FULL: VirtAddrRangeFrom = va_range!(..);
    assert_eq!(BOUNDED, AddrRange::new(va!(2), va!(5)));
    assert_eq!(TAIL, AddrRangeFrom::new(va!(2)));
    assert_eq!(INCLUSIVE, FULL.into_general());
}

/// Checks physical-address macros with new and legacy source types.
///
/// The macro binds the address type on the trait rather than its method.
#[test]
fn const_physical_macro() {
    const BOUNDED: PhysAddrRange = pa_range!(core::range::Range {
        start: 2usize,
        end: 5
    });
    const PREFIX: PhysAddrRange = pa_range!(..5usize);
    const INCLUSIVE: GeneralAddrRange<memory_addr::PhysAddr> = pa_range!(pa!(2)..=pa!(5));
    assert_eq!(BOUNDED, AddrRange::new(pa!(2), pa!(5)));
    assert_eq!(PREFIX, AddrRange::new(pa!(0), pa!(5)));
    assert_eq!(
        INCLUSIVE,
        GeneralAddrRange::Range(AddrRange::new(pa!(2), pa!(6)))
    );
}
