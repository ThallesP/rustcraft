//! com/mojang/renderpearl/backend/vulkan/DestructionQueue.java
//!
//! Objects to destroy, in one of `internalQueueCount` queues, the current one taking what's
//! added; each `rotate` moves on to the next queue and destroys what it holds, which was added
//! that many rotations before. Java's is generic in what it holds and how it destroys it; the
//! only one the game makes holds `Destroyable`s, which destroy themselves. A destroyable can add
//! to the queue (a copy's callback freeing what it read), so the queue is a `RefCell` whose
//! rotation is taken out before it's destroyed.

use crate::com::mojang::renderpearl::backend::vulkan::destroyable::Destroyable;
use std::cell::RefCell;

pub struct DestructionQueue {
    destruction_queues: RefCell<Vec<Vec<Destroyable>>>,
    current_destruction_queue_index: std::cell::Cell<usize>,
}

impl DestructionQueue {
    pub fn new(internal_queue_count: usize) -> DestructionQueue {
        DestructionQueue {
            destruction_queues: RefCell::new(
                (0..internal_queue_count).map(|_| Vec::new()).collect(),
            ),
            current_destruction_queue_index: std::cell::Cell::new(0),
        }
    }

    /// `close()`: rotates until a whole round finds nothing to destroy.
    pub fn close(&self) {
        let size = self.destruction_queues.borrow().len();
        let mut i = 0;
        while i < size {
            if self.rotate() {
                i = 0;
            }
            i += 1;
        }
    }

    /// Whether the queue it moved on to held anything.
    pub fn rotate(&self) -> bool {
        let current_queue = {
            let mut queues = self.destruction_queues.borrow_mut();
            let index = (self.current_destruction_queue_index.get() + 1) % queues.len();
            self.current_destruction_queue_index.set(index);
            std::mem::take(&mut queues[index])
        };
        if current_queue.is_empty() {
            return false;
        }

        for destroyable in current_queue {
            destroyable();
        }
        true
    }

    pub fn add(&self, t: Destroyable) {
        self.destruction_queues.borrow_mut()[self.current_destruction_queue_index.get()].push(t);
    }
}
