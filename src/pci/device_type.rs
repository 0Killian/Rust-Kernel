use crate::pci::pci_header::{ClassCode, SubClass};

#[derive(Debug)]
pub enum DeviceType
{
    SataController,

    VgaCompatibleController,

    HostBridge,
    IsaBridge,
    PciPciBridge,

    SmBusController,

    Unknown
}

impl From<(ClassCode, SubClass)> for DeviceType
{
    fn from((class, sub_class): (ClassCode, SubClass)) -> Self
    {
        match (class, sub_class)
        {
            (0x1, 0x6) => DeviceType::SataController,

            (0x3, 0x0) => DeviceType::VgaCompatibleController,

            (0x6, 0x0) => DeviceType::HostBridge,
            (0x6, 0x1) => DeviceType::IsaBridge,
            (0x6, 0x4) => DeviceType::PciPciBridge,

            (0xC, 0x5) => DeviceType::SmBusController,

            _ => DeviceType::Unknown
        }
    }
}