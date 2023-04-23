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

    // Initialize fs.img with all bytes written to zero
    fs_img.set_len((BSIZE * FSSIZE) as u64)?;
    zero_fs(&mut fs_img).unwrap();
    
    // Define values in the superblock struct and write it to disk
    let superblock = Superblock::new();
    write_superblock(&mut fs_img, &superblock)?;

    // Initialize the root directory by creating an Inode / allocating data blocks
    let mut root_inode = Inode::new(InodeType::Dir);
    let root_data_block = root_balloc(&mut fs_img);
    root_inode.data_addresses.0[0] = root_data_block.unwrap();
    write_inode(&mut fs_img, ROOT_INO, &root_inode, &superblock)?;
    
    // Create the "." and ".." directory entries
    let dot_entry = Dirent::new(ROOT_INO as u8, ".");
    let dotdot_entry = Dirent::new(ROOT_INO as u8, "..");
    
    // Write the "." and ".." directory entries to the root directory's data block
    write_dirent(&mut fs_img, &dot_entry, root_inode.data_addresses.0[0], 0)?;
    write_dirent(&mut fs_img, &dotdot_entry, root_inode.data_addresses.0[0], 1)?;
    

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

// Allocates the block for the root Inode and returns the block number (avoids searching bitmap
// because nothing has been written)
fn root_balloc(fs_img: &mut File) -> Result<u32> {
    fs_img.seek(SeekFrom::Start(((1 + LOGSIZE + NINODEBLOCKS) * BSIZE) as u64))?;
    let mut onebit: [u8; 1] = [0; 1];
    onebit[0] |= 1 << 7;
    fs_img.write(&onebit)?;
    Ok(NMETA)
}

// Writes a directory entry to a specified data block and index in the file system
fn write_dirent(fs_img: &mut File, dirent: &Dirent, block: u32, index: usize) -> Result<()> {
    let dirent_start = block * BSIZE + (index * std::mem::size_of::<Dirent>()) as u32;
    fs_img.seek(SeekFrom::Start(dirent_start as u64))?;

    let mut dirent_encoded = [0; std::mem::size_of::<Dirent>()];
    dirent.encode_as_le_bytes(&mut dirent_encoded);

    fs_img.write(&dirent_encoded)?;

    Ok(())
}

// fn ialloc(i_type: InodeType) {}
