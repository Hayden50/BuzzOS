use alloc::{
    collections::VecDeque,
    boxed::Box,
};
use crate::devices::defs::SECTOR_SIZE;
use hashbrown::HashMap;

pub const MAX_BUFS: usize = 10; // Maximum buffers in buffer cache

#[derive(Clone)]
pub struct Buf {
    pub dev: u32, // Device identifier (disk id)
    pub blockno: usize, // Block number of disk
    pub flags: u8, // Buffer flags (Dirty, Valid, etc.)
    pub data: [u8; SECTOR_SIZE], // Data in buffer
    pub qnext: Option<Box<Buf>>, // Pointer to the next buffer in idequeue
}

pub struct BufCache {
    pub cache: HashMap<(u32, usize), Buf>, // Maps buffer dev, blockno to a buffer
    pub keys: VecDeque<(u32, usize)>, // Sorts recency of buffer use
    pub capacity: usize, // Current amount of buffers in the cache
}
