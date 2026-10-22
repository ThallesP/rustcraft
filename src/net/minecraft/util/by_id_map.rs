//! net/minecraft/util/ByIdMap.java
//!
//! Java's `ToIntFunction` and `IntFunction` are closures, and the values a slice. The maps it
//! makes hand out copies of the values, which is what a caller of Java's (an enum's `BY_ID`)
//! sees.

use std::collections::HashMap;
use std::fmt::Debug;

use crate::net::minecraft::util::mth::Mth;

pub struct ByIdMap;

impl ByIdMap {
    fn create_map<T: Clone + Debug>(
        id_getter: impl Fn(&T) -> i32,
        values: &[T],
    ) -> HashMap<i32, T> {
        assert!(!values.is_empty(), "Empty value list");

        let mut result = HashMap::new();

        for value in values {
            let id = id_getter(value);
            let previous = result.insert(id, value.clone());
            if let Some(previous) = previous {
                panic!("Duplicate entry on id {id}: current={value:?}, previous={previous:?}");
            }
        }

        result
    }

    /// `sparse(ToIntFunction, Object[], Object)`: an id no value has gives the default. Panics
    /// where Java throws `IllegalArgumentException`, on no values or two with the same id.
    pub fn sparse<T: Clone + Debug>(
        id_getter: impl Fn(&T) -> i32,
        values: &[T],
        default: T,
    ) -> impl Fn(i32) -> T {
        let id_to_object = ByIdMap::create_map(id_getter, values);
        move |id| id_to_object.get(&id).unwrap_or(&default).clone()
    }

    // The values by id, which must run from 0 up without gaps.
    fn create_sorted_array<T: Clone + Debug>(
        id_getter: impl Fn(&T) -> i32,
        values: &[T],
    ) -> Vec<T> {
        let length = values.len() as i32;
        assert!(length != 0, "Empty value list");

        let mut result: Vec<Option<T>> = vec![None; values.len()];

        for value in values {
            let id = id_getter(value);
            assert!(
                id >= 0 && id < length,
                "Values are not continous, found index {id} for value {value:?}"
            );

            let previous = &result[id as usize];
            if let Some(previous) = previous {
                panic!("Duplicate entry on id {id}: current={value:?}, previous={previous:?}");
            }

            result[id as usize] = Some(value.clone());
        }

        for (i, value) in result.iter().enumerate() {
            assert!(value.is_some(), "Missing value at index: {i}");
        }

        result.into_iter().flatten().collect()
    }

    /// `continuous(ToIntFunction, Object[], OutOfBoundsStrategy)`: ids from 0 up, and the
    /// strategy for any other. Panics where Java throws `IllegalArgumentException`, on no values,
    /// or ids that aren't each of 0 to the last once.
    pub fn continuous<T: Clone + Debug>(
        id_getter: impl Fn(&T) -> i32,
        values: &[T],
        strategy: OutOfBoundsStrategy,
    ) -> impl Fn(i32) -> T {
        let sorted_values = ByIdMap::create_sorted_array(id_getter, values);
        let length = sorted_values.len() as i32;

        move |id| match strategy {
            OutOfBoundsStrategy::ZERO => {
                let zero_value = &sorted_values[0];
                if id >= 0 && id < length {
                    sorted_values[id as usize].clone()
                } else {
                    zero_value.clone()
                }
            }
            OutOfBoundsStrategy::WRAP => {
                sorted_values[Mth::positive_modulo_i32(id, length) as usize].clone()
            }
            OutOfBoundsStrategy::CLAMP => {
                sorted_values[Mth::clamp_i32(id, 0, length - 1) as usize].clone()
            }
        }
    }
}

/// What `continuous` gives for an id out of range: the first value, the id wrapped around, or
/// the nearest end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OutOfBoundsStrategy {
    ZERO,
    WRAP,
    CLAMP,
}

impl OutOfBoundsStrategy {
    pub const VALUES: [OutOfBoundsStrategy; 3] = [
        OutOfBoundsStrategy::ZERO,
        OutOfBoundsStrategy::WRAP,
        OutOfBoundsStrategy::CLAMP,
    ];

    pub fn values() -> [OutOfBoundsStrategy; 3] {
        OutOfBoundsStrategy::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> OutOfBoundsStrategy {
        match name {
            "ZERO" => OutOfBoundsStrategy::ZERO,
            "WRAP" => OutOfBoundsStrategy::WRAP,
            "CLAMP" => OutOfBoundsStrategy::CLAMP,
            _ => panic!("IllegalArgumentException: no OutOfBoundsStrategy constant {name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ByIdMap, OutOfBoundsStrategy};
    use crate::golden::{Golden, encode_names};
    use crate::net::minecraft::core::direction::Direction;

    encode_names!(OutOfBoundsStrategy { ZERO, WRAP, CLAMP });

    const CONTINUOUS: &str = "continuous(Ljava/util/function/ToIntFunction;[Ljava/lang/Object;Lnet/minecraft/util/ByIdMap$OutOfBoundsStrategy;)Ljava/util/function/IntFunction;";
    const SPARSE: &str = "sparse(Ljava/util/function/ToIntFunction;[Ljava/lang/Object;Ljava/lang/Object;)Ljava/util/function/IntFunction;";

    #[test]
    fn by_id_map_matches_vanilla() {
        let golden = Golden::load("net.minecraft.util.ByIdMap");
        golden.check_function_from(
            &format!("{CONTINUOUS}[get3DDataValue, values()]"),
            |strategy: OutOfBoundsStrategy| {
                ByIdMap::continuous(Direction::get_3d_data_value, &Direction::VALUES, strategy)
            },
        );
        golden.check_function_from(
            &format!("{CONTINUOUS}[get2DDataValue, values()]"),
            |strategy: OutOfBoundsStrategy| {
                ByIdMap::continuous(Direction::get_2d_data_value, &Direction::VALUES, strategy)
            },
        );
        golden.check_function_from(
            &format!("{CONTINUOUS}[get3DDataValue, []]"),
            |strategy: OutOfBoundsStrategy| {
                ByIdMap::continuous(Direction::get_3d_data_value, &[], strategy)
            },
        );
        golden.check_function(
            &format!("{SPARSE}[get2DDataValue, [NORTH, EAST, SOUTH, WEST], UP]"),
            || {
                ByIdMap::sparse(
                    Direction::get_2d_data_value,
                    &[
                        Direction::NORTH,
                        Direction::EAST,
                        Direction::SOUTH,
                        Direction::WEST,
                    ],
                    Direction::UP,
                )
            },
        );
        golden.check_function(&format!("{SPARSE}[get2DDataValue, [], UP]"), || {
            ByIdMap::sparse(Direction::get_2d_data_value, &[], Direction::UP)
        });
        golden.check_function(&format!("{SPARSE}[get2DDataValue, values(), UP]"), || {
            ByIdMap::sparse(
                Direction::get_2d_data_value,
                &Direction::VALUES,
                Direction::UP,
            )
        });
        golden.finish();
    }

    #[test]
    fn out_of_bounds_strategy_matches_vanilla() {
        let golden = Golden::load("net.minecraft.util.ByIdMap$OutOfBoundsStrategy");
        golden.field("ZERO", OutOfBoundsStrategy::ZERO);
        golden.field("WRAP", OutOfBoundsStrategy::WRAP);
        golden.field("CLAMP", OutOfBoundsStrategy::CLAMP);
        golden.check(
            "values()[Lnet/minecraft/util/ByIdMap$OutOfBoundsStrategy;",
            OutOfBoundsStrategy::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/util/ByIdMap$OutOfBoundsStrategy;",
            |name: String| OutOfBoundsStrategy::value_of(&name),
        );
        golden.finish();
    }
}
