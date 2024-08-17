use crate::{align_down, align_offset, align_up, is_aligned, PAGE_SIZE_4K};

/// A trait for memory addresses.
///
/// Memory addresses here include both physical and virtual addresses, as well as any other
/// similar types like guest physical addresses in a hypervisor.
pub trait MemoryAddr:
    // The address type should be trivially copyable. This implies `Clone`.
    Copy
    // The address type should be convertible to and from `usize`.
    + From<usize>
    + Into<usize>
{
    // Empty for now.
}

/// Implement the `MemoryAddr` trait for any type that satisfies the required bounds.
impl<T> MemoryAddr for T where T: Copy + From<usize> + Into<usize> {}

/// Creates a new address type by wrapping an `usize`.
#[macro_export]
macro_rules! def_addr_types {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident(usize);

        $($tt:tt)*
    ) => {
        #[repr(transparent)]
        #[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
        $(#[$meta])*
        pub struct $name(usize);

        impl $name {
            #[doc = "Converts an `usize` to the address."]
            #[inline]
            pub const fn from_usize(addr: usize) -> Self {
                Self(addr)
            }

            #[doc = "Converts the address to an `usize`."]
            #[inline]
            pub const fn as_usize(self) -> usize {
                self.0
            }
        }

        impl $name {
            /// Aligns the address downwards to the given alignment.
            ///
            /// See the [`align_down`] function for more information.
            #[inline]
            pub fn align_down<U>(self, align: U) -> Self
            where
                U: Into<usize>,
            {
                Self::from_usize(align_down(self.0, align.into()))
            }

            /// Aligns the address upwards to the given alignment.
            ///
            /// See the [`align_up`] function for more information.
            #[inline]
            pub fn align_up<U>(self, align: U) -> Self
            where
                U: Into<usize>,
            {
                Self::from_usize(align_up(self.0, align.into()))
            }

            /// Returns the offset of the address within the given alignment.
            ///
            /// See the [`align_offset`] function for more information.
            #[inline]
            pub fn align_offset<U>(self, align: U) -> usize
            where
                U: Into<usize>,
            {
                align_offset(self.0, align.into())
            }

            /// Checks whether the address has the demanded alignment.
            ///
            /// See the [`is_aligned`] function for more information.
            #[inline]
            pub fn is_aligned<U>(self, align: U) -> bool
            where
                U: Into<usize>,
            {
                is_aligned(self.0, align.into())
            }

            /// Aligns the address downwards to 4096 (bytes).
            #[inline]
            pub const fn align_down_4k(self) -> Self {
                Self::from_usize(align_down(self.0, PAGE_SIZE_4K))
            }

            /// Aligns the address upwards to 4096 (bytes).
            #[inline]
            pub const fn align_up_4k(self) -> Self {
                Self::from_usize(align_up(self.0, PAGE_SIZE_4K))
            }

            /// Returns the offset of the address within a 4K-sized page.
            #[inline]
            pub const fn align_offset_4k(self) -> usize {
                align_offset(self.0, PAGE_SIZE_4K)
            }

            /// Checks whether the address is 4K-aligned.
            #[inline]
            pub const fn is_aligned_4k(self) -> bool {
                is_aligned(self.0, PAGE_SIZE_4K)
            }
        }

        impl From<usize> for $name {
            #[inline]
            fn from(addr: usize) -> Self {
                Self(addr)
            }
        }

        impl From<$name> for usize {
            #[inline]
            fn from(addr: $name) -> usize {
                addr.0
            }
        }

        impl core::ops::Add<usize> for $name {
            type Output = Self;
            #[inline]
            fn add(self, rhs: usize) -> Self {
                Self(self.0 + rhs)
            }
        }

        impl core::ops::AddAssign<usize> for $name {
            #[inline]
            fn add_assign(&mut self, rhs: usize) {
                self.0 += rhs;
            }
        }

        impl core::ops::Sub<usize> for $name {
            type Output = Self;
            #[inline]
            fn sub(self, rhs: usize) -> Self {
                Self(self.0 - rhs)
            }
        }

        impl core::ops::SubAssign<usize> for $name {
            #[inline]
            fn sub_assign(&mut self, rhs: usize) {
                self.0 -= rhs;
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_fmt(format_args!(concat!(stringify!($name), "{:#x}"), self.0))
            }
        }

        impl core::fmt::LowerHex for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_fmt(format_args!(concat!(stringify!($name), "{:#x}"), self.0))
            }
        }

        impl core::fmt::UpperHex for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_fmt(format_args!(concat!(stringify!($name), "{:#X}"), self.0))
            }
        }

        $crate::def_addr_types!($($tt)*);
    };
    () => {};
}

def_addr_types! {
    #[doc = "A physical memory address."]
    pub struct PhysAddr(usize);

    #[doc = "A virtual memory address."]
    pub struct VirtAddr(usize);
}

/// Alias for [`PhysAddr::from`].
#[macro_export]
macro_rules! pa {
    ($addr:expr) => {
        $crate::PhysAddr::from_usize($addr)
    };
}

/// Alias for [`VirtAddr::from`].
#[macro_export]
macro_rules! va {
    ($addr:expr) => {
        $crate::VirtAddr::from_usize($addr)
    };
}
