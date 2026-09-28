# memory_addr

[![Crates.io](https://img.shields.io/crates/v/memory_addr)](https://crates.io/crates/memory_addr)
[![Docs.rs](https://docs.rs/memory_addr/badge.svg)](https://docs.rs/memory_addr)
[![CI](https://github.com/arceos-org/axmm_crates/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/arceos-org/axmm_crates/actions/workflows/ci.yml)

## Overview

`memory_addr` provides address types, address ranges, and page iterators for
low-level memory management. It is `no_std` and does not require allocation.

The types describe numerical addresses and ranges. They do not allocate or map
memory, check architecture-specific address limits, or establish whether an
address is safe to dereference.

## Address Types and Operations

### Built-in Address Types

`PhysAddr` and `VirtAddr` distinguish physical and virtual addresses. Both wrap
a `usize` and **preserve its full range of values**. Construct them with `pa!` and
`va!`, or use `from_usize` and `From<usize>`. Convert back with `as_usize` or
`Into<usize>`. `VirtAddr` also provides conversions to and from raw pointers.

`MemoryAddr` provides alignment helpers and checked, wrapping, and overflowing
arithmetic. Address addition and subtraction use byte offsets. The ordinary
`+` and `-` operators follow the corresponding `usize` operators, including
their overflow behavior. Methods such as `checked_add` specify their overflow
behavior explicitly.

```rust
use memory_addr::{pa, va, MemoryAddr, PhysAddr};

let physical = PhysAddr::from_usize(0x1234);
let virtual_addr = va!(0x2000);

assert_eq!(physical.align_down_4k(), pa!(0x1000));
assert_eq!(physical.align_offset_4k(), 0x234);
assert_eq!(virtual_addr + 0x100, va!(0x2100));
assert_eq!(virtual_addr.offset(-0x100), va!(0x1f00));
assert_eq!(va!(usize::MAX).checked_add(1), None);
```

### Creating New Address Types

`def_usize_addr!` creates a distinct address type with conversions, ordering,
and arithmetic. It automatically satisfies `MemoryAddr` and can be used with
the generic range and iterator types. `def_usize_addr_formatter!` adds debug
and hexadecimal formatting with a custom prefix.

```rust
use memory_addr::{def_usize_addr, def_usize_addr_formatter, MemoryAddr};

def_usize_addr! {
    /// A guest physical address.
    ///
    /// Distinguishes guest addresses from host physical addresses.
    pub type GuestPhysAddr;
}

def_usize_addr_formatter! {
    GuestPhysAddr = "GPA:{}";
}

let addr = GuestPhysAddr::from_usize(0x1234);
assert_eq!(addr.align_down_4k(), GuestPhysAddr::from_usize(0x1000));
assert_eq!(format!("{addr:?}"), "GPA:0x1234");
```

## Address Ranges

### Range Types

All address ranges have an inclusive start and an exclusive end. The three
representations differ in how they store the end:

| Type | Representation |
| --- | --- |
| `AddrRange<A>` | Stores `start` and a representable exclusive `end`. |
| `AddrRangeFrom<A>` | Stores `start` and extends through `usize::MAX`. Its implicit exclusive end is one past the maximum address. |
| `GeneralAddrRange<A>` | Holds either representation in its `Range` or `RangeFrom` variant. |

`VirtAddrRange` and `PhysAddrRange` are bounded ranges with the corresponding
address type. `VirtAddrRangeFrom` and `PhysAddrRangeFrom` are their open-ended
counterparts. `GeneralAddrRange<A>` is useful when one value must hold either
kind of range.

```rust
use memory_addr::{va, AddrRangeBounds, GeneralAddrRange, VirtAddrRange, VirtAddrRangeFrom};

let bounded = VirtAddrRange::new(va!(0x1000), va!(0x2000));
let last_byte = VirtAddrRangeFrom::new(va!(usize::MAX));
let ranges = [GeneralAddrRange::from(bounded), GeneralAddrRange::from(last_byte)];

assert_eq!(ranges[0].checked_end(), Some(va!(0x2000)));
assert_eq!(ranges[1].checked_end(), None);
assert_eq!(ranges[1].size(), 1);
```

A bounded range is empty when `start == end` and invalid when `start > end`.
Its checked constructors reject reversed endpoints. Public fields and range
conversions can produce invalid ranges, which can be detected with `is_valid`.
Unless a method documents otherwise, its queries require a valid range.
Open-ended ranges are always valid and nonempty.

### Operations

`AddrRangeBounds` provides the common queries for all three range types:

- `start`, `end`, and `size` return the endpoints and byte count.
- `checked_end` and `checked_size` return `None` when their result is not
  representable. The corresponding `end` and `size` methods panic instead.
- `is_valid` and `is_empty` check validity and emptiness.
- `contains` tests an address. `contains_range`, `contained_in`, and `overlaps`
  compare ranges, including ranges of different representations.

An open-ended range never has a representable exclusive end. Its size is
representable unless it starts at zero and covers the entire address space.

Every valid range contains every empty range, regardless of the empty range's
endpoints. Empty ranges never overlap any range, including another empty
range. Equality still compares the stored endpoints and, for
`GeneralAddrRange`, the variant.

```rust
use memory_addr::{va, va_range, AddrRangeBounds};

let range = va_range!(0x1000..0x3000);
let empty = va_range!(usize::MAX..usize::MAX);

assert_eq!(range.size(), 0x2000);
assert!(range.contains(va!(0x1000)));
assert!(!range.contains(va!(0x3000)));
assert!(range.contains_range(va_range!(0x1800..0x2000)));
assert!(range.contains_range(empty));
assert!(empty.contained_in(range));
assert!(!range.overlaps(empty));
```

### Conversions

`IntoAddrRange` converts standard library ranges with address or integer
endpoints. It supports the legacy `core::ops` ranges and the new `core::range`
types. Endpoints are converted with `Into<A>`, and a missing start becomes zero.

| Source range | Result of `into_addr_range` |
| --- | --- |
| `start..end`, `..end` | `AddrRange<A>` |
| `start..`, `..` | `AddrRangeFrom<A>` |
| `start..=last`, `..=last` | `GeneralAddrRange<A>` |

Exclusive ends are preserved. Inclusive ends are converted to exclusive ends
by adding one, with `usize::MAX` selecting an open-ended representation.
Reversed exclusive ranges remain invalid. Reversed inclusive ranges and
exhausted legacy inclusive ranges become empty bounded ranges.

`into_general_addr_range` always returns `GeneralAddrRange<A>`. Existing address
ranges can also use `into_general` to obtain that common representation.
`addr_range!`, `va_range!`, and `pa_range!` are shorthand for the corresponding
conversions, with the latter two selecting virtual and physical address types.

```rust
use memory_addr::{va, va_range, AddrRangeBounds, GeneralAddrRange, IntoAddrRange, VirtAddr};

let bounded = va_range!(0x1000..0x2000);
let tail = va_range!(usize::MAX..);
let full: GeneralAddrRange<VirtAddr> = (..=usize::MAX).into_general_addr_range();

assert_eq!(bounded.checked_end(), Some(va!(0x2000)));
assert_eq!(tail.size(), 1);
assert!(full.contains(va!(usize::MAX)));
assert_eq!(full.checked_size(), None);
```

### Iteration

`iter(page_size)` returns an iterator over page start addresses. The page size
must be a nonzero power of two, and each representable endpoint must be aligned
to it. Invalid ranges, invalid page sizes, and unaligned endpoints return
`None`. Empty aligned ranges produce an empty iterator.

The associated iterator types are `AddrRangeIter`, `AddrRangeFromIter`, and
`GeneralAddrRangeIter`. They own their iteration state and yield complete pages
in ascending order. Open-ended iteration includes the final page and then
terminates without wrapping to zero.

```rust
use memory_addr::{va, va_range, AddrRangeBounds, PAGE_SIZE_4K};

let mut pages = va_range!(0x1000..0x3000).iter(PAGE_SIZE_4K).unwrap();
assert_eq!(pages.next(), Some(va!(0x1000)));
assert_eq!(pages.next(), Some(va!(0x2000)));
assert_eq!(pages.next(), None);

let last_page = va!(usize::MAX - (PAGE_SIZE_4K - 1));
let mut tail = va_range!(last_page..).iter(PAGE_SIZE_4K).unwrap();
assert_eq!(tail.next(), Some(last_page));
assert_eq!(tail.next(), None);
```

`PageIter`, `DynPageIter`, and the fixed-size aliases remain for compatibility.
Prefer the range iterators for new code.

## Const Support

The built-in address types support const arithmetic and conversions. Address
ranges support const construction, queries, conversions, and iteration when
their address type provides the required const operations. The same APIs also
accept runtime-only address types in ordinary runtime code.

Runtime use does not itself require feature attributes in the calling crate.
Const trait calls and const iteration require the relevant nightly features,
as shown below. The inherent `from_usize` and `as_usize` methods do not require
these feature attributes, even in const expressions.

```rust
#![feature(const_trait_impl, const_iter)]

use memory_addr::{va, AddrRangeBounds, MemoryAddr, VirtAddr, VirtAddrRange, PAGE_SIZE_4K};

const START: VirtAddr = va!(0x1234).align_down_4k();
const PAGES: [Option<VirtAddr>; 3] = {
    let range = VirtAddrRange::new(START, va!(0x3000));
    let mut iter = range.iter(PAGE_SIZE_4K).unwrap();
    [iter.next(), iter.next(), iter.next()]
};

assert_eq!(PAGES, [Some(va!(0x1000)), Some(va!(0x2000)), None]);
```

For a custom address type, `def_usize_addr! { pub const type Name; }` enables
const trait implementations. The ordinary `pub type Name;` form generates
runtime trait implementations, while retaining const `from_usize` and
`as_usize` methods. See the macro documentation for the feature attributes
required by its const form.

## Toolchain Requirements

The recommended toolchain is **1.98-nightly**. CI uses
`nightly-2026-06-01` (`rustc 1.98.0-nightly (14210df0e 2026-05-31)`).
Select the toolchain explicitly, for example with
`cargo +nightly-2026-06-01 test -p memory_addr`.

There is no supported stable Rust version: the crate unconditionally uses
unstable const traits, const iterators, and const derives. Cargo's
`rust-version = "1.98"` declares a conservative minimum version, not a claim
that stable Rust 1.98 can build the crate. Cargo cannot express a nightly
date in that field, and some 1.97-nightly releases lack the required support.

The earliest nightly verified to compile the source is
**nightly-2026-04-28** (`rustc 1.97.0-nightly (52b6e2c20 2026-04-27)`).
The preceding nightly fails to compile the const closure in
`AddrRange::try_from_start_size`, which requires
[Rust PR #155772](https://github.com/rust-lang/rust/pull/155772).
Reproducing this source-level minimum requires `--ignore-rust-version`
because Cargo's declared minimum is intentionally higher. Newer nightlies
may still change these unstable APIs.

Downstream crates must select a compatible nightly themselves. The const
features described above do not make the crate usable with stable Rust.
