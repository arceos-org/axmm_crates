//! Address representation and pointer conversion tests.
//!
//! Public conversions, comparisons, formatting, and const derives retain their
//! behavior.

#![feature(
    const_trait_impl,
    const_clone,
    const_cmp,
    const_convert,
    const_default,
    const_ops,
    derive_const
)]

use core::{mem::size_of, ops::Add};

use memory_addr::VirtAddr;

mod common;

use common::{AnotherAddr, ConstAddr, ExampleAddr};

/// Checks custom address conversions and ordering.
///
/// The generated traits preserve integer values and comparison results.
#[test]
fn test_addr_convert_and_comparison() {
    let example1 = ExampleAddr::from_usize(0x1234);
    let example2 = ExampleAddr::from(0x5678);
    let another1 = AnotherAddr::from_usize(0x9abc);
    let another2 = AnotherAddr::from(0xdef0);

    assert_eq!(example1.as_usize(), 0x1234);
    assert_eq!(Into::<usize>::into(example2), 0x5678);
    assert_eq!(Into::<usize>::into(another1), 0x9abc);
    assert_eq!(another2.as_usize(), 0xdef0);

    assert_eq!(example1, ExampleAddr::from(0x1234));
    assert_eq!(example2, ExampleAddr::from_usize(0x5678));
    assert_eq!(another1, AnotherAddr::from_usize(0x9abc));
    assert_eq!(another2, AnotherAddr::from(0xdef0));

    assert!(example1 < example2);
    assert!(example1 <= example2);
    assert!(example2 > example1);
    assert!(example2 >= example1);
    assert!(example1 != example2);
}

/// Checks custom address formatting.
///
/// Debug and hexadecimal formatters preserve configured address prefixes.
#[test]
fn test_addr_fmt() {
    assert_eq!(format!("{:?}", ExampleAddr::from(0x1abc)), "EA:0x1abc");
    assert_eq!(format!("{:x}", AnotherAddr::from(0x1abc)), "AA:0x1abc");
    assert_eq!(format!("{:X}", ExampleAddr::from(0x1abc)), "EA:0x1ABC");
}

/// Checks virtual address conversions to and from pointers.
///
/// Typed and untyped pointers retain access to the original backing array.
#[test]
fn test_virt_addr_ptr() {
    let mut a: [usize; 4] = [0x1234, 0x5678, 0x9abc, 0xdef0];

    let va0 = VirtAddr::from_mut_ptr_of(a.as_mut_ptr());
    let va1 = va0.add(size_of::<usize>());
    let va2 = va1.add(size_of::<usize>());
    let va3 = va2.add(size_of::<usize>());

    let p0 = va0.as_ptr() as *const usize;
    let p1 = va1.as_ptr_of::<usize>();
    let p2 = va2.as_mut_ptr() as *mut usize;
    let p3 = va3.as_mut_ptr_of::<usize>();

    // testing conversion back to virt addr
    assert_eq!(va0, VirtAddr::from_ptr_of(p0));
    assert_eq!(va1, VirtAddr::from_ptr_of(p1));
    assert_eq!(va2, VirtAddr::from_mut_ptr_of(p2));
    assert_eq!(va3, VirtAddr::from_mut_ptr_of(p3));

    // testing pointer read/write
    assert!(unsafe { *p0 } == a[0]);
    assert!(unsafe { *p1 } == a[1]);
    assert!(unsafe { *p2 } == a[2]);
    assert!(unsafe { *p3 } == a[3]);

    unsafe {
        *p2 = 0xdeadbeef;
    }
    unsafe {
        *p3 = 0xcafebabe;
    }
    assert_eq!(a[2], 0xdeadbeef);
    assert_eq!(a[3], 0xcafebabe);
}

/// Checks const conversions for a macro-generated address.
///
/// Both conversion directions must be evaluated at compile time.
#[test]
fn const_address_conversion() {
    const RAW: usize = ConstAddr::from(0x1234).into();
    assert_eq!(RAW, 0x1234);
}

/// Checks the const default address.
///
/// Default initialization must produce the zero address.
#[test]
fn const_address_default() {
    const DEFAULT: ConstAddr = ConstAddr::default();
    assert_eq!(DEFAULT.as_usize(), 0);
}

/// Checks const address cloning.
///
/// Explicit cloning guards the trait implementation rather than implicit
/// copying.
#[test]
#[allow(clippy::clone_on_copy)]
fn const_address_clone() {
    const CLONE: ConstAddr = ConstAddr::from_usize(0x1234).clone();
    assert_eq!(CLONE.as_usize(), 0x1234);
}

/// Checks const address comparisons.
///
/// Equality and ordering must work on the generated address type.
#[test]
fn const_address_comparison() {
    const EQUAL: bool = ConstAddr::from(0x1000) == ConstAddr::from(0x1000);
    const LESS: bool = ConstAddr::from(0x1000) < ConstAddr::from(0x2000);
    assert_eq!((EQUAL, LESS), (true, true));
}
