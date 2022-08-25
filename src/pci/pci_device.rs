use alloc::format;
use alloc::string::{String, ToString};
use core::fmt::{Display, Formatter};
use pci_types::{BaseClass, DeviceId, Interface, PciAddress, PciHeader, SubClass, VendorId};
use pci_types::device_type::DeviceType;
use crate::pci::PciHandler;

#[derive(Debug, Clone)]
pub struct PciDevice
{
    vendor_id: VendorId,
    device_id: DeviceId,
    class_code: BaseClass,
    subclass_code: SubClass,
    prog_interface: Interface,
    address: PciAddress
}

impl PciDevice
{
    pub fn new(address: PciAddress, handler: &PciHandler) -> Option<Self>
    {
        let header = PciHeader::new(address);
        let (vendor_id, device_id) = header.id(handler);
        let (_, class, sub_class, interface) = header.revision_and_class(handler);

        let device_type = DeviceType::from((class, sub_class));

        match device_type
        {
            DeviceType::Unknown => None,
            _ => {
                Some(PciDevice {
                    vendor_id,
                    device_id,
                    class_code: class,
                    subclass_code: sub_class,
                    prog_interface: interface,
                    address
                })
            }
        }
    }

    fn get_vendor_name(&self) -> &str
    {
        match self.vendor_id
        {
            0x1022 => "AMD",
            0x8086 => "Intel Corporation",
            0x10DE => "NVIDIA Corporation",
            _ => "Unknown"
        }
    }

    fn get_device_name(&self) -> &str
    {
        match (self.vendor_id, self.device_id)
        {
            (0x8086, 0x29C0) => "82G33/G31/P35/P31 Express DRAM Controller",
            (0x8086, 0x2918) => "82801IB (ICH9) LPC Interface Controller",
            (0x8086, 0x2922) => "82801IR/IO/IH (ICH9R/DO/DH) 6 port SATA Controller [AHCI mode]",
            (0x8086, 0x2930) => "82801I (ICH9 Family) SMBus Controller",
            (0x1234, 0x1111) => "QEMU Virtual Video Controller [\"-vga std\"]",
            (_, _) => "Unknown"
        }
    }

    fn get_interface_name(&self) -> String
    {
        match (DeviceType::from((self.class_code, self.subclass_code)), self.prog_interface)
        {
            (DeviceType::SataController, 0x0) => format!("{:?} Vendor Specific Interface", DeviceType::SataController),
            (DeviceType::SataController, 0x1) => format!("{:?} AHCI 1.0 Controller", DeviceType::SataController),
            (DeviceType::SataController, 0x2) => format!("{:?} Serial Storage Bus", DeviceType::SataController),
            (DeviceType::VgaCompatibleController, 0x0) => format!("{:?} VGA Controller", DeviceType::VgaCompatibleController),
            (DeviceType::VgaCompatibleController, 0x1) => format!("{:?} 8514 Compatible VGA Controller", DeviceType::VgaCompatibleController),
            (DeviceType::HostBridge, _) => format!("{:?}", DeviceType::HostBridge),
            (DeviceType::IsaBridge, _) => format!("{:?}", DeviceType::IsaBridge),
            (DeviceType::SmBusController, _) => format!("{:?}", DeviceType::SmBusController),
            _ => "".to_string()
        }
    }
}

impl Display for PciDevice
{
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result
    {
        let vendor = self.get_vendor_name();
        let device = self.get_device_name();
        let interface = self.get_interface_name();

        write!(f, "PCI Device: | {: ^20} | {: ^70} | {: ^50} | at {}", vendor, device, interface, self.address)
    }
}