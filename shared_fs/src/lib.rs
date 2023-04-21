#![no_std]
pub const ROOTINO: u32 = 1;  // root i-number
pub const BSIZE: u32 = 512;  // block size
pub const NDIRECT: u32 = 12;
 
#[derive(Debug, PartialEq)]
pub struct Superblock {
    pub size: u32,          // Size of FS Image 
    pub nblocks: u32,       // Number of data blocks
    pub ninodes: u32,       // Number of inodes
    pub nlog: u32,          // Number of log blocks
    pub logstart: u32,      // Block number of first log block
    pub inodestart: u32,    // Block number of first inode block
    pub bmapstart: u32,     // Block number of first free map block
}

#[derive(Debug, PartialEq)]
pub struct Inode {
    pub inode_type: InodeType,                      // Represents type of file the Inode references
    pub major_dev: u8,                              // Major device number
    pub minor_dev: u8,                              // Minor device number
    pub num_links: u8,                              // Number of symbolic links associated with this file 
    pub size: u32,                                  // Total number of bytes of content of a file
    pub data_addreses: [u32; NDIRECT as usize + 1], // Records the block numbers of the disk 
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

impl Inode {
    pub fn new(inode_type: InodeType) -> Self {
        Inode {
            inode_type,
            major_dev: 0,
            minor_dev: 0,
            num_links: 1,
            size: 0,
            data_addreses: [0; NDIRECT as usize + 1],
        }
    }
}
