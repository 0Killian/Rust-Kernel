#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::ops::Deref;
use core::ptr::NonNull;
use crate::interrupts::init_idt;
use crate::vmm::VMM;
use crate::logger::SERIAL_LOGGER;

mod serial;
mod interrupts;
mod vmm;
mod pmm;
mod allocator;
mod logger;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> !
{
    serial_println!("{}", _info);
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

    serial_println!("Hello from kernel!");
    serial_print!("Initializing IDT...");
    init_idt();
    serial_println!(" [ok]");

    serial_print!("Initializing heap...");
    allocator::init().expect("Heap initialization failed");
    serial_println!(" [ok]");

    log::set_logger(&SERIAL_LOGGER).map(|()| log::set_max_level(log::LevelFilter::Trace)).expect("Failed to set logger");

    loop {}
}