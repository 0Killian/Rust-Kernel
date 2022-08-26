use core::fmt::Debug;
use pci_types::PciHeader;
use crate::pci::PciDevice;

pub trait PciDriver: Debug
{
    fn init(pci_header: PciDevice) -> Result<Self, &'static str> where Self: Sized;
}
