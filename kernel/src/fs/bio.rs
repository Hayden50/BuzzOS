use super::defs::*; 
use spin::Mutex;
use alloc::{
    boxed::Box,
    collections::VecDeque,
};
use lazy_static::lazy_static;
use crate::println;
use hashbrown::HashMap;

impl BufCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: HashMap::new(),
            keys: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get(&mut self, key: &(u32, usize)) -> Option<&Buf> {
        if let Some(value) = self.cache.get(key) {
            self.keys.retain(|val| val != key);
            self.keys.push_front(*key);
            Some(value)
        } else {
            None
        }
    }

    pub fn put(&mut self, key: (u32, usize), value: Buf) {
        if self.keys.len() == self.capacity {
            if let Some(oldest_key) = self.keys.pop_back() {
                self.cache.remove(&oldest_key);
            }
        }
        self.keys.push_front(key);
        self.cache.insert(key, value);
    }

    pub fn buf_get() {}
    pub fn buf_read() {}
    pub fn buf_write() {}
}

lazy_static! {
    pub static ref BUF_CACHE: Mutex<BufCache> = Mutex::new(BufCache::new(MAX_BUFS));
}

// Currenly useless, might remove 
pub fn setup_bcache() {
    BUF_CACHE.lock();
    println!("[KERNEL] Buffer Cache Initialized");
}
