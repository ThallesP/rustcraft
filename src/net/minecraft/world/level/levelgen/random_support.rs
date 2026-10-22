//! net/minecraft/world/level/levelgen/RandomSupport.java

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use md5::{Digest, Md5};

pub struct RandomSupport;

impl RandomSupport {
    pub const GOLDEN_RATIO_64: i64 = -7046029254386353131;
    pub const SILVER_RATIO_64: i64 = 7640891576956012809;

    pub fn mix_stafford13(z: i64) -> i64 {
        let z = (z ^ ((z as u64) >> 30) as i64).wrapping_mul(-4658895280553007687);
        let z = (z ^ ((z as u64) >> 27) as i64).wrapping_mul(-7723592293110705685);
        z ^ ((z as u64) >> 31) as i64
    }

    pub fn upgrade_seed_to128bit_unmixed(legacy_seed: i64) -> Seed128bit {
        let low_bits = legacy_seed ^ RandomSupport::SILVER_RATIO_64;
        let high_bits = low_bits.wrapping_add(RandomSupport::GOLDEN_RATIO_64);
        Seed128bit::new(low_bits, high_bits)
    }

    pub fn upgrade_seed_to128bit(legacy_seed: i64) -> Seed128bit {
        RandomSupport::upgrade_seed_to128bit_unmixed(legacy_seed).mixed()
    }

    pub fn seed_from_hash_of(input: &str) -> Seed128bit {
        let hash_code: [u8; 16] = Md5::digest(input.as_bytes()).into();
        let (hash_lo, hash_hi) = hash_code.split_at(8);
        Seed128bit::new(
            i64::from_be_bytes(hash_lo.try_into().unwrap()),
            i64::from_be_bytes(hash_hi.try_into().unwrap()),
        )
    }

    // Vanilla xors in System.nanoTime(); any clock reading is as good.
    pub fn generate_unique_seed() -> i64 {
        static SEED_UNIQUIFIER: AtomicI64 = AtomicI64::new(8682522807148012);
        let step = |current: i64| current.wrapping_mul(1181783497276652981);
        let previous = SEED_UNIQUIFIER.update(Ordering::Relaxed, Ordering::Relaxed, step);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos() as i64);
        step(previous) ^ nanos
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Seed128bit {
    seed_lo: i64,
    seed_hi: i64,
}

impl Seed128bit {
    pub const fn new(seed_lo: i64, seed_hi: i64) -> Seed128bit {
        Seed128bit { seed_lo, seed_hi }
    }

    pub const fn seed_lo(&self) -> i64 {
        self.seed_lo
    }

    pub const fn seed_hi(&self) -> i64 {
        self.seed_hi
    }

    pub fn xor(&self, lo: i64, hi: i64) -> Seed128bit {
        Seed128bit::new(self.seed_lo ^ lo, self.seed_hi ^ hi)
    }

    pub fn xor_seed(&self, other: Seed128bit) -> Seed128bit {
        self.xor(other.seed_lo, other.seed_hi)
    }

    pub fn mixed(&self) -> Seed128bit {
        Seed128bit::new(
            RandomSupport::mix_stafford13(self.seed_lo),
            RandomSupport::mix_stafford13(self.seed_hi),
        )
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{RandomSupport, Seed128bit};
    use crate::golden::{Encode, Golden};

    // Records are arrays of their components.
    impl Encode for Seed128bit {
        fn encode(&self) -> Value {
            Value::from(vec![self.seed_lo, self.seed_hi])
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.levelgen.RandomSupport");
        golden.field("GOLDEN_RATIO_64", RandomSupport::GOLDEN_RATIO_64);
        golden.field("SILVER_RATIO_64", RandomSupport::SILVER_RATIO_64);
        golden.check("mixStafford13(J)J", RandomSupport::mix_stafford13);
        golden.check("upgradeSeedTo128bitUnmixed(J)Lnet/minecraft/world/level/levelgen/RandomSupport$Seed128bit;", RandomSupport::upgrade_seed_to128bit_unmixed);
        golden.check(
            "upgradeSeedTo128bit(J)Lnet/minecraft/world/level/levelgen/RandomSupport$Seed128bit;",
            RandomSupport::upgrade_seed_to128bit,
        );
        golden.check("seedFromHashOf(Ljava/lang/String;)Lnet/minecraft/world/level/levelgen/RandomSupport$Seed128bit;", |input: String| {
            RandomSupport::seed_from_hash_of(&input)
        });
        golden.pending(
            "generateUniqueSeed()J",
            "seeded from the clock, so there's nothing to compare",
        );
        golden.finish();
    }
}
