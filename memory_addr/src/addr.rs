//! Memory address types, arithmetic, and declaration macros.
//!
//! Address wrappers distinguish physical and virtual addresses while retaining
//! lossless conversions to and from `usize`.

use core::{
    cmp::Ord,
    marker::Destruct,
    ops::{Add, Sub},
};

/// A trait for memory address types.
///
/// Memory address types here include both physical and virtual addresses, as
/// well as any other similar types like guest physical addresses in a
/// hypervisor.
///
/// This trait is automatically implemented for any type that is `Copy`,
/// `From<usize>`, `Into<usize>`, and `Ord`, and supports `Add<usize>` and
/// `Sub<usize>` with itself as the output type. It provides utility methods
/// for address alignment and arithmetic.
///
/// Address types must satisfy the following semantic requirements.
/// These requirements are not enforced by the blanket implementation:
///
/// - Conversions to and from `usize` are lossless and succeed for every value.
/// - Converting an address to `usize` and back produces the same address.
///   Converting a `usize` to an address and back produces the same integer.
/// - Address ordering agrees with the ordering of the corresponding `usize`
///   values.
/// - `Add<usize>` and `Sub<usize>` use offsets in bytes.
/// - Without overflow, converting the result of addition or subtraction to
///   `usize` must give exactly the same value as converting the address to
///   `usize` first and performing the same addition or subtraction.
/// - On overflow, `Add<usize>` and `Sub<usize>` must behave like the
///   corresponding `usize` operators.
pub const trait MemoryAddr:
    // The address type should be trivially copyable. This implies `Clone`.
    Copy
    // The address type should be convertible to and from `usize`.
    + [const] From<usize>
    + [const] Into<usize>
    // The address type should be able to perform arithmetic with `usize`.
    + [const] Add<usize, Output = Self>
    + [const] Sub<usize, Output = Self>
    // The address type should be comparable.
    + [const] Ord
    // The address type should be const-destructible when used in a const context.
    + [const] Destruct
{
    // No required methods for now. Following are some utility methods.

    //
    // This section contains utility methods for address alignment.
    //

    /// Aligns the address downwards to the given alignment.
    ///
    /// The alignment must convert to a nonzero power of two. Delegates to [`crate::align_down`].
    #[inline]
    #[must_use = "this returns a new address, without modifying the original"]
    fn align_down<U>(self, align: U) -> Self
    where
        U: [const] Into<usize>,
    {
        Self::from(crate::align_down(self.into(), align.into()))
    }

    /// Aligns the address upwards to the given alignment.
    ///
    /// The alignment must convert to a nonzero power of two. Delegates to [`crate::align_up`],
    /// including its overflow behavior.
    #[inline]
    #[must_use = "this returns a new address, without modifying the original"]
    fn align_up<U>(self, align: U) -> Self
    where
        U: [const] Into<usize>,
    {
        Self::from(crate::align_up(self.into(), align.into()))
    }

    /// Aligns the address upwards to the given alignment in a checked manner.
    ///
    /// The alignment must convert to a nonzero power of two. Delegates to [`crate::align_up_checked`],
    /// returning `None` if the resulting address would overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_addr::{va, MemoryAddr};
    ///
    /// assert_eq!(va!(0x1234).align_up_checked(0x1000usize), Some(va!(0x2000)));
    /// assert_eq!(va!(usize::MAX - 1).align_up_checked(4usize), None);
    /// ```
    #[inline]
    fn align_up_checked<U>(self, align: U) -> Option<Self>
    where
        U: [const] Into<usize>,
    {
        crate::align_up_checked(self.into(), align.into()).map(Self::from)
    }

    /// Returns the offset of the address within the given alignment.
    ///
    /// The alignment must convert to a nonzero power of two. The result is measured in bytes.
    #[inline]
    #[must_use = "this function has no side effects, so it can be removed if the return value is not used"]
    fn align_offset<U>(self, align: U) -> usize
    where
        U: [const] Into<usize>,
    {
        crate::align_offset(self.into(), align.into())
    }

    /// Checks whether the address has the requested alignment.
    ///
    /// The alignment must convert to a nonzero power of two. Aligned addresses have offset zero.
    #[inline]
    #[must_use = "this function has no side effects, so it can be removed if the return value is not used"]
    fn is_aligned<U>(self, align: U) -> bool
    where
        U: [const] Into<usize>,
    {
        crate::is_aligned(self.into(), align.into())
    }

    /// Aligns the address downwards to a 4K boundary.
    ///
    /// Equivalent to [`align_down`](Self::align_down) with [`crate::PAGE_SIZE_4K`].
    #[inline]
    #[must_use = "this returns a new address, without modifying the original"]
    fn align_down_4k(self) -> Self {
        Self::from(crate::align_down(self.into(), crate::PAGE_SIZE_4K))
    }

    /// Aligns the address upwards to a 4K boundary.
    ///
    /// Equivalent to [`align_up`](Self::align_up) with [`crate::PAGE_SIZE_4K`].
    #[inline]
    #[must_use = "this returns a new address, without modifying the original"]
    fn align_up_4k(self) -> Self {
        Self::from(crate::align_up(self.into(), crate::PAGE_SIZE_4K))
    }

    /// Returns the offset of the address within a 4K-sized page.
    ///
    /// The byte offset is always less than [`crate::PAGE_SIZE_4K`].
    #[inline]
    #[must_use = "this function has no side effects, so it can be removed if the return value is not used"]
    fn align_offset_4k(self) -> usize {
        crate::align_offset(self.into(), crate::PAGE_SIZE_4K)
    }

    /// Checks whether the address is 4K-aligned.
    ///
    /// Equivalent to [`is_aligned`](Self::is_aligned) with [`crate::PAGE_SIZE_4K`].
    #[inline]
    #[must_use = "this function has no side effects, so it can be removed if the return value is not used"]
    fn is_aligned_4k(self) -> bool {
        crate::is_aligned(self.into(), crate::PAGE_SIZE_4K)
    }

    //
    // This section contains utility methods for address arithmetic.
    //

    /// Adds a signed byte offset to the address.
    ///
    /// Negative offsets move towards lower addresses.
    ///
    /// # Panics
    ///
    /// Panics if the result overflows.
    #[inline]
    #[must_use = "this returns a new address, without modifying the original"]
    fn offset(self, offset: isize) -> Self {
        // todo: use `strict_add_signed` when it's stable.
        Self::from(usize::checked_add_signed(self.into(), offset).expect("overflow in `MemoryAddr::offset`"))
    }

    /// Adds a signed byte offset with wrapping arithmetic.
    ///
    /// Wraps around the address space on overflow instead of panicking.
    #[inline]
    #[must_use = "this returns a new address, without modifying the original"]
    fn wrapping_offset(self, offset: isize) -> Self {
        Self::from(usize::wrapping_add_signed(self.into(), offset))
    }

    /// Returns the signed byte offset from `base` to this address.
    ///
    /// Addresses below `base` produce negative offsets. Equal addresses produce zero.
    ///
    /// # Panics
    ///
    /// Panics if the result is not representable by `isize`.
    #[inline]
    #[must_use = "this function has no side effects, so it can be removed if the return value is not used"]
    fn offset_from(self, base: Self) -> isize {
        let result = usize::wrapping_sub(self.into(), base.into()) as isize;
        if (result > 0) ^ (base < self) {
            // The result has overflowed.
            panic!("overflow in `MemoryAddr::offset_from`");
        } else {
            result
        }
    }

    /// Adds an unsigned byte offset with wrapping arithmetic.
    ///
    /// Wraps around the address space on overflow instead of panicking.
    #[inline]
    #[must_use = "this returns a new address, without modifying the original"]
    fn wrapping_add(self, rhs: usize) -> Self {
        Self::from(usize::wrapping_add(self.into(), rhs))
    }

    /// Adds an unsigned byte offset and reports overflow.
    ///
    /// Returns the wrapped result and a flag indicating whether the addition overflowed.
    #[inline]
    #[must_use = "this returns a new address, without modifying the original"]
    fn overflowing_add(self, rhs: usize) -> (Self, bool) {
        let (result, overflow) = self.into().overflowing_add(rhs);
        (Self::from(result), overflow)
    }

    /// Adds an unsigned byte offset if the result is representable.
    ///
    /// Returns `None` on overflow.
    #[inline]
    #[must_use = "this returns a new address, without modifying the original"]
    fn checked_add(self, rhs: usize) -> Option<Self> {
        usize::checked_add(self.into(), rhs).map(Self::from)
    }

    /// Subtracts an unsigned byte offset with wrapping arithmetic.
    ///
    /// Wraps around the address space on underflow instead of panicking.
    #[inline]
    #[must_use = "this returns a new address, without modifying the original"]
    fn wrapping_sub(self, rhs: usize) -> Self {
        Self::from(usize::wrapping_sub(self.into(), rhs))
    }

    /// Subtracts an unsigned byte offset and reports underflow.
    ///
    /// Returns the wrapped result and a flag indicating whether the subtraction underflowed.
    #[inline]
    #[must_use = "this returns a new address, without modifying the original"]
    fn overflowing_sub(self, rhs: usize) -> (Self, bool) {
        let (result, overflow) = self.into().overflowing_sub(rhs);
        (Self::from(result), overflow)
    }

    /// Subtracts an unsigned byte offset if the result is representable.
    ///
    /// Returns `None` on underflow.
    #[inline]
    #[must_use = "this returns a new address, without modifying the original"]
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        usize::checked_sub(self.into(), rhs).map(Self::from)
    }

    /// Returns the unsigned byte offset from `rhs` to this address.
    ///
    /// Equal addresses produce zero.
    ///
    /// # Panics
    ///
    /// Panics if this address is below `rhs`.
    #[inline]
    #[must_use = "this function has no side effects, so it can be removed if the return value is not used"]
    fn sub_addr(self, rhs: Self) -> usize {
        usize::checked_sub(self.into(), rhs.into()).expect("overflow in `MemoryAddr::sub_addr`")
    }

    /// Returns the wrapping unsigned byte offset from `rhs` to this address.
    ///
    /// Wraps around on underflow instead of panicking when this address is below `rhs`.
    #[inline]
    #[must_use = "this function has no side effects, so it can be removed if the return value is not used"]
    fn wrapping_sub_addr(self, rhs: Self) -> usize {
        usize::wrapping_sub(self.into(), rhs.into())
    }

    /// Returns the unsigned byte offset from `rhs` and reports underflow.
    ///
    /// Returns the wrapped offset and a flag indicating whether this address is below `rhs`.
    #[inline]
    #[must_use = "this function has no side effects, so it can be removed if the return value is not used"]
    fn overflowing_sub_addr(self, rhs: Self) -> (usize, bool) {
        usize::overflowing_sub(self.into(), rhs.into())
    }

    /// Returns the unsigned byte offset from `rhs` if it is representable.
    ///
    /// Returns `None` if this address is below `rhs`.
    #[inline]
    #[must_use = "this function has no side effects, so it can be removed if the return value is not used"]
    fn checked_sub_addr(self, rhs: Self) -> Option<usize> {
        usize::checked_sub(self.into(), rhs.into())
    }
}

/// Implements address utilities for types satisfying the address trait bounds.
///
/// In addition to copying, conversions, and ordering, address types support
/// addition and subtraction of unsigned offsets with an address output.
const impl<T> MemoryAddr for T where
    T: Copy
        + [const] From<usize>
        + [const] Into<usize>
        + [const] Add<usize, Output = Self>
        + [const] Sub<usize, Output = Self>
        + [const] Ord
        + [const] Destruct
{
}

/// Creates a new address type by wrapping a `usize`.
///
/// For each `$vis type $name;`, this macro generates the following items:
/// - Definition of the new address type `$name`, which contains a single
///   private unnamed field of type `usize`.
/// - Default implementations (i.e. derived implementations) for the following
///   traits:
///   - `Copy`, `Clone`,
///   - `Default`,
///   - `Ord`, `PartialOrd`, `Eq`, and `PartialEq`.
/// - Implementations for the following traits:
///   - `From<usize>`, `Into<usize>` (by implementing `From<$name> for usize`),
///   - `Add<usize>`, `AddAssign<usize>`, `Sub<usize>`, `SubAssign<usize>`, and
///   - `Sub<$name>`.
/// - Two `const` methods to convert between the address type and `usize`:
///   - `from_usize`, which converts a `usize` to the address type, and
///   - `as_usize`, which converts the address type to a `usize`.
///
/// # Example
///
/// ```
/// use memory_addr::{def_usize_addr, MemoryAddr};
///
/// def_usize_addr! {
///     /// An example address type.
///     ///
///     /// Stores a raw address for the conversion and alignment examples.
///     #[derive(Debug)]
///     pub type ExampleAddr;
/// }
///
/// # fn main() {
/// const EXAMPLE: ExampleAddr = ExampleAddr::from_usize(0x1234);
/// const EXAMPLE_USIZE: usize = EXAMPLE.as_usize();
/// assert_eq!(EXAMPLE_USIZE, 0x1234);
/// assert_eq!(EXAMPLE.align_down(0x10usize), ExampleAddr::from_usize(0x1230));
/// assert_eq!(EXAMPLE.align_up_4k(), ExampleAddr::from_usize(0x2000));
/// # }
/// ```
///
/// # Const implementations
///
/// Write `$vis const type $name;` to generate const implementations of the
/// derived traits other than `Copy`, conversions, and arithmetic operators.
/// Without `const`, those implementations are ordinary runtime implementations.
///
/// The const form requires the calling crate to enable `const_trait_impl`,
/// `derive_const`, `const_clone`, `const_default`, `const_cmp`,
/// `const_convert`, and `const_ops`. The ordinary form requires no feature
/// attributes in the calling crate.
///
/// The two inherent conversion methods remain const in both forms.
///
/// ```
/// #![feature(const_trait_impl, derive_const, const_clone, const_default)]
/// #![feature(const_cmp, const_convert, const_ops)]
/// use memory_addr::{def_usize_addr, MemoryAddr};
///
/// def_usize_addr! {
///     pub const type ConstAddr;
/// }
///
/// const ALIGNED: ConstAddr = ConstAddr::from(0x1234usize).align_down_4k();
/// const NEXT: ConstAddr = ALIGNED + 0x1000;
/// assert_eq!(NEXT.as_usize(), 0x2000);
/// ```
///
/// # Visibility
///
/// The generated type preserves `$vis`. Private types remain private to the
/// module declaring them.
///
/// ```compile_fail,E0603
/// mod inner {
///     memory_addr::def_usize_addr! { type PrivateAddr; }
/// }
/// let _ = inner::PrivateAddr::from_usize(0);
/// ```
///
/// Restricted visibility is preserved as well.
///
/// ```compile_fail,E0603
/// mod outer {
///     pub mod inner {
///         memory_addr::def_usize_addr! { pub(super) type ParentAddr; }
///     }
/// }
/// let _ = outer::inner::ParentAddr::from_usize(0);
/// ```
#[macro_export]
macro_rules! def_usize_addr {
    // "const" is specified, we generate const implementations of traits and use `derive_const` to
    // derive the const implementations.
    (
        $(#[$meta:meta])*
        $vis:vis const type $name:ident;
        $($tt:tt)*
    ) => {
        $crate::def_usize_addr!(@define [const] [derive_const] $(#[$meta])* $vis type $name;);
        $crate::def_usize_addr!($($tt)*);
    };
    // "const" is not specified, we generate runtime implementations of traits.
    (
        $(#[$meta:meta])*
        $vis:vis type $name:ident;
        $($tt:tt)*
    ) => {
        $crate::def_usize_addr!(@define [] [derive] $(#[$meta])* $vis type $name;);
        $crate::def_usize_addr!($($tt)*);
    };
    // The actual implementation of the macro.
    (
        @define [$($constness:tt)?] [$derive_constness:ident]
        $(#[$meta:meta])*
        $vis:vis type $name:ident;
    ) => {
        #[repr(transparent)]
        #[derive(::core::marker::Copy)]
        #[$derive_constness(
            ::core::clone::Clone,
            ::core::default::Default,
            ::core::cmp::Ord,
            ::core::cmp::PartialOrd,
            ::core::cmp::Eq,
            ::core::cmp::PartialEq
        )]
        $(#[$meta])*
        $vis struct $name(::core::primitive::usize);

        impl $name {
            #[doc = ::core::concat!("Converts a `usize` to [`", ::core::stringify!($name), "`].")]
            #[doc = ""]
            #[doc = "Preserves the raw value without validating architecture-specific address limits."]
            #[inline]
            pub const fn from_usize(addr: ::core::primitive::usize) -> Self {
                Self(addr)
            }

            #[doc = ::core::concat!("Converts [`", ::core::stringify!($name), "`] to a `usize`.")]
            #[doc = ""]
            #[doc = "Returns the unchanged raw address value."]
            #[inline]
            pub const fn as_usize(self) -> ::core::primitive::usize {
                self.0
            }
        }

        $($constness)? impl ::core::convert::From<::core::primitive::usize> for $name {
            #[inline]
            fn from(addr: ::core::primitive::usize) -> Self {
                Self(addr)
            }
        }

        $($constness)? impl ::core::convert::From<$name> for ::core::primitive::usize {
            #[inline]
            fn from(addr: $name) -> ::core::primitive::usize {
                addr.0
            }
        }

        $($constness)? impl ::core::ops::Add<::core::primitive::usize> for $name {
            type Output = Self;
            #[inline]
            fn add(self, rhs: ::core::primitive::usize) -> Self {
                Self(self.0 + rhs)
            }
        }

        $($constness)? impl ::core::ops::AddAssign<::core::primitive::usize> for $name {
            #[inline]
            fn add_assign(&mut self, rhs: ::core::primitive::usize) {
                self.0 += rhs;
            }
        }

        $($constness)? impl ::core::ops::Sub<::core::primitive::usize> for $name {
            type Output = Self;
            #[inline]
            fn sub(self, rhs: ::core::primitive::usize) -> Self {
                Self(self.0 - rhs)
            }
        }

        $($constness)? impl ::core::ops::SubAssign<::core::primitive::usize> for $name {
            #[inline]
            fn sub_assign(&mut self, rhs: ::core::primitive::usize) {
                self.0 -= rhs;
            }
        }

        $($constness)? impl ::core::ops::Sub<$name> for $name {
            type Output = ::core::primitive::usize;
            #[inline]
            fn sub(self, rhs: $name) -> ::core::primitive::usize {
                self.0 - rhs.0
            }
        }

    };
    () => {};
}

/// Creates debug and hexadecimal formatting implementations for address types.
///
/// Implements [`Debug`](core::fmt::Debug), [`LowerHex`](core::fmt::LowerHex),
/// and [`UpperHex`](core::fmt::UpperHex) for types defined by
/// [`def_usize_addr`](crate::def_usize_addr).
///
/// For each `$name = $format;`, this macro generates the following items:
/// - An implementation of [`core::fmt::Debug`] for the address type `$name`,
///   which formats the address with `format_args!($format,
///   format_args!("{:#x}", self.0))`,
/// - An implementation of [`core::fmt::LowerHex`] for the address type `$name`,
///   which formats the address in the same way as [`core::fmt::Debug`],
/// - An implementation of [`core::fmt::UpperHex`] for the address type `$name`,
///   which formats the address with `format_args!($format,
///   format_args!("{:#X}", self.0))`.
///
/// # Example
///
/// ```
/// use memory_addr::{PhysAddr, VirtAddr, def_usize_addr, def_usize_addr_formatter};
///
/// def_usize_addr! {
///     /// An example address type.
///     ///
///     /// Uses the custom `EA` prefix in its hexadecimal representation.
///     pub type ExampleAddr;
/// }
///
/// def_usize_addr_formatter! {
///     ExampleAddr = "EA:{}";
/// }
///
/// # fn main() {
/// assert_eq!(format!("{:?}", PhysAddr::from(0x1abc)), "PA:0x1abc");
/// assert_eq!(format!("{:x}", VirtAddr::from(0x1abc)), "VA:0x1abc");
/// assert_eq!(format!("{:X}", ExampleAddr::from(0x1abc)), "EA:0x1ABC");
/// # }
/// ```
#[macro_export]
macro_rules! def_usize_addr_formatter {
    (
        $name:ident = $format:literal;

        $($tt:tt)*
    ) => {
        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.write_fmt(::core::format_args!($format, ::core::format_args!("{:#x}", self.0)))
            }
        }

        impl ::core::fmt::LowerHex for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.write_fmt(::core::format_args!($format, ::core::format_args!("{:#x}", self.0)))
            }
        }

        impl ::core::fmt::UpperHex for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.write_fmt(::core::format_args!($format, ::core::format_args!("{:#X}", self.0)))
            }
        }

        $crate::def_usize_addr_formatter!($($tt)*);
    };
    () => {};
}

def_usize_addr! {
    /// A physical memory address.
    ///
    /// Stores a raw physical address without validating architecture-specific limits.
    pub const type PhysAddr;

    /// A virtual memory address.
    ///
    /// Stores a raw virtual address without checking whether it is canonical or mapped.
    pub const type VirtAddr;
}

def_usize_addr_formatter! {
    PhysAddr = "PA:{}";
    VirtAddr = "VA:{}";
}

/// Raw pointer conversions for virtual addresses.
///
/// These conversions do not dereference pointers or validate their targets.
impl VirtAddr {
    /// Creates a new virtual address from a raw pointer.
    ///
    /// Records the pointer's address without accessing its target.
    #[inline]
    pub fn from_ptr_of<T>(ptr: *const T) -> Self {
        Self(ptr as usize)
    }

    /// Creates a new virtual address from a mutable raw pointer.
    ///
    /// Records the pointer's address without accessing its target.
    #[inline]
    pub fn from_mut_ptr_of<T>(ptr: *mut T) -> Self {
        Self(ptr as usize)
    }

    /// Converts the virtual address to a raw pointer.
    ///
    /// The result points to bytes. This conversion does not validate
    /// dereference safety.
    #[inline]
    pub const fn as_ptr(self) -> *const u8 {
        self.0 as *const u8
    }

    /// Converts the virtual address to a raw pointer of a specific type.
    ///
    /// This conversion does not check alignment or whether the target contains
    /// a valid `T`.
    #[inline]
    pub const fn as_ptr_of<T>(self) -> *const T {
        self.0 as *const T
    }

    /// Converts the virtual address to a mutable raw pointer.
    ///
    /// The result points to bytes. This conversion does not validate
    /// dereference safety.
    #[inline]
    pub const fn as_mut_ptr(self) -> *mut u8 {
        self.0 as *mut u8
    }

    /// Converts the virtual address to a mutable raw pointer to `T`.
    ///
    /// This conversion does not check alignment or whether the target contains
    /// a valid `T`.
    #[inline]
    pub const fn as_mut_ptr_of<T>(self) -> *mut T {
        self.0 as *mut T
    }
}

/// Creates a physical address from a `usize` expression.
///
/// Expands to [`PhysAddr::from_usize`] without validating the address.
#[macro_export]
macro_rules! pa {
    ($addr:expr) => {
        $crate::PhysAddr::from_usize($addr)
    };
}

/// Creates a virtual address from a `usize` expression.
///
/// Expands to [`VirtAddr::from_usize`] without validating the address.
#[macro_export]
macro_rules! va {
    ($addr:expr) => {
        $crate::VirtAddr::from_usize($addr)
    };
}
