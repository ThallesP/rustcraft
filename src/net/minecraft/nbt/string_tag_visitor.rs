//! net/minecraft/nbt/StringTagVisitor.java
//!
//! Java's is a `TagVisitor` each tag's `accept` calls back; the visitors aren't ported yet, so
//! here it visits a `Tag` by matching on it (`visit`), and each tag's `toString` calls its own
//! `visit_*`. What it builds is SNBT: numbers with their suffix, strings quoted, a compound's keys
//! sorted and quoted unless they're plain names.

use crate::java::lang::double::Double;
use crate::java::lang::float::Float;
use crate::java::lang::string::JavaString;
use crate::net::minecraft::nbt::byte_array_tag::ByteArrayTag;
use crate::net::minecraft::nbt::byte_tag::ByteTag;
use crate::net::minecraft::nbt::compound_tag::CompoundTag;
use crate::net::minecraft::nbt::double_tag::DoubleTag;
use crate::net::minecraft::nbt::float_tag::FloatTag;
use crate::net::minecraft::nbt::int_array_tag::IntArrayTag;
use crate::net::minecraft::nbt::int_tag::IntTag;
use crate::net::minecraft::nbt::list_tag::ListTag;
use crate::net::minecraft::nbt::long_array_tag::LongArrayTag;
use crate::net::minecraft::nbt::long_tag::LongTag;
use crate::net::minecraft::nbt::short_tag::ShortTag;
use crate::net::minecraft::nbt::string_tag::StringTag;
use crate::net::minecraft::nbt::tag::Tag;

/// Writes tags as SNBT.
#[derive(Debug, Default)]
pub struct StringTagVisitor {
    builder: String,
}

impl StringTagVisitor {
    pub fn new() -> StringTagVisitor {
        StringTagVisitor::default()
    }

    pub fn build(self) -> String {
        self.builder
    }

    /// `tag.accept(this)`.
    pub fn visit(&mut self, tag: &Tag) {
        match tag {
            Tag::End(_) => self.visit_end(),
            Tag::Byte(tag) => self.visit_byte(tag),
            Tag::Short(tag) => self.visit_short(tag),
            Tag::Int(tag) => self.visit_int(tag),
            Tag::Long(tag) => self.visit_long(tag),
            Tag::Float(tag) => self.visit_float(tag),
            Tag::Double(tag) => self.visit_double(tag),
            Tag::ByteArray(tag) => self.visit_byte_array(tag),
            Tag::String(tag) => self.visit_string(tag),
            Tag::List(tag) => self.visit_list(tag),
            Tag::Compound(tag) => self.visit_compound(tag),
            Tag::IntArray(tag) => self.visit_int_array(tag),
            Tag::LongArray(tag) => self.visit_long_array(tag),
        }
    }

    pub fn visit_string(&mut self, tag: &StringTag) {
        StringTag::quote_and_escape_into(tag.value(), &mut self.builder);
    }

    pub fn visit_byte(&mut self, tag: &ByteTag) {
        self.builder.push_str(&tag.value().to_string());
        self.builder.push('b');
    }

    pub fn visit_short(&mut self, tag: &ShortTag) {
        self.builder.push_str(&tag.value().to_string());
        self.builder.push('s');
    }

    pub fn visit_int(&mut self, tag: &IntTag) {
        self.builder.push_str(&tag.value().to_string());
    }

    pub fn visit_long(&mut self, tag: &LongTag) {
        self.builder.push_str(&tag.value().to_string());
        self.builder.push('L');
    }

    pub fn visit_float(&mut self, tag: &FloatTag) {
        self.builder.push_str(&Float::to_string(tag.value()));
        self.builder.push('f');
    }

    pub fn visit_double(&mut self, tag: &DoubleTag) {
        self.builder.push_str(&Double::to_string(tag.value()));
        self.builder.push('d');
    }

    pub fn visit_byte_array(&mut self, tag: &ByteArrayTag) {
        self.builder.push_str("[B;");
        for (i, value) in tag.get_as_byte_array().iter().enumerate() {
            if i != 0 {
                self.builder.push(',');
            }
            self.builder.push_str(&value.to_string());
            self.builder.push('B');
        }
        self.builder.push(']');
    }

    pub fn visit_int_array(&mut self, tag: &IntArrayTag) {
        self.builder.push_str("[I;");
        for (i, value) in tag.get_as_int_array().iter().enumerate() {
            if i != 0 {
                self.builder.push(',');
            }
            self.builder.push_str(&value.to_string());
        }
        self.builder.push(']');
    }

    pub fn visit_long_array(&mut self, tag: &LongArrayTag) {
        self.builder.push_str("[L;");
        for (i, value) in tag.get_as_long_array().iter().enumerate() {
            if i != 0 {
                self.builder.push(',');
            }
            self.builder.push_str(&value.to_string());
            self.builder.push('L');
        }
        self.builder.push(']');
    }

    pub fn visit_list(&mut self, tag: &ListTag) {
        self.builder.push('[');
        for i in 0..tag.size() {
            if i != 0 {
                self.builder.push(',');
            }
            self.visit(tag.get(i));
        }
        self.builder.push(']');
    }

    /// Its entries sorted by key (`String.compareTo`).
    pub fn visit_compound(&mut self, tag: &CompoundTag) {
        self.builder.push('{');
        let mut entries: Vec<(&String, &Tag)> = tag.entry_set().collect();
        entries.sort_by(|(a, _), (b, _)| a.as_str().compare_to(b).cmp(&0));
        for (i, (key, value)) in entries.into_iter().enumerate() {
            if i != 0 {
                self.builder.push(',');
            }
            self.handle_key_escape(key);
            self.builder.push(':');
            self.visit(value);
        }
        self.builder.push('}');
    }

    /// A key as it is when it's a plain name (`[A-Za-z._]+[A-Za-z0-9._+-]*`) that isn't a
    /// boolean, and quoted otherwise.
    fn handle_key_escape(&mut self, input: &str) {
        let mut chars = input.chars();
        let unquoted = chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '.' || c == '_')
            && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '+' | '-'));
        if !input.eq_ignore_ascii_case("true") && !input.eq_ignore_ascii_case("false") && unquoted {
            self.builder.push_str(input);
        } else {
            StringTag::quote_and_escape_into(input, &mut self.builder);
        }
    }

    pub fn visit_end(&mut self) {
        self.builder.push_str("END");
    }
}
