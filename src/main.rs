#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use crate::interrupts::init_idt;

mod serial;
mod interrupts;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> !
{
    serial_println!("{}", _info);
    loop {}
}

bootloader::entry_point!(kernel_main);

fn kernel_main(boot_info : &'static mut bootloader::BootInfo) -> !
{
    if let Some(framebuffer) = boot_info.framebuffer.as_mut()
    {
        framebuffer.buffer_mut().fill(0x90);
    }

    serial_println!("Hello from kernel!");
    serial_print!("Initializing IDT...");
    init_idt();
    serial_println!(" [ok]");

    x86_64::instructions::interrupts::int3();

    loop {}
}