//! Generic address range iterator contract tests.
//!
//! Associated iterators support const and runtime generic dispatch and fused
//! iteration.

#![feature(const_trait_impl, const_iter, const_destruct, const_convert)]

use core::{iter::FusedIterator, marker::Destruct};

use memory_addr::{AddrRange, AddrRangeBounds, AddrRangeFrom, GeneralAddrRange, MemoryAddr};

memory_addr::def_usize_addr! {
    /// A runtime-only address for compatibility checks.
    ///
    /// Public range APIs must not require const address traits at runtime.
    #[derive(Debug)]
    type RuntimeAddr;
}

/// Reads a page through the public const range interface.
///
/// The associated iterator is dropped after yielding its first page.
const fn first<A: [const] MemoryAddr, R: [const] AddrRangeBounds<A>>(range: R) -> Option<A>
where
    R::Iterator: [const] Destruct,
{
    range.iter(1).unwrap().next()
}

/// Checks associated iterator dispatch through generic const bounds.
///
/// Both enum variants satisfy the same interface as the concrete ranges.
#[test]
fn const_generic_general_iteration() {
    const BOUNDED: Option<usize> = first(GeneralAddrRange::from(AddrRange::new(0usize, 1)));
    const TAIL: Option<usize> = first(GeneralAddrRange::from(AddrRangeFrom::new(usize::MAX)));
    assert_eq!(BOUNDED, Some(0));
    assert_eq!(TAIL, Some(usize::MAX));
}

/// Checks conversion and iteration for runtime-only endpoint traits.
///
/// Conditional const bounds do not restrict ordinary runtime calls.
#[test]
fn runtime_endpoints() {
    let bounded = GeneralAddrRange::from(AddrRange::new(
        RuntimeAddr::from_usize(1),
        RuntimeAddr::from_usize(2),
    ));
    let tail = GeneralAddrRange::from(AddrRangeFrom::new(RuntimeAddr::from_usize(usize::MAX)));
    assert_eq!(bounded.size(), 1);
    assert_eq!(tail.size(), 1);
    assert_eq!(first(bounded).unwrap().as_usize(), 1);
    assert_eq!(first(tail).unwrap().as_usize(), usize::MAX);
}

/// Requires permanent exhaustion through the standard marker trait.
///
/// This bound verifies that the enum retains the concrete iterators'
/// guarantee.
fn require_fused<I: FusedIterator>(_: I) {}

/// Checks the fused marker on both iterator variants.
///
/// Generic consumers can rely on permanent exhaustion.
#[test]
fn fused_iteration() {
    require_fused(
        GeneralAddrRange::from(AddrRange::new(0usize, 1))
            .iter(1)
            .unwrap(),
    );
    require_fused(
        GeneralAddrRange::from(AddrRangeFrom::new(usize::MAX))
            .iter(1)
            .unwrap(),
    );
}

/// Reads a page through the iterator associated with a generic range.
///
/// Both construction and advancement must be available in const contexts.
/// Conditional const destruction permits dropping the local iterator early.
const fn first_page<A: [const] MemoryAddr, R: [const] AddrRangeBounds<A>>(
    range: R,
    page_size: usize,
) -> Option<A>
where
    R::Iterator: [const] Destruct,
{
    range.iter(page_size).unwrap().next()
}

/// Checks iteration through const generic range bounds.
///
/// Both endpoint representations expose the associated iterator operations.
#[test]
fn const_generic_concrete_iteration() {
    const BOUNDED: Option<usize> = first_page(AddrRange::new(0usize, 1), 1);
    const TAIL: Option<usize> = first_page(AddrRangeFrom::new(usize::MAX), 1);
    assert_eq!(BOUNDED, Some(0));
    assert_eq!(TAIL, Some(usize::MAX));
}

/// Checks the const generic iterator helper with runtime-only addresses.
///
/// The public trait's conditional const bounds preserve runtime use.
#[test]
fn runtime_generic_iteration() {
    let bounded = AddrRange::new(RuntimeAddr::from_usize(0), RuntimeAddr::from_usize(1));
    let tail = AddrRangeFrom::new(RuntimeAddr::from_usize(usize::MAX));
    assert_eq!(first_page(bounded, 1).unwrap().as_usize(), 0);
    assert_eq!(first_page(tail, 1).unwrap().as_usize(), usize::MAX);
}
