#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

mod builder;
mod iterator;

pub use builder::BlockBuilder;
use bytes::{Buf, BufMut, Bytes};
pub use iterator::BlockIterator;

/// A block is the smallest unit of read and caching in LSM tree. It is a collection of sorted key-value pairs.
pub struct Block {
    pub(crate) data: Vec<u8>,
    pub(crate) offsets: Vec<u16>,
}

impl Block {
    /// Encode the internal data to the data layout illustrated in the tutorial
    /// Note: You may want to recheck if any of the expected field is missing from your output
    pub fn encode(&self) -> Bytes {
        let mut buf = self.data.clone();
        for offset in &self.offsets {
            buf.put_u16(*offset);
        }
        buf.put_u16(self.offsets.len() as u16);
        buf.into()
    }

    /// Decode from the data layout, transform the input `data` to a single `Block`
    pub fn decode(data: &[u8]) -> Self {
        // Decode the number of elements
        let n_idx = data.len() - size_of::<u16>();
        let num = (&data[n_idx..]).get_u16() as usize;

        // Decode offsets
        let o_idx = data.len() - size_of::<u16>() * (1 + num);
        let offsets = &data[o_idx..n_idx];
        let offsets = offsets
            .chunks(size_of::<u16>())
            .map(|mut x| x.get_u16())
            .collect();

        // Decode data
        let data = data[..o_idx].to_vec();

        Self { data, offsets }
    }
}
