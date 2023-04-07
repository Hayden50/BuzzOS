use alloc::boxed::Box;
use crate::devices::defs::SECTOR_SIZE;

#[derive(Clone)]
pub struct Buf {
    pub dev: u32,
    pub blockno: usize,
    pub flags: u8,
    pub data: [u8; SECTOR_SIZE],
    pub qnext: Option<Box<Buf>>,
}
