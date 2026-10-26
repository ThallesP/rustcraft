//! com/mojang/renderpearl/backend/vulkan/checkpoints/CheckpointExtension.java
//!
//! Markers the command buffers leave in the queue, which say where a lost device was. Only the
//! no-op one is ported: AMD's buffer markers and NVIDIA's diagnostic checkpoints
//! (`AmdCheckpointExtension`, `NvidiaCheckpointExtension`) only say more when a device is lost,
//! and neither is on lavapipe, so their devices get no checkpoints here.

use ash::vk;

pub trait CheckpointExtension {
    fn create_storage(&self, max_frames_in_flight: i32) -> Box<dyn CheckpointStorage>;

    fn retrieve_checkpoints(&self, is_device_lost: bool) -> Vec<QueueCheckpoints>;
}

/// `CheckpointExtension.CheckpointStorage`.
pub trait CheckpointStorage {
    fn rotate(&self);

    fn record_checkpoint(
        &self,
        command_buffer: vk::CommandBuffer,
        r#type: CheckpointType,
        label: &str,
    );
}

/// `CheckpointExtension.CheckpointType`.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckpointType {
    BEGIN_RENDER_PASS,
    END_RENDER_PASS,
}

/// `CheckpointExtension.QueueCheckpoints`.
#[derive(Clone, Debug)]
pub struct QueueCheckpoints {
    pub queue: i64,
    pub checkpoints: Vec<StageCheckpoint>,
}

/// `CheckpointExtension.StageCheckpoint`.
#[derive(Clone, Debug)]
pub struct StageCheckpoint {
    pub stage: i64,
    pub r#type: CheckpointType,
    pub label: String,
}
