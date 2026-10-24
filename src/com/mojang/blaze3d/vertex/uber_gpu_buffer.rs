//! com/mojang/blaze3d/vertex/UberGpuBuffer.java
//!
//! Keyed allocations in GPU buffers of a fixed size, the heaps, each with its `TlsfAllocator`: an
//! allocation goes in the first heap with room for it, or a new heap, and a heap left empty is let
//! go. Java stages an allocation's bytes in a `StagingBuffer` and copies them into the heap on the
//! GPU; here the bytes wait in the uber buffer until the upload and are written straight into the
//! heap's buffer (`writeToBuffer`), which leaves the heap holding the same bytes, so a `false` from
//! `addAllocation` (a full staging buffer) can't happen. Java keeps its staged allocations in a map
//! by the keys' identity, which iterates in an order their identity hashes decide; here they're
//! uploaded in the order their keys were first staged, which only decides where they go in the
//! heaps. A heap is its buffer and an id, which an allocation names. The upload callbacks are the
//! keys `upload_staged_allocations` returns. Removing an allocation drops what its key has staged
//! too, where Java notes the key to skip at the next upload: the same, for keys that aren't staged
//! again after, and a key here may be a value another key had (a freed mesh's address), which
//! Java's objects can't. `printStatistics`, a debug log, isn't ported.

use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::com::mojang::blaze3d::vertex::tlsf_allocator::{self, TlsfAllocator};
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;

pub struct UberGpuBuffer<K> {
    buffer_usage: i32,
    heap_size: i32,
    align_size: i32,
    name: String,
    nodes: Vec<UberGpuBufferHeap>,
    next_heap_id: u64,
    staged_allocations: Vec<(K, Vec<u8>)>,
    allocation_map: HashMap<K, Allocation>,
}

/// `TlsfAllocator.Allocation` with its heap.
pub struct Allocation {
    heap: u64,
    allocation: tlsf_allocator::Allocation,
}

impl Allocation {
    pub fn get_offset_from_heap(&self) -> i64 {
        self.allocation.get_offset_from_heap()
    }
}

/// `UberGpuBuffer.UberGpuBufferHeap` with its allocator, Java's node.
struct UberGpuBufferHeap {
    id: u64,
    allocator: TlsfAllocator,
    gpu_buffer: Rc<dyn GpuBuffer>,
}

impl<K: Copy + Eq + Hash> UberGpuBuffer<K> {
    pub fn new(name: &str, buffer_usage: i32, heap_size: i32, align_size: i32) -> UberGpuBuffer<K> {
        UberGpuBuffer {
            buffer_usage,
            heap_size,
            align_size,
            name: format!("UberBuffer {name}"),
            nodes: Vec::new(),
            next_heap_id: 0,
            staged_allocations: Vec::new(),
            allocation_map: HashMap::new(),
        }
    }

    /// `addAllocation`: the key's bytes, for the next upload, in place of what it had staged.
    pub fn add_allocation(&mut self, allocation_key: K, buffer: Vec<u8>) {
        match self
            .staged_allocations
            .iter_mut()
            .find(|(key, _)| *key == allocation_key)
        {
            Some((_, staged)) => *staged = buffer,
            None => self.staged_allocations.push((allocation_key, buffer)),
        }
    }

    /// `uploadStagedAllocations`: each staged key's old allocation freed and its bytes put in a
    /// heap, then a heap left empty let go. Returns whether a heap
    /// was made or let go, and the keys uploaded, in order (Java's callbacks).
    pub fn upload_staged_allocations(&mut self, gpu_device: &dyn GpuDevice) -> (bool, Vec<K>) {
        let staged = std::mem::take(&mut self.staged_allocations);
        for (key, _) in &staged {
            self.free_allocation(*key);
        }

        let mut new_heap_created_or_destroyed = false;
        let mut uploaded = Vec::new();
        let encoder = gpu_device.create_command_encoder();
        for (key, buffer) in staged {
            let allocation_size = buffer.len() as i64;
            let mut allocation = None;
            for node in &mut self.nodes {
                if let Some(found) = node.allocator.allocate(allocation_size, self.align_size) {
                    allocation = Some((node.id, found));
                    break;
                }
            }

            if allocation.is_none() {
                assert!(allocation_size <= i64::from(self.heap_size));
                let heap_name = format!("{} {}", self.name, self.nodes.len());
                let mut node = UberGpuBufferHeap {
                    id: self.next_heap_id,
                    allocator: TlsfAllocator::new(i64::from(self.heap_size)),
                    gpu_buffer: gpu_device.create_buffer(
                        Some(&heap_name),
                        self.buffer_usage | gpu_buffer::USAGE_COPY_DST | gpu_buffer::USAGE_COPY_SRC,
                        i64::from(self.heap_size),
                    ),
                };
                self.next_heap_id += 1;
                allocation = node
                    .allocator
                    .allocate(allocation_size, self.align_size)
                    .map(|found| (node.id, found));
                self.nodes.push(node);
                new_heap_created_or_destroyed = true;
            }

            if let Some((heap, allocation)) = allocation {
                let destination = &self.heap(heap).gpu_buffer;
                encoder.write_to_buffer(
                    &gpu_buffer::slice_range(
                        destination,
                        allocation.get_offset_from_heap(),
                        allocation_size,
                    ),
                    &buffer,
                );
                self.allocation_map
                    .insert(key, Allocation { heap, allocation });
                uploaded.push(key);
            }
        }

        if let Some(index) = self
            .nodes
            .iter()
            .position(|node| node.allocator.is_completely_free())
        {
            self.nodes.remove(index);
            new_heap_created_or_destroyed = true;
        }

        (new_heap_created_or_destroyed, uploaded)
    }

    pub fn get_allocation(&self, allocation_key: K) -> Option<&Allocation> {
        self.allocation_map.get(&allocation_key)
    }

    pub fn remove_allocation(&mut self, allocation_key: K) {
        self.staged_allocations
            .retain(|(key, _)| *key != allocation_key);
        self.free_allocation(allocation_key);
    }

    fn free_allocation(&mut self, allocation_key: K) {
        if let Some(allocation) = self.allocation_map.remove(&allocation_key)
            && let Some(node) = self
                .nodes
                .iter_mut()
                .find(|node| node.id == allocation.heap)
        {
            node.allocator.free(allocation.allocation);
        }
    }

    pub fn get_gpu_buffer(&self, allocation: &Allocation) -> &Rc<dyn GpuBuffer> {
        &self.heap(allocation.heap).gpu_buffer
    }

    fn heap(&self, id: u64) -> &UberGpuBufferHeap {
        self.nodes
            .iter()
            .find(|node| node.id == id)
            .expect("an allocation's heap is there while it is")
    }
}
