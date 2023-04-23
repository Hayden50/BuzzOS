use crate::{FSSIZE, NBLOCKS, NINODES, LOGSIZE, NINODEBLOCKS};
use endian_codec::{PackedSize, EncodeLE, DecodeLE};

#[derive(Debug, PartialEq, EncodeLE, DecodeLE, PackedSize)]
pub struct Superblock {
    pub size: u32,          // Size of FS Image 
    pub nblocks: u32,       // Number of data blocks
    pub ninodes: u32,       // Number of inodes
    pub nlog: u32,          // Number of log blocks
    pub logstart: u32,      // Block number of first log block
    pub inodestart: u32,    // Block number of first inode block
    pub bmapstart: u32,     // Block number of first free map block
}

impl Superblock {
    pub fn new() -> Self {
        Superblock {
            size: FSSIZE as u32,
            nblocks: NBLOCKS,
            ninodes: NINODES,
            nlog: LOGSIZE,
            logstart: 1,
            inodestart: 1 + LOGSIZE,
            bmapstart: 1 + LOGSIZE + NINODEBLOCKS,
        }
    }
}
