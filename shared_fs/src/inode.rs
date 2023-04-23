use crate::NDIRECT;
use core::{mem, fmt};
use endian_codec::{PackedSize, EncodeLE, DecodeLE};

#[derive(Debug, PartialEq, EncodeLE, DecodeLE, PackedSize)]
pub struct Inode {
    pub inode_type: InodeType,          // Represents type of file the Inode references
    pub major_dev: u8,                  // Major device number
    pub minor_dev: u8,                  // Minor device number
    pub num_links: u8,                  // Number of symbolic links associated with this file 
    pub size: u32,                      // Total number of bytes of content of a file
    pub data_addresses: DataAddresses,   // Records the block numbers of the disk
}

impl Inode {
    pub fn new(inode_type: InodeType) -> Self {
        Inode {
            inode_type,
            major_dev: 0,
            minor_dev: 0,
            num_links: 1,
            size: 0,
            data_addresses: DataAddresses([0; NDIRECT as usize + 1]),
        }
    }
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
pub struct DataAddresses(pub [u32; 13]);

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

