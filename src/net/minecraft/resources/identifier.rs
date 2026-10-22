//! net/minecraft/resources/Identifier.java
//!
//! Not ported yet: `CODEC` and `read(String)`, which wait for DataFixerUpper; `ERROR_INVALID`, `read(StringReader)` and
//! `readNonEmpty`, which wait for brigadier; `withPath(UnaryOperator)`, which the golden harness
//! can't generate an argument for; and `resolveAgainst(Path)`, which comes with resource loading.
//!
//! Java throws `IdentifierException` for an invalid namespace or path, which is a panic here. Its
//! message goes through commons-lang's `escapeJava`, which isn't ported: goldens only record the
//! exception's class. Java's `assert`s in the constructor are off unless the JVM runs with `-ea`,
//! so they aren't ported either.

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::object::HashCode;
use crate::java::lang::string::JavaString;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};

/// A namespaced name, `namespace:path`, like `minecraft:block/stone`. Every one that exists is
/// valid, so its parts are ASCII.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier {
    namespace: String,
    path: String,
}

// As a string `parse` takes, so one without a namespace is `minecraft`'s.
static STREAM_CODEC: StaticStreamCodec<ByteBuf, Identifier> = LazyLock::new(|| {
    Box::new(ByteBufCodecs::STRING_UTF8.map(
        |identifier: String| Identifier::parse(&identifier),
        Identifier::to_string,
    ))
});

impl Identifier {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, Identifier> = &STREAM_CODEC;
    pub const NAMESPACE_SEPARATOR: u16 = b':' as u16;
    pub const DEFAULT_NAMESPACE: &str = "minecraft";
    pub const REALMS_NAMESPACE: &str = "realms";
    pub const ALLOWED_NAMESPACE_CHARACTERS: &str = "[a-z0-9_.-]";

    fn new(namespace: &str, path: &str) -> Identifier {
        Identifier {
            namespace: namespace.to_owned(),
            path: path.to_owned(),
        }
    }

    /// Checks the namespace first, then the path.
    fn create_untrusted(namespace: &str, path: &str) -> Identifier {
        Identifier::new(
            Identifier::assert_valid_namespace(namespace, path),
            Identifier::assert_valid_path(namespace, path),
        )
    }

    pub fn from_namespace_and_path(namespace: &str, path: &str) -> Identifier {
        Identifier::create_untrusted(namespace, path)
    }

    pub fn parse(identifier: &str) -> Identifier {
        Identifier::by_separator(identifier, b':' as u16)
    }

    pub fn with_default_namespace(path: &str) -> Identifier {
        Identifier::new(
            "minecraft",
            Identifier::assert_valid_path("minecraft", path),
        )
    }

    pub fn try_parse(identifier: &str) -> Option<Identifier> {
        Identifier::try_by_separator(identifier, b':' as u16)
    }

    pub fn try_build(namespace: &str, path: &str) -> Option<Identifier> {
        (Identifier::is_valid_namespace(namespace) && Identifier::is_valid_path(path))
            .then(|| Identifier::new(namespace, path))
    }

    /// Splits at the first separator. With none, or with nothing before it, the namespace is
    /// `minecraft`.
    pub fn by_separator(identifier: &str, separator: u16) -> Identifier {
        match split_at_separator(identifier, separator) {
            Some((namespace, path)) if !namespace.is_empty() => {
                Identifier::create_untrusted(namespace, path)
            }
            Some((_, path)) => Identifier::with_default_namespace(path),
            None => Identifier::with_default_namespace(identifier),
        }
    }

    /// Like `by_separator`, with `None` where it panics. The path is checked first.
    pub fn try_by_separator(identifier: &str, separator: u16) -> Option<Identifier> {
        match split_at_separator(identifier, separator) {
            Some((namespace, path)) => {
                if !Identifier::is_valid_path(path) {
                    None
                } else if !namespace.is_empty() {
                    Identifier::is_valid_namespace(namespace)
                        .then(|| Identifier::new(namespace, path))
                } else {
                    Some(Identifier::new("minecraft", path))
                }
            }
            None => Identifier::is_valid_path(identifier)
                .then(|| Identifier::new("minecraft", identifier)),
        }
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn get_namespace(&self) -> &str {
        &self.namespace
    }

    pub fn with_path(&self, new_path: &str) -> Identifier {
        Identifier::new(
            &self.namespace,
            Identifier::assert_valid_path(&self.namespace, new_path),
        )
    }

    pub fn with_prefix(&self, prefix: &str) -> Identifier {
        self.with_path(&format!("{prefix}{}", self.path))
    }

    pub fn with_suffix(&self, suffix: &str) -> Identifier {
        self.with_path(&format!("{}{suffix}", self.path))
    }

    pub fn hash_code(&self) -> i32 {
        31i32
            .wrapping_mul(self.namespace.hash_code())
            .wrapping_add(self.path.hash_code())
    }

    /// By path first, then by namespace, each with `String.compareTo`'s difference.
    pub fn compare_to(&self, o: &Identifier) -> i32 {
        let result = self.path.compare_to(&o.path);
        if result == 0 {
            self.namespace.compare_to(&o.namespace)
        } else {
            result
        }
    }

    /// Java replaces `/`, then `:`, which is the same as both at once.
    pub fn to_debug_file_name(&self) -> String {
        self.to_string().replace(['/', ':'], "_")
    }

    pub fn to_language_key(&self) -> String {
        format!("{}.{}", self.namespace, self.path)
    }

    pub fn to_short_language_key(&self) -> String {
        if self.namespace == "minecraft" {
            self.path.clone()
        } else {
            self.to_language_key()
        }
    }

    pub fn to_short_string(&self) -> String {
        if self.namespace == "minecraft" {
            self.path.clone()
        } else {
            self.to_string()
        }
    }

    pub fn to_language_key_prefix(&self, prefix: &str) -> String {
        format!("{prefix}.{}", self.to_language_key())
    }

    pub fn to_language_key_prefix_suffix(&self, prefix: &str, suffix: &str) -> String {
        format!("{prefix}.{}.{suffix}", self.to_language_key())
    }

    /// What a command reads as part of an identifier: a path's characters and the separator.
    pub fn is_allowed_in_identifier(c: u16) -> bool {
        u8::try_from(c).is_ok_and(|c| {
            c.is_ascii_digit()
                || c.is_ascii_lowercase()
                || c == b'_'
                || c == b':'
                || c == b'/'
                || c == b'.'
                || c == b'-'
        })
    }

    /// Checks each UTF-16 code unit, so anything outside ASCII fails.
    pub fn is_valid_path(path: &str) -> bool {
        path.encode_utf16().all(Identifier::valid_path_char)
    }

    /// `..` is the one namespace made of allowed characters that isn't valid. The empty one is.
    pub fn is_valid_namespace(namespace: &str) -> bool {
        if namespace == ".." {
            return false;
        }

        namespace
            .encode_utf16()
            .all(Identifier::valid_namespace_char)
    }

    /// Panics on an invalid namespace, where Java throws `IdentifierException`.
    fn assert_valid_namespace<'a>(namespace: &'a str, path: &str) -> &'a str {
        if !Identifier::is_valid_namespace(namespace) {
            panic!(
                "IdentifierException: Non [a-z0-9_.-] character in namespace of identifier: {namespace}:{path}"
            );
        }

        namespace
    }

    pub fn valid_path_char(c: u16) -> bool {
        u8::try_from(c).is_ok_and(|c| {
            c == b'_'
                || c == b'-'
                || c.is_ascii_lowercase()
                || c.is_ascii_digit()
                || c == b'/'
                || c == b'.'
        })
    }

    fn valid_namespace_char(c: u16) -> bool {
        u8::try_from(c).is_ok_and(|c| {
            c == b'_' || c == b'-' || c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'.'
        })
    }

    /// Panics on an invalid path, where Java throws `IdentifierException`.
    fn assert_valid_path<'a>(namespace: &str, path: &'a str) -> &'a str {
        if !Identifier::is_valid_path(path) {
            panic!(
                "IdentifierException: Non [a-z0-9/._-] character in path of location: {namespace}:{path}"
            );
        }

        path
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

/// For the Java collections that hash it (a `HashSet` of features).
impl HashCode for Identifier {
    fn hash_code(&self) -> i32 {
        Identifier::hash_code(self)
    }
}

/// `identifier.indexOf(separator)` and the strings either side of it. A surrogate is never
/// found here, where Java can find one inside a supplementary character. Either way the answer
/// is the same: splitting there leaves a lone surrogate in the namespace or the path, which
/// neither allows, and without a split the whole string, supplementary character and all, isn't
/// a valid path.
fn split_at_separator(identifier: &str, separator: u16) -> Option<(&str, &str)> {
    let separator = char::from_u32(u32::from(separator))?;
    let index = identifier.find(separator)?;
    Some((
        &identifier[..index],
        &identifier[index + separator.len_utf8()..],
    ))
}

#[cfg(test)]
mod tests {
    use super::Identifier;
    use crate::golden::{Golden, encode_fields};

    encode_fields!(Identifier {
        namespace: String,
        path: String
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.resources.Identifier");
        golden.pending("CODEC", "waits for DataFixerUpper's codecs");
        golden.check_codec("STREAM_CODEC", || Identifier::STREAM_CODEC);
        golden.pending("ERROR_INVALID", "waits for brigadier");
        golden.pending(
            "read(Ljava/lang/String;)Lcom/mojang/serialization/DataResult;",
            "waits for DataFixerUpper's DataResult",
        );
        golden.pending(
            "read(Lcom/mojang/brigadier/StringReader;)Lnet/minecraft/resources/Identifier;",
            "waits for brigadier",
        );
        golden.pending(
            "readNonEmpty(Lcom/mojang/brigadier/StringReader;)Lnet/minecraft/resources/Identifier;",
            "waits for brigadier",
        );
        golden.pending(
            "withPath(Ljava/util/function/UnaryOperator;)Lnet/minecraft/resources/Identifier;",
            "vanilla can't generate UnaryOperators",
        );
        golden.pending(
            "resolveAgainst(Ljava/nio/file/Path;)Ljava/nio/file/Path;",
            "comes with resource loading",
        );
        golden.field("NAMESPACE_SEPARATOR", Identifier::NAMESPACE_SEPARATOR);
        golden.field(
            "DEFAULT_NAMESPACE",
            Identifier::DEFAULT_NAMESPACE.to_owned(),
        );
        golden.field("REALMS_NAMESPACE", Identifier::REALMS_NAMESPACE.to_owned());
        golden.field(
            "ALLOWED_NAMESPACE_CHARACTERS",
            Identifier::ALLOWED_NAMESPACE_CHARACTERS.to_owned(),
        );
        golden.check(
            "fromNamespaceAndPath(Ljava/lang/String;Ljava/lang/String;)Lnet/minecraft/resources/Identifier;",
            |namespace: String, path: String| Identifier::from_namespace_and_path(&namespace, &path),
        );
        golden.check(
            "parse(Ljava/lang/String;)Lnet/minecraft/resources/Identifier;",
            |identifier: String| Identifier::parse(&identifier),
        );
        golden.check(
            "withDefaultNamespace(Ljava/lang/String;)Lnet/minecraft/resources/Identifier;",
            |path: String| Identifier::with_default_namespace(&path),
        );
        golden.check(
            "tryParse(Ljava/lang/String;)Lnet/minecraft/resources/Identifier;",
            |identifier: String| Identifier::try_parse(&identifier),
        );
        golden.check(
            "tryBuild(Ljava/lang/String;Ljava/lang/String;)Lnet/minecraft/resources/Identifier;",
            |namespace: String, path: String| Identifier::try_build(&namespace, &path),
        );
        golden.check(
            "bySeparator(Ljava/lang/String;C)Lnet/minecraft/resources/Identifier;",
            |identifier: String, separator| Identifier::by_separator(&identifier, separator),
        );
        golden.check(
            "tryBySeparator(Ljava/lang/String;C)Lnet/minecraft/resources/Identifier;",
            |identifier: String, separator| Identifier::try_by_separator(&identifier, separator),
        );
        golden.check_method("getPath()Ljava/lang/String;", |this: &mut Identifier| {
            this.get_path().to_owned()
        });
        golden.check_method(
            "getNamespace()Ljava/lang/String;",
            |this: &mut Identifier| this.get_namespace().to_owned(),
        );
        golden.check_method(
            "withPath(Ljava/lang/String;)Lnet/minecraft/resources/Identifier;",
            |this: &mut Identifier, new_path: String| this.with_path(&new_path),
        );
        golden.check_method(
            "withPrefix(Ljava/lang/String;)Lnet/minecraft/resources/Identifier;",
            |this: &mut Identifier, prefix: String| this.with_prefix(&prefix),
        );
        golden.check_method(
            "withSuffix(Ljava/lang/String;)Lnet/minecraft/resources/Identifier;",
            |this: &mut Identifier, suffix: String| this.with_suffix(&suffix),
        );
        golden.check_method("toString()Ljava/lang/String;", |this: &mut Identifier| {
            this.to_string()
        });
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut Identifier, o: Identifier| *this == o,
        );
        golden.check_method("hashCode()I", |this: &mut Identifier| this.hash_code());
        golden.check_method(
            "compareTo(Lnet/minecraft/resources/Identifier;)I",
            |this: &mut Identifier, o: Identifier| this.compare_to(&o),
        );
        golden.check_method(
            "toDebugFileName()Ljava/lang/String;",
            |this: &mut Identifier| this.to_debug_file_name(),
        );
        golden.check_method(
            "toLanguageKey()Ljava/lang/String;",
            |this: &mut Identifier| this.to_language_key(),
        );
        golden.check_method(
            "toShortLanguageKey()Ljava/lang/String;",
            |this: &mut Identifier| this.to_short_language_key(),
        );
        golden.check_method(
            "toShortString()Ljava/lang/String;",
            |this: &mut Identifier| this.to_short_string(),
        );
        golden.check_method(
            "toLanguageKey(Ljava/lang/String;)Ljava/lang/String;",
            |this: &mut Identifier, prefix: String| this.to_language_key_prefix(&prefix),
        );
        golden.check_method(
            "toLanguageKey(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            |this: &mut Identifier, prefix: String, suffix: String| {
                this.to_language_key_prefix_suffix(&prefix, &suffix)
            },
        );
        golden.check(
            "isAllowedInIdentifier(C)Z",
            Identifier::is_allowed_in_identifier,
        );
        golden.check("isValidPath(Ljava/lang/String;)Z", |path: String| {
            Identifier::is_valid_path(&path)
        });
        golden.check(
            "isValidNamespace(Ljava/lang/String;)Z",
            |namespace: String| Identifier::is_valid_namespace(&namespace),
        );
        golden.check("validPathChar(C)Z", Identifier::valid_path_char);
        golden.finish();
    }
}
