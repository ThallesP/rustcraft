//! net/minecraft/world/level/biome/BiomeSpecialEffects.java
//!
//! A biome's colours: its water's, and the overrides of its grass's, leaves' and dry leaves' and
//! the modifier of its grass's. Not ported yet: `CODEC` (DataFixerUpper's), which `Biome`'s
//! temporary readers stand in for, and the `Builder`, which only the server's biomes use.

use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::biome::biome::BIOME_INFO_NOISE;

/// A biome's colours.
#[derive(Clone, Debug, PartialEq)]
pub struct BiomeSpecialEffects {
    water_color: i32,
    foliage_color_override: Option<i32>,
    dry_foliage_color_override: Option<i32>,
    grass_color_override: Option<i32>,
    grass_color_modifier: GrassColorModifier,
}

impl BiomeSpecialEffects {
    pub fn new(
        water_color: i32,
        foliage_color_override: Option<i32>,
        dry_foliage_color_override: Option<i32>,
        grass_color_override: Option<i32>,
        grass_color_modifier: GrassColorModifier,
    ) -> BiomeSpecialEffects {
        BiomeSpecialEffects {
            water_color,
            foliage_color_override,
            dry_foliage_color_override,
            grass_color_override,
            grass_color_modifier,
        }
    }

    pub fn water_color(&self) -> i32 {
        self.water_color
    }

    pub fn foliage_color_override(&self) -> Option<i32> {
        self.foliage_color_override
    }

    pub fn dry_foliage_color_override(&self) -> Option<i32> {
        self.dry_foliage_color_override
    }

    pub fn grass_color_override(&self) -> Option<i32> {
        self.grass_color_override
    }

    pub fn grass_color_modifier(&self) -> GrassColorModifier {
        self.grass_color_modifier
    }
}

/// `BiomeSpecialEffects.GrassColorModifier`: how a biome changes its grass's colour where it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum GrassColorModifier {
    NONE,
    DARK_FOREST,
    SWAMP,
}

impl GrassColorModifier {
    pub const VALUES: [GrassColorModifier; 3] = [
        GrassColorModifier::NONE,
        GrassColorModifier::DARK_FOREST,
        GrassColorModifier::SWAMP,
    ];

    pub fn modify_color(&self, x: f64, z: f64, base_color: i32) -> i32 {
        match self {
            GrassColorModifier::NONE => base_color,
            GrassColorModifier::DARK_FOREST => {
                ARGB::opaque(((base_color & 16711422) + 2634762) >> 1)
            }
            GrassColorModifier::SWAMP => {
                let ground_value = f64::from(BIOME_INFO_NOISE.get_2d(x * 0.0225, z * 0.0225));
                if ground_value < -0.1 {
                    -11766212
                } else {
                    -9801671
                }
            }
        }
    }

    /// `getName()`.
    pub fn get_name(&self) -> &'static str {
        self.get_serialized_name()
    }

    /// The constant with that name, as the codec reads it.
    pub fn by_name(name: &str) -> Option<GrassColorModifier> {
        GrassColorModifier::VALUES
            .into_iter()
            .find(|modifier| modifier.get_serialized_name() == name)
    }
}

impl StringRepresentable for GrassColorModifier {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            GrassColorModifier::NONE => "none",
            GrassColorModifier::DARK_FOREST => "dark_forest",
            GrassColorModifier::SWAMP => "swamp",
        }
    }
}
