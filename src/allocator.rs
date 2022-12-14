use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::VirtAddr;
use crate::VMM;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 1 << 20; // 1MiB 0b100000000000000000000

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() -> Result<(), MapToError<Size4KiB>>
{
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range
    {
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { VMM.lock().map(page, flags)?.flush() };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    Ok(())
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> !
{
    panic!("allocation error: {:?}", layout);
}