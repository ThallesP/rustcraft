//! net/minecraft/client/gui/font/CodepointMap.java
//!
//! A map from codepoints, in blocks of 256 made when first written. `keySet` is a fastutil
//! `IntOpenHashSet` in Java, which only `FontSet`'s lists of glyphs by width read the order of;
//! here it's the codepoints in order.

const BLOCK_SIZE: usize = 256;
const BLOCK_COUNT: usize = 4352;

#[derive(Clone)]
pub struct CodepointMap<T> {
    block_map: Vec<Option<Box<[Option<T>]>>>,
}

impl<T: Clone> Default for CodepointMap<T> {
    fn default() -> CodepointMap<T> {
        CodepointMap::new()
    }
}

impl<T: Clone> CodepointMap<T> {
    pub fn new() -> CodepointMap<T> {
        CodepointMap {
            block_map: vec![None; BLOCK_COUNT],
        }
    }

    pub fn clear(&mut self) {
        self.block_map.iter_mut().for_each(|block| *block = None);
    }

    /// Java's arrays throw `ArrayIndexOutOfBoundsException` past U+10FFFF, the panic.
    pub fn get(&self, codepoint: i32) -> Option<&T> {
        let block = (codepoint >> 8) as usize;
        let offset = (codepoint & 0xFF) as usize;
        self.block_map[block].as_ref()?[offset].as_ref()
    }

    pub fn put(&mut self, codepoint: i32, value: T) -> Option<T> {
        let block = (codepoint >> 8) as usize;
        let offset = (codepoint & 0xFF) as usize;
        let block_data =
            self.block_map[block].get_or_insert_with(|| vec![None; BLOCK_SIZE].into_boxed_slice());
        block_data[offset].replace(value)
    }

    pub fn compute_if_absent(&mut self, codepoint: i32, mapper: impl FnOnce(i32) -> T) -> &T {
        let block = (codepoint >> 8) as usize;
        let offset = (codepoint & 0xFF) as usize;
        let block_data =
            self.block_map[block].get_or_insert_with(|| vec![None; BLOCK_SIZE].into_boxed_slice());
        block_data[offset].get_or_insert_with(|| mapper(codepoint))
    }

    pub fn remove(&mut self, codepoint: i32) -> Option<T> {
        let block = (codepoint >> 8) as usize;
        let offset = (codepoint & 0xFF) as usize;
        self.block_map[block].as_mut()?[offset].take()
    }

    /// `forEach`, in codepoint order.
    pub fn for_each(&self, mut output: impl FnMut(i32, &T)) {
        for (block, block_data) in self.block_map.iter().enumerate() {
            if let Some(block_data) = block_data {
                for (offset, value) in block_data.iter().enumerate() {
                    if let Some(value) = value {
                        output((block << 8 | offset) as i32, value);
                    }
                }
            }
        }
    }

    pub fn key_set(&self) -> Vec<i32> {
        let mut result = Vec::new();
        self.for_each(|codepoint, _| result.push(codepoint));
        result
    }
}
