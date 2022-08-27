use alloc::format;
use core::fmt::{Display, Formatter};
use x86_64::PhysAddr;

#[derive(Debug)]
pub enum Bar
{
    Memory {
        base: PhysAddr,
        size: u64,
        prefetchable: bool
    },
    Io {
        port: u32
    }
}

impl Display for Bar
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result
    {
        write!(f, "{}", match self
        {
            Bar::Memory { base, size, prefetchable } => format!("{}:{}", base.as_u64(), size),
            Bar::Io { port } => format!("{}", port)
        })
    }
}