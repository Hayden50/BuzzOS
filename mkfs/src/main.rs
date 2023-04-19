use shared_fs::{Inode, InodeType, Superblock};
use std::env;
use std::fs::File;
use std::io::{Result, Seek, SeekFrom, Write};
use std::path::Path;

const FS_IMG: &str = "fs.img";
const BSIZE: u32 = 512; // Size of one block
const FSSIZE: u32 = 1000; // Number of blocks in FS
const LOGSIZE: u32 = 30; // Number of blocks dedicated to log

const NDIR: u32 = 12;
const NINDIRECT: u32 = BSIZE as u32 / 4;
const MAXFILE: u32 = NDIR + NINDIRECT;

const ROOT_INO: u32 = 1; // Root Inode number
const IBLOCK: u32 = 2;
const BBLOCK: u32 = 3;

const NINODES: u32 = 200; // Number of Inodes
const IPB: u32 = BSIZE as u32 / 64; // Inodes per block
const NINODEBLOCKS: u32 = NINODES / IPB; // Number of blocks holding Inodes

fn main() -> Result<()> {
    // Gathers args into a Vec and makes sure the minimum is met
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: mkfs fs.img files...");
        std::process::exit(1);
    }

    // Create the fs.img file
    let fs_img_path = Path::new(FS_IMG);
    let mut fs_img = File::create(&fs_img_path)?;

    // Initialize fs.img with zero bytes
    fs_img.set_len((BSIZE * FSSIZE) as u64)?;
    zero_fs(&mut fs_img).unwrap();

    let nlog = LOGSIZE;
    let nbitmap = FSSIZE / (BSIZE * 8) + 1;
    let nmeta = 2 + nlog + NINODEBLOCKS + nbitmap;
    let nblocks = FSSIZE - nmeta;

    // Define values in the superblock struct
    let superblock = Superblock {
        size: FSSIZE as u32,
        nblocks,
        ninodes: NINODES,
        nlog: LOGSIZE,
        logstart: 2,
        inodestart: 2 + LOGSIZE,
        bmapstart: 2 + LOGSIZE + NINODEBLOCKS,
    };
    write_superblock(&mut fs_img, &superblock)?;

    // Initialize the root directory
    let root_inode = Inode::new(InodeType::Dir);
    write_inode(&mut fs_img, ROOT_INO, &root_inode)?;

    // Add any additional files, directories, or user programs to the file system
    // ...

    fs_img.flush()?;
    println!("File system created successfully.");
    Ok(())
}

// Clears file system by setting everything to 0
fn zero_fs(fs_img: &mut File) -> Result<()> {
    let zeroes = vec![0; BSIZE as usize];
    for i in 0..FSSIZE {
        fs_img.seek(SeekFrom::Start((BSIZE * i) as u64))?;
        fs_img.write_all(&zeroes)?;
    }
    Ok(())
}

fn write_superblock(fs_img: &mut File, superblock: &Superblock) -> Result<()> {
    fs_img.seek(SeekFrom::Start(BSIZE as u64))?;
    let superblock_data = bincode::serialize(superblock).unwrap();
    fs_img.write_all(&superblock_data)
}

// fn ialloc(i_type: InodeType) {}
// fn balloc() {}

fn write_inode(fs_img: &mut File, inum: u32, inode: &Inode) -> Result<()> {
    fs_img.seek(SeekFrom::Start(
        BSIZE as u64 * (IBLOCK as u64 + inum as u64 / IPB as u64),
    ))?;
    let inode_data = bincode::serialize(inode).unwrap();
    fs_img.write_all(&inode_data)
}

// Implement additional helper functions to handle file system operations
// like creating files, directories, writing data blocks, etc.
