#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use crate::interrupts::init_idt;
use crate::vmm::Vmm;

mod serial;
mod interrupts;
mod vmm;
mod pmm;
mod allocator;

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

    serial_print!("Initializing PMM and VMM...");
    let mut vmm = unsafe { Vmm::init(boot_info.recursive_index.into_option().expect("No recursive index"), pmm::BootInfoFrameAllocator::init(&boot_info.memory_regions)) };
    serial_println!(" [ok]");

    serial_print!("Initializing heap...");
    allocator::init(&mut vmm).expect("Heap initialization failed");
    serial_println!(" [ok]");


    loop {}
}