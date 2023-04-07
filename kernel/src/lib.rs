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
pub mod structures;
pub mod threading;
pub mod x86;
pub mod fs;

extern crate alloc;

// Interface definition of panic in Rust. Core represents the core library
use core::panic::PanicInfo;

use crate::devices::defs::*;
use crate::fs::defs::Buf;
use crate::devices::ide::GLOBAL_IDE;
// use alloc::vec::Vec;

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

    // test_write();
    test_read();

    println!("Goodbye");
    
    loop {}
}

fn test_write() {
    println!("-------------------START WRITE---------------------");
    let block_number = 24;
    let mut write_buf = Buf::new(0, block_number);
    let pattern: [u8; 512] = [0xAB; 512];

    // Write the pattern to the disk
    write_buf.data.copy_from_slice(&pattern);
    write_buf.flags |= B_DIRTY;
    GLOBAL_IDE.lock().iderw(&mut write_buf);
    print!("Buffer Data: ");
    write_buf.data.map(|num| print!("{} ", num));
    println!("\nBuffer flags: DIRTY = {}, VALID = {}", write_buf.flags & B_DIRTY, write_buf.flags & B_VALID);
    println!("-------------------END WRITE---------------------");
}

fn test_read() {
    println!("-------------------START READ---------------------");
    let block_number = 0;
    // Read the same block from the disk
    let mut read_buf = Buf::new(0, block_number);
    GLOBAL_IDE.lock().iderw(&mut read_buf);
    print!("Buffer Data: ");
    read_buf.data.map(|num| print!("{} ", num));
    println!("\nBuffer flags: DIRTY = {}, VALID = {}", read_buf.flags & B_DIRTY, read_buf.flags & B_VALID);
    println!("-------------------END READ---------------------");
}

// fn test_disk_operations(ide: &mut Ide) {
//     
//     
//     let block_number = 24;
//     let mut write_buf = Buf::new(0, block_number);
//     let pattern: [u8; 512] = [0xAB; 512];
//
//     println!("LIB: Before first iderw");
//     // Write the pattern to the disk
//     write_buf.data.copy_from_slice(&pattern);
//     write_buf.flags |= B_DIRTY;
//     ide.iderw(&mut write_buf);
//
//     println!("LIB: second iderw");
//     // Read the same block from the disk
//     let mut read_buf = Buf::new(0, block_number);
//     ide.iderw(&mut read_buf);
//
//     // Verify that the written and read data match
//     assert_eq!(write_buf.data, read_buf.data, "Data mismatch after reading the written block");
//
//     // Test writing and reading multiple blocks
//     let block_count = 10;
//     let mut write_bufs = Vec::with_capacity(block_count);
//     let mut read_bufs = Vec::with_capacity(block_count);
//
//     for i in 0..block_count {
//         let mut buf = Buf::new(0, block_number + i);
//         buf.data.iter_mut().enumerate().for_each(|(j, b)| *b = (i + j) as u8);
//         buf.flags |= B_DIRTY;
//         write_bufs.push(buf);
//     }
//
//     for buf in &mut write_bufs {
//         ide.iderw(buf);
//     }
//
//     for i in 0..block_count {
//         let mut buf = Buf::new(0, block_number + i);
//         read_bufs.push(buf);
//     }
//
//     for buf in &mut read_bufs {
//         ide.iderw(buf);
//     }
//
//     for (i, (write_buf, read_buf)) in write_bufs.iter().zip(read_bufs.iter()).enumerate() {
//         assert_eq!(write_buf.data, read_buf.data, "Data mismatch in block {} after reading the written block", i);
//     }
//
//     // Test handling of out-of-bounds block numbers
//     let invalid_block_number = 1_000_000; // Assuming this is an invalid block number
//     let mut invalid_buf = Buf::new(0, invalid_block_number);
//     invalid_buf.flags |= B_DIRTY;
//
//     // This should either panic or return an error, depending on your implementation
//     ide.iderw(&mut invalid_buf);
// }


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
