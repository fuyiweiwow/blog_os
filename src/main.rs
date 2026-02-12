#![no_std]  // Disable Rust standard library
#![no_main] // Disable all Rust-level entry points

use core::panic::PanicInfo;
mod vga_buffer;

// Used as the entry point of the OS
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    
    loop {}
}

// Will be called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}