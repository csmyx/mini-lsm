#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

use crate::key::{KeySlice, KeyVec};
use bytes::Buf;
use std::sync::Arc;

use super::Block;

/// Iterates on a block.
pub struct BlockIterator {
    /// The internal `Block`, wrapped by an `Arc`
    block: Arc<Block>,
    /// The current key, empty represents the iterator is invalid
    key: KeyVec,
    /// the current value range in the block.data, corresponds to the current key
    value_range: (usize, usize),
    /// Current index of the key-value pair, should be in range of [0, num_of_elements)
    idx: usize,
    /// The first key in the block
    first_key: KeyVec,
}

impl BlockIterator {
    fn new(block: Arc<Block>) -> Self {
        Self {
            block,
            key: KeyVec::new(),
            value_range: (0, 0),
            idx: 0,
            first_key: KeyVec::new(),
        }
    }

    /// Creates a block iterator and seek to the first entry.
    pub fn create_and_seek_to_first(block: Arc<Block>) -> Self {
        let mut iter = Self::new(block.clone());
        iter.set_fisrt_key();
        iter.set_key_value_at(0);
        iter
    }

    fn parse_key_value(data: &[u8], offset: usize) -> (&[u8], (usize, usize)) {
        let mut beg = offset;
        let mut end = beg + size_of::<u16>();
        debug_assert!(end <= data.len());
        let key_len = (&data[beg..end]).get_u16() as usize;

        beg = end;
        end = beg + key_len;
        debug_assert!(end <= data.len());
        let key = &data[beg..end];

        beg = end;
        end = beg + size_of::<u16>();
        debug_assert!(end <= data.len());
        let value_len = (&data[beg..end]).get_u16() as usize;

        beg = end;
        end = beg + value_len;
        debug_assert!(end <= data.len());
        // let value = &data[beg..end];

        (key, (beg, end))
    }

    fn set_fisrt_key(&mut self) {
        self.first_key
            .set_from_ref(Self::parse_key_value(&self.block.data, 0).0);
    }

    fn set_key_value_at(&mut self, idx: usize) {
        // set key to empty to indicate having moved to an invalid position
        if idx >= self.block.offsets.len() {
            self.key.clear();
            self.value_range = (0, 0);
            return;
        }
        self.idx = idx;
        let offset = self.block.offsets[idx] as usize;
        let key_slice;
        let value_range;
        (key_slice, value_range) = Self::parse_key_value(&self.block.data, offset);
        self.key.set_from_slice(KeySlice::from_slice(key_slice));
        self.value_range = value_range;
    }

    /// Creates a block iterator and seek to the first key that >= `key`.
    pub fn create_and_seek_to_key(block: Arc<Block>, key: KeySlice) -> Self {
        unimplemented!()
    }

    /// Returns the key of the current entry.
    pub fn key(&self) -> KeySlice {
        self.key.as_key_slice()
    }

    /// Returns the value of the current entry.
    pub fn value(&self) -> &[u8] {
        let (beg, end) = self.value_range;
        &self.block.data[beg..end]
    }

    /// Returns true if the iterator is valid.
    /// Note: You may want to make use of `key`
    pub fn is_valid(&self) -> bool {
        !self.key.is_empty()
    }

    /// Seeks to the first key in the block.
    pub fn seek_to_first(&mut self) {
        self.set_key_value_at(0);
    }

    /// Move to the next key in the block.
    pub fn next(&mut self) {
        self.set_key_value_at(self.idx + 1);
    }

    /// Seek to the first key that >= `key`.
    /// Note: You should assume the key-value pairs in the block are sorted when being added by
    /// callers.
    pub fn seek_to_key(&mut self, key: KeySlice) {}
}
