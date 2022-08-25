use alloc::rc::Rc;
use core::cell::RefCell;
use core::ptr::NonNull;
use acpi::{AcpiHandler, PhysicalMapping};
use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};
use crate::VMM;

pub struct KernelAcpiHandler
{
    pub next_page: VirtAddr
}

#[derive(Clone)]
pub struct RcKernelAcpiHandler
{
    rc: Rc<RefCell<KernelAcpiHandler>>
}

impl KernelAcpiHandler
{
    pub fn new() -> KernelAcpiHandler
    {
        KernelAcpiHandler {
            next_page: VirtAddr::new(0x_4444_5444_0000)
        }
    }
}

impl RcKernelAcpiHandler
{
    pub fn new() -> RcKernelAcpiHandler
    {
        RcKernelAcpiHandler {
            rc: Rc::new(RefCell::new(KernelAcpiHandler::new()))
        }
    }
}

impl AcpiHandler for RcKernelAcpiHandler
{
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T>
    {
        let offset = physical_address % 0x1000;
        let start_page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(self.rc.borrow_mut().next_page.as_u64()));

        VMM.lock().map_region(
            PhysAddr::new(physical_address as u64),
            VirtAddr::new(self.rc.borrow_mut().next_page.as_u64() + offset as u64),
            size as u64, PageTableFlags::PRESENT | PageTableFlags::WRITABLE).expect("[ACPI] Failed to map ACPI region");

        let page_range = {
            let end_page = Page::containing_address(VirtAddr::new(self.rc.borrow_mut().next_page.as_u64() + (offset + size) as u64 - 1 as u64));
            Page::range_inclusive(start_page, end_page)
        };
        self.rc.borrow_mut().next_page += page_range.count() * 0x1000;

        PhysicalMapping::new(
            physical_address,
            NonNull::new_unchecked((start_page.start_address().as_u64() + offset as u64) as *mut T),
            size,
            page_range.count() * 0x1000,
            self.clone()
        )
    }

    fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>)
    {
        VMM.lock().unmap_region(VirtAddr::from_ptr(region.virtual_start().as_ptr()), region.region_length() as u64).expect("[ACPI] Failed to unmap memory");
    }
}