#![no_std]
#![no_main]

mod serial;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> !
{
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

    loop {}
}