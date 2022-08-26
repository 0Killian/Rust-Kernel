use core::fmt::Debug;
use aml::AmlValue;

pub trait AcpiDriver: Debug
{
    fn init(hid: AmlValue) -> Result<Self, &'static str> where Self: Sized;
}