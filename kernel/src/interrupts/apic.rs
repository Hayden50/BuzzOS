/// Advanced Programmable Interrupt Controller (APIC) is a hardware component that manages the
/// delivery of interrupt requests to processors. Multiple components external to the processor
/// (keyboard, timer, peripherals, etc) may try to generate an IRQ, and it is the APIC's job to
/// deliver those. You can find more information on PIC here: https://wiki.osdev.org/8259_PIC

use pic8259_x86::ChainedPics;
use spin;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,   // 0x20
    Keyboard,               // 0x21
    Cascade,                // 0x22
    COM2,                   // 0x23
    COM1,                   // 0x24
    LPT2,                   // 0x25
    FloppyDiskController,   // 0x26
    LPT1,                   // 0x27
    CMOSRealTimeClock,      // 0x28
    Free1,                  // 0x29
    Free2,                  // 0x2a
    Free3,                  // 0x2b
    PS2Mouse,               // 0x2c
    FPU,                    // 0x2d
    PrimaryATAHardDisk,     // 0x2e
    SecondaryATAHardDisk,   // 0x2f
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}
