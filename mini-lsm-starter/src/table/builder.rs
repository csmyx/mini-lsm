#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

use anyhow::Result;
use bytes::BufMut;
use std::path::Path;
use std::sync::Arc;

use super::bloom::Bloom;
use super::{BlockMeta, FileObject, SsTable, SsTableIterator};
use crate::iterators::StorageIterator;
use crate::key::KeyBytes;
use crate::{block::BlockBuilder, key::KeySlice, lsm_storage::BlockCache};

/// Builds an SSTable from key-value pairs.
pub struct SsTableBuilder {
    builder: BlockBuilder,
    first_key: Vec<u8>, // the first key of current builder
    last_key: Vec<u8>,  // the last key of current builder
    data: Vec<u8>,
    pub(crate) meta: Vec<BlockMeta>,
    block_size: usize,
    key_hashes: Vec<u32>,
}

impl SsTableBuilder {
    /// Create a builder based on target block size.
    pub fn new(block_size: usize) -> Self {
        Self {
            builder: BlockBuilder::new(block_size),
            first_key: Vec::new(),
            last_key: Vec::new(),
            data: Vec::new(),
            meta: Vec::new(),
            block_size,
            key_hashes: Vec::new(),
        }
    }

    /// Adds a key-value pair to SSTable.
    ///
    /// Note: You should split a new block when the current block is full.(`std::mem::replace` may
    /// be helpful here)
    pub fn add(&mut self, key: KeySlice, value: &[u8]) {
        self.key_hashes
            .push(farmhash::fingerprint32(key.into_inner()));

        if self.first_key.is_empty() {
            self.first_key.clear();
            self.first_key.extend_from_slice(key.into_inner());
        }

        if self.builder.add(key, value) {
            self.last_key.clear();
            self.last_key.extend_from_slice(key.into_inner());
            return;
        }

        // The block is full, so we need to archivie it first and add to new block
        self.archive_block();

        assert!(self.builder.add(key, value));
        self.first_key.clear();
        self.first_key.extend_from_slice(key.into_inner());
        self.last_key.clear();
        self.last_key.extend_from_slice(key.into_inner());
    }

    fn archive_block(&mut self) {
        let builder = std::mem::replace(&mut self.builder, BlockBuilder::new(self.block_size));
        let block_encode = builder.build().encode();
        self.meta.push(BlockMeta {
            offset: self.data.len(),
            first_key: KeyBytes::from_bytes(std::mem::take(&mut self.first_key).into()),
            last_key: KeyBytes::from_bytes(std::mem::take(&mut self.last_key).into()),
        });
        self.data.extend_from_slice(&block_encode);
    }

    /// Get the estimated size of the SSTable.
    ///
    /// Since the data blocks contain much more data than meta blocks, just return the size of data
    /// blocks here.
    pub fn estimated_size(&self) -> usize {
        unimplemented!()
    }

    /// Builds the SSTable and writes it to the given path. Use the `FileObject` structure to manipulate the disk objects.
    pub fn build(
        mut self,
        id: usize,
        block_cache: Option<Arc<BlockCache>>,
        path: impl AsRef<Path>,
    ) -> Result<SsTable> {
        // Need to archive the last block, it may be an empty block
        self.archive_block();

        // Encoding the SSTable
        let mut buf = self.data;
        let block_meta_offset = buf.len();
        BlockMeta::encode_block_meta(&self.meta, &mut buf);
        buf.put_u32(block_meta_offset as u32);

        let bits_per_key = Bloom::bloom_bits_per_key(self.key_hashes.len(), 0.01);
        let bloom = Bloom::build_from_key_hashes(&self.key_hashes, bits_per_key);

        let bloom_offset = buf.len();
        bloom.encode(&mut buf);
        buf.put_u32(bloom_offset as u32);

        Ok(SsTable {
            file: FileObject::create(path.as_ref(), buf)?,
            first_key: self.meta.first().unwrap().first_key.clone(),
            last_key: self.meta.last().unwrap().last_key.clone(),
            block_meta: self.meta,
            block_meta_offset,
            id,
            block_cache,
            bloom: Some(bloom),
            max_ts: 0,
        })
    }

    #[cfg(test)]
    pub(crate) fn build_for_test(self, path: impl AsRef<Path>) -> Result<SsTable> {
        self.build(0, None, path)
    }
}
