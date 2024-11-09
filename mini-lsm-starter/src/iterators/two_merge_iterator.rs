#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

use anyhow::Result;

use super::StorageIterator;

/// Merges two iterators of different types into one. If the two iterators have the same key, only
/// produce the key once and prefer the entry from A.
pub struct TwoMergeIterator<A: StorageIterator, B: StorageIterator> {
    a: A,
    b: B,
    // Add fields as need
    flag: u8, // 1 for a, 2 for b, 0 for none
}

impl<
        A: 'static + StorageIterator,
        B: 'static + for<'a> StorageIterator<KeyType<'a> = A::KeyType<'a>>,
    > TwoMergeIterator<A, B>
{
    pub fn create(a: A, b: B) -> Result<Self> {
        let flag = if a.is_valid() && b.is_valid() {
            if a.key() <= b.key() {
                1
            } else {
                2
            }
        } else if a.is_valid() {
            1
        } else if b.is_valid() {
            2
        } else {
            0
        };
        Ok(Self { a, b, flag })
    }

    fn choose_a(&self) -> bool {
        self.flag == 1
    }
    fn choose_b(&self) -> bool {
        self.flag == 2
    }
}

impl<
        A: 'static + StorageIterator,
        B: 'static + for<'a> StorageIterator<KeyType<'a> = A::KeyType<'a>>,
    > StorageIterator for TwoMergeIterator<A, B>
{
    type KeyType<'a> = A::KeyType<'a>;

    fn key(&self) -> Self::KeyType<'_> {
        if self.choose_a() {
            self.a.key()
        } else {
            self.b.key()
        }
    }

    fn value(&self) -> &[u8] {
        if self.choose_a() {
            self.a.value()
        } else {
            self.b.value()
        }
    }

    fn is_valid(&self) -> bool {
        self.choose_a() && self.a.is_valid() || self.choose_b() && self.b.is_valid()
    }

    fn next(&mut self) -> Result<()> {
        if self.choose_a() {
            while self.b.is_valid() && self.a.key() == self.b.key() {
                self.b.next()?;
            }
            self.a.next()?;
            if !self.a.is_valid() && self.b.is_valid() {
                self.flag = 2;
            }
        } else {
            self.b.next()?;
            if !self.b.is_valid() {
                self.flag = 1;
            }
        }

        Ok(())
    }
}
