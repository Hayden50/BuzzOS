use super::defs::*; 
use spin::Mutex;
use alloc::{
    boxed::Box,
    collections::VecDeque,
};
use lazy_static::lazy_static;
use crate::{println, devices::{defs::B_VALID, ide::GLOBAL_IDE}};
use hashbrown::HashMap;

impl BufCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: HashMap::new(),
            keys: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    // Gets a buffer from the buffer cache
    pub fn get(&mut self, key: &(u32, usize)) -> Option<&mut Buf> {
        if let Some(value) = self.cache.get_mut(key) {
            self.keys.retain(|val| val != key);
            self.keys.push_front(*key);
            Some(value)
        } else {
            None
        }
    }

    // Puts a buffer into the buffer cache
    pub fn put(&mut self, key: (u32, usize), value: Buf) {
        if self.keys.len() == self.capacity {
            if let Some(oldest_key) = self.keys.pop_back() {
                self.cache.remove(&oldest_key);
            }
        }
        self.keys.push_front(key);
        self.cache.insert(key, value);
    }

    // Gets a buffer from the cache and returns it. If the buffer does not 
    // exist in the cache, it will create a new one. This may result in a buffer 
    // release if the cache is already full.
    pub fn buf_get(&mut self, dev: u32, blockno: usize) -> &mut Buf {
        if self.get(&(dev, blockno)).is_some() {
            self.get(&(dev, blockno)).unwrap()
        } else {
            let new_buf = Buf::new(dev, blockno);
            self.put((dev, blockno), new_buf);
            self.get(&(dev, blockno)).unwrap()
        }
    }
    
    // Returns a buffer that contains the data from the disk at the specified dev / block
    pub fn buf_read(&mut self, dev: u32, blockno: usize) -> &Buf {
        let mut buffer = self.buf_get(dev, blockno);

        if buffer.flags & B_VALID == 0 {
            GLOBAL_IDE.lock().iderw(buffer);
        }

        buffer
    }
    
    pub fn buf_write() {}
    pub fn buf_release() {}
    
}

lazy_static! {
    pub static ref BUF_CACHE: Mutex<BufCache> = Mutex::new(BufCache::new(MAX_BUFS));
}

// Currenly useless, might remove 
pub fn setup_bcache() {
    BUF_CACHE.lock();
    println!("[KERNEL] Buffer Cache Initialized");
}
