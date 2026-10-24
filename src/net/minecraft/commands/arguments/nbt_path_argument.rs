//! net/minecraft/commands/arguments/NbtPathArgument.java
//!
//! Only `parse`, which the NBT contents' path is read with, its nodes as what each holds and its
//! errors. What the nodes do with tags (`getTag`, `getOrCreateTag`, `setTag`, `removeTag`, the
//! predicates from their patterns) is the server's, and `NbtPath`'s methods using them and its
//! `CODEC` come with their callers. Java maps each node to where it ends in the path by identity,
//! so every `[]` (one shared `AllElementsNode`) keeps the last one's; here each node keeps its own.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::exceptions::simple_command_exception_type::SimpleCommandExceptionType;
use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::net::minecraft::nbt::compound_tag::CompoundTag;
use crate::net::minecraft::nbt::tag_parser::TagParser;
use crate::net::minecraft::network::chat::component::Component;

pub static ERROR_INVALID_NODE: LazyLock<SimpleCommandExceptionType> = LazyLock::new(|| {
    SimpleCommandExceptionType::new(Component::from(Component::translatable(
        "arguments.nbtpath.node.invalid",
    )))
});

const INDEX_MATCH_START: u16 = b'[' as u16;
const INDEX_MATCH_END: u16 = b']' as u16;
const KEY_MATCH_START: u16 = b'{' as u16;
const QUOTED_KEY_START: u16 = b'"' as u16;
const SINGLE_QUOTED_KEY_START: u16 = b'\'' as u16;
const SPACE: u16 = b' ' as u16;
const DOT: u16 = b'.' as u16;

/// A step of a path: Java's node classes.
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    /// `[]`.
    AllElements,
    /// `name`.
    CompoundChild(String),
    /// `[index]`.
    IndexedElement(i32),
    /// `[{pattern}]`.
    MatchElement(CompoundTag),
    /// `name{pattern}`.
    MatchObject(String, CompoundTag),
    /// `{pattern}`, first.
    MatchRootObject(CompoundTag),
}

/// `NbtPathArgument.NbtPath`: the path's text and its nodes, each with where it ends in the text.
#[derive(Clone, Debug, PartialEq)]
pub struct NbtPath {
    original: String,
    nodes: Vec<(Node, i32)>,
}

impl NbtPath {
    pub fn as_string(&self) -> &str {
        &self.original
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter().map(|(node, _)| node)
    }

    /// `of(String)`.
    pub fn of(string: &str) -> Result<NbtPath, CommandSyntaxException> {
        NbtPathArgument.parse(&mut StringReader::new(string))
    }
}

/// `toString`: the text.
impl fmt::Display for NbtPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.original)
    }
}

pub struct NbtPathArgument;

impl NbtPathArgument {
    pub fn nbt_path() -> NbtPathArgument {
        NbtPathArgument
    }

    /// Nodes up to a space or the end, separated by dots (none before `[` or `{`). Panics where
    /// Java throws `StringIndexOutOfBoundsException`: a path ending in `[`.
    pub fn parse(&self, reader: &mut StringReader) -> Result<NbtPath, CommandSyntaxException> {
        let mut nodes = Vec::new();
        let start = reader.get_cursor();
        let mut first_node = true;
        while reader.can_read() && reader.peek() != SPACE {
            let node = NbtPathArgument::parse_node(reader, first_node)?;
            nodes.push((node, reader.get_cursor() - start));
            first_node = false;
            if reader.can_read() {
                let next = reader.peek();
                if next != SPACE && next != INDEX_MATCH_START && next != KEY_MATCH_START {
                    reader.expect(DOT)?;
                }
            }
        }
        Ok(NbtPath {
            original: reader.substring(start, reader.get_cursor()),
            nodes,
        })
    }

    fn parse_node(
        reader: &mut StringReader,
        first_node: bool,
    ) -> Result<Node, CommandSyntaxException> {
        match reader.peek() {
            QUOTED_KEY_START | SINGLE_QUOTED_KEY_START => {
                let name = reader.read_string()?;
                NbtPathArgument::read_object_node(reader, name)
            }
            INDEX_MATCH_START => {
                reader.skip();
                let next = reader.peek();
                if next == KEY_MATCH_START {
                    let pattern = TagParser::parse_compound_as_argument(reader)?;
                    reader.expect(INDEX_MATCH_END)?;
                    Ok(Node::MatchElement(pattern))
                } else if next == INDEX_MATCH_END {
                    reader.skip();
                    Ok(Node::AllElements)
                } else {
                    let index = reader.read_int()?;
                    reader.expect(INDEX_MATCH_END)?;
                    Ok(Node::IndexedElement(index))
                }
            }
            KEY_MATCH_START => {
                if !first_node {
                    return Err(ERROR_INVALID_NODE.create_with_context(reader));
                }
                let pattern = TagParser::parse_compound_as_argument(reader)?;
                Ok(Node::MatchRootObject(pattern))
            }
            _ => {
                let name = NbtPathArgument::read_unquoted_name(reader)?;
                NbtPathArgument::read_object_node(reader, name)
            }
        }
    }

    fn read_object_node(
        reader: &mut StringReader,
        name: String,
    ) -> Result<Node, CommandSyntaxException> {
        if name.is_empty() {
            return Err(ERROR_INVALID_NODE.create_with_context(reader));
        }
        if reader.can_read() && reader.peek() == KEY_MATCH_START {
            let pattern = TagParser::parse_compound_as_argument(reader)?;
            Ok(Node::MatchObject(name, pattern))
        } else {
            Ok(Node::CompoundChild(name))
        }
    }

    fn read_unquoted_name(reader: &mut StringReader) -> Result<String, CommandSyntaxException> {
        let start = reader.get_cursor();
        while reader.can_read() && NbtPathArgument::is_allowed_in_unquoted_name(reader.peek()) {
            reader.skip();
        }
        if reader.get_cursor() == start {
            return Err(ERROR_INVALID_NODE.create_with_context(reader));
        }
        Ok(reader.substring(start, reader.get_cursor()))
    }

    fn is_allowed_in_unquoted_name(c: u16) -> bool {
        !b" \"'[].{}"
            .iter()
            .any(|&disallowed| c == u16::from(disallowed))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{NbtPathArgument, Node};
    use crate::com::mojang::brigadier::string_reader::StringReader;
    use crate::golden::{Encode, Golden};

    fn node(node: &Node) -> Value {
        match node {
            Node::AllElements => json!(["AllElementsNode"]),
            Node::CompoundChild(name) => json!(["CompoundChildNode", name]),
            Node::IndexedElement(index) => json!(["IndexedElementNode", index]),
            Node::MatchElement(pattern) => json!(["MatchElementNode", pattern.encode()]),
            Node::MatchObject(name, pattern) => json!(["MatchObjectNode", name, pattern.encode()]),
            // Java's keeps only the pattern's predicate.
            Node::MatchRootObject(_) => json!(["MatchRootObjectNode"]),
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.commands.arguments.NbtPathArgument");
        golden.check("parse", |input: String| {
            let mut reader = StringReader::new(&input);
            match NbtPathArgument.parse(&mut reader) {
                Ok(path) => json!([
                    path.to_string(),
                    path.nodes().map(node).collect::<Vec<_>>(),
                    reader.get_cursor()
                ]),
                Err(exception) => json!(["error", exception.get_message(), reader.get_cursor()]),
            }
        });
        golden.finish();
    }
}
