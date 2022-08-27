use alloc::string::String;
use core::fmt::Debug;
use aml::AmlValue;

pub trait AcpiDriver: Debug
{
    fn init(hid: AmlValue) -> Result<Self, String> where Self: Sized;
}