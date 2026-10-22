//! net/minecraft/client/multiplayer/ChunkBatchSizeCalculator.java

use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::util::Util;

/// How many chunks a tick the client can take, from how long the server's chunk batches took to
/// arrive, which the client tells the server after each batch.
#[derive(Clone, Debug)]
pub struct ChunkBatchSizeCalculator {
    aggregated_nanos_per_chunk: f64,
    old_samples_weight: i32,
    chunk_batch_start_time: i64,
}

impl Default for ChunkBatchSizeCalculator {
    fn default() -> ChunkBatchSizeCalculator {
        ChunkBatchSizeCalculator::new()
    }
}

impl ChunkBatchSizeCalculator {
    const MAX_OLD_SAMPLES_WEIGHT: i32 = 49;
    const CLAMP_COEFFICIENT: i32 = 3;

    pub fn new() -> ChunkBatchSizeCalculator {
        ChunkBatchSizeCalculator {
            aggregated_nanos_per_chunk: 2000000.0,
            old_samples_weight: 1,
            chunk_batch_start_time: Util::get_nanos(),
        }
    }

    pub fn on_batch_start(&mut self) {
        self.chunk_batch_start_time = Util::get_nanos();
    }

    pub fn on_batch_finished(&mut self, batch_size: i32) {
        self.on_batch_finished_at(batch_size, Util::get_nanos());
    }

    /// `onBatchFinished`, with the clock's time given.
    fn on_batch_finished_at(&mut self, batch_size: i32, now: i64) {
        if batch_size > 0 {
            let batch_duration = (now - self.chunk_batch_start_time) as f64;
            let nanos_per_chunk = batch_duration / f64::from(batch_size);
            let coefficient = f64::from(ChunkBatchSizeCalculator::CLAMP_COEFFICIENT);
            let clamped_nanos_per_chunk = Mth::clamp_f64(
                nanos_per_chunk,
                self.aggregated_nanos_per_chunk / coefficient,
                self.aggregated_nanos_per_chunk * coefficient,
            );
            self.aggregated_nanos_per_chunk = (self.aggregated_nanos_per_chunk
                * f64::from(self.old_samples_weight)
                + clamped_nanos_per_chunk)
                / f64::from(self.old_samples_weight + 1);
            self.old_samples_weight =
                ChunkBatchSizeCalculator::MAX_OLD_SAMPLES_WEIGHT.min(self.old_samples_weight + 1);
        }
    }

    pub fn get_desired_chunks_per_tick(&self) -> f32 {
        (7000000.0 / self.aggregated_nanos_per_chunk) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::ChunkBatchSizeCalculator;

    // vanilla's capture of the walk scenario: the first batch's answer, 0x40a80000.
    #[test]
    fn starts_at_three_and_a_half_chunks_a_tick() {
        let mut calculator = ChunkBatchSizeCalculator::new();
        assert_eq!(
            calculator.get_desired_chunks_per_tick().to_bits(),
            0x40600000
        );
        calculator.chunk_batch_start_time = 0;
        // 9 chunks in 1.5 ms is clamped to a third of 2 ms a chunk, and averaged with it.
        calculator.on_batch_finished_at(9, 1500000);
        assert_eq!(
            calculator.get_desired_chunks_per_tick().to_bits(),
            0x40a80000
        );
    }
}
