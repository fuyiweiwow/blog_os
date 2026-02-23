#![no_std]
#![cfg_attr(test, no_main)]                                
#![feature(custom_test_frameworks)]             
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]    // Act like redirect the "test_main" to test_runner
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
pub mod vga_buffer;
pub mod serial;
pub mod interrupts;
pub mod gdt;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success     = 0x10,
    Failed      = 0x11
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4); // 0xf4 is iobase of isa-debug-exit device
        port.write(exit_code as u32);                                           // Write to port to tell that we need exit
    }

}


pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T 
where 
    T : Fn(), 
{
    fn run(&self) -> () {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_print!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

// Will be called on panic
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

/*
    Halt the CPU until the next interrupt arrives
 */
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

// Entry point for `cargo test`
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}