use alloc::rc::Rc;
use core::cell::RefCell;
use core::ptr::NonNull;
use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};
use log::{error, info};
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};
use crate::{BOOT_INFO, VMM};

#[derive(Clone)]
struct KernelAcpiHandler
{
    pub next_page: VirtAddr
}

#[derive(Clone)]
struct RcKernelAcpiHandler
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

pub unsafe fn init()
{
    let handler = RcKernelAcpiHandler::new();
    let rsdp_phys_addr = BOOT_INFO.rsdp_addr.into_option().unwrap();

    match AcpiTables::from_rsdp(handler, rsdp_phys_addr as usize)
    {
        Err(err) => error!("[ACPI] {:?}", err),
        Ok(tables) =>
            {
                info!("[ACPI] Found Tables");
                match tables.platform_info()
                {
                    Err(err) => error!("[ACPI] {:?}", err),
                    Ok(platform_info) =>
                        {
                            info!("[ACPI] Platform info: power_profile: {:?}, interrupt_model: {:?}",
                                platform_info.power_profile,
                                platform_info.interrupt_model
                            );

                            if let Some(cpu_info) = platform_info.processor_info
                            {
                                info!("[ACPI] CPU info: boot_processor: {:?}, application_processors: {:?}", cpu_info.boot_processor, cpu_info.application_processors);
                            }
                            else
                            {
                                error!("[ACPI] No CPU info");
                            }

                            if let Some(pm_timer) = platform_info.pm_timer
                            {
                                info!("[ACPI] PM timer: base: {:?}, supports_32bit: {}", pm_timer.base, pm_timer.supports_32bit);
                            }
                            else
                            {
                                error!("[ACPI] No PM timer");
                            }
                        }
                }
            }
    }
}

impl AcpiHandler for RcKernelAcpiHandler
{
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T>
    {
        let physical_address_frame = PhysFrame::containing_address(PhysAddr::new(physical_address as u64));
        let offset = physical_address % 0x1000;
        let start_page = Page::containing_address(VirtAddr::new(self.rc.borrow_mut().next_page.as_u64()));
        let page_range = {
            let end_page = Page::containing_address(VirtAddr::new(self.rc.borrow_mut().next_page.as_u64() + (offset + size) as u64 - 1 as u64));
            Page::range_inclusive(start_page, end_page)
        };

        for (i, page) in page_range.enumerate()
        {
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            VMM.lock().map_to(page, physical_address_frame + i as u64, flags).expect("[ACPI] Failed to map memory").flush();
        }

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
        let page_range = {
            let start_page = Page::containing_address(VirtAddr::new(region.virtual_start().as_ptr() as u64));
            let end_page = Page::containing_address(VirtAddr::new(region.virtual_start().as_ptr() as u64 + region.region_length() as u64 - 1));
            Page::range_inclusive(start_page, end_page)
        };

        for page in page_range {
            unsafe { VMM.lock().unmap(page).expect("Failed to unmap page").1.flush() };
        }
    }
}