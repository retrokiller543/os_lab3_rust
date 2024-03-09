#![allow(clippy::upper_case_acronyms)]
#![allow(unused_variables)]

use std::fmt::Debug;
use std::ops::{Index, IndexMut};

use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "py-bindings")]
use pyo3::prelude::*;

use logger_macro::trace_log;
use rustic_disk::Disk;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
//#[cfg_attr(feature = "py-bindings", pyclass)]
pub enum FatType {
    Free,
    Taken(u16),
    EOF,
}

#[derive(Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "py-bindings", pyclass)]
pub struct FAT(
    //#[serde(with = "BigArray")]
    Vec<FatType>,
);

impl Debug for FAT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // get number of free blocks
        let num_free = self.0.iter().filter(|&x| *x == FatType::Free).count();
        // get number of EOF blocks
        let num_eof = self.0.iter().filter(|&x| *x == FatType::EOF).count();
        // get number of taken blocks
        let num_taken = self
            .0
            .iter()
            .filter(|&x| matches!(x, FatType::Taken(_)))
            .count();
        // get number of blocks
        let num_blocks = self.0.len();
        write!(
            f,
            "FAT{{Free: {}, Taken: {}, EOF: {}, Total: {}}}",
            num_free, num_taken, num_eof, num_blocks
        )
    }
}

impl FAT {
    #[trace_log]
    pub fn new() -> Self {
        let mut fat = vec![FatType::Free; (Disk::BLOCK_SIZE >> 2) - 8]; // 8 bytes is from padding in FAT struct
        fat.fill(FatType::Free);
        FAT(fat)
    }

    // Create an iterator
    #[trace_log]
    pub fn iter(&self) -> FatIterator {
        FatIterator {
            fat: self,
            position: 0,
        }
    }

    #[trace_log]
    pub fn get(&self, index: usize) -> Option<&FatType> {
        self.0.get(index)
    }
}

impl Default for FAT {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for FAT {
    type Output = FatType;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for FAT {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

// Define the iterator struct
pub struct FatIterator<'a> {
    fat: &'a FAT,
    position: usize,
}

// Implement the Iterator trait for FatIterator
impl<'a> Iterator for FatIterator<'a> {
    type Item = &'a FatType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= Disk::NUM_BLOCKS {
            None
        } else {
            let result = &self.fat.0[self.position];
            self.position += 1;
            Some(result)
        }
    }
}
