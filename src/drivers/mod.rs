use alloc::boxed::Box;
use crate::pci;
use crate::acpi;

mod sata_controller_ahci;

pub use sata_controller_ahci::SataControllerAhci;

#[derive(Debug)]
pub enum Driver
{
    PciDriver(Box<dyn pci::PciDriver>),
    AcpiDrier(Box<dyn acpi::AcpiDriver>)
}