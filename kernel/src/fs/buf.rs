use super::defs::*;
use crate::devices::defs::B_SIZE;

impl Buf {
    pub fn new(dev: u32, blockno: usize) -> Buf {
        Buf {
            flags: 0,
            dev,
            blockno,
            data: [0; B_SIZE],
            qnext: None,
        }
    }
}

