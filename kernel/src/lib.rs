#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(alloc_error_handler)]
#[macro_use]

pub mod devices;
pub mod interrupts;
pub mod memory;
pub mod misc;
pub mod scheduler;
pub mod structures;
pub mod threading;
pub mod x86;
pub mod fs;

extern crate alloc;

// Interface definition of panic in Rust. Core represents the core library
use core::panic::PanicInfo;

// Uses C calling convention instead of Rust. no_mangle removes name mangling when compiled.
// _start is the default entry point for most systems. Function is diverging as the Kernel should
// never return
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    // Initialize debugging method (VGA or Console)
    devices::debug::debug_init();
    misc::logo::print_logo();

    // Setup Segmentation and Virtual Memory
    memory::vm::setup_vm();
    memory::gdt::setup_gdt();
    memory::heap::setup_heap();

    // Setup Interrupts
    interrupts::idt::setup_idt();

    // Setup PIC and Enable Interrupts
    unsafe {interrupts::apic::PICS.lock().initialize()};
    interrupts::intrpt::enable();

    // Initialize IDE Device
    devices::ide::setup_ide();

    // Enable Buffer Caching
    fs::bio::setup_bcache();

    // Scheduler
    // scheduler::process::spawn_init_process();
    // scheduler::scheduler::setup_scheduler();

    // Should never proceeed
    // panic!("[FATAL] Returned from Scheduler");
    loop {}
}

// Once the Kernel panics, enter an infinite loop
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print!("{}", _info);
    loop {}
}

#[alloc_error_handler]
fn alloc_panic(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
