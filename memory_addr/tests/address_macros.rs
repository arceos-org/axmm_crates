//! Address macro name resolution tests.
//!
//! Generated implementations use standard library names independently of the
//! caller's scope.

#![feature(
    const_trait_impl,
    const_clone,
    const_cmp,
    const_convert,
    const_default,
    const_ops,
    derive_const
)]

/// Caller types whose names overlap with conversion traits.
///
/// Both declaration forms must remain independent of the local `From` type.
mod shadowed_conversion {
    /// A caller-defined type sharing the conversion trait's name.
    ///
    /// It must not be selected by generated conversion implementations.
    pub struct From;

    memory_addr::def_usize_addr! {
        /// A runtime address declared alongside a conflicting trait name.
        ///
        /// Its conversions must still use the standard library trait.
        pub type RuntimeAddr;
        /// A const address declared alongside a conflicting trait name.
        ///
        /// Its conversions and operators must remain usable in const evaluation.
        pub const type ConstAddr;
    }
}

/// Caller names overlapping with standard library paths and primitive types.
///
/// Generated address storage and operators must retain machine-word semantics.
mod shadowed_paths {
    /// A local module hiding relative paths to the standard library.
    ///
    /// Absolute `::core` paths must bypass this empty module.
    mod core {}

    /// A local alias hiding the primitive machine-word type.
    ///
    /// The address macro must not use this smaller integer representation.
    #[allow(non_camel_case_types)]
    pub type usize = u8;

    memory_addr::def_usize_addr! {
        /// An address declared alongside conflicting standard library names.
        ///
        /// Its storage, conversions, and operators must use the actual primitive type.
        pub const type Addr;
    }

    memory_addr::def_usize_addr_formatter! {
        Addr = "PA:{}";
    }
}

/// Caller macros overlapping with standard derives and helper macros.
///
/// Neither address declarations nor formatters may resolve these local macros.
mod shadowed_macros {
    /// Defines local macros that reject accidental selection by generated code.
    ///
    /// The names are intentionally unused when standard library paths resolve
    /// correctly.
    macro_rules! shadow_names {
        ($($name:ident),+ $(,)?) => {
            $(
                /// A conflicting caller macro.
                ///
                /// Address macro expansion must resolve the standard library definition instead.
                #[allow(unused_macros)]
                macro_rules! $name {
                    () => { compile_error!("a caller macro was selected") };
                }
            )+
        };
    }

    shadow_names!(
        Copy,
        Clone,
        Default,
        Ord,
        PartialOrd,
        Eq,
        PartialEq,
        concat,
        stringify,
        format_args
    );

    memory_addr::def_usize_addr! {
        /// A runtime address declared alongside conflicting macro names.
        ///
        /// Its derives and generated documentation must use standard library macros.
        pub type RuntimeAddr;
        /// A const address declared alongside conflicting macro names.
        ///
        /// Its const derives must use standard library macros as well.
        pub const type ConstAddr;
    }

    memory_addr::def_usize_addr_formatter! {
        RuntimeAddr = "RT:{}";
        ConstAddr = "CT:{}";
    }
}

/// Checks runtime conversions when the caller shadows `From`.
///
/// Both conversion directions preserve machine-sized address values.
#[test]
fn runtime_conversion_ignores_shadowed_from() {
    let _ = shadowed_conversion::From;
    let addr = shadowed_conversion::RuntimeAddr::from(0x1234usize);
    let raw: usize = addr.into();
    assert_eq!(raw, 0x1234);
    assert_eq!(
        shadowed_conversion::RuntimeAddr::from_usize(raw).as_usize(),
        raw
    );
}

/// Checks const conversions when the caller shadows `From`.
///
/// Conversion and address arithmetic retain their const implementations.
#[test]
fn const_conversion_ignores_shadowed_from() {
    const RAW: usize = (shadowed_conversion::ConstAddr::from(0x1234usize) + 1).into();
    const ADDR: shadowed_conversion::ConstAddr = shadowed_conversion::ConstAddr::from_usize(RAW);
    assert_eq!(ADDR.as_usize(), 0x1235);
}

/// Checks machine-word storage and arithmetic despite conflicting type paths.
///
/// Conversion, assignment operators, address subtraction, and formatting use
/// absolute paths.
#[test]
fn standard_paths_ignore_caller_names() {
    let _: shadowed_paths::usize = 0;
    const RAW: usize = {
        let mut addr = shadowed_paths::Addr::from_usize(0x1234);
        addr += 2;
        addr -= 1;
        ((addr + 1) - 1).into()
    };
    assert_eq!(RAW, 0x1235);
    assert_eq!(size_of::<shadowed_paths::Addr>(), size_of::<usize>());
    let addr = shadowed_paths::Addr::from(RAW);
    assert_eq!(addr.as_usize(), RAW);
    assert_eq!(addr - shadowed_paths::Addr::from(0x1234), 1);
    assert_eq!(format!("{addr:?}"), "PA:0x1235");
}

/// Checks ordinary derives despite conflicting caller macros.
///
/// Explicit cloning and comparison exercise the generated trait
/// implementations.
#[test]
#[allow(clippy::clone_on_copy)]
fn runtime_derives_ignore_caller_macros() {
    let zero = shadowed_macros::RuntimeAddr::default();
    let addr = shadowed_macros::RuntimeAddr::from_usize(0x12ab);
    assert_eq!(addr.clone(), addr);
    assert!(zero < addr);
    assert_eq!(addr.as_usize(), 0x12ab);
}

/// Checks const derives despite conflicting caller macros.
///
/// Default construction, cloning, and ordering remain available during const
/// evaluation.
#[test]
#[allow(clippy::clone_on_copy)]
fn const_derives_ignore_caller_macros() {
    const RAW: usize = {
        let zero = shadowed_macros::ConstAddr::default();
        let addr = shadowed_macros::ConstAddr::from_usize(0x12ab);
        assert!(zero < addr);
        assert!(addr.clone() == addr);
        addr.as_usize()
    };
    assert_eq!(RAW, 0x12ab);
}

/// Checks formatting despite a caller-defined `format_args` macro.
///
/// Both declaration forms support debug, lowercase, and uppercase hexadecimal
/// output.
#[test]
fn formatting_ignores_caller_macros() {
    let runtime = shadowed_macros::RuntimeAddr::from_usize(0x12ab);
    let constant = shadowed_macros::ConstAddr::from_usize(0x12ab);
    assert_eq!(
        format!("{runtime:?} {runtime:x} {runtime:X}"),
        "RT:0x12ab RT:0x12ab RT:0x12AB"
    );
    assert_eq!(
        format!("{constant:?} {constant:x} {constant:X}"),
        "CT:0x12ab CT:0x12ab CT:0x12AB"
    );
}
