#![no_std]  // Disable Rust standard library
#![no_main] // Disable all Rust-level entry points

use core::{fmt::Write, panic::PanicInfo};
mod vga_buffer;

// Used as the entry point of the OS
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    vga_buffer::WRITER.lock().write_str("Hello again").unwrap();
    write!(vga_buffer::WRITER.lock(), ", some numbers: {} {}", 42, 1.337).unwrap();
    
    loop {}
}

// Will be called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}