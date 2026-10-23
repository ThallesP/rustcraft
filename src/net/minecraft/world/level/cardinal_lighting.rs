//! net/minecraft/world/level/CardinalLighting.java

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::StaticCodec;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::util::string_representable::{self, StringRepresentable};

/// How bright a face looks by the way it faces, in a dimension.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardinalLighting {
    pub down: f32,
    pub up: f32,
    pub north: f32,
    pub south: f32,
    pub west: f32,
    pub east: f32,
}

impl CardinalLighting {
    pub const DEFAULT: CardinalLighting = CardinalLighting {
        down: 0.5,
        up: 1.0,
        north: 0.8,
        south: 0.8,
        west: 0.6,
        east: 0.6,
    };
    pub const NETHER: CardinalLighting = CardinalLighting {
        down: 0.9,
        up: 0.9,
        north: 0.8,
        south: 0.8,
        west: 0.6,
        east: 0.6,
    };

    pub fn by_face(&self, direction: Direction) -> f32 {
        match direction {
            Direction::DOWN => self.down,
            Direction::UP => self.up,
            Direction::NORTH => self.north,
            Direction::SOUTH => self.south,
            Direction::WEST => self.west,
            Direction::EAST => self.east,
        }
    }
}

/// `CardinalLighting.Type`: a dimension type's lighting, by name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    DEFAULT,
    NETHER,
}

static TYPE_CODEC: StaticCodec<Type> =
    LazyLock::new(|| string_representable::from_values(&Type::VALUES));

impl Type {
    pub const VALUES: [Type; 2] = [Type::DEFAULT, Type::NETHER];
    pub const CODEC: &'static StaticCodec<Type> = &TYPE_CODEC;

    pub fn get(&self) -> CardinalLighting {
        match self {
            Type::DEFAULT => CardinalLighting::DEFAULT,
            Type::NETHER => CardinalLighting::NETHER,
        }
    }
}

impl StringRepresentable for Type {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            Type::DEFAULT => "default",
            Type::NETHER => "nether",
        }
    }
}
