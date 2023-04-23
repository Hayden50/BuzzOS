#![no_std]

pub mod superblock;
pub mod inode;
pub mod dirent;

pub use superblock::Superblock;
pub use inode::{Inode, InodeType, DataAddresses};
pub use dirent::Dirent;

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
 
