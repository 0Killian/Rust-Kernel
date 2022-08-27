use core::fmt::Display;
use bit_field::BitField;

#[derive(Copy, Clone, Debug)]
pub struct PciAddress(u32);

impl PciAddress
{
    pub fn new(segment: u16, bus: u8, device: u8, function: u8) -> Self
    {
        let mut result: u32 = 0;
        result.set_bits(0..3, function as u32);
        result.set_bits(3..8, device as u32);
        result.set_bits(8..16, bus as u32);
        result.set_bits(16..32, segment as u32);
        PciAddress(result)
    }

    pub fn segment(&self) -> u16
    {
        self.0.get_bits(16..32) as u16
    }

    pub fn bus(&self) -> u8
    {
        self.0.get_bits(8..16) as u8
    }

    pub fn device(&self) -> u8
    {
        self.0.get_bits(3..8) as u8
    }

    pub fn function(&self) -> u8
    {
        self.0.get_bits(0..3) as u8
    }
}

impl Display for PciAddress
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result
    {
        write!(f, "{:04x}:{:02x}:{:02x}.{:01x}", self.segment(), self.bus(), self.device(), self.function())
    }
}