use alloc::boxed::Box;
use core::sync::atomic::AtomicBool;
use spin::Mutex;

pub const COM1: u16 = 0x3F8; // Base port address for first serial communication port

pub const SECTOR_SIZE: usize = 512; // Size of disk sector
pub const IDE_BSY: u8 = 0x80; // IDE busy bit 
pub const IDE_DRDY: u8 = 0x40; // IDE ready bit
pub const IDE_DF: u8 = 0x20; // IDE fault bit
pub const IDE_ERR: u8 = 0x01; // IDE error bit

pub const IDE_CMD_READ: u8 = 0x20; // Command code to read data
pub const IDE_CMD_WRITE: u8 = 0x30; // Command code to write data 
pub const IDE_CMD_RDMUL: u8 = 0xc4; // Command code to read multiple sectors
pub const IDE_CMD_WRMUL: u8 = 0xc5; // Command code to write to multiple sectors

pub const IDE_PORT_BASE_PRIMARY: u16 = 0x1F0; // Base I/O Address for primary IDE controller
pub const IDE_PORT_BASE_SECONDARY: u16 = 0x170; // Base I/O Adress for secondary IDE controller

pub const B_VALID: u8 = 0x2; // Buffer valid bit
pub const B_DIRTY: u8 = 0x4; // Buffer dirty bit
pub const B_SIZE: usize = 512; // Size of one block

pub const FS_SIZE: usize = 1000; // Size of file system in blocks

pub struct Ide {
    pub idelock: Mutex<()>,
    pub idequeue: Mutex<Option<Box<Buf>>>,
    pub havedisk1: AtomicBool,
}

#[derive(Clone)]
pub struct Buf {
    pub dev: u32,
    pub blockno: usize,
    pub flags: u8,
    pub data: [u8; SECTOR_SIZE],
    pub qnext: Option<Box<Buf>>,
}
