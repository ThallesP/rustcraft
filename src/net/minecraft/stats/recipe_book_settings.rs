//! net/minecraft/stats/RecipeBookSettings.java
//!
//! Not ported yet: `MAP_CODEC` and `TypeSettings`' map codecs, which wait for DataFixerUpper's
//! codecs (the player's saved settings, the server's).

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::world::inventory::recipe_book_type::RecipeBookType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, RecipeBookSettings> = LazyLock::new(|| {
    Box::new(stream_codec::composite4(
        TypeSettings::STREAM_CODEC,
        |o: &RecipeBookSettings| o.crafting,
        TypeSettings::STREAM_CODEC,
        |o: &RecipeBookSettings| o.furnace,
        TypeSettings::STREAM_CODEC,
        |o: &RecipeBookSettings| o.blast_furnace,
        TypeSettings::STREAM_CODEC,
        |o: &RecipeBookSettings| o.smoker,
        RecipeBookSettings::from_settings,
    ))
});

static TYPE_SETTINGS_STREAM_CODEC: StaticStreamCodec<ByteBuf, TypeSettings> = LazyLock::new(|| {
    Box::new(stream_codec::composite2(
        ByteBufCodecs::BOOL,
        TypeSettings::open,
        ByteBufCodecs::BOOL,
        TypeSettings::filtering,
        TypeSettings::new,
    ))
});

/// Whether each recipe book is open and shows only what can be made.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecipeBookSettings {
    crafting: TypeSettings,
    furnace: TypeSettings,
    blast_furnace: TypeSettings,
    smoker: TypeSettings,
}

impl RecipeBookSettings {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, RecipeBookSettings> =
        &STREAM_CODEC;

    /// Every book closed and not filtering.
    pub fn new() -> RecipeBookSettings {
        RecipeBookSettings::from_settings(
            TypeSettings::DEFAULT,
            TypeSettings::DEFAULT,
            TypeSettings::DEFAULT,
            TypeSettings::DEFAULT,
        )
    }

    /// The private constructor taking each book's settings.
    fn from_settings(
        crafting: TypeSettings,
        furnace: TypeSettings,
        blast_furnace: TypeSettings,
        smoker: TypeSettings,
    ) -> RecipeBookSettings {
        RecipeBookSettings {
            crafting,
            furnace,
            blast_furnace,
            smoker,
        }
    }

    pub fn get_settings(&self, r#type: RecipeBookType) -> TypeSettings {
        match r#type {
            RecipeBookType::CRAFTING => self.crafting,
            RecipeBookType::FURNACE => self.furnace,
            RecipeBookType::BLAST_FURNACE => self.blast_furnace,
            RecipeBookType::SMOKER => self.smoker,
        }
    }

    fn update_settings(
        &mut self,
        recipe_book_type: RecipeBookType,
        operator: impl FnOnce(TypeSettings) -> TypeSettings,
    ) {
        let settings = match recipe_book_type {
            RecipeBookType::CRAFTING => &mut self.crafting,
            RecipeBookType::FURNACE => &mut self.furnace,
            RecipeBookType::BLAST_FURNACE => &mut self.blast_furnace,
            RecipeBookType::SMOKER => &mut self.smoker,
        };
        *settings = operator(*settings);
    }

    pub fn is_open(&self, r#type: RecipeBookType) -> bool {
        self.get_settings(r#type).open
    }

    pub fn set_open(&mut self, r#type: RecipeBookType, open: bool) {
        self.update_settings(r#type, |settings| settings.set_open(open));
    }

    pub fn is_filtering(&self, r#type: RecipeBookType) -> bool {
        self.get_settings(r#type).filtering
    }

    pub fn set_filtering(&mut self, r#type: RecipeBookType, filtering: bool) {
        self.update_settings(r#type, |settings| settings.set_filtering(filtering));
    }

    pub fn copy(&self) -> RecipeBookSettings {
        *self
    }

    pub fn replace_from(&mut self, other: &RecipeBookSettings) {
        *self = *other;
    }
}

impl Default for RecipeBookSettings {
    fn default() -> RecipeBookSettings {
        RecipeBookSettings::new()
    }
}

/// `RecipeBookSettings.TypeSettings`: whether a book is open, and whether it shows only what can
/// be made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeSettings {
    open: bool,
    filtering: bool,
}

impl TypeSettings {
    pub const DEFAULT: TypeSettings = TypeSettings {
        open: false,
        filtering: false,
    };
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, TypeSettings> =
        &TYPE_SETTINGS_STREAM_CODEC;

    pub fn new(open: bool, filtering: bool) -> TypeSettings {
        TypeSettings { open, filtering }
    }

    pub fn open(&self) -> bool {
        self.open
    }

    pub fn filtering(&self) -> bool {
        self.filtering
    }

    pub fn set_open(&self, open: bool) -> TypeSettings {
        TypeSettings::new(open, self.filtering)
    }

    pub fn set_filtering(&self, filtering: bool) -> TypeSettings {
        TypeSettings::new(self.open, filtering)
    }

    /// A record's: each boolean's `Boolean.hashCode` (1231 or 1237), the second added to 31 times
    /// the first.
    pub fn hash_code(&self) -> i32 {
        let hash = |value: bool| -> i32 { if value { 1231 } else { 1237 } };
        hash(self.open)
            .wrapping_mul(31)
            .wrapping_add(hash(self.filtering))
    }
}

/// `[open=false, filtering=false]`.
impl fmt::Display for TypeSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[open={}, filtering={}]", self.open, self.filtering)
    }
}

#[cfg(test)]
mod tests {
    use super::{RecipeBookSettings, TypeSettings};
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::world::inventory::recipe_book_type::RecipeBookType;

    // Its fields, by reflection.
    encode_fields!(RecipeBookSettings {
        crafting: TypeSettings,
        furnace: TypeSettings,
        blast_furnace: TypeSettings,
        smoker: TypeSettings,
    });
    encode_fields!(TypeSettings {
        open: bool,
        filtering: bool,
    });

    type This = RecipeBookSettings;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.stats.RecipeBookSettings");
        golden.pending("MAP_CODEC", "waits for DataFixerUpper's codecs");
        golden.check_codec("STREAM_CODEC", || RecipeBookSettings::STREAM_CODEC);
        golden.check("<init>()V", RecipeBookSettings::new);
        golden.check_method(
            "getSettings(Lnet/minecraft/world/inventory/RecipeBookType;)Lnet/minecraft/stats/RecipeBookSettings$TypeSettings;",
            |this: &mut This, r#type: RecipeBookType| this.get_settings(r#type),
        );
        golden.check_method(
            "isOpen(Lnet/minecraft/world/inventory/RecipeBookType;)Z",
            |this: &mut This, r#type: RecipeBookType| this.is_open(r#type),
        );
        golden.check_method(
            "setOpen(Lnet/minecraft/world/inventory/RecipeBookType;Z)V",
            |this: &mut This, r#type: RecipeBookType, open: bool| this.set_open(r#type, open),
        );
        golden.check_method(
            "isFiltering(Lnet/minecraft/world/inventory/RecipeBookType;)Z",
            |this: &mut This, r#type: RecipeBookType| this.is_filtering(r#type),
        );
        golden.check_method(
            "setFiltering(Lnet/minecraft/world/inventory/RecipeBookType;Z)V",
            |this: &mut This, r#type: RecipeBookType, filtering: bool| {
                this.set_filtering(r#type, filtering)
            },
        );
        golden.check_method(
            "copy()Lnet/minecraft/stats/RecipeBookSettings;",
            |this: &mut This| this.copy(),
        );
        golden.check_mut(
            "replaceFrom(Lnet/minecraft/stats/RecipeBookSettings;)V",
            |this: &mut This, other: &mut This| this.replace_from(other),
        );
        golden.finish();
    }

    #[test]
    fn type_settings_match_vanilla() {
        let golden = Golden::load("net.minecraft.stats.RecipeBookSettings$TypeSettings");
        for name in [
            "BLAST_FURNACE_MAP_CODEC",
            "CRAFTING_MAP_CODEC",
            "FURNACE_MAP_CODEC",
            "SMOKER_MAP_CODEC",
        ] {
            golden.pending(name, "waits for DataFixerUpper's codecs");
        }
        golden.field("DEFAULT", TypeSettings::DEFAULT);
        golden.check_codec("STREAM_CODEC", || TypeSettings::STREAM_CODEC);
        golden.check("<init>(ZZ)V", TypeSettings::new);
        golden.check_method("open()Z", |this: &mut TypeSettings| this.open());
        golden.check_method("filtering()Z", |this: &mut TypeSettings| this.filtering());
        golden.check_method(
            "setOpen(Z)Lnet/minecraft/stats/RecipeBookSettings$TypeSettings;",
            |this: &mut TypeSettings, open: bool| this.set_open(open),
        );
        golden.check_method(
            "setFiltering(Z)Lnet/minecraft/stats/RecipeBookSettings$TypeSettings;",
            |this: &mut TypeSettings, filtering: bool| this.set_filtering(filtering),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut TypeSettings, other: TypeSettings| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut TypeSettings| this.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut TypeSettings| {
            this.to_string()
        });
        golden.finish();
    }
}
