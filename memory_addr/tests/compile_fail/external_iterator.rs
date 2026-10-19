//! A downstream iterator implementation rejected by the sealed iterator trait.
//!
//! The ordinary iterator implementation is valid without the sealed extension.

use memory_addr::{AddrRange, AddrRangeIterator};

/// A downstream iterator with no items.
///
/// This type satisfies all public supertrait requirements.
struct ExternalIterator;

/// An ordinary empty iterator.
///
/// Exhaustion is permanent and independent of the sealed trait.
impl Iterator for ExternalIterator {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        None
    }
}

/// An attempted implementation of the public range iterator trait.
///
/// The private `RangeIterator` marker must prevent this implementation.
impl AddrRangeIterator<usize> for ExternalIterator {
    type AddrRange = AddrRange<usize>;

    fn new(_: AddrRange<usize>, _: usize) -> Option<Self> {
        Some(Self)
    }
}

/// Provides the fixture's entry point.
///
/// Compilation must fail before this function can run.
fn main() {}
