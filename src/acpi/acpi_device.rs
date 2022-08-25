use alloc::format;
use alloc::string::String;
use core::fmt::{Debug, Display, Error, Formatter};
use aml::AmlValue;

pub struct AcpiDevice
{
    hid: AmlValue
}

impl AcpiDevice
{
    pub fn new(hid: AmlValue) -> Self
    {
        AcpiDevice {
            hid
        }
    }

    fn decompress_eisa_id(eisa_id: u32) -> String
    {
        let eisa_id_1 = eisa_id.to_be();
        format!("{}{}{}{:03X}{:X}",
                // Manufacturer ID
                ((((eisa_id_1 >> 26) & 0b11111) as u8) + 0x40) as char,
                ((((eisa_id_1 >> 21) & 0b11111) as u8) + 0x40) as char,
                ((((eisa_id_1 >> 16) & 0b11111) as u8) + 0x40) as char,
                // Product ID
                (((eisa_id_1 >> 8) & 0xFF) as u8),
                // Revision ID
                ((eisa_id_1 & 0xFF) as u8)
        )
    }
}

impl Debug for AcpiDevice
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>
    {
        match &self.hid
        {
            AmlValue::String(s) => write!(f, "ACPI Device: hid={}", s),
            AmlValue::Integer(i) => write!(f, "ACPI Device: hid={}", AcpiDevice::decompress_eisa_id(*i as u32)),
            v => write!(f, "Unknown value : {:?}", v)
        }
    }
}

impl Display for AcpiDevice
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>
    {
        write!(f, "{:?}", self)
    }
}