// This is a standalone utility that creates the second disk structure (fs.img) when the makefile
// is run. It runs on the host OS and defines the disk structure of the file system.

use shared_fs::*;
use endian_codec::{PackedSize, EncodeLE, DecodeLE};
use std::env;
use std::fs::{OpenOptions, File};
use std::io::{Read, Result, Seek, SeekFrom, Write};
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
    let mut fs_img = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&fs_img_path)?;

    // Initialize fs.img with all bytes written to zero
    fs_img.set_len((BSIZE * FSSIZE) as u64)?;
    zero_fs(&mut fs_img).unwrap();
    
    // Define values in the superblock struct and write it to disk
    let superblock = Superblock::new();
    write_superblock(&mut fs_img, &superblock)?;

    // Initialize the root directory by creating an Inode / allocating data blocks
    let mut root_inode = Inode::new(InodeType::Dir);
    let root_data_block = balloc(&mut fs_img, &superblock);
    root_inode.data_addresses.0[0] = root_data_block.unwrap();
    write_inode(&mut fs_img, ROOT_INO, &root_inode, &superblock)?;
    
    // Create the "." and ".." directory entries
    let dot_entry = Dirent::new(ROOT_INO as u8, ".");
    let dotdot_entry = Dirent::new(ROOT_INO as u8, "..");
    
    // Write the "." and ".." directory entries to the root directory's data block
    write_dirent(&mut fs_img, &dot_entry, root_inode.data_addresses.0[0], 0)?;
    write_dirent(&mut fs_img, &dotdot_entry, root_inode.data_addresses.0[0], 1)?;

    // Adds additional user programs to the file system
    // let path_start: &str = "../../";
    // for i in 2..args.len() {
    //     let file_path: &str = &args[i];
    //     let full_path = path_start.to_owned() + file_path;
    //     println!("Adding file: {}", full_path);
    //     add_file_to_fs(&mut fs_img, &(full_path), &superblock)?;
    // }

    fs_img.flush()?;
    println!("File system created successfully.");
    Ok(())
}

// TODO: THIS WILL ADD THE USER BINARIES TO THE OS (BUGGY SO COMMENTED OUT)
// fn add_file_to_fs(fs_img: &mut File, file_path: &str, superblock: &Superblock) -> Result<()> {
//     let path = Path::new(file_path);
//     let file_name = path.file_name().unwrap().to_str().unwrap();
//
//     if file_name.len() > 15 as usize {
//         return Err(std::io::Error::new(
//             std::io::ErrorKind::InvalidInput,
//             "File name is too long",
//         ));
//     }
//
//     let mut file = File::open(file_path)?;
//
//     // Allocate an inode for the new file
//     let inum = ialloc(fs_img, superblock)?;
//     let mut new_inode = Inode::new(InodeType::File);
//
//     // Allocate and write data blocks
//     let mut buffer = vec![0; BSIZE as usize];
//     let mut file_size = 0;
//     let mut block_index = 0;
//
//     while let Ok(bytes_read) = file.read(&mut buffer) {
//         if bytes_read == 0 {
//             break;
//         }
//
//         let data_block = balloc(fs_img, superblock)?;
//         fs_img.seek(SeekFrom::Start((data_block * BSIZE) as u64))?;
//         fs_img.write_all(&buffer[0..bytes_read])?;
//
//         new_inode.data_addresses.0[block_index] = data_block;
//         file_size += bytes_read;
//         block_index += 1;
//     }
//
//     new_inode.size = file_size as u32;
//     write_inode(fs_img, inum, &new_inode, superblock)?;
//
//     // Add directory entry in the root directory
//     let root_inode = read_inode(fs_img, ROOT_INO, superblock)?;
//     let mut dirent_added = false;
//
//     for i in 0..(root_inode.size as usize / std::mem::size_of::<Dirent>()) {
//         let dirent = read_dirent(fs_img, root_inode.data_addresses.0[0], i)?;
//         if dirent.inum == 0 {
//             let new_dirent = Dirent::new(inum as u8, file_name);
//             write_dirent(fs_img, &new_dirent, root_inode.data_addresses.0[0], i)?;
//             dirent_added = true;
//             break;
//         }
//     }
//
//     if !dirent_added {
//         return Err(std::io::Error::new(
//             std::io::ErrorKind::Other,
//             "No space in root directory",
//         ));
//     }
//
//     Ok(())
// }

// Helper function to read directory entries from disk
fn read_dirent(fs_img: &mut File, block: u32, index: usize) -> Result<Dirent> {
    let dirent_start = block * BSIZE + (index * std::mem::size_of::<Dirent>()) as u32;
    fs_img.seek(SeekFrom::Start(dirent_start as u64))?;

    let mut dirent_encoded = [0; Dirent::PACKED_LEN];
    fs_img.read_exact(&mut dirent_encoded)?;

    let dirent = Dirent::decode_from_le_bytes(&dirent_encoded);

    Ok(dirent)
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

// Writes a directory entry to a specified data block and index in the file system
fn write_dirent(fs_img: &mut File, dirent: &Dirent, block: u32, index: usize) -> Result<()> {
    let dirent_start = block * BSIZE + (index * std::mem::size_of::<Dirent>()) as u32;
    fs_img.seek(SeekFrom::Start(dirent_start as u64))?;

    let mut dirent_encoded = [0; Dirent::PACKED_LEN];
    dirent.encode_as_le_bytes(&mut dirent_encoded);

    fs_img.write(&dirent_encoded)?;

    Ok(())
}

// Allocate an i number
fn ialloc(fs_img: &mut File, superblock: &Superblock) -> Result<u32> {
    for inum in 1..superblock.ninodes {
        // Read the inode from the disk
        let inode = read_inode(fs_img, inum, superblock)?;

        // Check if the inode is unused
        if let InodeType::Free = inode.inode_type {
            return Ok(inum);
        }
    }

    // If no unused inodes are found, return an error
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "No unused inodes available",
    ))
}

// Helper method for ialloc to check if the current inode is free or not
fn read_inode(fs_img: &mut File, inum: u32, superblock: &Superblock) -> Result<Inode> {
    let inode_block = (inum / IPB) + superblock.inodestart;
    let inode_offset = (inum % IPB) * INODE_SIZE;

    fs_img.seek(SeekFrom::Start((inode_block * BSIZE + inode_offset) as u64))?;

    let mut inode_encoded: [u8; Inode::PACKED_LEN] = [0; Inode::PACKED_LEN];
    fs_img.read_exact(&mut inode_encoded)?;

    let inode = Inode::decode_from_le_bytes(&inode_encoded);

    Ok(inode)
}

// Checks bitmap for an available block and then allocates it
fn balloc(fs_img: &mut File, superblock: &Superblock) -> Result<u32> {
    let bitmap_start = superblock.bmapstart * BSIZE;
    let mut bitmap = vec![0; BSIZE as usize];
    fs_img.seek(SeekFrom::Start(bitmap_start as u64))?;
    fs_img.read_exact(&mut bitmap)?;

    let data_start_block = superblock.datastart;

    let mut free_block: Option<u32> = None;
    for (i, byte) in bitmap.iter().enumerate() {
        for j in 0..8 {
            if byte & (1 << j) == 0 {
                free_block = Some((i * 8 + j) as u32 + data_start_block);
                break;
            }
        }
        if free_block.is_some() {
            break;
        }
    }

    let free_block = free_block.ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "No free blocks available"))?;

    let index = (free_block - data_start_block) / 8;
    let bit = (free_block - data_start_block) % 8;
    bitmap[index as usize] |= 1 << bit;

    fs_img.seek(SeekFrom::Start(bitmap_start as u64))?;
    fs_img.write_all(&bitmap)?;

    fs_img.seek(SeekFrom::Start((free_block * BSIZE) as u64))?;
    fs_img.write_all(&vec![0; BSIZE as usize])?;

    Ok(free_block)
}
