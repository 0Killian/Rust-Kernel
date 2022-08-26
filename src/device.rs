use alloc::boxed::Box;
use core::fmt::{Display, Formatter};
use crate::acpi::AcpiDevice;
use crate::drivers::Driver;
use crate::pci::PciDevice;

#[derive(Debug)]
pub enum Device
{
    Acpi(AcpiDevice),
    Pci(PciDevice)
}

impl Display for Device
{
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result
    {
        match self
        {
            Device::Acpi(device) => write!(f, "{}", device),
            Device::Pci(device) => write!(f, "{}", device)
        }
    }
}

impl Device
{
    pub fn find_driver(&self) -> Option<Driver>
    {
        match self
        {
            Device::Acpi(device) => device.find_driver(),
            Device::Pci(device) => device.find_driver()
        }
    }
}