use crate::print;
use crate::println;
use crate::interrupts::apic;

use super::defs::{InterruptStackFrame, PageFaultErr};

pub extern "x86-interrupt" fn div_by_zero_handler(frame: InterruptStackFrame) {
    println!("EXCEPTION: DIVISION BY ZERO\n{:#?}", frame);
}

pub extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", frame);
}

pub extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, _error_code: PageFaultErr) {
    println!("EXCEPTION: PAGE FAULT\n{:#?}", frame);
}

pub extern "x86-interrupt" fn disk_access_handler(frame: InterruptStackFrame) {
    println!("EXCEPTION: DISK INTERRUPT\n{:#?}", frame);
    unsafe {
        apic::PICS.lock().notify_end_of_interrupt(apic::InterruptIndex::PrimaryATAHardDisk.as_u8());
    }
}

pub extern "x86-interrupt" fn timer_interrupt_handler(frame: InterruptStackFrame) {
    unsafe {
        apic::PICS.lock().notify_end_of_interrupt(apic::InterruptIndex::Timer.as_u8());
    }
}

pub extern "x86-interrupt" fn non_maskable(frame: InterruptStackFrame) {
    println!("EXCEPTION: NON MASKABLE INTERRUPT\n{:#?}", frame);
}

pub extern "x86-interrupt" fn overflow(frame: InterruptStackFrame) {
    println!("EXCEPTION: OVERFLOW\n{:#?}", frame);
}

pub extern "x86-interrupt" fn bound_range(frame: InterruptStackFrame) {
    println!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", frame);
}

pub extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _err: u32) {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#X?}", frame);
}

pub extern "x86-interrupt" fn gen_protection_fault(frame: InterruptStackFrame, _err: u32) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", frame);
}
