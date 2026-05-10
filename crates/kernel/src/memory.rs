use core::sync::atomic::{AtomicU64, Ordering};

use limine::memory_map::{Entry, EntryType};
use linked_list_allocator::LockedHeap;
use spin::Mutex;
use x86_64::VirtAddr;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::PhysAddr;

const MAX_REGIONS: usize = 128;
const HEAP_START: u64 = 0xFFFF_9000_0000_0000;
const HEAP_SIZE: usize = 1024 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[derive(Clone, Copy, Default)]
struct Region {
    start: u64,
    end: u64,
}

struct BootFrameAllocator {
    regions: [Region; MAX_REGIONS],
    region_count: usize,
    region_idx: usize,
    next_addr: u64,
    allocated: u64,
}

impl BootFrameAllocator {
    fn new(memory_map: &'static [&'static Entry]) -> Self {
        let mut regions = [Region::default(); MAX_REGIONS];
        let mut region_count = 0usize;

        for entry in memory_map {
            if entry.entry_type != EntryType::USABLE || entry.length < 4096 {
                continue;
            }
            if region_count == MAX_REGIONS {
                break;
            }
            let start = align_up(entry.base, 4096);
            let end = align_down(entry.base + entry.length, 4096);
            if start >= end {
                continue;
            }
            regions[region_count] = Region { start, end };
            region_count += 1;
        }

        regions[..region_count].sort_by_key(|v| v.start);

        let next_addr = if region_count > 0 {
            regions[0].start
        } else {
            0
        };

        Self {
            regions,
            region_count,
            region_idx: 0,
            next_addr,
            allocated: 0,
        }
    }

    fn allocate_phys_addr(&mut self) -> Option<u64> {
        while self.region_idx < self.region_count {
            let region = self.regions[self.region_idx];
            if self.next_addr < region.end {
                let addr = self.next_addr;
                self.next_addr += 4096;
                self.allocated += 1;
                return Some(addr);
            }
            self.region_idx += 1;
            if self.region_idx < self.region_count {
                self.next_addr = self.regions[self.region_idx].start;
            }
        }
        None
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let addr = self.allocate_phys_addr()?;
        Some(PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

static HHDM_OFFSET: AtomicU64 = AtomicU64::new(0);
static FRAME_ALLOCATOR: Mutex<Option<BootFrameAllocator>> = Mutex::new(None);

pub fn init_boot_memory(hhdm_offset: u64, memory_map: &'static [&'static Entry]) {
    HHDM_OFFSET.store(hhdm_offset, Ordering::Release);
    *FRAME_ALLOCATOR.lock() = Some(BootFrameAllocator::new(memory_map));
}

pub fn init_heap() {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1;
        let start_page = Page::containing_address(heap_start);
        let end_page = Page::containing_address(heap_end);
        Page::range_inclusive(start_page, end_page)
    };

    for page in page_range {
        let frame = allocate_frame().expect("out of frames while creating heap");
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { map_page(page, frame, flags).expect("failed to map heap page") };
    }

    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_START as usize as *mut u8, HEAP_SIZE);
    }
}

pub fn allocate_frame() -> Option<PhysFrame<Size4KiB>> {
    FRAME_ALLOCATOR.lock().as_mut()?.allocate_frame()
}

pub unsafe fn map_user_page(
    virtual_addr: u64,
    writable: bool,
) -> Result<PhysFrame, MapToError<Size4KiB>> {
    let page = Page::containing_address(VirtAddr::new(virtual_addr));
    let frame = allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
    let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
    if writable {
        flags |= PageTableFlags::WRITABLE;
    }
    map_page(page, frame, flags)?;
    Ok(frame)
}

pub unsafe fn map_page(
    page: Page<Size4KiB>,
    frame: PhysFrame<Size4KiB>,
    flags: PageTableFlags,
) -> Result<(), MapToError<Size4KiB>> {
    let mut frame_alloc_guard = FRAME_ALLOCATOR.lock();
    let frame_alloc = frame_alloc_guard
        .as_mut()
        .expect("frame allocator not initialized");
    let mut mapper = offset_page_table();
    let map_flush = mapper.map_to(page, frame, flags, frame_alloc)?;
    map_flush.flush();
    Ok(())
}

pub unsafe fn write_to_virtual(addr: u64, bytes: &[u8]) {
    let ptr = addr as *mut u8;
    core::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
}

pub fn frame_allocation_count() -> u64 {
    FRAME_ALLOCATOR
        .lock()
        .as_ref()
        .map(|v| v.allocated)
        .unwrap_or(0)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_ptr
}

unsafe fn offset_page_table() -> OffsetPageTable<'static> {
    let physical_memory_offset = VirtAddr::new(HHDM_OFFSET.load(Ordering::Acquire));
    OffsetPageTable::new(
        active_level_4_table(physical_memory_offset),
        physical_memory_offset,
    )
}

const fn align_up(value: u64, align: u64) -> u64 {
    (value + align - 1) & !(align - 1)
}

const fn align_down(value: u64, align: u64) -> u64 {
    value & !(align - 1)
}
