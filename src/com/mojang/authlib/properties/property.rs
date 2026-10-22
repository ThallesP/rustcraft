//! com/mojang/authlib/properties/Property.java
//!
//! Not ported yet: `isSignatureValid`, which checks the signature with `java.security`'s
//! SHA1withRSA, and which the client never calls.

use std::fmt;

use crate::java::lang::object::HashCode;

/// A property of a player's profile, like the skin's `textures`: a name, a value, and the
/// signature of a server that vouches for it, if any.
#[derive(Clone, Debug, PartialEq)]
pub struct Property {
    name: String,
    value: String,
    signature: Option<String>,
}

impl Property {
    pub fn new(name: String, value: String) -> Property {
        Property::from_signature(name, value, None)
    }

    pub fn from_signature(name: String, value: String, signature: Option<String>) -> Property {
        Property {
            name,
            value,
            signature,
        }
    }

    pub fn has_signature(&self) -> bool {
        self.signature.is_some()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn signature(&self) -> Option<&str> {
        self.signature.as_deref()
    }
}

/// A record's: each component's hash, added to 31 times the ones before, and 0 for no signature.
impl HashCode for Property {
    fn hash_code(&self) -> i32 {
        let signature = self.signature.as_ref().map_or(0, HashCode::hash_code);
        self.name
            .hash_code()
            .wrapping_mul(31)
            .wrapping_add(self.value.hash_code())
            .wrapping_mul(31)
            .wrapping_add(signature)
    }
}

/// A record's: `Property[name=textures, value=…, signature=null]`.
impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Property[name={}, value={}, signature={}]",
            self.name,
            self.value,
            self.signature.as_deref().unwrap_or("null")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Property;
    use crate::golden::{Golden, encode_fields};
    use crate::java::lang::object::HashCode;

    // A record: its components.
    encode_fields!(Property {
        name: String,
        value: String,
        signature: Option<String>,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.authlib.properties.Property");
        golden.check(
            "<init>(Ljava/lang/String;Ljava/lang/String;)V",
            Property::new,
        );
        golden.check(
            "<init>(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
            Property::from_signature,
        );
        golden.check_method("hasSignature()Z", |this: &mut Property| {
            this.has_signature()
        });
        golden.pending(
            "isSignatureValid(Ljava/security/PublicKey;)Z",
            "needs java.security's SHA1withRSA, and the client never calls it",
        );
        golden.check_method("name()Ljava/lang/String;", |this: &mut Property| {
            this.name().to_owned()
        });
        golden.check_method("value()Ljava/lang/String;", |this: &mut Property| {
            this.value().to_owned()
        });
        golden.check_method("signature()Ljava/lang/String;", |this: &mut Property| {
            this.signature().map(str::to_owned)
        });
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut Property, other: Property| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut Property| this.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut Property| {
            this.to_string()
        });
        golden.finish();
    }
}
