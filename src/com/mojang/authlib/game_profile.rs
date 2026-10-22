//! com/mojang/authlib/GameProfile.java

use std::fmt;

use crate::com::mojang::authlib::properties::property_map::PropertyMap;
use crate::java::lang::string::JavaString;
use crate::java::util::uuid::UUID;

/// A player's profile: its id, its name, and properties like its skin. Java's constructor
/// rejects a null component, which Rust can't have.
#[derive(Clone, Debug, PartialEq)]
pub struct GameProfile {
    id: UUID,
    name: String,
    properties: PropertyMap,
}

impl GameProfile {
    pub fn new(id: UUID, name: String) -> GameProfile {
        GameProfile::from_properties(id, name, PropertyMap::EMPTY)
    }

    pub fn from_properties(id: UUID, name: String, properties: PropertyMap) -> GameProfile {
        GameProfile {
            id,
            name,
            properties,
        }
    }

    pub fn id(&self) -> UUID {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn properties(&self) -> &PropertyMap {
        &self.properties
    }

    /// A record's: each component's hash, added to 31 times the ones before.
    pub fn hash_code(&self) -> i32 {
        self.id
            .hash_code()
            .wrapping_mul(31)
            .wrapping_add(self.name.hash_code())
            .wrapping_mul(31)
            .wrapping_add(self.properties.hash_code())
    }
}

/// A record's: `GameProfile[id=…, name=rustcraft, properties={}]`.
impl fmt::Display for GameProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GameProfile[id={}, name={}, properties={}]",
            self.id, self.name, self.properties
        )
    }
}

#[cfg(test)]
mod tests {
    use super::GameProfile;
    use crate::com::mojang::authlib::properties::property_map::PropertyMap;
    use crate::golden::{Golden, encode_fields};
    use crate::java::util::uuid::UUID;

    // A record: its components.
    encode_fields!(GameProfile {
        id: UUID,
        name: String,
        properties: PropertyMap,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.authlib.GameProfile");
        golden.check(
            "<init>(Ljava/util/UUID;Ljava/lang/String;)V",
            GameProfile::new,
        );
        golden.check(
            "<init>(Ljava/util/UUID;Ljava/lang/String;Lcom/mojang/authlib/properties/PropertyMap;)V",
            GameProfile::from_properties,
        );
        golden.check_method("id()Ljava/util/UUID;", |this: &mut GameProfile| this.id());
        golden.check_method("name()Ljava/lang/String;", |this: &mut GameProfile| {
            this.name().to_owned()
        });
        golden.check_method(
            "properties()Lcom/mojang/authlib/properties/PropertyMap;",
            |this: &mut GameProfile| this.properties().clone(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut GameProfile, other: GameProfile| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut GameProfile| this.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut GameProfile| {
            this.to_string()
        });
        golden.finish();
    }
}
