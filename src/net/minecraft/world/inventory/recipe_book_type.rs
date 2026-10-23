//! net/minecraft/world/inventory/RecipeBookType.java

use std::fmt;

/// The recipe books: the crafting table's and the three furnaces'.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecipeBookType {
    CRAFTING,
    FURNACE,
    BLAST_FURNACE,
    SMOKER,
}

impl RecipeBookType {
    pub const VALUES: [RecipeBookType; 4] = [
        RecipeBookType::CRAFTING,
        RecipeBookType::FURNACE,
        RecipeBookType::BLAST_FURNACE,
        RecipeBookType::SMOKER,
    ];

    pub fn values() -> [RecipeBookType; 4] {
        RecipeBookType::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> RecipeBookType {
        RecipeBookType::VALUES
            .into_iter()
            .find(|r#type| format!("{type:?}") == name)
            .unwrap_or_else(|| {
                panic!("IllegalArgumentException: no RecipeBookType constant {name}")
            })
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for RecipeBookType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::RecipeBookType;
    use crate::golden::{Golden, encode_names};

    encode_names!(RecipeBookType {
        CRAFTING,
        FURNACE,
        BLAST_FURNACE,
        SMOKER
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.inventory.RecipeBookType");
        for r#type in RecipeBookType::VALUES {
            golden.field(&r#type.to_string(), r#type);
        }
        golden.check(
            "values()[Lnet/minecraft/world/inventory/RecipeBookType;",
            RecipeBookType::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/world/inventory/RecipeBookType;",
            |name: String| RecipeBookType::value_of(&name),
        );
        golden.finish();
    }
}
