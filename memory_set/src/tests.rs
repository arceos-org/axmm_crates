use memory_addr::{va_range, MemoryAddr, VirtAddr, PAGE_SIZE_2M, PAGE_SIZE_1G};

use crate::{MappingBackend, MappingError, MemoryArea, MemorySet};

const MAX_ADDR: usize = 0x10000;

type MockFlags = u8;
type MockPageTable = [MockFlags; MAX_ADDR];

#[derive(Clone)]
struct MockBackend;

type MockMemorySet = MemorySet<MockBackend>;

impl MappingBackend for MockBackend {
    type Addr = VirtAddr;
    type Flags = MockFlags;
    type PageTable = MockPageTable;

    fn map(&self, start: VirtAddr, size: usize, flags: MockFlags, pt: &mut MockPageTable) -> bool {
        for entry in pt.iter_mut().skip(start.as_usize()).take(size) {
            if *entry != 0 {
                return false;
            }
            *entry = flags;
        }
        true
    }

    fn unmap(&self, start: VirtAddr, size: usize, pt: &mut MockPageTable) -> bool {
        for entry in pt.iter_mut().skip(start.as_usize()).take(size) {
            if *entry == 0 {
                return false;
            }
            *entry = 0;
        }
        true
    }

    fn protect(
        &self,
        start: VirtAddr,
        size: usize,
        new_flags: MockFlags,
        pt: &mut MockPageTable,
    ) -> bool {
        for entry in pt.iter_mut().skip(start.as_usize()).take(size) {
            if *entry == 0 {
                return false;
            }
            *entry = new_flags;
        }
        true
    }
}

#[derive(Clone)]
struct HugeBackend {
    page_size: usize,
}

type HugeMemorySet = MemorySet<HugeBackend>;

impl MappingBackend for HugeBackend {
    type Addr = VirtAddr;
    type Flags = MockFlags;
    type PageTable = ();

    fn page_size(&self) -> usize {
        self.page_size
    }

    fn map(
        &self,
        start: VirtAddr,
        size: usize,
        flags: MockFlags,
        _pt: &mut (),
    ) -> bool {
        assert!(start.is_aligned(self.page_size));
        assert_eq!(size % self.page_size, 0);
        let _ = flags; // flags are not checked in this mock backend
        true
    }

    fn unmap(&self, start: VirtAddr, size: usize, _pt: &mut ()) -> bool {
        assert!(start.is_aligned(self.page_size));
        assert_eq!(size % self.page_size, 0);
        true
    }

    fn protect(
        &self,
        start: VirtAddr,
        size: usize,
        new_flags: MockFlags,
        _pt: &mut (),
    ) -> bool {
        assert!(start.is_aligned(self.page_size));
        assert_eq!(size % self.page_size, 0);
        let _ = new_flags;
        true
    }
}

macro_rules! assert_ok {
    ($expr: expr) => {
        assert!(($expr).is_ok())
    };
}

macro_rules! assert_err {
    ($expr: expr) => {
        assert!(($expr).is_err())
    };
    ($expr: expr, $err: ident) => {
        assert_eq!(($expr).err(), Some(MappingError::$err))
    };
}

fn dump_memory_set(set: &MockMemorySet) {
    use std::sync::Mutex;
    static DUMP_LOCK: Mutex<()> = Mutex::new(());

    let _lock = DUMP_LOCK.lock().unwrap();
    println!("Number of areas: {}", set.len());
    for area in set.iter() {
        println!("{:?}", area);
    }
}

#[test]
fn test_map_unmap() {
    let mut set = MockMemorySet::new();
    let mut pt = [0; MAX_ADDR];

    // Map [0, 0x1000), [0x2000, 0x3000), [0x4000, 0x5000), ...
    for start in (0..MAX_ADDR).step_by(0x2000) {
        assert_ok!(set.map(
            MemoryArea::new(start.into(), 0x1000, 1, MockBackend),
            &mut pt,
            false,
        ));
    }
    // Map [0x1000, 0x2000), [0x3000, 0x4000), [0x5000, 0x6000), ...
    for start in (0x1000..MAX_ADDR).step_by(0x2000) {
        assert_ok!(set.map(
            MemoryArea::new(start.into(), 0x1000, 2, MockBackend),
            &mut pt,
            false,
        ));
    }
    dump_memory_set(&set);
    assert_eq!(set.len(), 16);
    for &e in &pt[0..MAX_ADDR] {
        assert!(e == 1 || e == 2);
    }

    // Found [0x4000, 0x5000), flags = 1.
    let area = set.find(0x4100.into()).unwrap();
    assert_eq!(area.start(), 0x4000.into());
    assert_eq!(area.end(), 0x5000.into());
    assert_eq!(area.flags(), 1);
    assert_eq!(pt[0x4200], 1);

    // The area [0x4000, 0x8000) is already mapped, map returns an error.
    assert_err!(
        set.map(
            MemoryArea::new(0x4000.into(), 0x4000, 3, MockBackend),
            &mut pt,
            false
        ),
        AlreadyExists
    );
    // Unmap overlapped areas before adding the new mapping [0x4000, 0x8000).
    assert_ok!(set.map(
        MemoryArea::new(0x4000.into(), 0x4000, 3, MockBackend),
        &mut pt,
        true
    ));
    dump_memory_set(&set);
    assert_eq!(set.len(), 13);

    // Found [0x4000, 0x8000), flags = 3.
    let area = set.find(0x4100.into()).unwrap();
    assert_eq!(area.start(), 0x4000.into());
    assert_eq!(area.end(), 0x8000.into());
    assert_eq!(area.flags(), 3);
    for &e in &pt[0x4000..0x8000] {
        assert_eq!(e, 3);
    }

    // Unmap areas in the middle.
    assert_ok!(set.unmap(0x4000.into(), 0x8000, &mut pt));
    assert_eq!(set.len(), 8);
    // Unmap the remaining areas, including the unmapped ranges.
    assert_ok!(set.unmap(0.into(), MAX_ADDR * 2, &mut pt));
    assert_eq!(set.len(), 0);
    for &e in &pt[0..MAX_ADDR] {
        assert_eq!(e, 0);
    }
}

#[test]
fn test_unmap_split() {
    let mut set = MockMemorySet::new();
    let mut pt = [0; MAX_ADDR];

    // Map [0, 0x1000), [0x2000, 0x3000), [0x4000, 0x5000), ...
    for start in (0..MAX_ADDR).step_by(0x2000) {
        assert_ok!(set.map(
            MemoryArea::new(start.into(), 0x1000, 1, MockBackend),
            &mut pt,
            false,
        ));
    }
    assert_eq!(set.len(), 8);

    // Unmap [0xc00, 0x2400), [0x2c00, 0x4400), [0x4c00, 0x6400), ...
    // The areas are shrinked at the left and right boundaries.
    for start in (0..MAX_ADDR).step_by(0x2000) {
        assert_ok!(set.unmap((start + 0xc00).into(), 0x1800, &mut pt));
    }
    dump_memory_set(&set);
    assert_eq!(set.len(), 8);

    for area in set.iter() {
        if area.start().as_usize() == 0 {
            assert_eq!(area.size(), 0xc00);
        } else {
            assert_eq!(area.start().align_offset_4k(), 0x400);
            assert_eq!(area.end().align_offset_4k(), 0xc00);
            assert_eq!(area.size(), 0x800);
        }
        for &e in &pt[area.start().as_usize()..area.end().as_usize()] {
            assert_eq!(e, 1);
        }
    }

    // Unmap [0x800, 0x900), [0x2800, 0x2900), [0x4800, 0x4900), ...
    // The areas are split into two areas.
    for start in (0..MAX_ADDR).step_by(0x2000) {
        assert_ok!(set.unmap((start + 0x800).into(), 0x100, &mut pt));
    }
    dump_memory_set(&set);
    assert_eq!(set.len(), 16);

    for area in set.iter() {
        let off = area.start().align_offset_4k();
        if off == 0 {
            assert_eq!(area.size(), 0x800);
        } else if off == 0x400 {
            assert_eq!(area.size(), 0x400);
        } else if off == 0x900 {
            assert_eq!(area.size(), 0x300);
        } else {
            unreachable!();
        }
        for &e in &pt[area.start().as_usize()..area.end().as_usize()] {
            assert_eq!(e, 1);
        }
    }
    let mut iter = set.iter();
    while let Some(area) = iter.next() {
        if let Some(next) = iter.next() {
            for &e in &pt[area.end().as_usize()..next.start().as_usize()] {
                assert_eq!(e, 0);
            }
        }
    }
    drop(iter);

    // Unmap all areas.
    assert_ok!(set.unmap(0.into(), MAX_ADDR, &mut pt));
    assert_eq!(set.len(), 0);
    for &e in &pt[0..MAX_ADDR] {
        assert_eq!(e, 0);
    }
}

#[test]
fn test_protect() {
    let mut set = MockMemorySet::new();
    let mut pt = [0; MAX_ADDR];
    let update_flags = |new_flags: MockFlags| {
        move |old_flags: MockFlags| -> Option<MockFlags> {
            if (old_flags & 0x7) == (new_flags & 0x7) {
                return None;
            }
            let flags = (new_flags & 0x7) | (old_flags & !0x7);
            Some(flags)
        }
    };

    // Map [0, 0x1000), [0x2000, 0x3000), [0x4000, 0x5000), ...
    for start in (0..MAX_ADDR).step_by(0x2000) {
        assert_ok!(set.map(
            MemoryArea::new(start.into(), 0x1000, 0x7, MockBackend),
            &mut pt,
            false,
        ));
    }
    assert_eq!(set.len(), 8);

    // Protect [0xc00, 0x2400), [0x2c00, 0x4400), [0x4c00, 0x6400), ...
    // The areas are split into two areas.
    for start in (0..MAX_ADDR).step_by(0x2000) {
        assert_ok!(set.protect((start + 0xc00).into(), 0x1800, update_flags(0x1), &mut pt));
    }
    dump_memory_set(&set);
    assert_eq!(set.len(), 23);

    for area in set.iter() {
        let off = area.start().align_offset_4k();
        if area.start().as_usize() == 0 {
            assert_eq!(area.size(), 0xc00);
            assert_eq!(area.flags(), 0x7);
        } else if off == 0 {
            assert_eq!(area.size(), 0x400);
            assert_eq!(area.flags(), 0x1);
        } else if off == 0x400 {
            assert_eq!(area.size(), 0x800);
            assert_eq!(area.flags(), 0x7);
        } else if off == 0xc00 {
            assert_eq!(area.size(), 0x400);
            assert_eq!(area.flags(), 0x1);
        }
    }

    // Protect [0x800, 0x900), [0x2800, 0x2900), [0x4800, 0x4900), ...
    // The areas are split into three areas.
    for start in (0..MAX_ADDR).step_by(0x2000) {
        assert_ok!(set.protect((start + 0x800).into(), 0x100, update_flags(0x13), &mut pt));
    }
    dump_memory_set(&set);
    assert_eq!(set.len(), 39);

    for area in set.iter() {
        let off = area.start().align_offset_4k();
        if area.start().as_usize() == 0 {
            assert_eq!(area.size(), 0x800);
            assert_eq!(area.flags(), 0x7);
        } else if off == 0 {
            assert_eq!(area.size(), 0x400);
            assert_eq!(area.flags(), 0x1);
        } else if off == 0x400 {
            assert_eq!(area.size(), 0x400);
            assert_eq!(area.flags(), 0x7);
        } else if off == 0x800 {
            assert_eq!(area.size(), 0x100);
            assert_eq!(area.flags(), 0x3);
        } else if off == 0x900 {
            assert_eq!(area.size(), 0x300);
            assert_eq!(area.flags(), 0x7);
        } else if off == 0xc00 {
            assert_eq!(area.size(), 0x400);
            assert_eq!(area.flags(), 0x1);
        }
    }

    // Test skip [0x880, 0x900), [0x2880, 0x2900), [0x4880, 0x4900), ...
    for start in (0..MAX_ADDR).step_by(0x2000) {
        assert_ok!(set.protect((start + 0x880).into(), 0x80, update_flags(0x3), &mut pt));
    }
    assert_eq!(set.len(), 39);

    // Unmap all areas.
    assert_ok!(set.unmap(0.into(), MAX_ADDR, &mut pt));
    assert_eq!(set.len(), 0);
    for &e in &pt[0..MAX_ADDR] {
        assert_eq!(e, 0);
    }
}

#[test]
fn test_find_free_area() {
    let mut set = MockMemorySet::new();
    let mut pt = [0; MAX_ADDR];

    // Map [0, 0x1000), [0x2000, 0x3000), ..., [0xe000, 0xf000)
    for start in (0..MAX_ADDR).step_by(0x2000) {
        assert_ok!(set.map(
            MemoryArea::new(start.into(), 0x1000, 1, MockBackend),
            &mut pt,
            false,
        ));
    }

    let addr = set.find_free_area(0.into(), 0x1000, va_range!(0..MAX_ADDR), 1);
    assert_eq!(addr, Some(0x1000.into()));

    let addr = set.find_free_area(0x800.into(), 0x800, va_range!(0..MAX_ADDR), 0x800);
    assert_eq!(addr, Some(0x1000.into()));

    let addr = set.find_free_area(0x1800.into(), 0x800, va_range!(0..MAX_ADDR), 0x800);
    assert_eq!(addr, Some(0x1800.into()));

    let addr = set.find_free_area(0x1800.into(), 0x1000, va_range!(0..MAX_ADDR), 0x1000);
    assert_eq!(addr, Some(0x3000.into()));

    let addr = set.find_free_area(0x2000.into(), 0x1000, va_range!(0..MAX_ADDR), 0x1000);
    assert_eq!(addr, Some(0x3000.into()));

    let addr = set.find_free_area(0xf000.into(), 0x1000, va_range!(0..MAX_ADDR), 0x1000);
    assert_eq!(addr, Some(0xf000.into()));

    let addr = set.find_free_area(0xf001.into(), 0x1000, va_range!(0..MAX_ADDR), 0x1000);
    assert_eq!(addr, None);
}

/// `MemoryArea::split` must reject split positions that are not aligned to the
/// backend page size for large-page backends, while still accepting aligned
/// positions.
#[test]
fn test_large_page_split_alignment() {
    // Helper to test alignment behaviour for a given huge page size.
    fn check_alignment(page_size: usize) {
        let backend = HugeBackend { page_size };
        let mut area = MemoryArea::new(0.into(), 4 * page_size, 1, backend);

        // Unaligned split position should be rejected.
        assert!(area.split((page_size / 2).into()).is_none());

        // Aligned split position inside the area should succeed.
        let right = area
            .split((2 * page_size).into())
            .expect("split at aligned boundary must succeed");
        assert_eq!(area.va_range(), va_range!(0..2 * page_size));
        assert_eq!(
            right.va_range(),
            va_range!(2 * page_size..4 * page_size)
        );
    }

    // Simulate 2 MiB and 1 GiB huge pages by using their actual sizes as
    // `page_size` in the backend. We don't touch a page table here, so large
    // addresses are fine in this unit test.
    check_alignment(PAGE_SIZE_2M);
    check_alignment(PAGE_SIZE_1G);
}

/// `MemorySet::unmap` with huge-page backends (2 MiB / 1 GiB) should only
/// operate on page-size aligned ranges and produce correctly split areas for
/// left/middle/right boundary cases.
#[test]
fn test_huge_page_unmap_boundaries() {
    fn check_unmap_boundaries(page_size: usize) {
        // Helper to create a new set with a single area [0, 4 * page_size).
        fn new_set(page_size: usize) -> (HugeMemorySet, (), usize) {
            let backend = HugeBackend { page_size };
            let mut set = HugeMemorySet::new();
            let mut pt = ();
            let total_size = 4 * page_size;
            assert_ok!(set.map(
                MemoryArea::new(0.into(), total_size, 1, backend),
                &mut pt,
                false,
            ));
            assert_eq!(set.len(), 1);
            (set, pt, total_size)
        }

        // 1) Unmap left boundary: [0, page_size) in [0, 4 * page_size)
        {
            let (mut set, mut pt, total) = new_set(page_size);
            let unmap_start = 0usize;
            let unmap_size = page_size;
            assert_ok!(set.unmap(unmap_start.into(), unmap_size, &mut pt));

            let areas: Vec<_> = set.iter().collect();
            assert_eq!(areas.len(), 1);
            assert_eq!(
                areas[0].va_range(),
                va_range!(page_size..total)
            );
        }

        // 2) Unmap right boundary: [3 * page_size, 4 * page_size) in [0, 4 * page_size)
        {
            let (mut set, mut pt, total) = new_set(page_size);
            let unmap_start = 3 * page_size;
            let unmap_size = page_size;
            assert_ok!(set.unmap(unmap_start.into(), unmap_size, &mut pt));

            let areas: Vec<_> = set.iter().collect();
            assert_eq!(areas.len(), 1);
            assert_eq!(
                areas[0].va_range(),
                va_range!(0..total - page_size)
            );
        }

        // 3) Unmap a middle page: [page_size, 2 * page_size) in [0, 4 * page_size)
        //    Result should be two areas: [0, page_size) and [2 * page_size, 4 * page_size).
        {
            let (mut set, mut pt, total) = new_set(page_size);
            let unmap_start = page_size;
            let unmap_size = page_size;
            assert_ok!(set.unmap(unmap_start.into(), unmap_size, &mut pt));

            let mut areas: Vec<_> = set.iter().collect();
            areas.sort_by_key(|a| Into::<usize>::into(a.start()));
            assert_eq!(areas.len(), 2);
            assert_eq!(areas[0].va_range(), va_range!(0..page_size));
            assert_eq!(
                areas[1].va_range(),
                va_range!(2 * page_size..total)
            );
        }

        // 4) Unmap the entire area: [0, 4 * page_size).
        {
            let (mut set, mut pt, total) = new_set(page_size);
            assert_ok!(set.unmap(0.into(), total, &mut pt));
            assert_eq!(set.len(), 0);
        }
    }

    // Run the boundary tests for 2 MiB and 1 GiB huge pages.
    check_unmap_boundaries(PAGE_SIZE_2M);
    check_unmap_boundaries(PAGE_SIZE_1G);
}

/// Unmapping a sub-page (smaller than the backend page size) within a huge-page
/// area must fail with `InvalidParam` and leave the mappings unchanged.
#[test]
fn test_huge_page_unmap_small_range_error() {
    fn check(page_size: usize) {
        fn new_set(page_size: usize) -> (HugeMemorySet, (), usize) {
            let backend = HugeBackend { page_size };
            let mut set = HugeMemorySet::new();
            let mut pt = ();
            let total = 4 * page_size;

            assert_ok!(set.map(
                MemoryArea::new(0.into(), total, 1, backend),
                &mut pt,
                false,
            ));
            assert_eq!(set.len(), 1);
            (set, pt, total)
        }

        // 1) Left boundary: [0, page_size / 2)
        {
            let (mut set, mut pt, total) = new_set(page_size);
            let res_left = set.unmap(0.into(), page_size / 2, &mut pt);
            assert_eq!(res_left.err(), Some(MappingError::InvalidParam));
            // Mapping should remain intact.
            assert_eq!(set.len(), 1);
            let area = set.iter().next().unwrap();
            assert_eq!(area.va_range(), va_range!(0..total));
        }

        // 2) Middle: [page_size / 2, page_size)
        {
            let (mut set, mut pt, total) = new_set(page_size);
            let res_mid = set.unmap(
                (page_size / 2).into(),
                page_size / 2,
                &mut pt,
            );
            assert_eq!(res_mid.err(), Some(MappingError::InvalidParam));
            assert_eq!(set.len(), 1);
            let area = set.iter().next().unwrap();
            assert_eq!(area.va_range(), va_range!(0..total));
        }

        // 3) Right boundary: [4 * page_size - page_size / 2, 4 * page_size)
        {
            let (mut set, mut pt, total) = new_set(page_size);
            let res_right = set.unmap(
                (total - page_size / 2).into(),
                page_size / 2,
                &mut pt,
            );
            assert_eq!(res_right.err(), Some(MappingError::InvalidParam));
            assert_eq!(set.len(), 1);
            let area = set.iter().next().unwrap();
            assert_eq!(area.va_range(), va_range!(0..total));
        }
    }

    check(PAGE_SIZE_2M);
    check(PAGE_SIZE_1G);
}

/// Protecting a sub-page (smaller than the backend page size) within a
/// huge-page area must also fail with `InvalidParam` and leave the mappings
/// untouched.
#[test]
fn test_huge_page_protect_small_range_error() {
    fn check(page_size: usize) {
        let backend = HugeBackend { page_size };
        let mut set = HugeMemorySet::new();
        let mut pt = ();
        let total = 4 * page_size;

        assert_ok!(set.map(
            MemoryArea::new(0.into(), total, 0x1, backend),
            &mut pt,
            false,
        ));
        assert_eq!(set.len(), 1);

        let update_flags = |new_flags: MockFlags| {
            move |old_flags: MockFlags| -> Option<MockFlags> {
                if old_flags == new_flags {
                    None
                } else {
                    Some(new_flags)
                }
            }
        };

        // 1) Left boundary: [0, page_size / 2)
        assert_err!(
            set.protect(
                0.into(),
                page_size / 2,
                update_flags(0x2),
                &mut pt
            ),
            InvalidParam
        );

        // 2) Middle: [page_size / 2, page_size)
        assert_err!(
            set.protect(
                (page_size / 2).into(),
                page_size / 2,
                update_flags(0x3),
                &mut pt
            ),
            InvalidParam
        );

        // 3) Right boundary: [4 * page_size - page_size / 2, 4 * page_size)
        assert_err!(
            set.protect(
                (total - page_size / 2).into(),
                page_size / 2,
                update_flags(0x4),
                &mut pt
            ),
            InvalidParam
        );

        // The original mapping range should remain unchanged.
        assert_eq!(set.len(), 1);
        let area = set.iter().next().unwrap();
        assert_eq!(area.va_range(), va_range!(0..total));
        assert_eq!(area.flags(), 0x1);
    }

    check(PAGE_SIZE_2M);
    check(PAGE_SIZE_1G);
}
