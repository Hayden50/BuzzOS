use crate::x86::helpers::{inb, outb, inw, outw};
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;
use lazy_static::lazy_static;
use super::defs::*;
use crate::println;

impl Ide {
    // Constructor for Ide Struct
    pub fn new() -> Ide {
        Ide {
            idelock: Mutex::new(()),
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
        let _guard = self.idelock.lock();
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
            outb(0x1f7, write_cmd);
            for i in (0..B_SIZE).step_by(4) {
                let data = u32::from_le_bytes([b.data[i], b.data[i + 1], b.data[i + 2], b.data[i + 3]]);
                unsafe {
                    outw(0x1f0, data);
                }
            }
        } else {
            outb(0x1f7, read_cmd);
        }
    }

    // IDE interrupt handler. Atomically handles reads/writes to disk and updates queue
    // pub fn ideintr(&mut self) {
    //     let mut queue = self.idequeue.lock();
    //     if let Some(mut b) = queue.take() {
    //         // Read data if needed.
    //         if b.flags & B_DIRTY == 0 && self.idewait(true).is_ok() {
    //             for i in (0..B_SIZE).step_by(4) {
    //                 unsafe {
    //                     let data = inw(0x1f0);
    //                     b.data[i..i + 4].copy_from_slice(&data.to_le_bytes());
    //                 }
    //             }
    //         }
    //
    //         // Update buffer flags
    //         b.flags |= B_VALID;
    //         b.flags &= !B_DIRTY;
    //
    //         // Wake process waiting for this buf (if using a real scheduler).
    //
    //         // Start disk on next buf in queue
    //         if let Some(mut next_buf) = b.qnext.take() {
    //             self.idestart(&mut next_buf);
    //             *queue = Some(next_buf);
    //         }
    //     }
    // }
    
    pub fn ideintr(&mut self) {
        let idewait_result = self.idewait(true).is_ok();

        let mut queue = self.idequeue.lock();
        let mut next_buf_option = None;

        if let Some(mut b) = queue.take() {
            // Read data if needed.
            if b.flags & B_DIRTY == 0 && idewait_result {
                for i in (0..B_SIZE).step_by(4) {
                    unsafe {
                        let data = inw(0x1f0);
                        b.data[i..i + 4].copy_from_slice(&data.to_le_bytes());
                    }
                }
            }

            // Update buffer flags
            b.flags |= B_VALID;
            b.flags &= !B_DIRTY;

            // Wake process waiting for this buf (if using a real scheduler).

            // Start disk on next buf in queue
            if let Some(next_buf) = b.qnext.take() {
                next_buf_option = Some(next_buf);
            }

            *queue = Some(b);
        }

        drop(queue); // Explicitly drop the lock to release the borrow of self

        if let Some(mut next_buf) = next_buf_option {
            self.idestart(&mut next_buf);
        }
    }
    
    // Synces buffer and disk
    pub fn iderw(&mut self, b: &mut Buf) {
        let mut queue = self.idequeue.lock();
        let mut next_buf = None;

        // Append b to idequeue
        b.qnext = queue.take();
        *queue = Some(Box::new(b.clone()));

        // Check if disk needs to be started
        if queue.is_some() {
            next_buf = b.qnext.take();
        }
        
        drop(queue); // Explicitly drop the lock to release the borrow of self

        self.idestart(&mut next_buf.unwrap());
        
        // Wait for request to finish (if using a real scheduler).
    }

}

lazy_static! {
    pub static ref IDE: Mutex<Ide> = Mutex::new(Ide::new());
}

pub fn setup_ide() {
    IDE.lock().ideinit();
    println!("[KERNEL] Disk Initialized");
}
