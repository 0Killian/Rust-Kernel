use alloc::collections::BTreeMap;
use acpi::{AcpiError, AcpiHandler, AcpiTable, AcpiTables, PciConfigRegions};
use log::trace;
use pci_types::{ConfigRegionAccess, PciAddress, PciHeader};
use pci_types::device_type::DeviceType;
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame};
use x86_64::{align_down, PhysAddr, VirtAddr};
use crate::VMM;

pub struct PciPciBridgeHeader(PciAddress);

// Documentation for PCI

impl PciPciBridgeHeader {

    pub fn header(&self) -> PciHeader {
        PciHeader::new(self.0)
    }

    /// Get the secondary bus number of the PCI-PCI bridge
    pub fn secondary_bus(&self, access: &impl ConfigRegionAccess) -> u8 {
        ((unsafe { access.read(self.0, 0x18) } >> 8) & 0xFF) as u8
    }
}

#[derive(Clone)]
pub struct PciDevice
{
    device_type: DeviceType
}

#[derive(Clone)]
pub struct PciHandler
{
    pci_config_regions: PciConfigRegions,
    devices: BTreeMap<PciAddress, PciDevice>
}

static PCI_ADDRESS_TMP: u64 = 0x_4444_6444_0000;

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

        VMM.lock().remap_region(PhysAddr::new(physical_address), VirtAddr::new(PCI_ADDRESS_TMP + _offset), (offset + 3) as u64, PageTableFlags::PRESENT | PageTableFlags::WRITABLE).expect("[PCI] Failed to remap PCI region");

        let ptr = (PCI_ADDRESS_TMP + offset as u64 + _offset) as *mut u32;
        ptr.read_volatile()
    }

    unsafe fn write(&self, address: PciAddress, offset: u16, value: u32) {
        let physical_address = self.pci_config_regions.physical_address(address.segment(), address.bus(), address.device(), address.function())
            .unwrap();

        let _offset = physical_address % 0x1000;

        VMM.lock().remap_region(PhysAddr::new(physical_address), VirtAddr::new(PCI_ADDRESS_TMP + _offset), (offset + 3) as u64, PageTableFlags::PRESENT | PageTableFlags::WRITABLE).expect("[PCI] Failed to remap PCI region");

        let ptr = (PCI_ADDRESS_TMP + _offset + offset as u64) as *mut u32;
        ptr.write_volatile(value);
    }
}

impl PciHandler
{
    pub fn new<T>(acpi_tables: &AcpiTables<T>) -> Result<PciHandler, AcpiError>
    where
        T: AcpiHandler
    {
        match PciConfigRegions::new(acpi_tables) {
            Ok(regions) => Ok(PciHandler {
                pci_config_regions: regions,
                devices: BTreeMap::new()
            }),
            Err(err) => Err(err)
        }
    }

    pub fn init(&mut self)
    {
        if PciHeader::new(PciAddress::new(0, 0, 0, 0)).has_multiple_functions(self)
        {
            for bus in 0..8
            {
                self.check_bus(bus);
            }
        }
        else
        {
            self.check_bus(0);
        }
    }

    fn check_bus(&mut self, bus: u8)
    {
        for device in 0..32
        {
            let address = PciAddress::new(0, bus, device, 0);
            if self.function_exists(address)
            {
                self.check_device(bus, device);
            }
        }
    }

    fn check_device(&mut self, bus: u8, device: u8)
    {
        let address = PciAddress::new(0, bus, device, 0);
        self.check_function(address);

        let header = PciHeader::new(address);
        if header.has_multiple_functions(self)
        {
            for function in 1..8
            {
                self.check_function(PciAddress::new(0, bus, device, function));
            }
        }
    }

    fn check_function(&mut self, address: PciAddress)
    {
        if self.function_exists(address)
        {
            let header = PciHeader::new(address);
            let (vendor_id, device_id) = header.id(self);
            let (revision, class, sub_class, interface) = header.revision_and_class(self);

            let device_type = DeviceType::from((class, sub_class));

            if device_type != DeviceType::Unknown
            {
                self.devices.insert(address, PciDevice { device_type });
                trace!("Found device: {:?} at {}", device_type, address);

                if device_type == DeviceType::PciPciBridge
                {
                    let bridge = PciPciBridgeHeader(address);
                    let secondary_bus = bridge.secondary_bus(self);
                    self.check_bus(secondary_bus);
                }
            }
        }
    }

    pub unsafe fn read(&self, address: PciAddress, offset: u16) -> u32
    {
        self.read(address, offset)
    }

    pub unsafe fn write(&self, address: PciAddress, offset: u16, value: u32)
    {
        self.write(address, offset, value)
    }
}