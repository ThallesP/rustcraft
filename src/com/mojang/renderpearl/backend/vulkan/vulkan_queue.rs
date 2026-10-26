//! com/mojang/renderpearl/backend/vulkan/VulkanQueue.java
//!
//! A device queue and its submissions: command buffers with the semaphores they wait on and
//! signal, in stages, submitted together through `vkQueueSubmit2KHR` when the submission closes.

use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils;
use ash::vk;

#[derive(Clone, Copy, Debug)]
pub struct VulkanQueue {
    vk_queue: vk::Queue,
    queue_family_index: u32,
}

impl VulkanQueue {
    pub fn new(device: &ash::Device, queue_family_index: i32, queue_index: i32) -> VulkanQueue {
        VulkanQueue {
            // SAFETY: the device was made with the family's queues.
            vk_queue: unsafe {
                device.get_device_queue(queue_family_index as u32, queue_index as u32)
            },
            queue_family_index: queue_family_index as u32,
        }
    }

    pub fn vk_queue(&self) -> vk::Queue {
        self.vk_queue
    }

    pub fn queue_family_index(&self) -> u32 {
        self.queue_family_index
    }

    pub fn begin_submit(&self) -> Submission {
        Submission {
            queue: self.vk_queue,
            closed: false,
            stages: vec![SubmitStage::default()],
        }
    }

    pub fn wait_idle(&self, device: &ash::Device) {
        // SAFETY: the queue is the device's.
        let _ = unsafe { device.queue_wait_idle(self.vk_queue) };
    }
}

/// `VulkanQueue.Submission`.
pub struct Submission {
    queue: vk::Queue,
    closed: bool,
    stages: Vec<SubmitStage>,
}

/// `Submission.SemaphoreOp`.
#[derive(Clone, Copy)]
struct SemaphoreOp {
    vk_semaphore: vk::Semaphore,
    value: u64,
    stage_mask: vk::PipelineStageFlags2,
}

/// `Submission.SubmitStage`.
#[derive(Default)]
struct SubmitStage {
    waits: Vec<SemaphoreOp>,
    command_buffers: Vec<vk::CommandBuffer>,
    signals: Vec<SemaphoreOp>,
}

impl Submission {
    fn check_open(&self) {
        assert!(!self.closed, "Attempt to use closed Submission");
    }

    fn top(&mut self) -> &mut SubmitStage {
        self.stages.last_mut().expect("a stage")
    }

    pub fn wait_semaphore(
        &mut self,
        semaphore: vk::Semaphore,
        value: u64,
        stage_mask: vk::PipelineStageFlags2,
    ) {
        self.check_open();
        if !self.top().command_buffers.is_empty() || !self.top().signals.is_empty() {
            self.stages.push(SubmitStage::default());
        }
        self.top().waits.push(SemaphoreOp {
            vk_semaphore: semaphore,
            value,
            stage_mask,
        });
    }

    pub fn execute_commands(&mut self, command_buffer: vk::CommandBuffer) {
        self.check_open();
        if !self.top().signals.is_empty() {
            self.stages.push(SubmitStage::default());
        }
        self.top().command_buffers.push(command_buffer);
    }

    pub fn signal_semaphore(
        &mut self,
        semaphore: vk::Semaphore,
        value: u64,
        stage_mask: vk::PipelineStageFlags2,
    ) {
        self.check_open();
        self.top().signals.push(SemaphoreOp {
            vk_semaphore: semaphore,
            value,
            stage_mask,
        });
    }

    /// `close()`: every stage submitted in one `vkQueueSubmit2KHR`.
    pub fn close(&mut self, synchronization2: &ash::khr::synchronization2::Device) {
        let semaphore_infos = |ops: &[SemaphoreOp]| -> Vec<vk::SemaphoreSubmitInfo<'static>> {
            ops.iter()
                .map(|op| {
                    vk::SemaphoreSubmitInfo::default()
                        .semaphore(op.vk_semaphore)
                        .value(op.value)
                        .stage_mask(op.stage_mask)
                })
                .collect()
        };
        let waits: Vec<_> = self
            .stages
            .iter()
            .map(|stage| semaphore_infos(&stage.waits))
            .collect();
        let signals: Vec<_> = self
            .stages
            .iter()
            .map(|stage| semaphore_infos(&stage.signals))
            .collect();
        let buffers: Vec<Vec<vk::CommandBufferSubmitInfo<'static>>> = self
            .stages
            .iter()
            .map(|stage| {
                stage
                    .command_buffers
                    .iter()
                    .map(|buffer| vk::CommandBufferSubmitInfo::default().command_buffer(*buffer))
                    .collect()
            })
            .collect();
        let submits: Vec<vk::SubmitInfo2<'_>> = (0..self.stages.len())
            .map(|i| {
                vk::SubmitInfo2::default()
                    .wait_semaphore_infos(&waits[i])
                    .command_buffer_infos(&buffers[i])
                    .signal_semaphore_infos(&signals[i])
            })
            .collect();
        // SAFETY: the buffers have ended recording, and the infos live for the call.
        let result =
            unsafe { synchronization2.queue_submit2(self.queue, &submits, vk::Fence::null()) };
        if let Err(error) = result {
            VulkanUtils::crash_if_failure(error, "Failed to submit to queue");
        }
        self.closed = true;
    }
}
