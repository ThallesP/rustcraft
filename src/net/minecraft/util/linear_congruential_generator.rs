//! net/minecraft/util/LinearCongruentialGenerator.java

/// The step a biome lookup's seed hash mixes each coordinate in with.
pub struct LinearCongruentialGenerator;

impl LinearCongruentialGenerator {
    const MULTIPLIER: i64 = 6364136223846793005;
    const INCREMENT: i64 = 1442695040888963407;

    pub fn next(rval: i64, c: i64) -> i64 {
        let rval = rval.wrapping_mul(
            rval.wrapping_mul(LinearCongruentialGenerator::MULTIPLIER)
                .wrapping_add(LinearCongruentialGenerator::INCREMENT),
        );
        rval.wrapping_add(c)
    }
}
