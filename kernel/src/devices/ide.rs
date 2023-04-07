use crate::x86::helpers::{inb, outb, inw, outw};
use crate::fs::defs::Buf;
use crate::println;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, Ordering};
use alloc::sync::Arc;
use spin::Mutex;
use lazy_static::lazy_static;
use super::defs::*;

impl Ide {
    // Constructor for Ide Struct
    pub fn new() -> Ide {
        Ide {
            idequeue: Mutex::new(None),
            havedisk1: AtomicBool::new(false),
        }
    }

    // Waits for IDE to be ready for use
    fn idewait(&self, checkerr: bool) -> Result<(), ()> {
        while {
            let r = inb(0x1f7);
            (r & (IDE_BSY | IDE_DRDY)) != IDE_DRDY
        } {}
        
        if checkerr {
            let r = inb(0x1f7);
            if (r & (IDE_DF | IDE_ERR)) != 0 {
                return Err(());
            }
        }
        Ok(())
    }
    
    // Initializes IDE device and checks that disk 1 is present
    pub fn ideinit(&mut self) {
        self.idewait(false).unwrap();

        // Check if disk 1 is present
        outb(0x1f6, 0xe0 | (1 << 4));
        for _ in 0..1000 {
            if inb(0x1f7) != 0 {
                self.havedisk1.store(true, Ordering::SeqCst);
                break;
            }
        }

        // Switch back to disk 0.
        outb(0x1f6, 0xe0 | (0 << 4));
    }

    // Starts processing requests and sends necessary data to the IDE device
    fn idestart(&mut self, b: &mut Buf) {
        println!("IDESTART: START");
        println!("IDESTART: BEGINNING OF METHOD: B_VALID: {}, B_DIRTY: {}", b.flags & B_VALID, b.flags & B_DIRTY);
        
        if b.blockno >= FS_SIZE {
            panic!("incorrect blockno");
        }
        
        let sector_per_block = B_SIZE / SECTOR_SIZE;
        let sector = b.blockno * sector_per_block;
        let read_cmd = if sector_per_block == 1 {
            IDE_CMD_READ
        } else {
            IDE_CMD_RDMUL
        };
        
        let write_cmd = if sector_per_block == 1 {
            IDE_CMD_WRITE
        } else {
            IDE_CMD_WRMUL
        };

        if sector_per_block > 7 {
            panic!("idestart");
        }

        self.idewait(false).unwrap();
        outb(0x3f6, 0); // generate interrupt
        outb(0x1f2, sector_per_block as u8); // writes the sector_per_block value to the ide
        outb(0x1f3, (sector & 0xff) as u8); // writes the least significant 8 bits to the sector
        outb(0x1f4, ((sector >> 8) & 0xff) as u8); // writes the next 8 bits
        outb(0x1f5, ((sector >> 16) & 0xff) as u8); // writes the next 8 bits
        outb(0x1f6, 0xe0 | ((b.dev & 1) << 4 | ((sector >> 24) & 0x0f) as u32) as u8);
        
        // Checks if dirty bit is set in the buffer and if it is, write 
        if b.flags & B_DIRTY != 0 {
            println!("IDESTART: WRITING");
            outb(0x1f7, write_cmd);
            for i in (0..B_SIZE).step_by(4) {
                let data = u32::from_le_bytes([b.data[i], b.data[i + 1], b.data[i + 2], b.data[i + 3]]);
                unsafe {
                    outw(0x1f0, data);
                }
            }
        } else {
            println!("IDESTART: READING");
            outb(0x1f7, read_cmd);
        }

        self.ideintr(b);
        
        println!("IDESTART: ENDING OF METHOD: B_VALID: {}, B_DIRTY: {}", b.flags & B_VALID, b.flags & B_DIRTY);
        println!("IDESTART: END");
    }

    // Interrupt handler. Currently using busy waiting but can be edited by deleting the final
    // while loop and uncommenting the wakeup call.
    pub fn ideintr(&mut self, b: &mut Buf) {
        println!("IDEINTR: START");
        let idewait_result = self.idewait(true).is_ok();

        let mut queue = self.idequeue.lock();
        let mut next_buf_option = None;
        
        // If dirty bit is not set, it must be a read
        if b.flags & B_DIRTY == 0 && idewait_result {
            for i in (0..B_SIZE).step_by(4) {
                unsafe {
                    let data = inw(0x1f0);
                    b.data[i..i + 4].copy_from_slice(&data.to_le_bytes());
                }
            }
        }

        b.flags |= B_VALID; // Set valid bit
        b.flags &= !B_DIRTY; // Unset dirty bit
        
        println!("IDEINTR: UPDATED FLAGS: B_VALID: {}, B_DIRTY: {}", b.flags & B_VALID, b.flags & B_DIRTY);

        // In a real scheduler, you would wake up the process waiting for this buf here.
        // wakeup(b);

        // Set queue to next val
        *queue = b.qnext.clone();

        // Start disk on next buf in queue
        if let Some(next_buf) = queue.take() {
            next_buf_option = Some(next_buf);
        }
        
        drop(queue); // Explicitly drop the lock to release the borrow of self
        
        if let Some(mut next_buf) = next_buf_option {
            self.idestart(&mut next_buf);
        }

        // Busy-wait for the operation to complete
        while self.idewait(true).is_err() {
            println!("IDEINTR: WAITING");
        }
        println!("IDEINTR: END");
    }
    
    // Not working version that compiles
    pub fn iderw(&mut self, b: &mut Buf) {
        
        println!("IDERW CALLED");
        if (b.flags & (B_VALID | B_DIRTY)) == B_VALID {
            panic!("iderw: nothing to do");
        }
        if b.dev != 0 && !self.havedisk1.load(Ordering::SeqCst) {
            panic!("iderw: ide disk 1 not present");
        }
        
        // Acquire lock to queue
        let mut queue = self.idequeue.lock();
        let mut start_disk = false;

        // Check if the buffer will be the only value in the queue 
        if queue.is_none() {
            start_disk = true;
        }

        // Append b to back of queue
        b.qnext = None;
        let mut pp = &mut *queue;
        while let Some(p) = pp {
            pp = &mut p.qnext;
        }
        *pp = Some(Box::new(b.clone()));

        drop(queue); // Explicitly drop the lock to release the borrow of self
        
        if start_disk {
            println!("IDERW: BEFORE IDESTART: B_VALID: {}, B_DIRTY: {}", b.flags & B_VALID, b.flags & B_DIRTY);
            self.idestart(b);
            println!("IDERW: IDESTART: B_VALID: {}, B_DIRTY: {}", b.flags & B_VALID, b.flags & B_DIRTY);
        }
        
        // Wait for request to finish (if using a real scheduler).
        // sleep() called here once scheduler impl
        // while !done_flag.load(Ordering::SeqCst) {}
         
        while b.flags & (B_VALID | B_DIRTY) != B_VALID {}
        println!("IDERW: END");
    }

}

lazy_static! {
    pub static ref GLOBAL_IDE: Mutex<Ide> = Mutex::new(Ide::new());
}

pub fn setup_ide() {
    GLOBAL_IDE.lock().ideinit();
    println!("[KERNEL] Disk Initialized");
}
