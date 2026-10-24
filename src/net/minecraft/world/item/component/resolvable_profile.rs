//! net/minecraft/world/item/component/ResolvableProfile.java
//!
//! A player's profile as an item or a text component holds it: whole (`Static`, a stored
//! `GameProfile` or a partial one), or a name or an id to look up (`Dynamic`). Not ported yet:
//! `STREAM_CODEC` (with `ByteBufCodecs.GAME_PROFILE` and the skin patch's stream codec), resolving a
//! dynamic profile (`resolveProfile`, a `ProfileResolver`, Mojang's services), the tooltip and
//! `hashCode`. Java's sealed class of two is an enum here. Neither overrides `toString`, so Java
//! prints one by identity (`ResolvableProfile$Static@1b2c3d`); the port prints the class name.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::authlib::game_profile::GameProfile;
use crate::com::mojang::authlib::properties::property_map::PropertyMap;
use crate::com::mojang::datafixers::util::either::Either;
use crate::com::mojang::serialization::codec::{CodecExt, SharedCodec, StaticCodec};
use crate::com::mojang::serialization::codecs::either_map_codec::EitherMapCodec;
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, SharedMapCodec, StaticMapCodec};
use crate::java::util::uuid::UUID;
use crate::net::minecraft::core::uuid_util::UUIDUtil;
use crate::net::minecraft::util::extra_codecs::ExtraCodecs;
use crate::net::minecraft::world::entity::player::player_skin::Patch;

static PARTIAL_MAP_CODEC: StaticMapCodec<Partial> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (
            ExtraCodecs::PLAYER_NAME
                .shared()
                .optional_field_of("name")
                .for_getter(|partial: &Partial| partial.name.clone()),
            UUIDUtil::CODEC
                .shared()
                .optional_field_of("id")
                .for_getter(|partial: &Partial| partial.id),
            ExtraCodecs::PROPERTY_MAP
                .shared()
                .optional_field_of_default("properties", PropertyMap::EMPTY)
                .for_getter(|partial: &Partial| partial.properties.clone()),
        ),
        |name, id, properties| Partial {
            name,
            id,
            properties,
        },
    )
    .boxed()
});

static FULL_CODEC: StaticCodec<ResolvableProfile> = LazyLock::new(|| {
    record_codec_builder::create(
        (
            EitherMapCodec::new(
                ExtraCodecs::STORED_GAME_PROFILE.shared(),
                PARTIAL_MAP_CODEC.shared(),
            )
            .for_getter(ResolvableProfile::unpack),
            Patch::MAP_CODEC
                .shared()
                .for_getter(|profile: &ResolvableProfile| profile.skin_patch().clone()),
        ),
        ResolvableProfile::create,
    )
    .boxed()
});

// `Codec.withAlternative(FULL_CODEC, ExtraCodecs.PLAYER_NAME, ResolvableProfile::createUnresolved)`.
static CODEC: StaticCodec<ResolvableProfile> = LazyLock::new(|| {
    FULL_CODEC
        .shared()
        .with_alternative_converted(
            ExtraCodecs::PLAYER_NAME.shared(),
            ResolvableProfile::create_unresolved_name,
        )
        .boxed()
});

/// `ResolvableProfile.Partial`: what's known of a profile.
#[derive(Clone, Debug, PartialEq)]
pub struct Partial {
    pub name: Option<String>,
    pub id: Option<UUID>,
    pub properties: PropertyMap,
}

impl Partial {
    pub const EMPTY: Partial = Partial {
        name: None,
        id: None,
        properties: PropertyMap::EMPTY,
    };

    fn create_profile(&self) -> GameProfile {
        ResolvableProfile::create_partial_profile(
            self.name.clone(),
            self.id,
            self.properties.clone(),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResolvableProfile {
    Static {
        contents: Either<GameProfile, Partial>,
        partial_profile: GameProfile,
        skin_patch: Patch,
    },
    Dynamic {
        name_or_id: Either<String, UUID>,
        partial_profile: GameProfile,
        skin_patch: Patch,
    },
}

impl ResolvableProfile {
    pub const CODEC: &'static StaticCodec<ResolvableProfile> = &CODEC;

    // A partial profile with a name or an id alone and no properties is one to look up.
    fn create(value: Either<GameProfile, Partial>, patch: Patch) -> ResolvableProfile {
        match value {
            Either::Left(full) => ResolvableProfile::new_static(Either::Left(full), patch),
            Either::Right(partial) => {
                if partial.properties.is_empty() && partial.id.is_some() != partial.name.is_some() {
                    match partial.name {
                        Some(name) => ResolvableProfile::new_dynamic(Either::Left(name), patch),
                        None => ResolvableProfile::new_dynamic(
                            Either::Right(partial.id.expect("an id without a name")),
                            patch,
                        ),
                    }
                } else {
                    ResolvableProfile::new_static(Either::Right(partial), patch)
                }
            }
        }
    }

    fn new_static(contents: Either<GameProfile, Partial>, skin_patch: Patch) -> ResolvableProfile {
        let partial_profile = match &contents {
            Either::Left(profile) => profile.clone(),
            Either::Right(partial) => partial.create_profile(),
        };
        ResolvableProfile::Static {
            contents,
            partial_profile,
            skin_patch,
        }
    }

    fn new_dynamic(name_or_id: Either<String, UUID>, skin_patch: Patch) -> ResolvableProfile {
        let partial_profile = match &name_or_id {
            Either::Left(name) => ResolvableProfile::create_partial_profile(
                Some(name.clone()),
                None,
                PropertyMap::EMPTY,
            ),
            Either::Right(id) => {
                ResolvableProfile::create_partial_profile(None, Some(*id), PropertyMap::EMPTY)
            }
        };
        ResolvableProfile::Dynamic {
            name_or_id,
            partial_profile,
            skin_patch,
        }
    }

    pub fn create_resolved(game_profile: GameProfile) -> ResolvableProfile {
        ResolvableProfile::new_static(Either::Left(game_profile), Patch::EMPTY)
    }

    /// `createUnresolved(String)`.
    pub fn create_unresolved_name(name: String) -> ResolvableProfile {
        ResolvableProfile::new_dynamic(Either::Left(name), Patch::EMPTY)
    }

    /// `createUnresolved(UUID)`.
    pub fn create_unresolved_id(id: UUID) -> ResolvableProfile {
        ResolvableProfile::new_dynamic(Either::Right(id), Patch::EMPTY)
    }

    /// `unpack`: what the codec writes.
    fn unpack(&self) -> Either<GameProfile, Partial> {
        match self {
            ResolvableProfile::Static { contents, .. } => contents.clone(),
            ResolvableProfile::Dynamic { name_or_id, .. } => Either::Right(Partial {
                name: name_or_id.left().cloned(),
                id: name_or_id.right().copied(),
                properties: PropertyMap::EMPTY,
            }),
        }
    }

    pub fn partial_profile(&self) -> &GameProfile {
        match self {
            ResolvableProfile::Static {
                partial_profile, ..
            }
            | ResolvableProfile::Dynamic {
                partial_profile, ..
            } => partial_profile,
        }
    }

    pub fn skin_patch(&self) -> &Patch {
        match self {
            ResolvableProfile::Static { skin_patch, .. }
            | ResolvableProfile::Dynamic { skin_patch, .. } => skin_patch,
        }
    }

    // The name, or the empty name; the id, or the name's offline id, or the nil id.
    fn create_partial_profile(
        maybe_name: Option<String>,
        maybe_id: Option<UUID>,
        properties: PropertyMap,
    ) -> GameProfile {
        let id = maybe_id.unwrap_or_else(|| {
            maybe_name
                .as_deref()
                .map_or(UUID::new(0, 0), UUIDUtil::create_offline_player_uuid)
        });
        GameProfile::from_properties(id, maybe_name.unwrap_or_default(), properties)
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            ResolvableProfile::Static { contents, .. } => match contents {
                Either::Left(profile) => Some(profile.name()),
                Either::Right(partial) => partial.name.as_deref(),
            },
            ResolvableProfile::Dynamic { name_or_id, .. } => name_or_id.left().map(String::as_str),
        }
    }
}

impl Eq for ResolvableProfile {}

/// Consistent with `eq`: equal profiles have equal partial profiles. (Java's `hashCode` isn't
/// ported.)
impl std::hash::Hash for ResolvableProfile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let profile = self.partial_profile();
        profile.id().hash(state);
        profile.name().hash(state);
        self.skin_patch().hash(state);
    }
}

/// Java's identity `toString`, without the identity.
impl fmt::Display for ResolvableProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolvableProfile::Static { .. } => {
                f.write_str("net.minecraft.world.item.component.ResolvableProfile$Static")
            }
            ResolvableProfile::Dynamic { .. } => {
                f.write_str("net.minecraft.world.item.component.ResolvableProfile$Dynamic")
            }
        }
    }
}
