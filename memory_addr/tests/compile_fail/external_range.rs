//! A downstream range implementation rejected by the sealed range traits.
//!
//! All public methods are present so that only the private marker is missing.

use memory_addr::{AddrRange, AddrRangeBounds, AddrRangeIterator, GeneralAddrRange};

/// A downstream range with otherwise valid trait implementations.
///
/// The fixture represents the empty range at zero.
#[derive(Clone, Copy)]
struct ExternalRange;

/// An attempted implementation of the public range trait.
///
/// The private `Range` marker must prevent this implementation.
impl AddrRangeBounds<usize> for ExternalRange {
    type Iterator = ExternalIterator;

    fn start(&self) -> usize {
        0
    }

    fn checked_end(&self) -> Option<usize> {
        Some(0)
    }

    fn checked_size(&self) -> Option<usize> {
        Some(0)
    }

    fn into_general(self) -> GeneralAddrRange<usize> {
        GeneralAddrRange::Range(AddrRange::new(0, 0))
    }
}

/// An iterator paired with the downstream range.
///
/// Its associated range satisfies the public iterator-to-range equality bound.
struct ExternalIterator;

/// An ordinary empty iterator for the empty range.
///
/// Its items have the same address type as the range's endpoints.
impl Iterator for ExternalIterator {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        None
    }
}

/// An attempted pairing of the downstream range and iterator.
///
/// The private marker traits are the only missing requirements.
impl AddrRangeIterator<usize> for ExternalIterator {
    type AddrRange = ExternalRange;

    fn new(_: ExternalRange, _: usize) -> Option<Self> {
        Some(Self)
    }
}

/// Provides the fixture's entry point.
///
/// Compilation must fail before this function can run.
fn main() {}
