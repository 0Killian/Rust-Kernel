use bit_field::BitField;
use x86_64::PhysAddr;
use crate::pci::bar::Bar;
use crate::pci::pci_address::PciAddress;
use crate::pci::PciHandler;
use crate::pci::status_register::StatusRegister;

pub struct PciHeader(PciAddress);

pub type VendorId = u16;
pub type DeviceId = u16;
pub type ClassCode = u8;
pub type SubClass = u8;
pub type ProgramInterface = u8;
pub type DeviceRevision = u8;

pub enum HeaderType
{
    Standard,
    Bridge,
    CardBus,
    Unknown
}

impl TryFrom<u8> for HeaderType
{
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value
        {
            0 => Ok(HeaderType::Standard),
            1 => Ok(HeaderType::Bridge),
            2 => Ok(HeaderType::CardBus),
            _ => Err(())
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BistError
{
    NotSupported,
    Error(u8)
}

impl PciHeader
{
    pub fn new(address: PciAddress) -> Self
    {
        PciHeader(address)
    }

    pub fn has_multiple_functions(&self, pci_handler: &PciHandler) -> bool
    {
        unsafe { pci_handler.read(self.0, 0x0C) }.get_bit(23)
    }

    pub fn id(&self, pci_handler: &PciHandler) -> (VendorId, DeviceId)
    {
        let id = unsafe { pci_handler.read(self.0, 0x00) };
        (
            id.get_bits(16..32) as VendorId,
            id.get_bits(0..16) as DeviceId,
        )
    }

    pub fn revision_and_class(&self, pci_handler: &PciHandler) -> (DeviceRevision, ClassCode, SubClass, ProgramInterface)
    {
        let field = unsafe { pci_handler.read(self.0, 0x08) };
        (
            field.get_bits(0..8) as DeviceRevision,
            field.get_bits(24..32) as ClassCode,
            field.get_bits(16..24) as SubClass,
            field.get_bits(8..16) as ProgramInterface,
        )
    }

    pub fn status(&self, pci_handler: &PciHandler) -> StatusRegister
    {
        let data = unsafe { pci_handler.read(self.0, 0x04) }.get_bits(16..32);
        StatusRegister::new(data as u16)
    }

    pub fn header_type(&self, pci_handler: &PciHandler) -> Result<HeaderType, ()>
    {
        HeaderType::try_from(unsafe { pci_handler.read(self.0, 0x0C) }.get_bits(16..23) as u8)
    }

    pub fn function_exists(&self, pci_handler: &PciHandler) -> bool
    {
        unsafe { pci_handler.read(self.0, 0x0C) }.get_bit(7)
    }

    pub fn do_bist(&self, pci_handler: &PciHandler) -> Result<(), BistError>
    {
        if unsafe { pci_handler.read(self.0, 0x0C) }.get_bit(24 + 7)
        {
            let mut reg = unsafe { pci_handler.read(self.0, 0x0C) };
            unsafe { pci_handler.write(self.0, 0x0C, *reg.set_bit(24 + 6, true)) };

            while unsafe { pci_handler.read(self.0, 0x0C) }.get_bit(24 + 6) {}

            let result = unsafe { pci_handler.read(self.0, 0x0C) }.get_bits(24..27);
            if result == 0
            {
                return Ok(())
            }
            else
            {
                return Err(BistError::Error(result as u8))
            }
        }

        Err(BistError::NotSupported)
    }
}

#[derive(Debug)]
pub struct StandardHeader(PciAddress);

impl StandardHeader
{
    pub fn new(address: PciAddress) -> Self
    {
        StandardHeader(address)
    }

    pub fn bar(&self, pci_handler: &PciHandler, slot: u8) -> Option<Bar>
    {
        if slot >= 6 {
            return None;
        }

        let offset = 0x10 + (slot as u16) * 4;
        let bar = unsafe { pci_handler.read(self.0, offset) };

        /*
         * If bit 0 is `0`, the BAR is in memory. If it's `1`, it's in I/O.
         */
        if bar.get_bit(0) == false {
            let prefetchable = bar.get_bit(3);
            let address = bar.get_bits(4..32) << 4;

            match bar.get_bits(1..3)
            {
                0b00 => {
                    let size = unsafe {
                        pci_handler.write(self.0, offset, 0xFFFFFFF0);
                        let mut readback = pci_handler.read(self.0, offset).get_bits(4..32);
                        pci_handler.write(self.0, offset, address);

                        if readback == 0x0 {
                            return None;
                        }

                        readback.set_bits(0..4, 0);
                        1 << readback.trailing_zeros()
                    };
                    Some(Bar::Memory {
                        base: PhysAddr::new(address as u64),
                        size,
                        prefetchable,
                    })
                },

                0b10 => {
                    /*
                     * If the BAR is 64 bit-wide and this slot is the last, there is no second slot to read.
                     */
                    if slot >= 5 {
                        return None
                    }

                    let address_upper = unsafe { pci_handler.read(self.0, offset + 4) };

                    let size = unsafe {
                        pci_handler.write(self.0, offset, 0xfffffff0);
                        pci_handler.write(self.0, offset + 4, 0xffffffff);
                        let mut readback_low = pci_handler.read(self.0, offset);
                        let mut readback_high = pci_handler.read(self.0, offset + 4);
                        pci_handler.write(self.0, offset, address);
                        pci_handler.write(self.0, offset + 4, address_upper);

                        /*
                         * If the readback from the first slot is not 0, the size of the BAR is less than 4GiB.
                         */
                        if readback_low != 0
                        {
                            /*
                             * The readback is invalid in these conditions:
                             */
                            (1 << readback_low.trailing_zeros()) as u64
                        }
                        else
                        {
                            (1 << (readback_high.trailing_zeros() + 32)) as u64
                        }
                    };

                    let address = {
                        let mut address = address as u64;
                        address.set_bits(32..64, address_upper as u64);
                        address
                    };

                    Some(Bar::Memory {
                        base: PhysAddr::new(address),
                        size: size as u64,
                        prefetchable,
                    })
                },
                _ => panic!("BAR Memory type is reserved!"),
            }
        } else {
            Some(Bar::Io {
                port: bar.get_bits(2..32),
            })
        }
    }
}