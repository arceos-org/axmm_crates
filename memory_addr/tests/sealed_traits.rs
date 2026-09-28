//! Compile-fail tests for the sealed range traits.
//!
//! Downstream implementations must fail specifically because the private marker
//! traits are unavailable, rather than because public methods are missing.

/// Checks that downstream types cannot implement the range trait.
///
/// The fixture supplies all public requirements except the private marker.
#[test]
fn range_bounds_rejects_external_implementations() {
    trybuild::TestCases::new().compile_fail("tests/compile_fail/external_range.rs");
}

/// Checks that downstream types cannot implement the iterator trait.
///
/// A valid ordinary iterator still cannot satisfy the private marker bound.
#[test]
fn range_iterator_rejects_external_implementations() {
    trybuild::TestCases::new().compile_fail("tests/compile_fail/external_iterator.rs");
}
