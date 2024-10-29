#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

use bytes::BufMut;

use crate::key::{KeySlice, KeyVec};

use super::Block;

/// Builds a block.
pub struct BlockBuilder {
    /// Offsets of each key-value entries.
    offsets: Vec<u16>,
    /// All serialized key-value pairs in the block.
    data: Vec<u8>,
    /// The expected block size.
    block_size: usize,
    /// The first key in the block
    first_key: KeyVec,
}

impl BlockBuilder {
    /// Creates a new block builder.
    pub fn new(block_size: usize) -> Self {
        Self {
            offsets: vec![],
            data: vec![],
            block_size,
            first_key: KeyVec::new(),
        }
    }

    /// Adds a key-value pair to the block. Returns false when the block is full.
    #[must_use]
    pub fn add(&mut self, key: KeySlice, value: &[u8]) -> bool {
        if self.estimated_size() + Self::added_size(&key, value) > self.block_size && !self.is_empty() {
            return false;
        }

        // Try to set first_key
        if self.first_key.is_empty() {
            self.first_key = key.to_key_vec();
        }

        // Put offset
        self.offsets.push(self.data.len() as u16);

        // Put entry(key-value pair)
        self.data.put_u16(key.len() as u16);
        self.data.put(key.into_inner());
        self.data.put_u16(value.len() as u16);
        self.data.put(value);
        return true;
    }

    /// Check if there is no key-value pair in the block.
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// Finalize the block.
    pub fn build(self) -> Block {
        Block {
            data: self.data,
            offsets: self.offsets,
        }
    }

    fn estimated_size(&self) -> usize {
        return self.data.len() * size_of::<u8>()        /* for data */ 
                + self.offsets.len() * size_of::<u16>() /* for offsets */
                + size_of::<u16>();                     /* for number of elements */
    }

    fn added_size(key: &KeySlice, value: &[u8]) -> usize {
        key.len() + value.len() + size_of::<u16>() * 3  /* for key_len value_len and offset */
    }
}
