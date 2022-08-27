use bit_field::BitField;

pub enum DevselTiming
{
    Fast = 0,
    Medium = 1,
    Slow = 2,
}

impl TryFrom<u8> for DevselTiming
{
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error>
    {
        match value
        {
            0 => Ok(DevselTiming::Fast),
            1 => Ok(DevselTiming::Medium),
            2 => Ok(DevselTiming::Slow),
            _ => Err(()),
        }
    }
}

pub struct StatusRegister(u16);

impl StatusRegister
{
    pub fn new(data: u16) -> Self
    {
        StatusRegister(data)
    }

    pub fn parity_error_detected(&self) -> bool
    {
        self.0.get_bit(15)
    }

    pub fn signaled_system_error(&self) -> bool
    {
        self.0.get_bit(14)
    }

    pub fn received_master_abort(&self) -> bool
    {
        self.0.get_bit(13)
    }

    pub fn received_target_abort(&self) -> bool
    {
        self.0.get_bit(12)
    }

    pub fn signaled_target_abort(&self) -> bool
    {
        self.0.get_bit(11)
    }

    pub fn devsel_timing(&self) -> Result<DevselTiming, ()>
    {
        let bits = self.0.get_bits(9..11);
        DevselTiming::try_from(bits as u8)
    }

    pub fn master_data_parity_error(&self) -> bool
    {
        self.0.get_bit(8)
    }

    pub fn fast_back_to_back_capable(&self) -> bool
    {
        self.0.get_bit(7)
    }

    pub fn capable_66mhz(&self) -> bool
    {
        self.0.get_bit(5)
    }

    pub fn has_capability_list(&self) -> bool
    {
        self.0.get_bit(4)
    }

    pub fn interrupt_status(&self) -> bool
    {
        self.0.get_bit(3)
    }
}