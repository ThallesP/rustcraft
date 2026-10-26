//! net/minecraft/client/renderer/chunk/SectionTaskDynamicQueue.java
//!
//! The compile tasks waiting for a worker thread. A worker takes the one nearest the camera,
//! preferring a section's first compile to a recompile unless the recompile is nearer, and then
//! at most twice in a row. A cancelled task is dropped as the queue is searched. Java's methods
//! are `synchronized`; the dispatcher holds the queue in a `Mutex`.

use std::sync::Arc;

use crate::net::minecraft::client::renderer::chunk::section_render_dispatcher::CompileTask;
use crate::net::minecraft::world::phys::vec3::Vec3;

pub struct SectionTaskDynamicQueue {
    recompile_quota: i32,
    tasks: Vec<Arc<CompileTask>>,
}

impl Default for SectionTaskDynamicQueue {
    fn default() -> SectionTaskDynamicQueue {
        SectionTaskDynamicQueue::new()
    }
}

impl SectionTaskDynamicQueue {
    const MAX_RECOMPILE_QUOTA: i32 = 2;

    pub fn new() -> SectionTaskDynamicQueue {
        SectionTaskDynamicQueue {
            recompile_quota: SectionTaskDynamicQueue::MAX_RECOMPILE_QUOTA,
            tasks: Vec::new(),
        }
    }

    pub fn add(&mut self, task: Arc<CompileTask>) {
        self.tasks.push(task);
    }

    /// Java's `!(recompile < initial)`, which a NaN distance would make true, is kept as it is.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub fn poll(&mut self, camera_pos: &Vec3) -> Option<Arc<CompileTask>> {
        let mut best_initial_compile_task_index = None;
        let mut best_recompile_task_index = None;
        let mut best_initial_compile_distance = f64::MAX;
        let mut best_recompile_distance = f64::MAX;
        let mut task_index = 0;
        // ListIterator's walk, removing the cancelled tasks as it goes.
        while task_index < self.tasks.len() {
            let task = &self.tasks[task_index];
            if task.is_cancelled() {
                self.tasks.remove(task_index);
                continue;
            }

            let distance = task.get_render_origin().dist_to_center_sqr_xyz(
                camera_pos.x,
                camera_pos.y,
                camera_pos.z,
            );
            if !task.is_recompile() && distance < best_initial_compile_distance {
                best_initial_compile_distance = distance;
                best_initial_compile_task_index = Some(task_index);
            }

            if task.is_recompile() && distance < best_recompile_distance {
                best_recompile_distance = distance;
                best_recompile_task_index = Some(task_index);
            }
            task_index += 1;
        }

        let has_recompile_task = best_recompile_task_index.is_some();
        let has_initial_compile_task = best_initial_compile_task_index.is_some();
        if !has_recompile_task
            || has_initial_compile_task
                && (self.recompile_quota <= 0
                    || !(best_recompile_distance < best_initial_compile_distance))
        {
            self.recompile_quota = SectionTaskDynamicQueue::MAX_RECOMPILE_QUOTA;
            best_initial_compile_task_index.map(|index| self.tasks.remove(index))
        } else {
            self.recompile_quota -= 1;
            best_recompile_task_index.map(|index| self.tasks.remove(index))
        }
    }

    pub fn size(&self) -> usize {
        self.tasks.len()
    }

    pub fn clear(&mut self) {
        for task in &self.tasks {
            task.cancel();
        }

        self.tasks.clear();
    }
}
