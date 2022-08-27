use alloc::string::String;
use core::fmt::Debug;
use crate::pci::PciDevice;

pub trait PciDriver: Debug
{
    fn init(pci_header: PciDevice) -> Result<Self, String> where Self: Sized;
}
