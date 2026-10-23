//! net/minecraft/world/item/DyeColor.java
//!
//! Only each colour's id, name and map colours, which coloured blocks are made with, and `CODEC`
//! (by name). Not ported yet: the other colours (`getTextureDiffuseColor`, `getFireworkColor`,
//! `getTextColor`), `byId`, `byName`, `byFireworkColor`, `getMixedColor` and the stream codec.

use std::fmt;

use crate::net::minecraft::world::level::material::map_color::MapColor;

/// One of the sixteen dye colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DyeColor {
    WHITE,
    ORANGE,
    MAGENTA,
    LIGHT_BLUE,
    YELLOW,
    LIME,
    PINK,
    GRAY,
    LIGHT_GRAY,
    CYAN,
    PURPLE,
    BLUE,
    BROWN,
    GREEN,
    RED,
    BLACK,
}

struct Data {
    id: i32,
    name: &'static str,
    map_color: MapColor,
    terracotta_color: MapColor,
}

// Each constant's id, name and map colours, in declaration order.
const DATA: [Data; 16] = [
    Data {
        id: 0,
        name: "white",
        map_color: MapColor::SNOW,
        terracotta_color: MapColor::TERRACOTTA_WHITE,
    },
    Data {
        id: 1,
        name: "orange",
        map_color: MapColor::COLOR_ORANGE,
        terracotta_color: MapColor::TERRACOTTA_ORANGE,
    },
    Data {
        id: 2,
        name: "magenta",
        map_color: MapColor::COLOR_MAGENTA,
        terracotta_color: MapColor::TERRACOTTA_MAGENTA,
    },
    Data {
        id: 3,
        name: "light_blue",
        map_color: MapColor::COLOR_LIGHT_BLUE,
        terracotta_color: MapColor::TERRACOTTA_LIGHT_BLUE,
    },
    Data {
        id: 4,
        name: "yellow",
        map_color: MapColor::COLOR_YELLOW,
        terracotta_color: MapColor::TERRACOTTA_YELLOW,
    },
    Data {
        id: 5,
        name: "lime",
        map_color: MapColor::COLOR_LIGHT_GREEN,
        terracotta_color: MapColor::TERRACOTTA_LIGHT_GREEN,
    },
    Data {
        id: 6,
        name: "pink",
        map_color: MapColor::COLOR_PINK,
        terracotta_color: MapColor::TERRACOTTA_PINK,
    },
    Data {
        id: 7,
        name: "gray",
        map_color: MapColor::COLOR_GRAY,
        terracotta_color: MapColor::TERRACOTTA_GRAY,
    },
    Data {
        id: 8,
        name: "light_gray",
        map_color: MapColor::COLOR_LIGHT_GRAY,
        terracotta_color: MapColor::TERRACOTTA_LIGHT_GRAY,
    },
    Data {
        id: 9,
        name: "cyan",
        map_color: MapColor::COLOR_CYAN,
        terracotta_color: MapColor::TERRACOTTA_CYAN,
    },
    Data {
        id: 10,
        name: "purple",
        map_color: MapColor::COLOR_PURPLE,
        terracotta_color: MapColor::TERRACOTTA_PURPLE,
    },
    Data {
        id: 11,
        name: "blue",
        map_color: MapColor::COLOR_BLUE,
        terracotta_color: MapColor::TERRACOTTA_BLUE,
    },
    Data {
        id: 12,
        name: "brown",
        map_color: MapColor::COLOR_BROWN,
        terracotta_color: MapColor::TERRACOTTA_BROWN,
    },
    Data {
        id: 13,
        name: "green",
        map_color: MapColor::COLOR_GREEN,
        terracotta_color: MapColor::TERRACOTTA_GREEN,
    },
    Data {
        id: 14,
        name: "red",
        map_color: MapColor::COLOR_RED,
        terracotta_color: MapColor::TERRACOTTA_RED,
    },
    Data {
        id: 15,
        name: "black",
        map_color: MapColor::COLOR_BLACK,
        terracotta_color: MapColor::TERRACOTTA_BLACK,
    },
];

impl DyeColor {
    pub const VALUES: [DyeColor; 16] = [
        DyeColor::WHITE,
        DyeColor::ORANGE,
        DyeColor::MAGENTA,
        DyeColor::LIGHT_BLUE,
        DyeColor::YELLOW,
        DyeColor::LIME,
        DyeColor::PINK,
        DyeColor::GRAY,
        DyeColor::LIGHT_GRAY,
        DyeColor::CYAN,
        DyeColor::PURPLE,
        DyeColor::BLUE,
        DyeColor::BROWN,
        DyeColor::GREEN,
        DyeColor::RED,
        DyeColor::BLACK,
    ];

    fn data(&self) -> &'static Data {
        &DATA[*self as usize]
    }

    pub fn get_id(&self) -> i32 {
        self.data().id
    }

    pub fn get_name(&self) -> &'static str {
        self.data().name
    }

    pub fn get_map_color(&self) -> MapColor {
        self.data().map_color
    }

    pub fn get_terracotta_color(&self) -> MapColor {
        self.data().terracotta_color
    }
}

static CODEC: crate::com::mojang::serialization::codec::StaticCodec<DyeColor> =
    std::sync::LazyLock::new(|| {
        crate::net::minecraft::util::string_representable::from_values(&DyeColor::VALUES)
    });

impl DyeColor {
    /// `StringRepresentable.fromEnum(DyeColor::values)`.
    pub const CODEC: &'static crate::com::mojang::serialization::codec::StaticCodec<DyeColor> =
        &CODEC;
}

impl crate::net::minecraft::util::string_representable::StringRepresentable for DyeColor {
    fn get_serialized_name(&self) -> &'static str {
        self.get_name()
    }
}

/// Java's `toString`, the name.
impl fmt::Display for DyeColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_name())
    }
}
