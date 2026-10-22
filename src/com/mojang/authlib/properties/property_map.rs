//! com/mojang/authlib/properties/PropertyMap.java
//!
//! Not ported yet: its `Serializer`, which reads and writes properties as JSON with Gson, for
//! authlib's requests to Mojang's servers.
//!
//! A Guava `ForwardingMultimap`, whose methods all forward to the multimap it holds, so only the
//! ones the game calls are here: `values`, `size` and `isEmpty`, and `equals`, `hashCode` and
//! `toString`, which a `GameProfile`'s use.

use std::fmt;

use crate::com::google::common::collect::immutable_multimap::ImmutableMultimap;
use crate::com::mojang::authlib::properties::property::Property;

/// A profile's properties, by name.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyMap {
    properties: ImmutableMultimap<String, Property>,
}

impl PropertyMap {
    pub const EMPTY: PropertyMap = PropertyMap {
        properties: ImmutableMultimap::of(),
    };

    /// Java takes any `Multimap`, and keeps `ImmutableMultimap.copyOf` of it.
    pub fn new(properties: ImmutableMultimap<String, Property>) -> PropertyMap {
        PropertyMap {
            properties: ImmutableMultimap::copy_of(properties),
        }
    }

    /// Grouped by name, in the order each name first came.
    pub fn values(&self) -> impl Iterator<Item = &Property> {
        self.properties.values()
    }

    pub fn size(&self) -> i32 {
        self.properties.size()
    }

    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    pub fn hash_code(&self) -> i32 {
        self.properties.hash_code()
    }
}

/// The multimap's: `{textures=[Property[…]]}`.
impl fmt::Display for PropertyMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.properties, f)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::PropertyMap;
    use crate::com::google::common::collect::immutable_multimap::ImmutableMultimap;
    use crate::com::mojang::authlib::properties::property::Property;
    use crate::golden::{Decode, Encode, Golden};

    // Vanilla writes a property map as its multimap.
    impl Decode for PropertyMap {
        fn decode(value: &Value) -> Self {
            PropertyMap {
                properties: ImmutableMultimap::decode(value),
            }
        }
    }

    impl Encode for PropertyMap {
        fn encode(&self) -> Value {
            self.properties.encode()
        }

        fn matches(&self, expected: &Value) -> bool {
            self.properties.matches(expected)
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.authlib.properties.PropertyMap");
        golden.field("EMPTY", PropertyMap::EMPTY);
        golden.check(
            "<init>(Lcom/google/common/collect/Multimap;)V",
            PropertyMap::new,
        );
        golden.check_method(
            "values()Ljava/util/Collection;",
            |this: &mut PropertyMap| -> Vec<Property> { this.values().cloned().collect() },
        );
        golden.check_method("size()I", |this: &mut PropertyMap| this.size());
        golden.check_method("isEmpty()Z", |this: &mut PropertyMap| this.is_empty());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut PropertyMap, other: PropertyMap| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut PropertyMap| this.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut PropertyMap| {
            this.to_string()
        });
        golden.finish();
    }
}
