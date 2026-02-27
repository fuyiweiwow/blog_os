#![no_std]  // Disable Rust standard library
#![no_main] // Disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"] 

use core::panic::PanicInfo;
use blog_os::{allocator, memory, println, task::{Task, executor::Executor, keyboard}};
use bootloader::{BootInfo, entry_point};
use x86_64::{VirtAddr};

extern crate alloc;

entry_point!(kernel_main);

// Used as the entry point of the OS
fn kernel_main(boot_info: &'static BootInfo) -> ! {

    println!("Hello World{}", "!");
    blog_os::init();

    use blog_os::memory::BootInfoFrameAllocator;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(& mut mapper, & mut frame_allocator)
        .expect("heap initialization failed");

    // Test entry point
    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

// Will be called on panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}