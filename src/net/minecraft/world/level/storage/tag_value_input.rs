//! net/minecraft/world/level/storage/TagValueInput.java, with ValueInput.java
//!
//! What a block entity loads from (`loadAdditional`): a compound tag read through the registries'
//! ops, each value by name, a value of the wrong kind reported (`ProblemReporter`) and taken as
//! missing, and a codec's error reported with its partial value if it has one. `ValueInput` has
//! one implementation the client reads with, so it's this struct. Not ported yet: `child`,
//! `childOrEmpty`, the lists (`childrenList`, `list`, `listOrEmpty`), `keySet`, `getString`,
//! `getFloatOr`, `getDoubleOr` and `lookup`, which come with their callers.

use crate::com::mojang::serialization::codec::BoxedCodec;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::nbt::compound_tag::CompoundTag;
use crate::net::minecraft::nbt::nbt_ops::NbtOps;
use crate::net::minecraft::nbt::numeric_tag::NumericTag;
use crate::net::minecraft::nbt::tag::Tag;
use crate::net::minecraft::resources::registry_ops::RegistryOps;
use crate::net::minecraft::util::problem_reporter::ProblemReporter;

pub struct TagValueInput<'a> {
    problem_reporter: &'a dyn ProblemReporter,
    holders: &'a RegistryAccess,
    input: &'a CompoundTag,
}

impl<'a> TagValueInput<'a> {
    /// `create(ProblemReporter, HolderLookup.Provider, CompoundTag)`.
    pub fn create(
        problem_reporter: &'a dyn ProblemReporter,
        holders: &'a RegistryAccess,
        tag: &'a CompoundTag,
    ) -> TagValueInput<'a> {
        TagValueInput {
            problem_reporter,
            holders,
            input: tag,
        }
    }

    /// `read(String, Codec)`: the value, or none where the name is missing; a value the codec
    /// can't read is reported, its partial value kept if it has one.
    pub fn read<T: Clone>(&self, name: &str, codec: &BoxedCodec<T>) -> Option<T> {
        let tag = self.input.get(name)?;
        let ops = RegistryOps::create(&NbtOps::INSTANCE, self.holders);
        match codec.parse(&ops, tag) {
            DataResult::Success(success) => Some(success.into_value()),
            DataResult::Error(error) => {
                self.problem_reporter.report(format!(
                    "Failed to decode value '{tag}' from field '{name}': {}",
                    error.message()
                ));
                error.partial_value().cloned()
            }
        }
    }

    // A value that's a number, else reported and missing.
    fn get_numeric_tag(&self, name: &str) -> Option<&'a dyn NumericTag> {
        let tag = self.input.get(name)?;
        match tag.as_numeric_tag() {
            Some(numeric_tag) => Some(numeric_tag),
            None => {
                self.problem_reporter.report(format!(
                    "Expected field '{name}' to contain number, but got {}",
                    tag.get_type().get_name()
                ));
                None
            }
        }
    }

    pub fn get_boolean_or(&self, name: &str, default_value: bool) -> bool {
        self.get_numeric_tag(name)
            .map_or(default_value, |tag| tag.byte_value() != 0)
    }

    pub fn get_byte_or(&self, name: &str, default_value: i8) -> i8 {
        self.get_numeric_tag(name)
            .map_or(default_value, |tag| tag.byte_value())
    }

    /// `getShortOr`, which Java returns as an int.
    pub fn get_short_or(&self, name: &str, default_value: i16) -> i32 {
        i32::from(
            self.get_numeric_tag(name)
                .map_or(default_value, |tag| tag.short_value()),
        )
    }

    pub fn get_int(&self, name: &str) -> Option<i32> {
        self.get_numeric_tag(name).map(|tag| tag.int_value())
    }

    pub fn get_int_or(&self, name: &str, default_value: i32) -> i32 {
        self.get_numeric_tag(name)
            .map_or(default_value, |tag| tag.int_value())
    }

    pub fn get_float_or(&self, name: &str, default_value: f32) -> f32 {
        self.get_numeric_tag(name)
            .map_or(default_value, |tag| tag.float_value())
    }

    pub fn get_long_or(&self, name: &str, default_value: i64) -> i64 {
        self.get_numeric_tag(name)
            .map_or(default_value, |tag| tag.long_value())
    }

    /// `getIntArray`: an int array's values, else (another kind, reported) none.
    pub fn get_int_array(&self, name: &str) -> Option<Vec<i32>> {
        match self.input.get(name)? {
            Tag::IntArray(tag) => Some(tag.get_as_int_array().to_vec()),
            other => {
                self.problem_reporter.report(format!(
                    "Expected field '{name}' to contain value of type INT[], but got {}",
                    other.get_type().get_name()
                ));
                None
            }
        }
    }
}
