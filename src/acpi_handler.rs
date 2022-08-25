use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::ptr::NonNull;
use acpi::{AcpiHandler, AcpiTables, PciConfigRegions, PhysicalMapping};
use aml::{AmlContext, AmlName, DebugVerbosity, Handler, NamespaceLevel};
use aml::value::AmlType;
use log::{error, info, trace};
use pci_types::PciAddress;
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame};
use x86_64::{align_down, PhysAddr, VirtAddr};
use crate::{BOOT_INFO, VMM};
use crate::pci::PciHandler;

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

pub struct AmlHandler
{
    pci_handler: Option<PciHandler>
}

impl AmlHandler
{
    pub fn new_with_pci(pci_handler: PciHandler) -> AmlHandler
    {
        AmlHandler {
            pci_handler: Some(pci_handler)
        }
    }

    pub fn new() -> AmlHandler
    {
        AmlHandler {
            pci_handler: None
        }
    }
}

static AML_ADDRESS_TMP: u64 = 0x_4444_6444_2000;

impl Handler for AmlHandler
{
    fn read_u8(&self, physical_address: usize) -> u8
    {
        let physical_address_aligned = align_down(physical_address as u64, 0x1000);

        let page_range = {
            let page_start = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP));
            let page_end = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64));
            Page::range_inclusive(page_start, page_end)
        };

        for (i, page) in page_range.enumerate()
        {
            match VMM.lock().translate_addr(page.start_address())
            {
                Some(frame) => if frame.as_u64() != (physical_address_aligned as usize + i * 0x1000) as u64
                {
                    unsafe
                    {
                        VMM.lock().unmap(page).expect("[PCI] Failed to unmap memory");
                        VMM.lock().map_to(
                            page,
                            PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                        ).expect("[PCI] Failed to map memory").flush();
                    }
                },
                None => unsafe
                {
                    VMM.lock().map_to(
                        page,
                        PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                    ).expect("[PCI] Failed to map memory").flush();
                }
            }
        }

        let ptr = (AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64) as *mut u8;
        let v = unsafe { ptr.read() };

        v
    }

    fn read_u16(&self, physical_address: usize) -> u16 {
        let physical_address_aligned = align_down(physical_address as u64, 0x1000);

        let page_range = {
            let page_start = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP));
            let page_end = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64));
            Page::range_inclusive(page_start, page_end)
        };

        for (i, page) in page_range.enumerate()
        {
            match VMM.lock().translate_addr(page.start_address())
            {
                Some(frame) => if frame.as_u64() != (physical_address_aligned as usize + i * 0x1000) as u64
                {
                    unsafe
                        {
                            VMM.lock().unmap(page).expect("[PCI] Failed to unmap memory");
                            VMM.lock().map_to(
                                page,
                                PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                                PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                            ).expect("[PCI] Failed to map memory").flush();
                        }
                },
                None => unsafe
                    {
                        VMM.lock().map_to(
                            page,
                            PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                        ).expect("[PCI] Failed to map memory").flush();
                    }
            }
        }

        let ptr = (AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64) as *mut u16;
        let v = unsafe { ptr.read() };

        v
    }

    fn read_u32(&self, physical_address: usize) -> u32 {
        let physical_address_aligned = align_down(physical_address as u64, 0x1000);

        let page_range = {
            let page_start = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP));
            let page_end = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64));
            Page::range_inclusive(page_start, page_end)
        };

        for (i, page) in page_range.enumerate()
        {
            match VMM.lock().translate_addr(page.start_address())
            {
                Some(frame) => if frame.as_u64() != (physical_address_aligned as usize + i * 0x1000) as u64
                {
                    unsafe
                        {
                            VMM.lock().unmap(page).expect("[PCI] Failed to unmap memory");
                            VMM.lock().map_to(
                                page,
                                PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                                PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                            ).expect("[PCI] Failed to map memory").flush();
                        }
                },
                None => unsafe
                    {
                        VMM.lock().map_to(
                            page,
                            PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                        ).expect("[PCI] Failed to map memory").flush();
                    }
            }
        }

        let ptr = (AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64) as *mut u32;
        let v = unsafe { ptr.read() };

        v
    }

    fn read_u64(&self, physical_address: usize) -> u64 {
        let physical_address_aligned = align_down(physical_address as u64, 0x1000);

        let page_range = {
            let page_start = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP));
            let page_end = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64));
            Page::range_inclusive(page_start, page_end)
        };

        for (i, page) in page_range.enumerate()
        {
            match VMM.lock().translate_addr(page.start_address())
            {
                Some(frame) => if frame.as_u64() != (physical_address_aligned as usize + i * 0x1000) as u64
                {
                    unsafe
                        {
                            VMM.lock().unmap(page).expect("[PCI] Failed to unmap memory");
                            VMM.lock().map_to(
                                page,
                                PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                                PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                            ).expect("[PCI] Failed to map memory").flush();
                        }
                },
                None => unsafe
                    {
                        VMM.lock().map_to(
                            page,
                            PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                        ).expect("[PCI] Failed to map memory").flush();
                    }
            }
        }

        let ptr = (AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64) as *mut u64;
        let v = unsafe { ptr.read() };

        v
    }

    fn write_u8(&mut self, physical_address: usize, value: u8) {
        let physical_address_aligned = align_down(physical_address as u64, 0x1000);

        let page_range = {
            let page_start = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP));
            let page_end = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64));
            Page::range_inclusive(page_start, page_end)
        };

        for (i, page) in page_range.enumerate()
        {
            match VMM.lock().translate_addr(page.start_address())
            {
                Some(frame) => if frame.as_u64() != (physical_address_aligned as usize + i * 0x1000) as u64
                {
                    unsafe
                        {
                            VMM.lock().unmap(page).expect("[PCI] Failed to unmap memory");
                            VMM.lock().map_to(
                                page,
                                PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                                PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                            ).expect("[PCI] Failed to map memory").flush();
                        }
                },
                None => unsafe
                    {
                        VMM.lock().map_to(
                            page,
                            PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                        ).expect("[PCI] Failed to map memory").flush();
                    }
            }
        }

        let ptr = (AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64) as *mut u8;
        unsafe { ptr.write(value) };
    }

    fn write_u16(&mut self, physical_address: usize, value: u16) {
        let physical_address_aligned = align_down(physical_address as u64, 0x1000);

        let page_range = {
            let page_start = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP));
            let page_end = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64));
            Page::range_inclusive(page_start, page_end)
        };

        for (i, page) in page_range.enumerate()
        {
            match VMM.lock().translate_addr(page.start_address())
            {
                Some(frame) => if frame.as_u64() != (physical_address_aligned as usize + i * 0x1000) as u64
                {
                    unsafe
                        {
                            VMM.lock().unmap(page).expect("[PCI] Failed to unmap memory");
                            VMM.lock().map_to(
                                page,
                                PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                                PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                            ).expect("[PCI] Failed to map memory").flush();
                        }
                },
                None => unsafe
                    {
                        VMM.lock().map_to(
                            page,
                            PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                        ).expect("[PCI] Failed to map memory").flush();
                    }
            }
        }

        let ptr = (AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64) as *mut u16;
        unsafe { ptr.write(value) };
    }

    fn write_u32(&mut self, physical_address: usize, value: u32) {
        let physical_address_aligned = align_down(physical_address as u64, 0x1000);

        let page_range = {
            let page_start = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP));
            let page_end = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64));
            Page::range_inclusive(page_start, page_end)
        };

        for (i, page) in page_range.enumerate()
        {
            match VMM.lock().translate_addr(page.start_address())
            {
                Some(frame) => if frame.as_u64() != (physical_address_aligned as usize + i * 0x1000) as u64
                {
                    unsafe
                        {
                            VMM.lock().unmap(page).expect("[PCI] Failed to unmap memory");
                            VMM.lock().map_to(
                                page,
                                PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                                PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                            ).expect("[PCI] Failed to map memory").flush();
                        }
                },
                None => unsafe
                    {
                        VMM.lock().map_to(
                            page,
                            PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                        ).expect("[PCI] Failed to map memory").flush();
                    }
            }
        }

        let ptr = (AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64) as *mut u32;
        unsafe { ptr.write(value) };
    }

    fn write_u64(&mut self, physical_address: usize, value: u64) {
        let physical_address_aligned = align_down(physical_address as u64, 0x1000);

        let page_range = {
            let page_start = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP));
            let page_end = Page::containing_address(VirtAddr::new(AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64));
            Page::range_inclusive(page_start, page_end)
        };

        for (i, page) in page_range.enumerate()
        {
            match VMM.lock().translate_addr(page.start_address())
            {
                Some(frame) => if frame.as_u64() != (physical_address_aligned as usize + i * 0x1000) as u64
                {
                    unsafe
                        {
                            VMM.lock().unmap(page).expect("[PCI] Failed to unmap memory").1.flush();
                            VMM.lock().map_to(
                                page,
                                PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                                PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                            ).expect("[PCI] Failed to map memory").flush();
                        }
                },
                None => unsafe
                    {
                        VMM.lock().map_to(
                            page,
                            PhysFrame::containing_address(PhysAddr::new((physical_address_aligned as usize + i * 0x1000) as u64)),
                            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                        ).expect("[PCI] Failed to map memory").flush();
                    }
            }
        }

        let ptr = (AML_ADDRESS_TMP + (physical_address - physical_address_aligned as usize) as u64) as *mut u64;
        unsafe { ptr.write(value) };
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        unsafe { x86_64::instructions::port::Port::new(port).read() }
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        unsafe { x86_64::instructions::port::Port::new(port).read() }
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        unsafe { x86_64::instructions::port::Port::new(port).read() }
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        unsafe { x86_64::instructions::port::Port::new(port).write(value) }
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        unsafe { x86_64::instructions::port::Port::new(port).write(value) }
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        unsafe { x86_64::instructions::port::Port::new(port).write(value) }
    }

    fn read_pci_u8(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u8 {
        unsafe { self.pci_handler.as_ref().unwrap().read(PciAddress::new(segment, bus, device, function), offset) as u8 }
    }

    fn read_pci_u16(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u16 {
        unsafe { self.pci_handler.as_ref().unwrap().read(PciAddress::new(segment, bus, device, function), offset) as u16 }
    }

    fn read_pci_u32(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u32 {
        unsafe { self.pci_handler.as_ref().unwrap().read(PciAddress::new(segment, bus, device, function), offset) as u32}
    }

    fn write_pci_u8(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16, value: u8) {
        unsafe { self.pci_handler.as_ref().unwrap().write(PciAddress::new(segment, bus, device, function), offset, value as u32) }
    }

    fn write_pci_u16(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16, value: u16) {
        unsafe { self.pci_handler.as_ref().unwrap().write(PciAddress::new(segment, bus, device, function), offset, value as u32) }
    }

    fn write_pci_u32(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16, value: u32) {
        unsafe { self.pci_handler.as_ref().unwrap().write(PciAddress::new(segment, bus, device, function), offset, value as u32) }
    }
}

pub struct ACPI
{
    acpi_tables: AcpiTables<RcKernelAcpiHandler>,
    pci_config_regions: Option<PciHandler>,
    aml_context: AmlContext,
    acpi_handler: RcKernelAcpiHandler,
}

impl ACPI
{
    pub unsafe fn new() -> Self
    {
        let handler = RcKernelAcpiHandler::new();
        let rsdp_phys_addr = BOOT_INFO.rsdp_addr.into_option().unwrap();

        let tables = AcpiTables::from_rsdp(handler.clone(), rsdp_phys_addr as usize).expect("Failed to initialize ACPI");

        let mut acpi = match PciHandler::new(&tables)
        {
            Ok(pci_config_regions) => {
                ACPI {
                    acpi_tables: tables,
                    aml_context: AmlContext::new(Box::new(AmlHandler::new_with_pci(pci_config_regions.clone())),DebugVerbosity::None),
                    pci_config_regions: Some(pci_config_regions),
                    acpi_handler: handler,
                }
            },
            Err(err) => {
                error!("{:?}", err);
                ACPI {
                    acpi_tables: tables,
                    aml_context: AmlContext::new(Box::new(AmlHandler::new()),DebugVerbosity::None),
                    pci_config_regions: None,
                    acpi_handler: handler,
                }
            }
        };

        if let Some(dsdt) = &acpi.acpi_tables.dsdt
        {
            info!("[AML] Parsing DSDT");
            let phys_addr = dsdt.address;
            let mapping: PhysicalMapping<RcKernelAcpiHandler, u8> = acpi.acpi_handler.map_physical_region(phys_addr, dsdt.length as usize);
            acpi.aml_context.parse_table(core::slice::from_raw_parts(mapping.virtual_start().as_ptr(), dsdt.length as usize)).expect("[AML] Failed to parse DSDT");
        }

        for ssdt in &acpi.acpi_tables.ssdts
        {
            info!("[AML] Parsing SSDT");
            let phys_addr = ssdt.address;
            let mapping: PhysicalMapping<RcKernelAcpiHandler, u8> = acpi.acpi_handler.map_physical_region(phys_addr, ssdt.length as usize);
            acpi.aml_context.parse_table(core::slice::from_raw_parts(mapping.virtual_start().as_ptr(), ssdt.length as usize)).expect("[AML] Failed to parse SSDT");
        }

        acpi
    }

    pub fn init_pci(&mut self)
    {
        if let Some(pci) = self.pci_config_regions.as_mut()
        {
            pci.init();
        }
    }

    pub fn load_drivers(&mut self)
    {
        let mut devices = Vec::new();

        if let Err(e) = &self.aml_context.namespace.traverse(&mut |name: &AmlName, level: &NamespaceLevel| {
            info!("[AML] Found ACPI Namespace: {} ({} objects)", name.as_string(), level.values.len());

            for value in level.values.iter()
            {
                if value.0.as_str() == "_HID"
                {
                    devices.push(*value.1);
                }
            }

            Ok(true)
        })
        {
            error!("[AML] Failed to traverse namespace: {:?}", e);
            return
        }

        for device in devices.iter()
        {
            let hid = self.aml_context.namespace.get(*device).unwrap();
            match hid.type_of()
            {
                AmlType::Integer => {
                    // Compressed EISA type ID
                    let eisa_id = hid.as_integer(&self.aml_context).unwrap() as u32;
                    let acpi_id = Self::decompress_eisa_id(eisa_id);
                    info!("[AML] Found ACPI Device {{EISA_ID={:X}, ACPI_ID={}}}", eisa_id, acpi_id);
                },
                AmlType::String => {
                    // Full ACPI type ID
                    let acpi_id = hid.as_string(&self.aml_context).unwrap();
                    info!("[AML] Found ACPI Device {{ACPI_ID={}}}", acpi_id);
                },
                type_ => {
                    error!("[AML] Unknown ACPI Device type: {:?}", type_);
                }
            }
        }
    }

    fn decompress_eisa_id(eisa_id: u32) -> String
    {
        let eisa_id_1 = eisa_id.to_be();
        format!("{}{}{}{:03X}{:X}",
            // Manufacturer ID
            ((((eisa_id_1 >> 26) & 0b11111) as u8) + 0x40) as char,
            ((((eisa_id_1 >> 21) & 0b11111) as u8) + 0x40) as char,
            ((((eisa_id_1 >> 16) & 0b11111) as u8) + 0x40) as char,
            // Product ID
            (((eisa_id_1 >> 8) & 0xFF) as u8),
            // Revision ID
            ((eisa_id_1 & 0xFF) as u8)
        )
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
        VMM.lock().unmap_region(VirtAddr::from_ptr(region.virtual_start().as_ptr()), region.region_length() as u64).expect("[ACPI] Failed to unmap memory");
    }
}