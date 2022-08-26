use core::ptr::NonNull;
use acpi::{AcpiHandler, PhysicalMapping};
use log::info;
use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};
use crate::VMM;

#[derive(Clone)]
pub struct KernelAcpiHandler;

impl AcpiHandler for KernelAcpiHandler
{
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T>
    {
        let offset = physical_address % 0x1000;
        let virt_addr = VMM.lock().map_region(
            PhysAddr::new(physical_address as u64),
            size as u64,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[ACPI] Failed to map ACPI region");

        let page_range = {
            let start_page: Page<Size4KiB> = Page::containing_address(virt_addr);
            let end_page : Page<Size4KiB>= Page::containing_address(virt_addr + (offset + size) as u64 - 1u64);
            Page::range_inclusive(start_page, end_page)
        };

        info!("[ACPI] Mapped ACPI region: {:?} to {:?} ({:?}", page_range, physical_address, virt_addr);

        PhysicalMapping::new(
            physical_address,
            NonNull::new_unchecked(virt_addr.as_mut_ptr()),
            size,
            page_range.count() * 0x1000,
            self.clone()
        )
    }

    fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>)
    {
        info!("[ACPI] Unmapping ACPI region: {:?}", Page::range_inclusive(Page::<Size4KiB>::containing_address(VirtAddr::from_ptr(region.virtual_start().as_ptr())),Page::containing_address(VirtAddr::from_ptr(region.virtual_start().as_ptr()) + region.region_length() - 1u64)));
        VMM.lock().unmap_region(VirtAddr::from_ptr(region.virtual_start().as_ptr()), region.region_length() as u64).expect("[ACPI] Failed to unmap memory");
    }
}