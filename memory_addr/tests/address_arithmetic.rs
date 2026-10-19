//! Address arithmetic and overflow tests.
//!
//! Runtime and const arithmetic retain the selected checking, wrapping, and
//! offset semantics.

#![feature(
    const_trait_impl,
    const_clone,
    const_cmp,
    const_convert,
    const_default,
    const_ops,
    derive_const
)]

use core::ops::{Add, Sub};

use memory_addr::{pa, MemoryAddr, PhysAddr};

mod common;

use common::{ConstAddr, ExampleAddr};

/// Checks address and offset arithmetic.
///
/// Operators and utility methods agree on representable offset calculations.
#[test]
fn test_addr_arithmetic() {
    let base = 0x1234usize;
    let offset = 0x100usize;
    let with_offset = base + offset;

    let addr = ExampleAddr::from_usize(base);
    let offset_addr = ExampleAddr::from_usize(with_offset);

    assert_eq!(addr.offset(offset as isize), offset_addr);
    assert_eq!(addr.wrapping_offset(offset as isize), offset_addr);
    assert_eq!(offset_addr.offset_from(addr), offset as isize);
    assert_eq!(addr.add(offset), offset_addr);
    assert_eq!(addr.wrapping_add(offset), offset_addr);
    assert_eq!(offset_addr.sub(offset), addr);
    assert_eq!(offset_addr.wrapping_sub(offset), addr);
    assert_eq!(offset_addr.sub_addr(addr), offset);
    assert_eq!(offset_addr.wrapping_sub_addr(addr), offset);

    assert_eq!(addr + offset, offset_addr);
    assert_eq!(offset_addr - offset, addr);
    assert_eq!(offset_addr - addr, offset);
}

/// Checks wrapping address arithmetic.
///
/// Offsets crossing the maximum address wrap consistently in both directions.
#[test]
fn test_addr_wrapping_arithmetic() {
    let base = usize::MAX - 0x100usize;
    let offset = 0x200usize;
    let with_offset = base.wrapping_add(offset);

    let addr = ExampleAddr::from_usize(base);
    let offset_addr = ExampleAddr::from_usize(with_offset);

    assert_eq!(addr.wrapping_offset(offset as isize), offset_addr);
    assert_eq!(offset_addr.wrapping_offset(-(offset as isize)), addr);
    assert_eq!(addr.wrapping_add(offset), offset_addr);
    assert_eq!(offset_addr.wrapping_sub(offset), addr);
    assert_eq!(offset_addr.wrapping_sub_addr(addr), offset);
}

/// Checks fallible address arithmetic.
///
/// Representable results are returned and overflow is reported as `None`.
#[test]
fn test_addr_checked_arithmetic() {
    let low_addr = ExampleAddr::from_usize(0x100usize);
    let high_addr = ExampleAddr::from_usize(usize::MAX - 0x100usize);
    let small_offset = 0x50usize;
    let large_offset = 0x200usize;

    assert_eq!(
        low_addr.checked_sub(small_offset),
        Some(low_addr.wrapping_sub(small_offset))
    );
    assert_eq!(low_addr.checked_sub(large_offset), None);
    assert_eq!(
        high_addr.checked_add(small_offset),
        Some(high_addr.wrapping_add(small_offset))
    );
    assert_eq!(high_addr.checked_add(large_offset), None);

    assert_eq!(
        high_addr.checked_sub_addr(low_addr),
        Some(usize::MAX - 0x200usize)
    );
    assert_eq!(low_addr.checked_sub_addr(high_addr), None);
}

/// Checks overflow-reporting address arithmetic.
///
/// Each result combines its wrapped value with the correct overflow flag.
#[test]
fn test_addr_overflowing_arithmetic() {
    let low_addr = ExampleAddr::from_usize(0x100usize);
    let high_addr = ExampleAddr::from_usize(usize::MAX - 0x100usize);
    let small_offset = 0x50usize;
    let large_offset = 0x200usize;

    assert_eq!(
        low_addr.overflowing_sub(small_offset),
        (low_addr.wrapping_sub(small_offset), false)
    );
    assert_eq!(
        low_addr.overflowing_sub(large_offset),
        (low_addr.wrapping_sub(large_offset), true)
    );
    assert_eq!(
        high_addr.overflowing_add(small_offset),
        (high_addr.wrapping_add(small_offset), false)
    );
    assert_eq!(
        high_addr.overflowing_add(large_offset),
        (high_addr.wrapping_add(large_offset), true)
    );
    assert_eq!(
        high_addr.overflowing_sub_addr(low_addr),
        (high_addr.wrapping_sub_addr(low_addr), false)
    );
    assert_eq!(
        low_addr.overflowing_sub_addr(high_addr),
        (low_addr.wrapping_sub_addr(high_addr), true)
    );
}

/// Checks rejection of an overflowing signed offset.
///
/// An offset past the maximum address panics in every build profile.
#[test]
#[should_panic]
fn test_addr_offset_overflow() {
    let addr = ExampleAddr::from_usize(usize::MAX);
    let _ = addr.offset(1);
}

/// Checks a zero address difference during const evaluation.
///
/// Equal maximum addresses must produce zero rather than an overflow.
#[test]
fn const_offset_from_same_address() {
    const DIFFERENCE: isize = pa!(usize::MAX).offset_from(pa!(usize::MAX));
    assert_eq!(DIFFERENCE, 0);
}

/// Checks a negative address difference during const evaluation.
///
/// An address below the base produces a negative byte offset.
#[test]
fn const_offset_from_negative() {
    const DIFFERENCE: isize = pa!(0x1000).offset_from(pa!(0x2000));
    assert_eq!(DIFFERENCE, -0x1000);
}

/// Checks the largest representable positive address difference.
///
/// A difference exactly equal to `isize::MAX` must succeed in const code.
#[test]
fn const_offset_from_maximum() {
    const DIFFERENCE: isize = pa!(usize::MAX).offset_from(pa!(isize::MAX as usize + 1));
    assert_eq!(DIFFERENCE, isize::MAX);
}

/// Checks the smallest representable negative address difference.
///
/// A difference exactly equal to `isize::MIN` must succeed in const code.
#[test]
fn const_offset_from_minimum() {
    const DIFFERENCE: isize = pa!(0).offset_from(pa!(isize::MAX as usize + 1));
    assert_eq!(DIFFERENCE, isize::MIN);
}

/// Checks rejection just above the signed difference range.
///
/// A positive difference of `isize::MAX + 1` must panic in every profile.
#[test]
#[should_panic(expected = "overflow in `MemoryAddr::offset_from`")]
fn offset_from_just_above_maximum() {
    let _ = pa!(isize::MAX as usize + 1).offset_from(pa!(0));
}

/// Checks rejection just below the signed difference range.
///
/// A negative difference of `isize::MIN - 1` must panic in every profile.
#[test]
#[should_panic(expected = "overflow in `MemoryAddr::offset_from`")]
fn offset_from_just_below_minimum() {
    let _ = pa!(0).offset_from(pa!(isize::MAX as usize + 2));
}

/// Checks rejection of an excessive positive address difference.
///
/// A difference outside the signed offset range must panic.
#[test]
#[should_panic]
fn test_addr_offset_from_overflow() {
    let addr = ExampleAddr::from_usize(usize::MAX);
    let _ = addr.offset_from(ExampleAddr::from_usize(0));
}

/// Checks rejection of an excessive negative address difference.
///
/// A difference outside the signed offset range must panic.
#[test]
#[should_panic]
fn test_addr_offset_from_underflow() {
    let addr = ExampleAddr::from_usize(0);
    let _ = addr.offset_from(ExampleAddr::from_usize(usize::MAX));
}

/// Checks unsigned addition overflow against the underlying integer
/// operator.
///
/// The result follows the build's overflow checking policy in both
/// profiles.
#[test]
fn test_addr_add_overflow() {
    let raw = core::hint::black_box(usize::MAX);
    let expected = std::panic::catch_unwind(|| raw + 1).map_err(|_| ());
    let actual =
        std::panic::catch_unwind(|| (ExampleAddr::from_usize(raw) + 1).as_usize()).map_err(|_| ());
    assert_eq!(actual, expected);
}

/// Checks unsigned subtraction underflow against the underlying integer
/// operator.
///
/// The result follows the build's overflow checking policy in both
/// profiles.
#[test]
fn test_addr_sub_underflow() {
    let raw = core::hint::black_box(0usize);
    let expected = std::panic::catch_unwind(|| raw - 1).map_err(|_| ());
    let actual =
        std::panic::catch_unwind(|| (ExampleAddr::from_usize(raw) - 1).as_usize()).map_err(|_| ());
    assert_eq!(actual, expected);
}

/// Checks rejection of an unsigned negative address difference.
///
/// Subtracting a larger address must panic in every build profile.
#[test]
#[should_panic]
fn test_addr_sub_addr_overflow() {
    let addr = ExampleAddr::from_usize(0);
    let _ = addr.sub_addr(ExampleAddr::from_usize(1));
}

/// Checks const arithmetic operators.
///
/// Assignment, offset arithmetic, and address subtraction retain their
/// meanings.
#[test]
fn const_address_operators() {
    const OFFSET: usize = {
        let mut addr = ConstAddr::from(0x1000);
        addr += 0x200;
        addr -= 0x100;
        (addr + 0x20 - 0x10) - ConstAddr::from(0x1000)
    };
    assert_eq!(OFFSET, 0x110);
}

/// Checks const addition at the maximum address.
///
/// Reaching the maximum succeeds, but exceeding it returns `None`.
#[test]
fn const_checked_add_boundary() {
    const LAST: Option<PhysAddr> = PhysAddr::from_usize(usize::MAX - 1).checked_add(1);
    const OVERFLOW: Option<PhysAddr> = PhysAddr::from_usize(usize::MAX).checked_add(1);
    assert_eq!(LAST, Some(pa!(usize::MAX)));
    assert_eq!(OVERFLOW, None);
}

/// Checks const subtraction at the zero address.
///
/// Reaching zero succeeds, but going below it returns `None`.
#[test]
fn const_checked_sub_boundary() {
    const ZERO: Option<PhysAddr> = PhysAddr::from_usize(1).checked_sub(1);
    const UNDERFLOW: Option<PhysAddr> = PhysAddr::from_usize(0).checked_sub(1);
    assert_eq!(ZERO, Some(pa!(0)));
    assert_eq!(UNDERFLOW, None);
}

/// Checks const wrapping arithmetic across the address-space boundaries.
///
/// Wrapping addition and subtraction must not panic during const
/// evaluation.
#[test]
fn const_wrapping_boundaries() {
    const ZERO: PhysAddr = PhysAddr::from_usize(usize::MAX).wrapping_add(1);
    const LAST: PhysAddr = PhysAddr::from_usize(0).wrapping_sub(1);
    assert_eq!(ZERO, pa!(0));
    assert_eq!(LAST, pa!(usize::MAX));
}
