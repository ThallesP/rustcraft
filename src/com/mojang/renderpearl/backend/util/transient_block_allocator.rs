//! com/mojang/renderpearl/backend/util/TransientBlockAllocator.java
//!
//! Memory for one submit's data, carved out of blocks of a fixed size: each allocation goes after
//! the last in the current block, or in a new one, or in a block of its own when it's larger; a
//! rotation hands back what the submit used, to reuse once the GPU is done with it. Java's
//! `rotate` returns a `Runnable` doing that, which the encoder runs later; here it's the
//! `Rotation` it took, which `finish_rotation` gives back.

use crate::net::minecraft::util::mth::Mth;

/// `TransientBlockAllocator.Allocator.Block`.
pub trait Block: Clone {
    /// A block in a heap the allocator didn't want, which is freed rather than reused.
    fn suboptimal(&self) -> bool;
}

/// `TransientBlockAllocator.Allocator`.
pub trait Allocator<T> {
    fn alloc(&mut self, size: i64) -> T;

    fn free(&mut self, t: T);
}

/// `TransientBlockAllocator.Allocation`.
#[derive(Clone, Debug)]
pub struct Allocation<T> {
    pub block: T,
    pub offset: i64,
    pub size: i64,
}

/// What a rotation took from the allocator, to give back once the GPU is done with it.
pub struct Rotation<T> {
    blocks_used_this_rotation: Vec<T>,
    special_blocks_used_this_rotation: Vec<T>,
}

pub struct TransientBlockAllocator<T: Block> {
    block_size: i64,
    max_alignment: i64,
    allocator: Box<dyn Allocator<T>>,
    on_block_use: Box<dyn FnMut(&T)>,
    special_blocks: Vec<T>,
    free_blocks: Vec<T>,
    used_blocks: Vec<T>,
    current_block: Option<T>,
    current_offset: i64,
}

impl<T: Block> TransientBlockAllocator<T> {
    pub fn new(block_size: i64, max_alignment: i64, allocator: Box<dyn Allocator<T>>) -> Self {
        TransientBlockAllocator::with_on_block_use(
            block_size,
            max_alignment,
            allocator,
            Box::new(|_| {}),
        )
    }

    pub fn with_on_block_use(
        block_size: i64,
        max_alignment: i64,
        allocator: Box<dyn Allocator<T>>,
        on_block_use: Box<dyn FnMut(&T)>,
    ) -> Self {
        TransientBlockAllocator {
            block_size,
            max_alignment,
            allocator,
            on_block_use,
            special_blocks: Vec::new(),
            free_blocks: Vec::new(),
            used_blocks: Vec::new(),
            current_block: None,
            current_offset: 0,
        }
    }

    /// `close()`: two rotations, each given back at once.
    pub fn close(&mut self) {
        for _ in 0..2 {
            let rotation = self.rotate();
            self.finish_rotation(rotation);
        }
    }

    pub fn block_size(&self) -> i64 {
        self.block_size
    }

    /// `rotate()`: the blocks freed after the rotation before go back to the allocator, and the
    /// blocks this submit used are taken.
    pub fn rotate(&mut self) -> Rotation<T> {
        self.current_block = None;
        self.current_offset = self.block_size;
        for block in self.free_blocks.drain(..) {
            self.allocator.free(block);
        }
        Rotation {
            blocks_used_this_rotation: std::mem::take(&mut self.used_blocks),
            special_blocks_used_this_rotation: std::mem::take(&mut self.special_blocks),
        }
    }

    /// The rotation's `Runnable`: its blocks free to use again, the suboptimal and special ones
    /// freed.
    pub fn finish_rotation(&mut self, rotation: Rotation<T>) {
        for used_block in rotation.blocks_used_this_rotation {
            if used_block.suboptimal() {
                self.allocator.free(used_block);
            } else {
                self.free_blocks.push(used_block);
            }
        }
        for special_block in rotation.special_blocks_used_this_rotation {
            self.allocator.free(special_block);
        }
    }

    pub fn can_allocate_in_block(&self, size: i64, alignment: i64) -> bool {
        size <= self.block_size && alignment <= self.max_alignment
    }

    pub fn can_allocate_in_current_block(&self, size: i64, alignment: i64) -> bool {
        if self.current_block.is_none() && self.can_allocate_in_block(size, alignment) {
            return true;
        }
        let aligned_offset = Mth::round_toward_i64(self.current_offset, alignment);
        size <= self.block_size - aligned_offset && alignment <= self.max_alignment
    }

    fn allocate_block(&mut self) -> T {
        if self.free_blocks.is_empty() {
            let block = self.allocator.alloc(self.block_size);
            self.free_blocks.push(block);
        }
        let block = self.free_blocks.pop().expect("a free block");
        (self.on_block_use)(&block);
        self.used_blocks.push(block.clone());
        block
    }

    /// Panics where Java throws `IllegalArgumentException`, for an alignment over the maximum.
    pub fn allocate(
        &mut self,
        size: i64,
        alignment: i64,
        minimum_allocation: i64,
        element_size: i64,
    ) -> Allocation<T> {
        assert!(
            alignment <= self.max_alignment,
            "Alignment requirement over maximum supported alignment"
        );
        if size == self.block_size {
            return Allocation {
                block: self.allocate_block(),
                offset: 0,
                size: self.block_size,
            };
        }
        if !self.can_allocate_in_block(size, alignment) {
            let special_block = self.allocator.alloc(size);
            (self.on_block_use)(&special_block);
            self.special_blocks.push(special_block.clone());
            return Allocation {
                block: special_block,
                offset: 0,
                size,
            };
        }
        if self.current_block.is_none() {
            self.current_block = Some(self.allocate_block());
            self.current_offset = 0;
        }

        if self.can_allocate_in_current_block(size, alignment) {
            let aligned_offset = Mth::round_toward_i64(self.current_offset, alignment);
            self.current_offset = aligned_offset + size;
            return Allocation {
                block: self.current_block.clone().expect("a current block"),
                offset: aligned_offset,
                size,
            };
        }

        if self.can_allocate_in_current_block(minimum_allocation, alignment) {
            let aligned_offset = Mth::round_toward_i64(self.current_offset, alignment);
            let allocated_size = (self.block_size - aligned_offset) / element_size * element_size;
            self.current_offset = aligned_offset + allocated_size;
            return Allocation {
                block: self.current_block.clone().expect("a current block"),
                offset: aligned_offset,
                size: allocated_size,
            };
        }

        let new_block = self.allocate_block();
        if self.current_offset > size {
            self.current_block = Some(new_block.clone());
            self.current_offset = size;
        }
        Allocation {
            block: new_block,
            offset: 0,
            size,
        }
    }
}
