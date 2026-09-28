//! Address alignment tests.
//!
//! Raw alignment helpers and semantic address methods follow the same alignment
//! rules.

#![feature(
    const_trait_impl,
    const_clone,
    const_cmp,
    const_convert,
    const_default,
    const_ops,
    derive_const
)]

use memory_addr::{
    align_down, align_down_4k, align_offset, align_offset_4k, align_up, align_up_4k, is_aligned,
    is_aligned_4k, va, MemoryAddr, PhysAddr,
};

mod common;

use common::{AnotherAddr, ConstAddr, ExampleAddr};

/// Checks alignment of virtual addresses.
///
/// Aligned and unaligned addresses retain their expected page boundaries.
#[test]
fn test_addr() {
    let addr = va!(0x2000);
    assert!(addr.is_aligned_4k());
    assert!(!addr.is_aligned(0x10000usize));
    assert_eq!(addr.align_offset_4k(), 0);
    assert_eq!(addr.align_down_4k(), va!(0x2000));
    assert_eq!(addr.align_up_4k(), va!(0x2000));

    let addr = va!(0x2fff);
    assert!(!addr.is_aligned_4k());
    assert_eq!(addr.align_offset_4k(), 0xfff);
    assert_eq!(addr.align_down_4k(), va!(0x2000));
    assert_eq!(addr.align_up_4k(), va!(0x3000));

    let align = 0x100000;
    let addr = va!(align * 5) + 0x2000;
    assert!(addr.is_aligned_4k());
    assert!(!addr.is_aligned(align));
    assert_eq!(addr.align_offset(align), 0x2000);
    assert_eq!(addr.align_down(align), va!(align * 5));
    assert_eq!(addr.align_up(align), va!(align * 6));
}

/// Checks alignment of a custom address type.
///
/// Generic alignment operations preserve the address representation.
#[test]
fn test_alignment() {
    let alignment = 0x1000usize;
    let base = alignment * 2;
    let offset = 0x123usize;
    let addr = ExampleAddr::from_usize(base + offset);

    assert_eq!(addr.align_down(alignment), ExampleAddr::from_usize(base));
    assert_eq!(
        addr.align_up(alignment),
        ExampleAddr::from_usize(base + alignment)
    );
    assert_eq!(addr.align_offset(alignment), offset);
    assert!(!addr.is_aligned(alignment));
    assert!(ExampleAddr::from_usize(base).is_aligned(alignment));
    assert_eq!(
        ExampleAddr::from_usize(base).align_up(alignment),
        ExampleAddr::from_usize(base)
    );
}

/// Aligns an address through a conditionally const generic bound.
///
/// The same function accepts runtime-only address types outside const
/// evaluation.
const fn aligned<A: [const] MemoryAddr>(addr: A) -> usize {
    addr.align_down_4k().into()
}

/// Checks a generic address operation during const evaluation.
///
/// Both built-in and macro-generated const addresses satisfy the bound.
#[test]
fn const_generic_alignment() {
    const BUILTIN: usize = aligned(PhysAddr::from(0x1234));
    const CUSTOM: usize = aligned(ConstAddr::from(0x2345));
    assert_eq!((BUILTIN, CUSTOM), (0x1000, 0x2000));
}

/// Checks runtime calls through the same generic const function.
///
/// Both ordinary declarations in the mixed macro invocation remain usable.
#[test]
fn runtime_generic_alignment() {
    assert_eq!(aligned(ExampleAddr::from(0x1234)), 0x1000);
    assert_eq!(aligned(AnotherAddr::from(0x2345)), 0x2000);
}

/// Checks raw address alignment helpers.
///
/// General and four-kilobyte helpers agree on aligned boundaries and offsets.
#[test]
fn test_align() {
    assert_eq!(align_down(0x12345678, 0x1000), 0x12345000);
    assert_eq!(align_up(0x12345678, 0x1000), 0x12346000);
    assert_eq!(align_offset(0x12345678, 0x1000), 0x678);
    assert!(is_aligned(0x12345000, 0x1000));
    assert!(!is_aligned(0x12345678, 0x1000));

    assert_eq!(align_down_4k(0x12345678), 0x12345000);
    assert_eq!(align_up_4k(0x12345678), 0x12346000);
    assert_eq!(align_offset_4k(0x12345678), 0x678);
    assert!(is_aligned_4k(0x12345000));
    assert!(!is_aligned_4k(0x12345678));
}
