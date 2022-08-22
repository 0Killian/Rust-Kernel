@echo off

cd C:/Users/Killian/.cargo/registry/src/github.com-1ecc6299db9ec823/bootloader-0.10.12
cargo builder --kernel-manifest C:/Users/Killian/CLionProjects/Rust-Kernel/Cargo.toml --kernel-binary C:/Users/Killian/CLionProjects/Rust-Kernel/target/x86_64-os/debug/rust-kernel --target-dir C:/Users/Killian/CLionProjects/Rust-Kernel/target --out-dir C:/Users/Killian/CLionProjects/Rust-Kernel/target/x86_64-os/debug
cd C:/Users/Killian/CLionProjects/Rust-Kernel/
"C:/Program Files/qemu/qemu-system-x86_64" -drive file=target/x86_64-os/debug/boot-uefi-rust-kernel.img -m 256M -cpu qemu64 -drive if=pflash,format=raw,unit=0,file="C:/Program Files/qemu/share/edk2-x86_64-code.fd",readonly=on -drive if=pflash,format=raw,unit=1,file="C:/Program Files/qemu/share/edk2-i386-vars.fd" -net none
