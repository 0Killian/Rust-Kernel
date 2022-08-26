mod pci_pci_bridge;
mod pci_device;
mod pci_driver;

use alloc::vec::Vec;
use acpi::{AcpiError, AcpiHandler, AcpiTables, PciConfigRegions};
use lazy_static::lazy_static;
use pci_types::{ConfigRegionAccess, PciAddress, PciHeader};
use x86_64::structures::paging::{Page, PageTableFlags};
use x86_64::{PhysAddr, VirtAddr};
pub use crate::pci::pci_device::PciDevice;
use crate::VMM;
use spin::Mutex;
use crate::acpi::ACPI;
use log::{error, info};
pub use crate::pci::pci_driver::PciDriver;

#[derive(Clone)]
pub struct PciHandler
{
    pci_config_regions: PciConfigRegions,
    page: VirtAddr
}

impl ConfigRegionAccess for PciHandler
{
    fn function_exists(&self, address: PciAddress) -> bool {
        self.pci_config_regions.physical_address(address.segment(), address.bus(), address.device(), address.function())
            .is_some()
    }

    unsafe fn read(&self, address: PciAddress, offset: u16) -> u32
    {
        let physical_address = self.pci_config_regions.physical_address(address.segment(), address.bus(), address.device(), address.function())
            .unwrap();

        let _offset = physical_address % 0x1000;

        VMM.lock().remap_region(PhysAddr::new(physical_address), self.page + _offset, (offset + 3) as u64, PageTableFlags::PRESENT | PageTableFlags::WRITABLE).expect("[PCI] Failed to remap PCI region");

        let ptr = (self.page.as_u64() + offset as u64 + _offset) as *mut u32;
        ptr.read_volatile()
    }

    unsafe fn write(&self, address: PciAddress, offset: u16, value: u32) {
        let physical_address = self.pci_config_regions.physical_address(address.segment(), address.bus(), address.device(), address.function())
            .unwrap();

        let _offset = physical_address % 0x1000;

        VMM.lock().remap_region(PhysAddr::new(physical_address), self.page + _offset, (offset + 3) as u64, PageTableFlags::PRESENT | PageTableFlags::WRITABLE).expect("[PCI] Failed to remap PCI region");

        let ptr = (self.page.as_u64() + _offset + offset as u64) as *mut u32;
        ptr.write_volatile(value);
    }
}

impl PciHandler
{
    pub fn new<T>(acpi_tables: &AcpiTables<T>) -> Result<PciHandler, AcpiError> where T: AcpiHandler
    {
        info!("Acpi Tables :");
        for sdt in acpi_tables.sdts.iter() {
            info!("{:#?}", sdt.0);
        }

        match PciConfigRegions::new(acpi_tables) {
            Ok(regions) => Ok(PciHandler {
                pci_config_regions: regions,
                page: VMM.lock().map_region(
                    PhysAddr::new(0),
                    2,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                ).expect("[PCI] Failed to find free pages")
            }),
            Err(err) => Err(err)
        }
    }

    pub fn enumerate_devices(&mut self) -> Vec<PciDevice>
    {
        let mut devices = Vec::new();
        if PciHeader::new(PciAddress::new(0, 0, 0, 0)).has_multiple_functions(self)
        {
            for bus in 0..8
            {
                self.check_bus(bus, &mut devices);
            }
        }
        else
        {
            self.check_bus(0, &mut devices);
        }

        devices
    }

    fn check_bus(&mut self, bus: u8, devices: &mut Vec<PciDevice>)
    {
        for device in 0..32
        {
            let address = PciAddress::new(0, bus, device, 0);
            if self.function_exists(address)
            {
                self.check_device(bus, device, devices);
            }
        }
    }

    fn check_device(&mut self, bus: u8, device: u8, devices: &mut Vec<PciDevice>)
    {
        let address = PciAddress::new(0, bus, device, 0);
        self.check_function(address, devices);

        let header = PciHeader::new(address);
        if header.has_multiple_functions(self)
        {
            for function in 1..8
            {
                self.check_function(PciAddress::new(0, bus, device, function), devices);
            }
        }
    }

    fn check_function(&mut self, address: PciAddress, devices: &mut Vec<PciDevice>)
    {
        if self.function_exists(address)
        {
            if let Some(device) = PciDevice::new(address, self)
            {
                devices.push(device);
            }
        }
    }

    pub unsafe fn read(&self, address: PciAddress, offset: u16) -> u32
    {
        ConfigRegionAccess::read(self, address, offset)
    }

    pub unsafe fn write(&self, address: PciAddress, offset: u16, value: u32)
    {
        ConfigRegionAccess::write(self,address, offset, value)
    }
}

lazy_static!
{
    pub static ref PCI_HANDLER: Mutex<Option<PciHandler>> = Mutex::new(match PciHandler::new(&ACPI.lock().acpi_tables)
        {
            Ok(handler) => Some(handler),
            Err(err) => {
                error!("[PCI] Failed to initialize PCI handler: {:?}", err);
                None
            }
        }
    );
}