//! Private markers for the supported range and iterator implementations.
//!
//! Public traits inherit these markers so downstream code can use the traits
//! without adding implementations.

use crate::{
    AddrRange, AddrRangeFrom, AddrRangeFromIter, AddrRangeIter, GeneralAddrRange,
    GeneralAddrRangeIter, MemoryAddr,
};

/// A marker for crate-provided address ranges.
///
/// The private module prevents downstream implementations.
pub trait Range {}

/// A marker for crate-provided address range iterators.
///
/// Only iterators paired with supported range types implement this marker.
pub trait RangeIterator {}

/// The bounded range marker implementation.
///
/// External address types remain supported.
impl<A: MemoryAddr> Range for AddrRange<A> {}

/// The open-ended range marker implementation.
///
/// External address types remain supported.
impl<A: MemoryAddr> Range for AddrRangeFrom<A> {}

/// The general range marker implementation.
///
/// Both public enum variants belong to the supported range family.
impl<A: MemoryAddr> Range for GeneralAddrRange<A> {}

/// The bounded iterator marker implementation.
///
/// This iterator is paired with `AddrRange`.
impl<A: MemoryAddr> RangeIterator for AddrRangeIter<A> {}

/// The open-ended iterator marker implementation.
///
/// This iterator is paired with `AddrRangeFrom`.
impl<A: MemoryAddr> RangeIterator for AddrRangeFromIter<A> {}

/// The general iterator marker implementation.
///
/// This iterator is paired with `GeneralAddrRange`.
impl<A: MemoryAddr> RangeIterator for GeneralAddrRangeIter<A> {}
