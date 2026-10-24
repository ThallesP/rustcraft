//! com/mojang/blaze3d/vertex/TlsfAllocator.java
//!
//! A two-level segregated fit allocator over one heap: free blocks in lists by size class, found
//! through two levels of bitmaps, split when they're bigger than the size class asked for and
//! merged with their free neighbours when freed. Java's blocks are objects linked to each other;
//! here they're slots of the allocator's arena, linked by index, a slot that a merge left unused
//! being handed out again. The heap is its size: an allocation's heap is whichever the caller made
//! the allocator for. A freed allocation is consumed, which Java's `freed` flag guards against
//! freeing twice. `printAllocatorStatistics`, a debug log, isn't ported.

use crate::net::minecraft::util::mth::Mth;

pub struct TlsfAllocator {
    first_level_bitmap: i32,
    second_level_bitmap: [i32; 32],
    free_lists: [Option<usize>; 256],
    total_memory_size: i64,
    blocks: Vec<Block>,
    unused_blocks: Vec<usize>,
}

/// `TlsfAllocator.Allocation`: where in the heap the allocation starts, after its block's start
/// aligned up.
#[derive(Debug)]
pub struct Allocation {
    block: usize,
    offset_from_heap: i64,
}

impl Allocation {
    pub fn get_offset_from_heap(&self) -> i64 {
        self.offset_from_heap
    }
}

struct Block {
    /// The size, with the free flag in its lowest bit.
    size: i64,
    offset_from_heap: i64,
    next_free_block: Option<usize>,
    previous_free_block: Option<usize>,
    next_physical_block: Option<usize>,
    previous_physical_block: Option<usize>,
}

impl Block {
    const BLOCK_HEADER_FREE_BIT: i64 = 1;

    fn is_free(&self) -> bool {
        self.size & Block::BLOCK_HEADER_FREE_BIT == 1
    }

    fn set_free(&mut self) {
        self.size |= Block::BLOCK_HEADER_FREE_BIT;
    }

    fn set_used(&mut self) {
        self.size &= -2;
    }

    fn get_size(&self) -> i64 {
        self.size & -2
    }

    fn set_size(&mut self, size: i64) {
        let old_size = self.size;
        self.size = size | old_size & 1;
    }
}

/// `TlsfAllocator.IndexPair`.
#[derive(Clone, Copy)]
struct IndexPair {
    first_level_index: i32,
    second_level_index: i32,
}

impl TlsfAllocator {
    const SECOND_LEVEL_BIN_COUNT: i32 = 8;
    const SMALL_BLOCK_SIZE: i64 = 256;
    const ALIGN_SIZE: i64 = 32;

    pub fn new(heap_size: i64) -> TlsfAllocator {
        let mut allocator = TlsfAllocator {
            first_level_bitmap: 0,
            second_level_bitmap: [0; 32],
            free_lists: [None; 256],
            total_memory_size: 0,
            blocks: Vec::new(),
            unused_blocks: Vec::new(),
        };
        let aligned_heap_size = TlsfAllocator::align_down(heap_size, TlsfAllocator::ALIGN_SIZE);
        let free_block = allocator.new_block(aligned_heap_size, 0, None, None);
        allocator.blocks[free_block].set_free();
        allocator.insert_free_block(free_block);
        let remaining_heap_size = heap_size - aligned_heap_size;
        let sentinel_block = allocator.new_block(
            remaining_heap_size,
            aligned_heap_size,
            None,
            Some(free_block),
        );
        allocator.blocks[sentinel_block].set_used();
        allocator.blocks[free_block].next_physical_block = Some(sentinel_block);
        allocator.total_memory_size = aligned_heap_size;
        allocator
    }

    /// `new Block(size, heap, offsetFromHeap, null, null, next, previous)`, in a slot of the arena.
    fn new_block(
        &mut self,
        size: i64,
        offset_from_heap: i64,
        next_physical_block: Option<usize>,
        previous_physical_block: Option<usize>,
    ) -> usize {
        let block = Block {
            size,
            offset_from_heap,
            next_free_block: None,
            previous_free_block: None,
            next_physical_block,
            previous_physical_block,
        };
        match self.unused_blocks.pop() {
            Some(index) => {
                self.blocks[index] = block;
                index
            }
            None => {
                self.blocks.push(block);
                self.blocks.len() - 1
            }
        }
    }

    fn get_block_from_free_list(
        &self,
        first_level_index: i32,
        second_level_index: i32,
    ) -> Option<usize> {
        self.free_lists[(first_level_index * 8 + second_level_index) as usize]
    }

    fn set_block_free_list(
        &mut self,
        first_level_index: i32,
        second_level_index: i32,
        block: Option<usize>,
    ) {
        self.free_lists[(first_level_index * 8 + second_level_index) as usize] = block;
    }

    fn align_up(x: i64, align: i64) -> i64 {
        x.wrapping_add(align - 1) & !(align - 1)
    }

    fn align_down(x: i64, align: i64) -> i64 {
        x - (x & (align - 1))
    }

    fn find_last_significant_bit(x: i64) -> i32 {
        63 - x.leading_zeros() as i32
    }

    fn find_first_significant_bit(x: i32) -> i32 {
        x.trailing_zeros() as i32
    }

    fn get_level_index(size: i64) -> IndexPair {
        if (size as u64) < TlsfAllocator::SMALL_BLOCK_SIZE as u64 {
            IndexPair {
                first_level_index: 0,
                second_level_index: (size as u64 / TlsfAllocator::ALIGN_SIZE as u64) as i32,
            }
        } else {
            let first_level_index = TlsfAllocator::find_last_significant_bit(size);
            let second_level_index = (((size as u64) >> (first_level_index - 3)) as i32)
                ^ TlsfAllocator::SECOND_LEVEL_BIN_COUNT;
            IndexPair {
                first_level_index: first_level_index - 7,
                second_level_index,
            }
        }
    }

    fn mapping_search(size: i64) -> IndexPair {
        let mut rounded_size = size;
        if size as u64 >= TlsfAllocator::SMALL_BLOCK_SIZE as u64 {
            let round = (1i64 << (TlsfAllocator::find_last_significant_bit(size) - 3)) - 1;
            rounded_size = rounded_size.wrapping_add(round);
        }

        TlsfAllocator::get_level_index(rounded_size)
    }

    fn insert_free_block(&mut self, block: usize) {
        let level_index = TlsfAllocator::get_level_index(self.blocks[block].get_size());
        let current_block = self.get_block_from_free_list(
            level_index.first_level_index,
            level_index.second_level_index,
        );
        if let Some(current_block) = current_block {
            self.blocks[current_block].previous_free_block = Some(block);
        }

        self.blocks[block].next_free_block = current_block;
        self.first_level_bitmap |= 1i32.wrapping_shl(level_index.first_level_index as u32);
        self.second_level_bitmap[level_index.first_level_index as usize] |=
            1i32.wrapping_shl(level_index.second_level_index as u32);
        self.set_block_free_list(
            level_index.first_level_index,
            level_index.second_level_index,
            Some(block),
        );
    }

    fn remove_free_block(&mut self, block: usize, first_level: i32, second_level: i32) {
        let next = self.blocks[block].next_free_block;
        let previous = self.blocks[block].previous_free_block;
        if let Some(previous) = previous {
            self.blocks[previous].next_free_block = next;
        }

        if let Some(next) = next {
            self.blocks[next].previous_free_block = previous;
        }

        if self.get_block_from_free_list(first_level, second_level) == Some(block) {
            self.set_block_free_list(first_level, second_level, next);
            if next.is_none() {
                self.second_level_bitmap[first_level as usize] &=
                    !1i32.wrapping_shl(second_level as u32);
                if self.second_level_bitmap[first_level as usize] == 0 {
                    self.first_level_bitmap &= !1i32.wrapping_shl(first_level as u32);
                }
            }
        }
    }

    fn trim_block(&mut self, block: usize, size: i64) {
        if (self.blocks[block].get_size() as u64) > (size + TlsfAllocator::SMALL_BLOCK_SIZE) as u64
        {
            let remaining = self.blocks[block].get_size() - size;
            let next = self.blocks[block].next_physical_block;
            let remaining_block = self.new_block(
                remaining,
                self.blocks[block].offset_from_heap + size,
                next,
                Some(block),
            );
            self.blocks[remaining_block].set_free();
            if let Some(next) = next {
                debug_assert_eq!(self.blocks[next].previous_physical_block, Some(block));
                self.blocks[next].previous_physical_block = Some(remaining_block);
            }

            self.blocks[block].next_physical_block = Some(remaining_block);
            self.blocks[block].set_size(size);
            self.merge_block_with_next(remaining_block);
            self.insert_free_block(remaining_block);
        }
    }

    /// `allocate`: `size` bytes at an offset that's a multiple of `align`, or none when no free
    /// block is big enough.
    pub fn allocate(&mut self, size: i64, align: i32) -> Option<Allocation> {
        let is_power_of_two = Mth::is_power_of_two_i32(align);
        let size_padding = if is_power_of_two && align <= 32 {
            0
        } else {
            i64::from(align)
        };
        let aligned_size = TlsfAllocator::align_up(size + size_padding, TlsfAllocator::ALIGN_SIZE);
        debug_assert!(aligned_size <= 549755813888);
        let level_index = TlsfAllocator::mapping_search(aligned_size);
        let mut first_level_index = level_index.first_level_index;
        let mut second_level_index = level_index.second_level_index;
        let first_level_size = 1i64 << (first_level_index + 8 - 1);
        let second_level_interval = first_level_size / 8;
        let slot_size = first_level_size + i64::from(second_level_index) * second_level_interval;
        if first_level_index >= 32 {
            return None;
        }

        let mut sl_bitmap = self.second_level_bitmap[first_level_index as usize]
            & (-1i32).wrapping_shl(second_level_index as u32);
        if sl_bitmap == 0 {
            let fl_bitmap =
                self.first_level_bitmap & (-1i32).wrapping_shl((first_level_index + 1) as u32);
            if fl_bitmap == 0 {
                return None;
            }

            first_level_index = TlsfAllocator::find_first_significant_bit(fl_bitmap);
            sl_bitmap = self.second_level_bitmap[first_level_index as usize];
        }

        second_level_index = TlsfAllocator::find_first_significant_bit(sl_bitmap);
        let block = self
            .get_block_from_free_list(first_level_index, second_level_index)
            .expect("a set bit has a free list");
        debug_assert!(self.blocks[block].get_size() >= aligned_size);
        self.remove_free_block(block, first_level_index, second_level_index);
        self.trim_block(block, slot_size);
        self.blocks[block].set_used();
        let offset_from_heap = self.blocks[block].offset_from_heap;
        let gap = (offset_from_heap as u64 % align as u64) as i64;
        let alignment_offset = if gap == 0 { 0 } else { i64::from(align) - gap };
        Some(Allocation {
            block,
            offset_from_heap: offset_from_heap + alignment_offset,
        })
    }

    fn merge_block_with_previous(&mut self, block: usize) {
        let Some(previous) = self.blocks[block].previous_physical_block else {
            return;
        };
        if !self.blocks[previous].is_free() {
            return;
        }
        debug_assert_eq!(
            self.blocks[previous].offset_from_heap + self.blocks[previous].get_size(),
            self.blocks[block].offset_from_heap
        );
        let level_index = TlsfAllocator::get_level_index(self.blocks[previous].get_size());
        self.remove_free_block(
            previous,
            level_index.first_level_index,
            level_index.second_level_index,
        );
        let prevprev = self.blocks[previous].previous_physical_block;
        if let Some(prevprev) = prevprev {
            self.blocks[prevprev].next_physical_block = Some(block);
        }

        self.blocks[block].previous_physical_block = prevprev;
        let size = self.blocks[block].get_size() + self.blocks[previous].get_size();
        self.blocks[block].set_size(size);
        self.blocks[block].offset_from_heap = self.blocks[previous].offset_from_heap;
        self.unused_blocks.push(previous);
    }

    fn merge_block_with_next(&mut self, block: usize) {
        let Some(next) = self.blocks[block].next_physical_block else {
            return;
        };
        if !self.blocks[next].is_free() {
            return;
        }
        debug_assert_eq!(
            self.blocks[next].offset_from_heap - self.blocks[block].get_size(),
            self.blocks[block].offset_from_heap
        );
        let level_index = TlsfAllocator::get_level_index(self.blocks[next].get_size());
        self.remove_free_block(
            next,
            level_index.first_level_index,
            level_index.second_level_index,
        );
        let nextnext = self.blocks[next].next_physical_block;
        if let Some(nextnext) = nextnext {
            self.blocks[nextnext].previous_physical_block = Some(block);
        }

        self.blocks[block].next_physical_block = nextnext;
        let size = self.blocks[block].get_size() + self.blocks[next].get_size();
        self.blocks[block].set_size(size);
        self.unused_blocks.push(next);
    }

    pub fn free(&mut self, allocation: Allocation) {
        let block = allocation.block;
        self.blocks[block].set_free();
        self.merge_block_with_previous(block);
        self.merge_block_with_next(block);
        self.insert_free_block(block);
    }

    pub fn is_completely_free(&self) -> bool {
        if self.first_level_bitmap.count_ones() == 1 {
            let first_level_index =
                TlsfAllocator::find_first_significant_bit(self.first_level_bitmap);
            let sl_bitmap = self.second_level_bitmap[first_level_index as usize];
            if sl_bitmap.count_ones() == 1 {
                let second_level_index = TlsfAllocator::find_first_significant_bit(sl_bitmap);
                return self
                    .get_block_from_free_list(first_level_index, second_level_index)
                    .is_some_and(|free_block| {
                        self.blocks[free_block].get_size() == self.total_memory_size
                    });
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::TlsfAllocator;
    use crate::net::minecraft::util::random_source::RandomSource;
    use crate::net::minecraft::world::level::levelgen::legacy_random_source::LegacyRandomSource;

    /// Random allocations and frees of a heap's size: every live allocation lies inside the heap,
    /// aligned, without overlapping another, and freeing them all leaves the heap free again.
    #[test]
    fn allocates_without_overlapping_and_frees_back_to_one_block() {
        const HEAP_SIZE: i64 = 1 << 20;
        let mut random = LegacyRandomSource::new(42);
        for align in [8, 28, 32, 36] {
            let mut allocator = TlsfAllocator::new(HEAP_SIZE);
            assert!(allocator.is_completely_free());
            let mut live: Vec<(i64, i64, super::Allocation)> = Vec::new();
            for _ in 0..5000 {
                if !live.is_empty() && random.next_int_bounded(3) == 0 {
                    let index = random.next_int_bounded(live.len() as i32) as usize;
                    let (_, _, allocation) = live.swap_remove(index);
                    allocator.free(allocation);
                    continue;
                }
                let size = i64::from(align) * i64::from(1 + random.next_int_bounded(2000));
                let Some(allocation) = allocator.allocate(size, align) else {
                    continue;
                };
                let offset = allocation.get_offset_from_heap();
                assert_eq!(offset % i64::from(align), 0);
                assert!(offset >= 0 && offset + size <= HEAP_SIZE);
                for (other_offset, other_size, _) in &live {
                    assert!(offset + size <= *other_offset || other_offset + other_size <= offset);
                }
                live.push((offset, size, allocation));
            }
            assert!(!allocator.is_completely_free());
            for (_, _, allocation) in live {
                allocator.free(allocation);
            }
            assert!(allocator.is_completely_free());
        }
    }
}
