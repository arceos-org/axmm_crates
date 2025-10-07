use memory_addr::MemoryAddr;

/// Underlying operations to do when manipulating mappings within the specific
/// [`MemoryArea`](crate::MemoryArea).
///
/// The backend can be different for different memory areas. e.g., for linear
/// mappings, the target physical address is known when it is added to the page
/// table. For lazy mappings, an empty mapping needs to be added to the page
/// table to trigger a page fault.
pub trait MappingBackend: Clone {
    /// The address type used in the memory area.
    type Addr: MemoryAddr;
    /// The flags type used in the memory area.
    type Flags: Copy;
    /// The page table type used in the memory area.
    type PageTable;

    /// The page size (in bytes) used by this backend.
    ///
    /// This is mainly used by [`MemoryArea`](crate::MemoryArea) to avoid
    /// splitting a mapping at positions that are not page-aligned for this
    /// backend. By default, the value is `1`, which means "no additional
    /// restriction" (any position is treated as aligned).
    #[inline]
    fn page_size(&self) -> usize {
        1
    }

    /// What to do when mapping a region within the area with the given flags.
    fn map(
        &self,
        start: Self::Addr,
        size: usize,
        flags: Self::Flags,
        page_table: &mut Self::PageTable,
    ) -> bool;

    /// What to do when unmaping a memory region within the area.
    fn unmap(&self, start: Self::Addr, size: usize, page_table: &mut Self::PageTable) -> bool;

    /// What to do when changing access flags.
    fn protect(
        &self,
        start: Self::Addr,
        size: usize,
        new_flags: Self::Flags,
        page_table: &mut Self::PageTable,
    ) -> bool;
}
