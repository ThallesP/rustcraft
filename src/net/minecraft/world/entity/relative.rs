//! net/minecraft/world/entity/Relative.java
//!
//! Java's sets of these are `EnumSet`s (`unpack`, `rotation`, `position`, `direction`) and
//! `Set.of` (`ALL`, `ROTATION`, `DELTA`), and `union`'s a `HashSet`: here a set is a `Vec` of the
//! constants in their order, which is an `EnumSet`'s iteration order. Only `pack` iterates one,
//! and it ORs bits, so the order of the others never shows.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};

/// Which parts of a teleport are relative to where the entity was, as bits of an int.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Relative {
    X,
    Y,
    Z,
    Y_ROT,
    X_ROT,
    DELTA_X,
    DELTA_Y,
    DELTA_Z,
    ROTATE_DELTA,
}

static SET_STREAM_CODEC: StaticStreamCodec<ByteBuf, Vec<Relative>> = LazyLock::new(|| {
    Box::new(ByteBufCodecs::INT.map(Relative::unpack, |set: &Vec<Relative>| Relative::pack(set)))
});

impl Relative {
    pub const VALUES: [Relative; 9] = [
        Relative::X,
        Relative::Y,
        Relative::Z,
        Relative::Y_ROT,
        Relative::X_ROT,
        Relative::DELTA_X,
        Relative::DELTA_Y,
        Relative::DELTA_Z,
        Relative::ROTATE_DELTA,
    ];
    pub const ALL: [Relative; 9] = Relative::VALUES;
    pub const ROTATION: [Relative; 2] = [Relative::Y_ROT, Relative::X_ROT];
    pub const DELTA: [Relative; 4] = [
        Relative::DELTA_X,
        Relative::DELTA_Y,
        Relative::DELTA_Z,
        Relative::ROTATE_DELTA,
    ];
    pub const SET_STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, Vec<Relative>> =
        &SET_STREAM_CODEC;

    pub fn values() -> [Relative; 9] {
        Relative::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> Relative {
        Relative::VALUES
            .into_iter()
            .find(|relative| relative.name() == name)
            .unwrap_or_else(|| panic!("IllegalArgumentException: no Relative constant {name}"))
    }

    pub fn name(&self) -> &'static str {
        match self {
            Relative::X => "X",
            Relative::Y => "Y",
            Relative::Z => "Z",
            Relative::Y_ROT => "Y_ROT",
            Relative::X_ROT => "X_ROT",
            Relative::DELTA_X => "DELTA_X",
            Relative::DELTA_Y => "DELTA_Y",
            Relative::DELTA_Z => "DELTA_Z",
            Relative::ROTATE_DELTA => "ROTATE_DELTA",
        }
    }

    pub fn ordinal(&self) -> i32 {
        *self as i32
    }

    fn bit(&self) -> i32 {
        self.ordinal()
    }

    /// The sets' constants, each once, in their order.
    pub fn union(sets: &[&[Relative]]) -> Vec<Relative> {
        Relative::of(|relative| sets.iter().any(|set| set.contains(&relative)))
    }

    pub fn rotation(relative_y_rot: bool, relative_x_rot: bool) -> Vec<Relative> {
        Relative::of(|relative| match relative {
            Relative::Y_ROT => relative_y_rot,
            Relative::X_ROT => relative_x_rot,
            _ => false,
        })
    }

    pub fn position(relative_x: bool, relative_y: bool, relative_z: bool) -> Vec<Relative> {
        Relative::of(|relative| match relative {
            Relative::X => relative_x,
            Relative::Y => relative_y,
            Relative::Z => relative_z,
            _ => false,
        })
    }

    pub fn direction(relative_x: bool, relative_y: bool, relative_z: bool) -> Vec<Relative> {
        Relative::of(|relative| match relative {
            Relative::DELTA_X => relative_x,
            Relative::DELTA_Y => relative_y,
            Relative::DELTA_Z => relative_z,
            _ => false,
        })
    }

    /// The constants `include` picks, in their order: an `EnumSet`.
    fn of(include: impl Fn(Relative) -> bool) -> Vec<Relative> {
        Relative::VALUES
            .into_iter()
            .filter(|&relative| include(relative))
            .collect()
    }

    fn get_mask(&self) -> i32 {
        1 << self.bit()
    }

    fn is_set(&self, value: i32) -> bool {
        (value & self.get_mask()) == self.get_mask()
    }

    pub fn unpack(value: i32) -> Vec<Relative> {
        Relative::of(|relative| relative.is_set(value))
    }

    pub fn pack(set: &[Relative]) -> i32 {
        set.iter()
            .fold(0, |result, argument| result | argument.get_mask())
    }
}

#[cfg(test)]
mod tests {
    use super::Relative;
    use crate::golden::encode_names;

    // A set of them is the names of its constants in order, an `EnumSet`'s.
    encode_names!(Relative {
        X,
        Y,
        Z,
        Y_ROT,
        X_ROT,
        DELTA_X,
        DELTA_Y,
        DELTA_Z,
        ROTATE_DELTA
    });

    #[test]
    fn packs_each_constant_in_its_bit() {
        for (bit, relative) in Relative::VALUES.into_iter().enumerate() {
            assert_eq!(Relative::pack(&[relative]), 1 << bit);
            assert_eq!(Relative::unpack(1 << bit), [relative]);
        }
        assert_eq!(Relative::unpack(-1), Relative::VALUES);
        assert_eq!(Relative::pack(&Relative::ROTATION), 0b11000);
    }
}
