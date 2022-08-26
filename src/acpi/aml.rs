use aml::Handler;
use lazy_static::lazy_static;
use pci_types::{PciAddress};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{Page, PageTableFlags};
use crate::pci::PciHandler;
use crate::VMM;
use log::info;
use ::aml::{AmlContext, DebugVerbosity};
use spin::Mutex;
use alloc::boxed::Box;
use crate::pci::PCI_HANDLER;
use crate::acpi::ACPI;

pub struct AmlHandler
{
    pci_handler: Option<PciHandler>,
    virt_addr: VirtAddr,
}

impl AmlHandler
{
    pub fn new_with_pci(pci_handler: PciHandler) -> AmlHandler
    {
        info!("[AML] Initializing AML Handler");
        AmlHandler {
            pci_handler: Some(pci_handler),
            virt_addr: VMM.lock().map_region(
                PhysAddr::new(0),
                2,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE
            ).expect("[AML] Failed to find free pages")
        }
    }

    pub fn new() -> AmlHandler
    {
        AmlHandler {
            pci_handler: None,
            virt_addr: VMM.lock().map_region(
                PhysAddr::new(0),
                2,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE
            ).expect("[AML] Failed to find free pages")
        }
    }
}

impl Handler for AmlHandler
{
    fn read_u8(&self, physical_address: usize) -> u8
    {
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            self.virt_addr + offset,
            1,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (self.virt_addr.as_u64() + offset) as *mut u8;
        let v = unsafe { ptr.read() };

        v
    }

    fn read_u16(&self, physical_address: usize) -> u16
    {
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            self.virt_addr + offset,
            2,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (self.virt_addr.as_u64() + offset) as *mut u16;
        let v = unsafe { ptr.read() };

        v
    }

    fn read_u32(&self, physical_address: usize) -> u32
    {
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            self.virt_addr + offset,
            4,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (self.virt_addr.as_u64() + offset) as *mut u32;
        let v = unsafe { ptr.read() };

        v
    }

    fn read_u64(&self, physical_address: usize) -> u64
    {
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            self.virt_addr + offset,
            8,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (self.virt_addr.as_u64() + offset) as *mut u64;
        let v = unsafe { ptr.read() };

        v
    }

    fn write_u8(&mut self, physical_address: usize, value: u8)
    {
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            self.virt_addr + offset,
            1,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (self.virt_addr.as_u64() + offset) as *mut u8;
        unsafe { ptr.write(value) };
    }

    fn write_u16(&mut self, physical_address: usize, value: u16){
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            self.virt_addr + offset,
            2,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (self.virt_addr.as_u64() + offset) as *mut u16;
        unsafe { ptr.write(value) };
    }

    fn write_u32(&mut self, physical_address: usize, value: u32) {
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            self.virt_addr + offset,
            4,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (self.virt_addr.as_u64() + offset) as *mut u32;
        unsafe { ptr.write(value) };
    }

    fn write_u64(&mut self, physical_address: usize, value: u64) {
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            self.virt_addr + offset,
            8,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (self.virt_addr.as_u64() + offset) as *mut u64;
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

lazy_static!
{
    pub static ref AML_CONTEXT: Mutex<AmlContext> = {
        info!("[AML] Initializing AML context");
        let mut aml_context = match PCI_HANDLER.lock().as_ref()
        {
            Some(pci_handler) => AmlContext::new(Box::new(AmlHandler::new_with_pci(pci_handler.clone())), DebugVerbosity::None),
            None => AmlContext::new(Box::new(AmlHandler::new()), DebugVerbosity::None)
        };

        if let Some(dsdt) = &ACPI.lock().acpi_tables.dsdt
        {
            info!("[AML] Parsing DSDT");
            let phys_addr = dsdt.address;
            let virt_addr = VMM.lock().map_region(PhysAddr::new(phys_addr as u64), dsdt.length as u64, PageTableFlags::PRESENT | PageTableFlags::WRITABLE).expect("[AML] Failed to map DSDT");
            unsafe {
                aml_context.parse_table(core::slice::from_raw_parts(virt_addr.as_ptr(), dsdt.length as usize)).expect("[AML] Failed to parse DSDT");
            }
            VMM.lock().unmap_region(virt_addr, dsdt.length as u64).expect("[AML] Failed to unmap DSDT");
        }

        for ssdt in &ACPI.lock().acpi_tables.ssdts
        {
            info!("[AML] Parsing SSDT");
            let phys_addr = ssdt.address;
            let virt_addr = VMM.lock().map_region(PhysAddr::new(phys_addr as u64), ssdt.length as u64, PageTableFlags::PRESENT | PageTableFlags::WRITABLE).expect("[AML] Failed to map DSDT");
            unsafe {
                aml_context.parse_table(core::slice::from_raw_parts(virt_addr.as_ptr(), ssdt.length as usize)).expect("[AML] Failed to parse SSDT");
            }
            VMM.lock().unmap_region(virt_addr, ssdt.length as u64).expect("[AML] Failed to unmap SSDT");
        }

        Mutex::new(aml_context)
    };
}