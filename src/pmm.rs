use bootloader::boot_info::{MemoryRegion, MemoryRegionKind};
use lazy_static::lazy_static;
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use crate::BOOT_INFO;
use spin::Mutex;

pub struct BootInfoFrameAllocator {
    memory_regions: &'static [MemoryRegion],
    next: usize,
}

unsafe impl Send for BootInfoFrameAllocator {}

impl BootInfoFrameAllocator {
    pub unsafe fn new() -> Self {
        BootInfoFrameAllocator {
            memory_regions: core::slice::from_raw_parts(BOOT_INFO.memory_regions.as_ptr(), BOOT_INFO.memory_regions.len()),
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_
    {
        let regions = self.memory_regions.iter();
        let usable_regions = regions.filter(|r| r.kind == MemoryRegionKind::Usable);
        let addr_ranges = usable_regions.map(|r| r.start..r.end);
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame>
    {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

lazy_static! {
    pub static ref PMM: Mutex<BootInfoFrameAllocator> = unsafe { Mutex::new(BootInfoFrameAllocator::new()) };
}