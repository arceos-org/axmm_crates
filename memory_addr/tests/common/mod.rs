//! Shared custom address fixtures.
//!
//! Each integration test target uses the declarations relevant to its topic.

#![allow(
    dead_code,
    reason = "Independent test targets use different subsets of these fixtures."
)]

use memory_addr::{def_usize_addr, def_usize_addr_formatter};

def_usize_addr! {
    /// A runtime-only custom address.
    ///
    /// Ordinary macro declarations must not require const trait implementations.
    pub type ExampleAddr;
    /// A const address between two runtime-only macro declarations.
    ///
    /// Mixed declarations exercise the macro's per-type const opt-in.
    pub(crate) const type ConstAddr;
    /// A second runtime-only custom address.
    ///
    /// Its declaration checks continuation after the const declaration.
    pub type AnotherAddr;
}

def_usize_addr_formatter! {
    ExampleAddr = "EA:{}";
    AnotherAddr = "AA:{}";
}
