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
    HardDisk,               // 0x21
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}
