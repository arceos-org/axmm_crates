//! Compatibility page iterator tests.
//!
//! Fixed and dynamic wrappers preserve the bounded iterator's validation and
//! page sequence.

#![feature(const_trait_impl, const_iter)]

use memory_addr::{pa, DynPageIter, PageIter, PhysAddr};

memory_addr::def_usize_addr! {
    /// A runtime-only address for compatibility checks.
    ///
    /// Public range APIs must not require const address traits at runtime.
    #[derive(Debug)]
    type RuntimeAddr;
}

/// Checks rejection of reversed fixed-size iterator endpoints.
///
/// Aligned reversed bounds are invalid rather than an empty range.
#[test]
fn const_fixed_reversed_endpoints() {
    const {
        assert!(PageIter::<0x1000, _>::new(pa!(0x3000), pa!(0x1000)).is_none());
    }
}

/// Checks rejection of reversed dynamic iterator endpoints.
///
/// The result matches the bounded range iterator's validity check.
#[test]
fn const_dynamic_reversed_endpoints() {
    const {
        assert!(DynPageIter::new(pa!(0x3000), pa!(0x1000), 0x1000).is_none());
    }
}

/// Checks fixed-size iteration with runtime-only endpoints.
///
/// Delegation preserves ordinary runtime use of conditionally const
/// methods.
#[test]
fn runtime_fixed_page_sequence() {
    let pages: Vec<_> =
        PageIter::<1, _>::new(RuntimeAddr::from_usize(1), RuntimeAddr::from_usize(3))
            .unwrap()
            .map(RuntimeAddr::as_usize)
            .collect();
    assert_eq!(pages, [1, 2]);
}

/// Checks dynamic iteration with runtime-only endpoints.
///
/// Delegation preserves the supplied page size.
#[test]
fn runtime_dynamic_page_sequence() {
    let pages: Vec<_> = DynPageIter::new(RuntimeAddr::from_usize(2), RuntimeAddr::from_usize(6), 2)
        .unwrap()
        .map(RuntimeAddr::as_usize)
        .collect();
    assert_eq!(pages, [2, 4]);
}

/// Checks const iteration over a fixed-size page range.
///
/// The exclusive endpoint is omitted and exhaustion remains stable.
#[test]
fn const_fixed_page_sequence() {
    const PAGES: [Option<PhysAddr>; 4] = {
        let mut iter = PageIter::<0x1000, _>::new(pa!(0x1000), pa!(0x3000)).unwrap();
        [iter.next(), iter.next(), iter.next(), iter.next()]
    };
    assert_eq!(PAGES, [Some(pa!(0x1000)), Some(pa!(0x2000)), None, None]);
}

/// Checks const iteration with a runtime-configurable page size.
///
/// The supplied size determines the step and the endpoint remains
/// exclusive.
#[test]
fn const_dynamic_page_sequence() {
    const PAGES: [Option<PhysAddr>; 4] = {
        let mut iter = DynPageIter::new(pa!(0x2000), pa!(0x6000), 0x2000).unwrap();
        [iter.next(), iter.next(), iter.next(), iter.next()]
    };
    assert_eq!(PAGES, [Some(pa!(0x2000)), Some(pa!(0x4000)), None, None]);
}

/// Checks an empty fixed-size iterator during const evaluation.
///
/// Equal aligned endpoints produce no pages.
#[test]
fn const_fixed_empty_range() {
    const FIRST: Option<usize> = PageIter::<0x1000, _>::new(0usize, 0).unwrap().next();
    assert_eq!(FIRST, None);
}

/// Checks an empty dynamic iterator during const evaluation.
///
/// Equal aligned endpoints produce no pages.
#[test]
fn const_dynamic_empty_range() {
    const FIRST: Option<usize> = DynPageIter::new(0usize, 0, 0x1000).unwrap().next();
    assert_eq!(FIRST, None);
}

/// Checks invalid fixed page sizes during const evaluation.
///
/// Zero and non-power-of-two sizes must be rejected before alignment
/// arithmetic.
#[test]
fn const_fixed_invalid_page_size() {
    const ZERO: bool = PageIter::<0, _>::new(0usize, 0).is_none();
    const THREE: bool = PageIter::<3, _>::new(0usize, 0).is_none();
    assert_eq!((ZERO, THREE), (true, true));
}

/// Checks invalid dynamic page sizes during const evaluation.
///
/// Zero and non-power-of-two sizes must be rejected before alignment
/// arithmetic.
#[test]
fn const_dynamic_invalid_page_size() {
    const ZERO: bool = DynPageIter::new(0usize, 0, 0).is_none();
    const THREE: bool = DynPageIter::new(0usize, 0, 3).is_none();
    assert_eq!((ZERO, THREE), (true, true));
}

/// Checks alignment of each fixed iterator endpoint.
///
/// Either an unaligned start or an unaligned end prevents construction.
#[test]
fn const_fixed_unaligned_endpoints() {
    const START: bool = PageIter::<0x1000, _>::new(1usize, 0x2000).is_none();
    const END: bool = PageIter::<0x1000, _>::new(0usize, 0x2001).is_none();
    assert_eq!((START, END), (true, true));
}

/// Checks alignment of each dynamic iterator endpoint.
///
/// Either an unaligned start or an unaligned end prevents construction.
#[test]
fn const_dynamic_unaligned_endpoints() {
    const START: bool = DynPageIter::new(1usize, 0x2000, 0x1000).is_none();
    const END: bool = DynPageIter::new(0usize, 0x2001, 0x1000).is_none();
    assert_eq!((START, END), (true, true));
}

/// Checks fixed iteration next to the maximum address.
///
/// Advancing to the exclusive endpoint succeeds without wrapping or
/// stepping past it.
#[test]
fn const_fixed_upper_boundary() {
    const PAGES: [Option<usize>; 2] = {
        let mut iter = PageIter::<1, _>::new(usize::MAX - 1, usize::MAX).unwrap();
        [iter.next(), iter.next()]
    };
    assert_eq!(PAGES, [Some(usize::MAX - 1), None]);
}

/// Checks dynamic iteration next to the maximum address.
///
/// Advancing to the exclusive endpoint succeeds without wrapping or
/// stepping past it.
#[test]
fn const_dynamic_upper_boundary() {
    const PAGES: [Option<usize>; 2] = {
        let mut iter = DynPageIter::new(usize::MAX - 1, usize::MAX, 1).unwrap();
        [iter.next(), iter.next()]
    };
    assert_eq!(PAGES, [Some(usize::MAX - 1), None]);
}
