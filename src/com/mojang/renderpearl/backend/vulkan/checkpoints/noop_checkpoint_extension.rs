//! com/mojang/renderpearl/backend/vulkan/checkpoints/NoopCheckpointExtension.java

use crate::com::mojang::renderpearl::backend::vulkan::checkpoints::checkpoint_extension::{
    CheckpointExtension, CheckpointStorage, CheckpointType, QueueCheckpoints,
};
use ash::vk;

pub struct NoopCheckpointExtension;

impl NoopCheckpointExtension {
    pub const INSTANCE: NoopCheckpointExtension = NoopCheckpointExtension;
}

impl CheckpointExtension for NoopCheckpointExtension {
    fn create_storage(&self, _max_frames_in_flight: i32) -> Box<dyn CheckpointStorage> {
        Box::new(NoopCheckpointStorage)
    }

    fn retrieve_checkpoints(&self, _is_device_lost: bool) -> Vec<QueueCheckpoints> {
        Vec::new()
    }
}

/// `NoopCheckpointExtension.NoopCheckpointStorage`.
struct NoopCheckpointStorage;

impl CheckpointStorage for NoopCheckpointStorage {
    fn rotate(&self) {}

    fn record_checkpoint(
        &self,
        _command_buffer: vk::CommandBuffer,
        _type: CheckpointType,
        _label: &str,
    ) {
    }
}
