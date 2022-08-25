use aml::Handler;
use pci_types::{PciAddress};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::PageTableFlags;
use crate::pci::PciHandler;
use crate::VMM;

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
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            VirtAddr::new(AML_ADDRESS_TMP + offset),
            1,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (AML_ADDRESS_TMP + offset) as *mut u8;
        let v = unsafe { ptr.read() };

        v
    }

    fn read_u16(&self, physical_address: usize) -> u16
    {
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            VirtAddr::new(AML_ADDRESS_TMP + offset),
            2,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (AML_ADDRESS_TMP + offset) as *mut u16;
        let v = unsafe { ptr.read() };

        v
    }

    fn read_u32(&self, physical_address: usize) -> u32
    {
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            VirtAddr::new(AML_ADDRESS_TMP + offset),
            4,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (AML_ADDRESS_TMP + offset) as *mut u32;
        let v = unsafe { ptr.read() };

        v
    }

    fn read_u64(&self, physical_address: usize) -> u64
    {
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            VirtAddr::new(AML_ADDRESS_TMP + offset),
            8,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (AML_ADDRESS_TMP + offset) as *mut u64;
        let v = unsafe { ptr.read() };

        v
    }

    fn write_u8(&mut self, physical_address: usize, value: u8)
    {
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            VirtAddr::new(AML_ADDRESS_TMP + offset),
            1,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (AML_ADDRESS_TMP + offset) as *mut u8;
        unsafe { ptr.write(value) };
    }

    fn write_u16(&mut self, physical_address: usize, value: u16){
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            VirtAddr::new(AML_ADDRESS_TMP + offset),
            2,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (AML_ADDRESS_TMP + offset) as *mut u16;
        unsafe { ptr.write(value) };
    }

    fn write_u32(&mut self, physical_address: usize, value: u32) {
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            VirtAddr::new(AML_ADDRESS_TMP + offset),
            4,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (AML_ADDRESS_TMP + offset) as *mut u32;
        unsafe { ptr.write(value) };
    }

    fn write_u64(&mut self, physical_address: usize, value: u64) {
        let offset = (physical_address % 0x1000) as u64;

        VMM.lock().remap_region(
            PhysAddr::new(physical_address as u64),
            VirtAddr::new(AML_ADDRESS_TMP + offset),
            8,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        ).expect("[AML] Failed to remap AML region");

        let ptr = (AML_ADDRESS_TMP + offset) as *mut u64;
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