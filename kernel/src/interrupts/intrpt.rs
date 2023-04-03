use core::arch::asm;

// Assembly wrapping using STI to enable interrupts
#[inline]
pub fn enable() {
    unsafe {
        asm!("sti", options(nomem, nostack));
    }
}

// Assembly wrapper using CLI to disable interrupts 
#[inline]
pub fn disable() {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
}
