// This is a standalone utility that creates the second disk structure (fs.img) when the makefile
// is run. It runs on the host OS and defines the disk structure of the file system.

use shared_fs::*;
use endian_codec::{PackedSize, EncodeLE};
use std::env;
use std::fs::File;
use std::io::{Result, Seek, SeekFrom, Write};
use std::path::Path;

fn main() -> Result<()> {
    // Gathers args into a Vec and makes sure the minimum is met
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: mkfs fs.img files...");
        std::process::exit(1);
    }

    // Create the fs.img file
    let fs_img_path = Path::new("fs.img");
    let mut fs_img = File::create(&fs_img_path)?;

    // Initialize fs.img with zero bytes
    fs_img.set_len((BSIZE * FSSIZE) as u64)?;
    zero_fs(&mut fs_img).unwrap();
    
    // Define values in the superblock struct
    let superblock = Superblock::new();
    write_superblock(&mut fs_img, &superblock)?;

    // Initialize the root directory
    let root_inode = Inode::new(InodeType::Dir);
    write_inode(&mut fs_img, ROOT_INO, &root_inode, &superblock)?;

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

// Writes the superblock in to the second block on the disk
fn write_superblock(fs_img: &mut File, superblock: &Superblock) -> Result<()> {
    // This moves the pointer to the beginning of the first block 
    fs_img.seek(SeekFrom::Start(0))?;

    // Creates a buffer with the superblock written in little endian
    let mut sb_encoded = [0; Superblock::PACKED_LEN];
    superblock.encode_as_le_bytes(&mut sb_encoded);
    
    // Writes the buffer into disk
    fs_img.write(&sb_encoded)?;

    Ok(())
}

// Writes an inode to disk at a specified inum
fn write_inode(fs_img: &mut File, inum: u32, inode: &Inode, superblock: &Superblock) -> Result<()> {
    let inode_block = (inum / IPB) + superblock.inodestart;
    let inode_offset = (inum % IPB) * INODE_SIZE;
    
    fs_img.seek(SeekFrom::Start((inode_block * BSIZE + inode_offset) as u64))?;

    let mut inode_encoded = [0; Inode::PACKED_LEN];
    inode.encode_as_le_bytes(&mut inode_encoded);
    
    fs_img.write(&inode_encoded)?;

    Ok(())
}
 
// fn ialloc(i_type: InodeType) {}
// fn balloc() {}
