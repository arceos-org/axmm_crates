# `axmm_crates`

Crates for memory management.

This fork requires nightly Rust and recommends **1.98-nightly**. Cargo's declared minimum version is `1.98`, while the earliest nightly verified to
compile the current `memory_addr` source is `nightly-2026-04-28` (1.97-nightly). CI uses `nightly-2026-06-01` (1.98-nightly). Stable Rust is not supported. See the [toolchain requirements](memory_addr/README.md#toolchain-requirements) for the distinction between the source requirement and Cargo's version guard.

- `memory_addr`([docs](https://docs.rs/memory_addr)|[crates.io](https://crates.io/crates/memory_addr)|[readme](memory_addr/README.md)): Wrappers and helper functions for physical and virtual memory addresses.
- `memory_set`([docs](https://docs.rs/memory_set)|[crates.io](https://crates.io/crates/memory_set)|[readme](memory_set/README.md)): Data structures and operations for managing memory mappings.
