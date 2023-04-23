pub struct Dirent {
    pub inum: u8,           // Inum of the directory
    pub name: [u8; 15],     // Name of the inode, max 15 characters long
}

impl Dirent {
    pub fn new(inum: u8, str_name: &str) -> Self {
        let mut name: [u8; 15] = [0; 15];
        let bytes = str_name.as_bytes();
        for (i, &byte) in bytes.iter().enumerate() {
            if i >= 15 {
                break;
            }
            name[i] = byte;
        }
        Dirent {
            inum,
            name
        }
    }
}

