//! net/minecraft/util/RandomSource.java

use crate::net::minecraft::world::level::levelgen::legacy_random_source::LegacyRandomSource;
use crate::net::minecraft::world::level::levelgen::positional_random_factory::PositionalRandomFactory;
use crate::net::minecraft::world::level::levelgen::random_support::RandomSupport;
use crate::net::minecraft::world::level::levelgen::single_threaded_random_source::SingleThreadedRandomSource;
#[allow(deprecated)]
use crate::net::minecraft::world::level::levelgen::thread_safe_legacy_random_source::ThreadSafeLegacyRandomSource;

#[deprecated]
pub const GAUSSIAN_SPREAD_FACTOR: f64 = 2.297;

pub fn create() -> Box<dyn RandomSource> {
    create_with_seed(RandomSupport::generate_unique_seed())
}

#[deprecated]
#[allow(deprecated)]
pub fn create_thread_safe() -> Box<dyn RandomSource> {
    Box::new(ThreadSafeLegacyRandomSource::new(
        RandomSupport::generate_unique_seed(),
    ))
}

pub fn create_with_seed(seed: i64) -> Box<dyn RandomSource> {
    Box::new(LegacyRandomSource::new(seed))
}

// Vanilla seeds this from Netty's ThreadLocalRandom; any unpredictable seed behaves the same.
pub fn create_thread_local_instance() -> Box<dyn RandomSource> {
    create_thread_local_instance_with_seed(RandomSupport::generate_unique_seed())
}

pub fn create_thread_local_instance_with_seed(seed: i64) -> Box<dyn RandomSource> {
    Box::new(SingleThreadedRandomSource::new(seed))
}

pub trait RandomSource {
    fn fork(&mut self) -> Box<dyn RandomSource>;

    fn fork_positional(&mut self) -> Box<dyn PositionalRandomFactory>;

    fn set_seed(&mut self, seed: i64);

    fn next_int(&mut self) -> i32;

    fn next_int_bounded(&mut self, bound: i32) -> i32;

    fn next_int_between_inclusive(&mut self, min: i32, max_inclusive: i32) -> i32 {
        self.next_int_bounded(max_inclusive.wrapping_sub(min).wrapping_add(1))
            .wrapping_add(min)
    }

    fn next_long(&mut self) -> i64;

    fn next_boolean(&mut self) -> bool;

    fn next_float(&mut self) -> f32;

    fn next_double(&mut self) -> f64;

    fn next_gaussian(&mut self) -> f64;

    fn triangle_f64(&mut self, mean: f64, spread: f64) -> f64 {
        mean + spread * (self.next_double() - self.next_double())
    }

    fn triangle_f32(&mut self, mean: f32, spread: f32) -> f32 {
        mean + spread * (self.next_float() - self.next_float())
    }

    fn consume_count(&mut self, rounds: i32) {
        for _ in 0..rounds {
            self.next_int();
        }
    }

    fn next_int_range(&mut self, origin: i32, bound: i32) -> i32 {
        assert!(origin < bound, "bound - origin is non positive");
        origin.wrapping_add(self.next_int_bounded(bound.wrapping_sub(origin)))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::RandomSource;
    use crate::golden::{Decode, Encode, Golden, catch};
    use crate::net::minecraft::world::level::levelgen::legacy_random_source::LegacyRandomSource;
    use crate::net::minecraft::world::level::levelgen::single_threaded_random_source::SingleThreadedRandomSource;
    #[allow(deprecated)]
    use crate::net::minecraft::world::level::levelgen::thread_safe_legacy_random_source::ThreadSafeLegacyRandomSource;
    use crate::net::minecraft::world::level::levelgen::xoroshiro_random_source::XoroshiroRandomSource;

    #[test]
    #[allow(deprecated)]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.util.RandomSource");
        golden.check("LegacyRandomSource", |seed: i64, program: Value| {
            run(&mut LegacyRandomSource::new(seed), &program)
        });
        golden.check("XoroshiroRandomSource", |seed: i64, program: Value| {
            run(&mut XoroshiroRandomSource::new(seed), &program)
        });
        golden.check("SingleThreadedRandomSource", |seed: i64, program: Value| {
            run(&mut SingleThreadedRandomSource::new(seed), &program)
        });
        golden.check(
            "ThreadSafeLegacyRandomSource",
            |seed: i64, program: Value| run(&mut ThreadSafeLegacyRandomSource::new(seed), &program),
        );
        for source in [
            "LegacyRandomSource",
            "XoroshiroRandomSource",
            "SingleThreadedRandomSource",
            "ThreadSafeLegacyRandomSource",
        ] {
            golden.pending(
                &format!("{source}+nextGaussian"),
                "nextGaussian uses Math.log, which isn't exact yet",
            );
        }
        golden.finish();
    }

    // Mirrors RandomSourceSuite.apply in oracle/golden/; a panicking op ends the program like a thrown exception.
    fn run(source: &mut dyn RandomSource, program: &Value) -> Value {
        let mut outputs = Vec::new();
        for op in program.as_array().expect("a program is an array of ops") {
            match catch(|| apply(source, op.as_array().expect("an op is an array"))) {
                Ok(output) => outputs.push(output),
                Err(_) => {
                    outputs.push(json!({ "throws": "panic" }));
                    break;
                }
            }
        }
        Value::from(outputs)
    }

    fn apply(source: &mut dyn RandomSource, op: &[Value]) -> Value {
        let int = |index: usize| i32::decode(&op[index]);
        match op[0].as_str().expect("ops start with their name") {
            "nextInt" if op.len() == 1 => source.next_int().encode(),
            "nextInt" => source.next_int_bounded(int(1)).encode(),
            "nextIntRange" => source.next_int_range(int(1), int(2)).encode(),
            "nextIntBetweenInclusive" => source.next_int_between_inclusive(int(1), int(2)).encode(),
            "nextLong" => source.next_long().encode(),
            "nextBoolean" => source.next_boolean().encode(),
            "nextFloat" => source.next_float().encode(),
            "nextDouble" => source.next_double().encode(),
            "nextGaussian" => source.next_gaussian().encode(),
            "triangleDouble" => source
                .triangle_f64(f64::decode(&op[1]), f64::decode(&op[2]))
                .encode(),
            "triangleFloat" => source
                .triangle_f32(f32::decode(&op[1]), f32::decode(&op[2]))
                .encode(),
            "consumeCount" => {
                source.consume_count(int(1));
                Value::Null
            }
            "setSeed" => {
                source.set_seed(i64::decode(&op[1]));
                Value::Null
            }
            "fork" => probe(source.fork().as_mut()),
            "forkPositionalAt" => {
                probe(source.fork_positional().at(int(1), int(2), int(3)).as_mut())
            }
            "forkPositionalFromHashOf" => probe(
                source
                    .fork_positional()
                    .from_hash_of(&String::decode(&op[1]))
                    .as_mut(),
            ),
            "forkPositionalFromSeed" => probe(
                source
                    .fork_positional()
                    .from_seed(i64::decode(&op[1]))
                    .as_mut(),
            ),
            "forkPositionalParity" => {
                let mut builder = String::new();
                source.fork_positional().parity_config_string(&mut builder);
                Value::from(builder)
            }
            name => panic!("unknown op {name}"),
        }
    }

    fn probe(source: &mut dyn RandomSource) -> Value {
        json!([
            source.next_long(),
            source.next_int_bounded(100),
            source.next_float().encode(),
            source.next_double().encode()
        ])
    }
}
