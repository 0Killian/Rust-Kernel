#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(new_uninit)]

extern crate alloc;

use alloc::vec::Vec;
use core::ops::Deref;
use core::ptr::NonNull;
use log::{error, info};
use crate::acpi::{ACPI, AML_CONTEXT};
use crate::interrupts::init_idt;
use crate::vmm::VMM;
use crate::logger::SERIAL_LOGGER;
use crate::pci::PCI_HANDLER;

mod serial;
mod interrupts;
mod vmm;
mod pmm;
mod allocator;
mod logger;
mod pci;
mod acpi;
mod device;
mod drivers;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> !
{
    error!("{}", _info);
    loop {}
}

bootloader::entry_point!(kernel_main);

#[derive(Clone)]
struct BootInfoRef
{
    pub boot_info_ptr: *mut bootloader::BootInfo,
}

impl Copy for BootInfoRef {}

impl Deref for BootInfoRef
{
    type Target = bootloader::BootInfo;

    fn deref(&self) -> &Self::Target
    {
        unsafe { NonNull::new(self.boot_info_ptr).unwrap().as_ref() }
    }
}

static mut BOOT_INFO: BootInfoRef = BootInfoRef {
    boot_info_ptr: core::ptr::null::<bootloader::BootInfo>() as *mut bootloader::BootInfo,
};

fn kernel_main(boot_info : &'static mut bootloader::BootInfo) -> !
{
    unsafe { BOOT_INFO.boot_info_ptr = boot_info as *mut bootloader::BootInfo };
    if let Some(framebuffer) = boot_info.framebuffer.as_mut()
    {
        framebuffer.buffer_mut().fill(0x90);
    }

    log::set_logger(&SERIAL_LOGGER).map(|()| log::set_max_level(log::LevelFilter::Trace)).expect("Failed to set logger");

    init_idt();

    x86_64::instructions::interrupts::int3();

    allocator::init().expect("Heap initialization failed");
    // Initialize AML context and PCI handler before locking ACPI
    PCI_HANDLER.lock();
    AML_CONTEXT.lock();

    let devices = ACPI.lock().enumerate_devices();

    let drivers = devices.iter().map(|device|
        {
            match device.find_driver()
            {
                Some(driver) => {
                    info!("Found driver for {}", device);
                    Some(driver)
                }
                None => {
                    info!("No driver found for {}", device);
                    None
                }
            }
        });

    for driver in drivers
    {
        if let Some(driver) = driver
        {
            info!("Driver found : {:?}", driver);
        }
    }

    info!("Kernel initialized");

    loop {}
}