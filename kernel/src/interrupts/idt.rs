use core::{marker::PhantomData, mem::size_of};

use lazy_static::lazy_static;

use crate::{interrupts::handlers::*, println, x86::helpers::lidt};

use super::defs::*;

impl<F> Gate<F> {
    // Implementation of an empty gate. Used to initialized gates
    #[inline]
    pub const fn empty() -> Self {
        // Ensure our gate is an interrupt at startup.
        let flags = GateFlags::INTGATE as u8;

        Gate {
            fn_addr_low: 0,
            fn_addr_high: 0,
            segment_selector: 0,
            reserved: 0,
            handler: PhantomData,
            flags,
        }
    }

    // Set gate handler. Accepts the 64-bits address of the handler function
    #[inline]
    pub unsafe fn set_handler_addr(&mut self, addr: u32) -> &mut u8 {
        self.fn_addr_low = addr as u16;
        self.fn_addr_high = (addr >> 16) as u16;
        self.segment_selector = 0x8;
        self.flags |= GateFlags::PRESENT as u8;
        &mut self.flags
    }
}

impl Gate<InterruptHandler> {
    #[inline]
    pub fn set_handler_fn(&mut self, handler: InterruptHandler) {
        let handler = handler as u32;
        unsafe { self.set_handler_addr(handler) };
    }
}

impl Gate<InterruptHandlerWithErr> {
    #[inline]
    pub fn set_handler_fn(&mut self, handler: InterruptHandlerWithErr) {
        let handler = handler as u32;
        unsafe { self.set_handler_addr(handler) };
    }
}

impl Gate<PageFaultHandler> {
    #[inline]
    pub fn set_handler_fn(&mut self, handler: PageFaultHandler) {
        let handler = handler as u32;
        unsafe { self.set_handler_addr(handler) };
    }
}

impl IDT {
    // Initialization of our Interrupt Descriptor Table. Reserved gates must also be initialized.
    // Notice gp_interrupts are also intiialized, being composed of 224 elements. Those are
    // interrupts available for the OS (e.g. System Calls).
    #[inline]
    pub fn new() -> IDT {
        IDT {
            div_by_zero: Gate::empty(),                         // 0
            debug: Gate::empty(),                               // 1
            non_maskable_interrupt: Gate::empty(),              // 2
            breakpoint: Gate::empty(),                          // 3
            overflow: Gate::empty(),                            // 4
            bound_range_exceeded: Gate::empty(),                // 5
            invalid_opcode: Gate::empty(),                      // 6
            device_not_available: Gate::empty(),                // 7
            double_fault: Gate::empty(),                        // 8
            coprocessor_segment_overrun: Gate::empty(),         // 9
            invalid_tss: Gate::empty(),                         // 10
            segment_not_present: Gate::empty(),                 // 11
            stack_segment_fault: Gate::empty(),                 // 12
            gen_protection_fault: Gate::empty(),                // 13
            page_fault: Gate::empty(),                          // 14
            reserved_1: Gate::empty(),                          // 15
            x87_floating_point: Gate::empty(),                  // 16
            alignment_check: Gate::empty(),                     // 17
            machine_check: Gate::empty(),                       // 18
            simd_floating_point: Gate::empty(),                 // 19
            virtualization: Gate::empty(),                      // 20
            cp_protection_exception: Gate::empty(),             // 21
            reserved_7: [Gate::empty(); 7],                     // 22-28
            hv_injection_exception: Gate::empty(),              // 29
            vmm_communication_exception: Gate::empty(),         // 30
            security_exception: Gate::empty(),                  // 31
            timer: Gate::empty(),                               // 32
            keyboard: Gate::empty(),                            // 33
            cascade: Gate::empty(),                             // 34
            com2: Gate::empty(),                                // 35
            com1: Gate::empty(),                                // 36
            lpt2: Gate::empty(),                                // 37
            floppy_disk_controller: Gate::empty(),              // 38
            lpt1: Gate::empty(),                                // 39
            real_time_clock: Gate::empty(),                     // 40
            free1: Gate::empty(),                               // 41
            free2: Gate::empty(),                               // 42
            free3: Gate::empty(),                               // 43
            ps2_mouse: Gate::empty(),                           // 44
            fpu: Gate::empty(),                                 // 45
            primary_ata_hard_disk: Gate::empty(),               // 46
            secondary_ata_hard_disk: Gate::empty(),             // 47
            uprint: Gate::empty(),                              // 48
            gp_interrupts: [Gate::empty(); 256 - 49],           // 49-255
        }
    }

    /// Creates the descriptor pointer for this table. This pointer can only be
    /// safely used if the table is never modified or destroyed while in use.
    fn pointer(&self) -> InterruptDescriptorTablePointer {
        InterruptDescriptorTablePointer {
            base: self as *const _ as u32,
            limit: (size_of::<Self>() - 1) as u16,
        }
    }

    // This two-step load is necessary to ensure our IDT is available whenever
    // the CPU needs it. Notice a non-static reference would cause all sorts
    // of bugs related to free before use.
    #[inline]
    pub fn load(&'static self) {
        lidt(&self.pointer());
    }
}

lazy_static! {
    static ref GLOBAL_IDT: IDT = {
        let mut global_idt = IDT::new();

        // Setup Handler
        global_idt.div_by_zero.set_handler_fn(div_by_zero_handler);
        global_idt.breakpoint.set_handler_fn(breakpoint_handler);
        global_idt.gen_protection_fault.set_handler_fn(gen_protection_fault);
        // global_idt.double_fault.set_handler_fn(double_fault_handler);
        global_idt.page_fault.set_handler_fn(page_fault);
        global_idt.overflow.set_handler_fn(overflow);
        global_idt.bound_range_exceeded.set_handler_fn(bound_range);
        global_idt.primary_ata_hard_disk.set_handler_fn(primary_disk_access);
        global_idt.secondary_ata_hard_disk.set_handler_fn(secondary_disk_access);
        global_idt.timer.set_handler_fn(timer_interrupt);
        global_idt
    };
}

pub fn setup_idt() {
    GLOBAL_IDT.load();
    println!("[KERNEL] Interrupt Table Initialized");
}
