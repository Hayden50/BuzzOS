use crate::{BSIZE, FSSIZE, NBLOCKS, NINODES, LOGSIZE, NINODEBLOCKS};
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
    pub datastart: u32,
}

impl Superblock {
    pub fn new() -> Self {
        let logstart = 1;
        let inodestart = logstart + LOGSIZE;
        let bmapstart = inodestart + NINODEBLOCKS;
        let bitmap_blocks = NBLOCKS / (8 * BSIZE as u32) + ((NBLOCKS % (8 * BSIZE as u32)) != 0) as u32;
        let datastart = bmapstart + bitmap_blocks;

        Superblock {
            size: FSSIZE as u32,
            nblocks: NBLOCKS,
            ninodes: NINODES,
            nlog: LOGSIZE,
            logstart,
            inodestart,
            bmapstart,
            datastart,
        }
    }
}
