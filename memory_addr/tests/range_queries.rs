//! Address range membership and relation tests.
//!
//! Bounded, open-ended, and general ranges share validity, containment, and
//! overlap semantics.

#![feature(const_trait_impl, const_convert)]

use memory_addr::{
    addr_range, va, va_range, AddrRange, AddrRangeBounds, AddrRangeFrom, GeneralAddrRange,
    MemoryAddr, VirtAddrRange,
};

memory_addr::def_usize_addr! {
    /// A runtime-only address for compatibility checks.
    ///
    /// Public range APIs must not require const address traits at runtime.
    #[derive(Debug)]
    type RuntimeAddr;
}

/// Checks membership at and between the endpoints.
///
/// The lower endpoint is included and the upper endpoint is excluded.
#[test]
fn test_range_contains_address() {
    let range = va_range!(0x1000usize..0x2000);
    assert!(range.contains(va!(0x1000)));
    assert!(range.contains(va!(0x1080)));
    assert!(!range.contains(va!(0x2000)));
}

/// Checks containment of nonempty ranges around each endpoint.
///
/// A contained range may share either bound but cannot extend beyond it.
#[test]
fn test_range_contains_range() {
    let range = va_range!(0x1000usize..0x2000);
    assert!(!range.contains_range(addr_range!(0xfff..0x1fff)));
    assert!(!range.contains_range(addr_range!(0xfff..0x2000)));
    assert!(!range.contains_range(va_range!(0xfff..0x2001))); // test both `va_range!` and `addr_range!`
    assert!(range.contains_range(va_range!(0x1000..0x1fff)));
    assert!(range.contains_range(addr_range!(0x1000..0x2000)));
    assert!(!range.contains_range(addr_range!(0x1000..0x2001)));
    assert!(range.contains_range(va_range!(0x1001..0x1fff)));
    assert!(range.contains_range(va_range!(0x1001..0x2000)));
    assert!(!range.contains_range(va_range!(0x1001..0x2001)));
    assert!(!range.contains_range(VirtAddrRange::from_start_size(0xfff.into(), 0x1)));
    assert!(!range.contains_range(VirtAddrRange::from_start_size(0x2000.into(), 0x1)));
}

/// Checks containment from the inner range's perspective.
///
/// Sharing a bound with the enclosing range remains valid.
#[test]
fn test_range_contained_in() {
    let range = va_range!(0x1000usize..0x2000);
    assert!(range.contained_in(addr_range!(0xfff..0x2000)));
    assert!(range.contained_in(addr_range!(0x1000..0x2000)));
    assert!(range.contained_in(va_range!(0x1000..0x2001)));
}

/// Checks overlap between nonempty ranges.
///
/// Touching exclusive endpoints do not create an overlap.
#[test]
fn test_nonempty_range_overlap() {
    let range = va_range!(0x1000usize..0x2000);
    assert!(!range.overlaps(addr_range!(0x800..0x1000)));
    assert!(range.overlaps(addr_range!(0x800..0x1001)));
    assert!(range.overlaps(addr_range!(0x1800..0x2000)));
    assert!(range.overlaps(va_range!(0x1800..0x2001)));
    assert!(!range.overlaps(va_range!(0x2000..0x2800)));
    assert!(range.overlaps(va_range!(0xfff..0x2001)));
}

/// Checks relations across enum and concrete range types.
///
/// The shared defaults preserve membership, containment, and overlap.
#[test]
fn const_mixed_relations() {
    const {
        let bounded = AddrRange::new(10usize, 20);
        let tail = AddrRangeFrom::new(15usize);
        let general = GeneralAddrRange::from(bounded);
        let general_tail = GeneralAddrRange::from(tail);
        assert!(general.contains(10));
        assert!(!general.contains(20));
        assert!(general.contains_range(bounded));
        assert!(bounded.contains_range(general));
        assert!(general.overlaps(general_tail));
        assert!(general_tail.overlaps(bounded));
        assert!(tail.overlaps(general));
        assert!(!general.contains_range(general_tail));
        assert!(AddrRange::new(15usize, 20).contained_in(general_tail));
    }
}

/// Checks subtraction from a bounded range, including empty and open-ended
/// operands.
#[test]
fn const_bounded_subtract() {
    const RANGE: AddrRange<usize> = AddrRange::new(10, 30);
    const SPLIT: (Option<AddrRange<usize>>, Option<AddrRange<usize>>) =
        RANGE.subtract(AddrRange::new(15, 20));
    const LEFT: (Option<AddrRange<usize>>, Option<AddrRange<usize>>) =
        RANGE.subtract(AddrRange::new(0, 15));
    const RIGHT: (Option<AddrRange<usize>>, Option<AddrRange<usize>>) =
        RANGE.subtract(AddrRange::new(25, 40));
    const COVERED: (Option<AddrRange<usize>>, Option<AddrRange<usize>>) =
        RANGE.subtract(AddrRange::new(0, 40));
    const BEFORE: (Option<AddrRange<usize>>, Option<AddrRange<usize>>) =
        RANGE.subtract(AddrRange::new(0, 10));
    const AFTER: (Option<AddrRange<usize>>, Option<AddrRange<usize>>) =
        RANGE.subtract(AddrRange::new(30, 40));
    const TAIL: (Option<AddrRange<usize>>, Option<AddrRange<usize>>) =
        RANGE.subtract(AddrRangeFrom::new(20));
    const EMPTY_BEFORE: (Option<AddrRange<usize>>, Option<AddrRange<usize>>) =
        RANGE.subtract(AddrRange::new(0, 0));
    const EMPTY_INSIDE: (Option<AddrRange<usize>>, Option<AddrRange<usize>>) =
        RANGE.subtract(AddrRange::new(17, 17));
    const EMPTY_AFTER: (Option<AddrRange<usize>>, Option<AddrRange<usize>>) =
        RANGE.subtract(AddrRange::new(30, 30));

    assert_eq!(
        SPLIT,
        (Some(AddrRange::new(10, 15)), Some(AddrRange::new(20, 30)))
    );
    assert_eq!(LEFT, (None, Some(AddrRange::new(15, 30))));
    assert_eq!(RIGHT, (Some(AddrRange::new(10, 25)), None));
    assert_eq!(COVERED, (None, None));
    assert_eq!(BEFORE, (None, Some(RANGE)));
    assert_eq!(AFTER, (Some(RANGE), None));
    assert_eq!(TAIL, (Some(AddrRange::new(10, 20)), None));
    assert_eq!(EMPTY_BEFORE, (None, Some(RANGE)));
    assert_eq!(EMPTY_INSIDE, (None, Some(RANGE)));
    assert_eq!(EMPTY_AFTER, (None, Some(RANGE)));
}

/// Checks subtraction from an open-ended range through the maximum address.
#[test]
fn const_open_ended_subtract() {
    const RANGE: AddrRangeFrom<usize> = AddrRangeFrom::new(10);
    const SPLIT: (Option<AddrRange<usize>>, Option<AddrRangeFrom<usize>>) =
        RANGE.subtract(AddrRange::new(15, 20));
    const LEFT: (Option<AddrRange<usize>>, Option<AddrRangeFrom<usize>>) =
        RANGE.subtract(AddrRange::new(0, 15));
    const RIGHT: (Option<AddrRange<usize>>, Option<AddrRangeFrom<usize>>) =
        RANGE.subtract(AddrRange::new(20, 30));
    const COVERED: (Option<AddrRange<usize>>, Option<AddrRangeFrom<usize>>) =
        RANGE.subtract(AddrRangeFrom::new(0));
    const TAIL: (Option<AddrRange<usize>>, Option<AddrRangeFrom<usize>>) =
        RANGE.subtract(AddrRangeFrom::new(20));
    const EMPTY_BEFORE: (Option<AddrRange<usize>>, Option<AddrRangeFrom<usize>>) =
        RANGE.subtract(AddrRange::new(0, 0));
    const EMPTY_INSIDE: (Option<AddrRange<usize>>, Option<AddrRangeFrom<usize>>) =
        RANGE.subtract(AddrRange::new(17, 17));
    const EMPTY_AT_MAX: (Option<AddrRange<usize>>, Option<AddrRangeFrom<usize>>) =
        RANGE.subtract(AddrRange::new(usize::MAX, usize::MAX));
    const FINAL: (Option<AddrRange<usize>>, Option<AddrRangeFrom<usize>>) =
        RANGE.subtract(AddrRange::new(usize::MAX - 1, usize::MAX));

    assert_eq!(
        SPLIT,
        (Some(AddrRange::new(10, 15)), Some(AddrRangeFrom::new(20))),
    );
    assert_eq!(LEFT, (None, Some(AddrRangeFrom::new(15))));
    assert_eq!(
        RIGHT,
        (Some(AddrRange::new(10, 20)), Some(AddrRangeFrom::new(30)))
    );
    assert_eq!(COVERED, (None, None));
    assert_eq!(TAIL, (Some(AddrRange::new(10, 20)), None));
    assert_eq!(EMPTY_BEFORE, (None, Some(RANGE)));
    assert_eq!(EMPTY_INSIDE, (None, Some(RANGE)));
    assert_eq!(EMPTY_AT_MAX, (None, Some(RANGE)));
    assert_eq!(
        FINAL,
        (
            Some(AddrRange::new(10, usize::MAX - 1)),
            Some(AddrRangeFrom::new(usize::MAX)),
        ),
    );
}

/// Checks that the general range delegates subtraction without changing its
/// endpoint representation.
#[test]
fn const_general_subtract_preserves_variant() {
    const BOUNDED: GeneralAddrRange<usize> = GeneralAddrRange::from(AddrRange::new(10, 30));
    const TAIL: GeneralAddrRange<usize> = GeneralAddrRange::from(AddrRangeFrom::new(10));
    const BOUNDED_EMPTY: (Option<AddrRange<usize>>, Option<GeneralAddrRange<usize>>) =
        BOUNDED.subtract(AddrRange::new(17, 17));
    const TAIL_EMPTY: (Option<AddrRange<usize>>, Option<GeneralAddrRange<usize>>) =
        TAIL.subtract(AddrRange::new(17, 17));
    const BOUNDED_SPLIT: (Option<AddrRange<usize>>, Option<GeneralAddrRange<usize>>) =
        BOUNDED.subtract(AddrRange::new(15, 20));
    const TAIL_SPLIT: (Option<AddrRange<usize>>, Option<GeneralAddrRange<usize>>) =
        TAIL.subtract(AddrRange::new(15, 20));

    assert_eq!(
        BOUNDED_EMPTY,
        (None, Some(GeneralAddrRange::Range(AddrRange::new(10, 30))),),
    );
    assert_eq!(TAIL_EMPTY, (None, Some(TAIL)));
    assert_eq!(
        BOUNDED_SPLIT,
        (
            Some(AddrRange::new(10, 15)),
            Some(GeneralAddrRange::Range(AddrRange::new(20, 30))),
        ),
    );
    assert_eq!(
        TAIL_SPLIT,
        (
            Some(AddrRange::new(10, 15)),
            Some(GeneralAddrRange::RangeFrom(AddrRangeFrom::new(20))),
        ),
    );
}

/// Checks empty-range semantics through the enum.
///
/// Empty ranges are subsets of all ranges and overlap none of them.
#[test]
fn const_empty_relations() {
    const {
        let empty = GeneralAddrRange::from(AddrRange::new(0usize, 0));
        let other = GeneralAddrRange::from(AddrRange::new(20usize, 20));
        let tail = GeneralAddrRange::from(AddrRangeFrom::new(usize::MAX));
        assert!(empty.is_empty());
        assert!(empty.contains_range(other));
        assert!(empty.contained_in(tail));
        assert!(!empty.contains_range(tail));
        assert!(!empty.overlaps(other));
        assert!(!tail.overlaps(empty));
    }
}

/// Checks containment through a generic const trait bound.
///
/// Both operands can independently choose their endpoint representation.
const fn contains<
    A: [const] MemoryAddr,
    R: [const] AddrRangeBounds<A>,
    S: [const] AddrRangeBounds<A>,
>(
    outer: R,
    inner: S,
) -> bool {
    outer.contains_range(inner)
}

/// Checks representable bounds through the common trait.
///
/// A bounded range ending at the maximum address still excludes it.
#[test]
fn const_bounded_queries() {
    const RANGE: AddrRange<usize> = AddrRange::new(0, usize::MAX);
    const START: usize = RANGE.start();
    const END: usize = RANGE.end();
    const CHECKED_END: Option<usize> = RANGE.checked_end();
    const SIZE: Option<usize> = RANGE.checked_size();
    const CONTAINS_MAX: bool = RANGE.contains(usize::MAX);
    assert_eq!(START, 0);
    assert_eq!(END, usize::MAX);
    assert_eq!(CHECKED_END, Some(usize::MAX));
    assert_eq!(SIZE, Some(usize::MAX));
    const { assert!(!CONTAINS_MAX) };
}

/// Checks empty bounded ranges at the address-space boundary.
///
/// The empty bounded range differs from the one-address open-ended range.
#[test]
fn const_empty_at_maximum() {
    const EMPTY: AddrRange<usize> = AddrRange::new(usize::MAX, usize::MAX);
    const IS_EMPTY: bool = EMPTY.is_empty();
    const SIZE: usize = EMPTY.size();
    const TAIL_EMPTY: bool = AddrRangeFrom::new(usize::MAX).is_empty();
    const { assert!(IS_EMPTY) };
    assert_eq!(SIZE, 0);
    const { assert!(!TAIL_EMPTY) };
}

/// Checks containment between bounded and open-ended ranges.
///
/// A bounded range cannot contain a range that includes the maximum
/// address.
#[test]
fn const_mixed_containment() {
    const BOUNDED: AddrRange<usize> = AddrRange::new(10, 20);
    const TAIL: AddrRangeFrom<usize> = AddrRangeFrom::new(10);
    const FORWARD: bool = contains(TAIL, BOUNDED);
    const REVERSE: bool = contains(BOUNDED, TAIL);
    const CONTAINED: bool = BOUNDED.contained_in(TAIL);
    const { assert!(FORWARD) };
    const { assert!(!REVERSE) };
    const { assert!(CONTAINED) };
    assert!(!AddrRangeFrom::new(11).contains_range(BOUNDED));
}

/// Checks containment between open-ended ranges.
///
/// Their identical conceptual ends reduce containment to comparing starts.
#[test]
fn const_tail_containment() {
    const FORWARD: bool = contains(AddrRangeFrom::new(10), AddrRangeFrom::new(20));
    const REVERSE: bool = contains(AddrRangeFrom::new(20), AddrRangeFrom::new(10));
    const { assert!(FORWARD) };
    const { assert!(!REVERSE) };
}

/// Checks nonempty overlap across both endpoint representations.
///
/// Touching exclusive endpoints do not overlap, while two tails always do.
#[test]
fn const_mixed_overlap() {
    const BOUNDED: AddrRange<usize> = AddrRange::new(10, 20);
    const TOUCHING: bool = BOUNDED.overlaps(AddrRangeFrom::new(20));
    const FORWARD: bool = BOUNDED.overlaps(AddrRangeFrom::new(19));
    const REVERSE: bool = AddrRangeFrom::new(19).overlaps(BOUNDED);
    const TAILS: bool = AddrRangeFrom::new(0usize).overlaps(AddrRangeFrom::new(usize::MAX));
    const { assert!(!TOUCHING) };
    const { assert!(FORWARD) };
    const { assert!(REVERSE) };
    const { assert!(TAILS) };
}

/// Checks adjacent nonempty ranges at the maximum address.
///
/// Each range contains exactly one of the final two addresses, without
/// overlapping in either operand order.
#[test]
fn const_adjacent_ranges_at_maximum() {
    const {
        let bounded = AddrRange::new(usize::MAX - 1, usize::MAX);
        let tail = AddrRangeFrom::new(usize::MAX);
        assert!(bounded.size() == 1);
        assert!(tail.size() == 1);
        assert!(bounded.contains(usize::MAX - 1));
        assert!(!bounded.contains(usize::MAX));
        assert!(!tail.contains(usize::MAX - 1));
        assert!(tail.contains(usize::MAX));
        assert!(!bounded.overlaps(tail));
        assert!(!tail.overlaps(bounded));
    }
}

/// Checks mixed containment with runtime-only addresses.
///
/// The generic const helper also accepts ordinary endpoint traits when
/// called at runtime.
#[test]
fn runtime_mixed_containment() {
    let bounded = AddrRange::new(RuntimeAddr::from_usize(10), RuntimeAddr::from_usize(20));
    let tail = AddrRangeFrom::new(RuntimeAddr::from_usize(10));
    assert_eq!(tail.start().as_usize(), 10);
    assert!(contains(tail, bounded));
    assert!(!contains(bounded, tail));
    assert!(bounded.contained_in(tail));
    assert!(!tail.contained_in(bounded));
}

/// Checks mixed overlap with runtime-only addresses.
///
/// Shared addresses overlap symmetrically, while adjacent endpoints do not.
#[test]
fn runtime_mixed_overlap() {
    let bounded = AddrRange::new(RuntimeAddr::from_usize(10), RuntimeAddr::from_usize(20));
    let overlapping = AddrRangeFrom::new(RuntimeAddr::from_usize(19));
    let adjacent = AddrRangeFrom::new(RuntimeAddr::from_usize(20));
    assert!(bounded.overlaps(overlapping));
    assert!(overlapping.overlaps(bounded));
    assert!(!bounded.overlaps(adjacent));
    assert!(!adjacent.overlaps(bounded));
}

/// Checks validity during constant evaluation.
///
/// Reversed endpoints are invalid, while empty ranges and tails are valid.
#[test]
fn const_validity() {
    const {
        assert!(AddrRange::new(0usize, usize::MAX).is_valid());
        assert!(AddrRange::new(usize::MAX, usize::MAX).is_valid());
        assert!(!AddrRange {
            start: 1usize,
            end: 0
        }
        .is_valid());
        assert!(AddrRangeFrom::new(0usize).is_valid());
        assert!(AddrRangeFrom::new(usize::MAX).is_valid());
    }
}

/// Checks containment of empty ranges independently of their position.
///
/// Both containment directions remain const across both range types.
#[test]
fn const_empty_range_containment() {
    const {
        let bounded = AddrRange::new(10usize, 20);
        let tail = AddrRangeFrom::new(20usize);
        let positions = [0, 9, 10, 15, 20, 21, usize::MAX];
        let mut index = 0;
        while index < positions.len() {
            let empty = AddrRange::new(positions[index], positions[index]);
            assert!(contains(bounded, empty));
            assert!(empty.contained_in(bounded));
            assert!(contains(tail, empty));
            assert!(empty.contained_in(tail));
            index += 1;
        }
    }
}

/// Checks containment between empty ranges at distinct endpoints.
///
/// Mutual containment does not require structural equality.
#[test]
fn const_empty_pair_containment() {
    const {
        let first = AddrRange::new(0usize, 0);
        let last = AddrRange::new(usize::MAX, usize::MAX);
        assert!(first.contains_range(first));
        assert!(first.contains_range(last));
        assert!(last.contains_range(first));
        assert!(first.contained_in(last));
        assert!(last.contained_in(first));
    }
}

/// Checks that empty ranges cannot contain nonempty ranges.
///
/// Both finite and open-ended inputs are rejected.
#[test]
fn const_empty_rejects_nonempty() {
    const {
        let empty = AddrRange::new(usize::MAX, usize::MAX);
        let bounded = AddrRange::new(usize::MAX - 1, usize::MAX);
        let tail = AddrRangeFrom::new(usize::MAX);
        assert!(!empty.contains_range(bounded));
        assert!(!empty.contains_range(tail));
        assert!(!bounded.contained_in(empty));
        assert!(!tail.contained_in(empty));
    }
}

/// Checks that empty ranges contain no addresses.
///
/// In particular, the stored start is not a member of an empty range.
#[test]
fn const_empty_address_membership() {
    const {
        let empty = AddrRange::new(10usize, 10);
        assert!(!empty.contains(0));
        assert!(!empty.contains(10));
        assert!(!empty.contains(usize::MAX));
        assert!(!AddrRange::new(0usize, 0).contains(0));
        assert!(!AddrRange::new(usize::MAX, usize::MAX).contains(usize::MAX));
    }
}

/// Checks disjointness of empty and nonempty ranges.
///
/// Empty ranges before, inside, and after the bounds never overlap in
/// either operand order.
#[test]
fn const_empty_nonempty_overlap() {
    const {
        let bounded = AddrRange::new(10usize, 20);
        let tail = AddrRangeFrom::new(20usize);
        let positions = [0, 9, 10, 15, 20, 21, usize::MAX];
        let mut index = 0;
        while index < positions.len() {
            let empty = AddrRange::new(positions[index], positions[index]);
            assert!(!bounded.overlaps(empty));
            assert!(!empty.overlaps(bounded));
            assert!(!tail.overlaps(empty));
            assert!(!empty.overlaps(tail));
            index += 1;
        }
    }
}

/// Checks that empty ranges never overlap each other.
///
/// Identical endpoints do not create a shared address.
#[test]
fn const_empty_pair_overlap() {
    const {
        let first = AddrRange::new(0usize, 0);
        let last = AddrRange::new(usize::MAX, usize::MAX);
        assert!(!first.overlaps(first));
        assert!(!last.overlaps(last));
        assert!(!first.overlaps(last));
        assert!(!last.overlaps(first));
    }
}

/// Checks finite-range relations against explicit address membership.
///
/// Exhausting a small domain verifies the set interpretation, including
/// all positions of empty ranges, without duplicating the bound formulas.
#[test]
fn bounded_relations_match_membership() {
    for start in 0usize..=4 {
        for end in start..=4 {
            let outer = AddrRange::new(start, end);
            for other_start in 0usize..=4 {
                for other_end in other_start..=4 {
                    let inner = AddrRange::new(other_start, other_end);
                    let contains =
                        (other_start..other_end).all(|addr| (start..end).contains(&addr));
                    let overlaps =
                        (other_start..other_end).any(|addr| (start..end).contains(&addr));
                    assert_eq!(outer.contains_range(inner), contains);
                    assert_eq!(inner.contained_in(outer), contains);
                    assert_eq!(outer.overlaps(inner), overlaps);
                    assert_eq!(inner.overlaps(outer), overlaps);
                }
            }
        }
    }
}
