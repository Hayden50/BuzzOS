#![no_std]
use endian_codec::{PackedSize, EncodeLE, DecodeLE};
use core::mem;

pub const BSIZE: u32 = 512;  // Block size pub const
pub const NDIRECT: u32 = 12; // Number of directly referenced blocks for each Inode
pub const FSSIZE: u32 = 1000; // Number of blocks in FS
pub const LOGSIZE: u32 = 10; // Number of blocks dedicated to log

// const NINDIRECT: u32 = BSIZE as u32 / 4;
// const MAXFILE: u32 = NDIR + NINDIRECT;

pub const ROOT_INO: u32 = 1; // Root Inode number
// const IBLOCK: u32 = 2;
// const BBLOCK: u32 = 3;

pub const NINODES: u32 = 200; // Number of Inodes
pub const INODE_SIZE: u32 = 60; // Size of each Inode in bytes
pub const IPB: u32 = BSIZE as u32 / INODE_SIZE; // Inodes per block
pub const NINODEBLOCKS: u32 = NINODES / IPB; // Number of blocks holding Inodes
 
pub const NBITMAP: u32 = FSSIZE / (BSIZE * 8) + 1;
pub const NMETA: u32 = 1 + LOGSIZE + NINODEBLOCKS + NBITMAP;
pub const NBLOCKS: u32 = FSSIZE - NMETA;
 
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

#[derive(Debug, PartialEq, EncodeLE, DecodeLE, PackedSize)]
pub struct Inode {
    pub inode_type: InodeType,          // Represents type of file the Inode references
    pub major_dev: u8,                  // Major device number
    pub minor_dev: u8,                  // Minor device number
    pub num_links: u8,                  // Number of symbolic links associated with this file 
    pub size: u32,                      // Total number of bytes of content of a file
    pub data_addreses: DataAddresses,   // Records the block numbers of the disk
}

pub struct Dirent {
    pub inum: u8,           // Inum of the directory
    pub name: [u8; 15],     // Name of the inode, max 15 characters long
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InodeType {
    Free = 0,
    Dir = 1,
    File = 2,
    Device = 3,
}

// Wrapper for u32 array for serialization / deserialization purposes
#[derive(Debug, PartialEq)]
pub struct DataAddresses([u32; 13]);

impl PackedSize for DataAddresses {
   const PACKED_LEN: usize = 13 * 4; 
}

impl EncodeLE for DataAddresses {
    fn encode_as_le_bytes(&self, bytes: &mut [u8]) {
        let mut idx = 0;
        for value in &self.0 {
            value.encode_as_le_bytes(&mut bytes[idx..idx + mem::size_of::<u32>()]);
            idx += mem::size_of::<u32>();
        }
    }
}

impl DecodeLE for DataAddresses {
    fn decode_from_le_bytes(bytes: &[u8]) -> Self {
        let mut data_addresses = [0u32; 13];
        let mut idx = 0;
        for value in data_addresses.iter_mut() {
            *value = u32::decode_from_le_bytes(&bytes[idx..idx + mem::size_of::<u32>()]);
            idx += mem::size_of::<u32>();
        }
        DataAddresses(data_addresses)
    }
}

impl PackedSize for InodeType {
    const PACKED_LEN: usize = 1;
}

impl EncodeLE for InodeType {
    fn encode_as_le_bytes(&self, bytes: &mut[u8]) {
        bytes[0] = *self as u8;
    }
}

impl DecodeLE for InodeType {
    fn decode_from_le_bytes(bytes: &[u8]) -> Self {
        let value = bytes[0];
        match value {
            1 => Self::Dir,
            2 => Self::File,
            3 => Self::Device,
            _ => Self::Free,
        }
    }
}

impl Inode {
    pub fn new(inode_type: InodeType) -> Self {
        Inode {
            inode_type,
            major_dev: 0,
            minor_dev: 0,
            num_links: 1,
            size: 0,
            data_addreses: DataAddresses([0; NDIRECT as usize + 1]),
        }
    }
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
