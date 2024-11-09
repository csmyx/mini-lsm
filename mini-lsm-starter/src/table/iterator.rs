#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

use std::cmp::min;
use std::sync::Arc;

use anyhow::Result;

use super::SsTable;
use crate::{block::BlockIterator, iterators::StorageIterator, key::KeySlice};

/// An iterator over the contents of an SSTable.
pub struct SsTableIterator {
    table: Arc<SsTable>,
    blk_iter: BlockIterator,
    blk_idx: usize,
}

impl SsTableIterator {
    /// Create a new iterator and seek to the first key-value pair in the first data block.
    pub fn create_and_seek_to_first(table: Arc<SsTable>) -> Result<Self> {
        let blk = table.read_block(0)?;
        let blk_iter = BlockIterator::create_and_seek_to_first(blk);
        Ok(Self {
            table,
            blk_iter,
            blk_idx: 0,
        })
    }

    /// Seek to the first key-value pair in the first data block.
    pub fn seek_to_first(&mut self) -> Result<()> {
        self.blk_idx = 0;
        let blk = self.table.read_block(0)?;
        self.blk_iter = BlockIterator::create_and_seek_to_first(blk);
        Ok(())
    }

    /// Create a new iterator and seek to the first key-value pair which >= `key`.
    pub fn create_and_seek_to_key(table: Arc<SsTable>, key: KeySlice) -> Result<Self> {
        // Try seek to the last block if the key is larger than the first key of all blocks
        let blk_idx = min(
            Self::get_idx_gt(table.clone(), key),
            table.num_of_blocks() - 1,
        );
        let blk = table.read_block(blk_idx)?;
        let blk_iter = BlockIterator::create_and_seek_to_key(blk, key);
        Ok(Self {
            table,
            blk_iter,
            blk_idx: 0,
        })
    }

    /// Seek to the first key-value pair which >= `key`.
    /// Note: You probably want to review the handout for detailed explanation when implementing
    /// this function.
    pub fn seek_to_key(&mut self, key: KeySlice) -> Result<()> {
        let mut blk_idx = Self::get_idx_gt(self.table.clone(), key);
        // Try to seek in block whose next block's first key > 'key'
        if blk_idx != 0 {
            blk_idx -= 1;
        }
        debug_assert!(blk_idx < self.table.num_of_blocks());
        // Try to move to next block if the last key of current block < 'key'
        let last_key = self.table.block_meta[blk_idx].last_key.as_key_slice();
        if last_key < key && blk_idx + 1 < self.table.num_of_blocks() {
            blk_idx += 1;
        }
        self.blk_idx = blk_idx;
        let blk = self.table.read_block(self.blk_idx)?;
        self.blk_iter = BlockIterator::create_and_seek_to_key(blk, key);
        Ok(())
    }

    /// Get the index of the first block who's key > 'key'
    fn get_idx_gt(table: Arc<SsTable>, key: KeySlice) -> usize {
        let mut left = 0;
        let mut right = table.num_of_blocks();
        while left < right {
            let mid = left + (right - left) / 2;
            let first_key = table.block_meta[mid].first_key.as_key_slice();
            if first_key >= key {
                right = mid;
            } else {
                left = mid + 1;
            }
        }
        // Notice: left will equal to num_of_blocks when all first keys of blocks <= 'key'
        left
    }
}

impl StorageIterator for SsTableIterator {
    type KeyType<'a> = KeySlice<'a>;

    /// Return the `key` that's held by the underlying block iterator.
    fn key(&self) -> KeySlice {
        self.blk_iter.key()
    }

    /// Return the `value` that's held by the underlying block iterator.
    fn value(&self) -> &[u8] {
        self.blk_iter.value()
    }

    /// Return whether the current block iterator is valid or not.
    fn is_valid(&self) -> bool {
        self.blk_iter.is_valid()
    }

    /// Move to the next `key` in the block.
    /// Note: You may want to check if the current block iterator is valid after the move.
    fn next(&mut self) -> Result<()> {
        self.blk_iter.next();
        if !self.blk_iter.is_valid() {
            self.blk_idx += 1;
            if self.blk_idx < self.table.num_of_blocks() {
                let blk = self.table.read_block(self.blk_idx)?;
                self.blk_iter = BlockIterator::create_and_seek_to_first(blk);
            }
        }
        Ok(())
    }
}
