//! net/minecraft/nbt/NbtIo.java
//!
//! Not ported yet: what reads and writes files (`read(Path)`, `write(CompoundTag, Path)`, the
//! compressed ones and `parseCompressed`), which come with the client's saves, and `parse`, which
//! streams to a `StreamTagVisitor` and comes with the visitors.
//!
//! Java wraps an `IOException` a tag's load throws in a `ReportedNbtException` with a crash
//! report: they're all panics here.

use crate::java::io::data_input::DataInput;
use crate::java::io::data_output::DataOutput;
use crate::java::io::data_output_stream::DataOutputStream;
use crate::net::minecraft::nbt::compound_tag::CompoundTag;
use crate::net::minecraft::nbt::end_tag::EndTag;
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::nbt::string_tag::StringTag;
use crate::net::minecraft::nbt::tag::Tag;
use crate::net::minecraft::nbt::tag_types::TagTypes;
use crate::net::minecraft::util::delegate_data_output::DelegateDataOutput;

pub struct NbtIo;

impl NbtIo {
    /// `read(DataInput)`: a root compound, with no quota to speak of.
    pub fn read(input: &mut dyn DataInput) -> CompoundTag {
        NbtIo::read_accounted(input, &mut NbtAccounter::unlimited_heap())
    }

    /// `read(DataInput, NbtAccounter)`: a root compound, panicking for any other tag.
    pub fn read_accounted(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> CompoundTag {
        match NbtIo::read_unnamed_tag(input, accounter) {
            Tag::Compound(compound_tag) => compound_tag,
            _ => panic!("Root tag must be a named compound tag"),
        }
    }

    /// A root compound with an empty name, writing too long a string as an empty one:
    /// `writeUnnamedTagWithFallback` of a compound, whose id isn't 0.
    pub fn write(tag: &CompoundTag, output: &mut dyn DataOutput) {
        let mut output = StringFallbackDataOutput::new(output);
        output.write_byte(i32::from(tag.get_id()));
        output.write_utf("");
        tag.write(&mut output);
    }

    /// A tag the way a packet holds one: its id, then the tag, without a name. An id of 0 is an
    /// `EndTag`, which reads nothing more.
    pub fn read_any_tag(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> Tag {
        let r#type = input.read_byte();
        if r#type == 0 {
            Tag::End(EndTag::INSTANCE)
        } else {
            NbtIo::read_tag_safe(input, accounter, r#type)
        }
    }

    pub fn write_any_tag(tag: &Tag, output: &mut dyn DataOutput) {
        output.write_byte(i32::from(tag.get_id()));
        if tag.get_id() != 0 {
            tag.write(output);
        }
    }

    /// A tag the way a file holds one: its id, an empty name, then the tag.
    pub fn write_unnamed_tag(tag: &Tag, output: &mut dyn DataOutput) {
        output.write_byte(i32::from(tag.get_id()));
        if tag.get_id() != 0 {
            output.write_utf("");
            tag.write(output);
        }
    }

    /// Like `write_unnamed_tag`, writing any string too long for modified UTF-8 as an empty one.
    pub fn write_unnamed_tag_with_fallback(tag: &Tag, output: &mut dyn DataOutput) {
        NbtIo::write_unnamed_tag(tag, &mut StringFallbackDataOutput::new(output));
    }

    /// A tag after its id and a name, whatever the name is.
    pub fn read_unnamed_tag(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> Tag {
        let r#type = input.read_byte();
        if r#type == 0 {
            return Tag::End(EndTag::INSTANCE);
        }

        StringTag::skip_string(input);
        NbtIo::read_tag_safe(input, accounter, r#type)
    }

    fn read_tag_safe(input: &mut dyn DataInput, accounter: &mut NbtAccounter, r#type: i8) -> Tag {
        TagTypes::get_type(i32::from(r#type)).load(input, accounter)
    }
}

/// `NbtIo.StringFallbackDataOutput`: an output that writes a string too long for modified UTF-8
/// as an empty one. Java catches the `UTFDataFormatException` its parent's `writeUTF` throws, and
/// logs it; a `writeUTF` only throws that for too long a string, before it writes anything, so
/// this one checks the length first.
pub struct StringFallbackDataOutput<'a> {
    parent: DelegateDataOutput<'a>,
}

impl<'a> StringFallbackDataOutput<'a> {
    pub fn new(parent: &'a mut dyn DataOutput) -> StringFallbackDataOutput<'a> {
        StringFallbackDataOutput {
            parent: DelegateDataOutput::new(parent),
        }
    }
}

impl DataOutput for StringFallbackDataOutput<'_> {
    fn write(&mut self, b: &[i8]) {
        self.parent.write(b);
    }

    fn write_range(&mut self, b: &[i8], off: i32, len: i32) {
        self.parent.write_range(b, off, len);
    }

    fn write_byte(&mut self, v: i32) {
        self.parent.write_byte(v);
    }

    fn write_short(&mut self, v: i32) {
        self.parent.write_short(v);
    }

    fn write_int(&mut self, v: i32) {
        self.parent.write_int(v);
    }

    fn write_long(&mut self, v: i64) {
        self.parent.write_long(v);
    }

    fn write_float(&mut self, v: f32) {
        self.parent.write_float(v);
    }

    fn write_double(&mut self, v: f64) {
        self.parent.write_double(v);
    }

    fn write_utf(&mut self, s: &str) {
        if DataOutputStream::utf_len(s) > 65535 {
            self.parent.write_utf("");
        } else {
            self.parent.write_utf(s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NbtIo;
    use crate::golden::Golden;
    use crate::io::netty::buffer::byte_buf::ByteBuf;
    use crate::io::netty::buffer::byte_buf_input_stream::ByteBufInputStream;
    use crate::io::netty::buffer::byte_buf_output_stream::ByteBufOutputStream;
    use crate::net::minecraft::nbt::compound_tag::CompoundTag;
    use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
    use crate::net::minecraft::nbt::tag::Tag;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.NbtIo");
        for name in [
            "read(Ljava/nio/file/Path;)Lnet/minecraft/nbt/CompoundTag;",
            "readCompressed(Ljava/io/InputStream;Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/CompoundTag;",
            "readCompressed(Ljava/nio/file/Path;Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/CompoundTag;",
            "write(Lnet/minecraft/nbt/CompoundTag;Ljava/nio/file/Path;)V",
            "writeCompressed(Lnet/minecraft/nbt/CompoundTag;Ljava/io/OutputStream;)V",
            "writeCompressed(Lnet/minecraft/nbt/CompoundTag;Ljava/nio/file/Path;)V",
            "parseCompressed(Ljava/io/InputStream;Lnet/minecraft/nbt/StreamTagVisitor;Lnet/minecraft/nbt/NbtAccounter;)V",
            "parseCompressed(Ljava/nio/file/Path;Lnet/minecraft/nbt/StreamTagVisitor;Lnet/minecraft/nbt/NbtAccounter;)V",
        ] {
            golden.pending(
                name,
                "reads and writes files, which come with the client's saves",
            );
        }
        golden.pending(
            "parse(Ljava/io/DataInput;Lnet/minecraft/nbt/StreamTagVisitor;Lnet/minecraft/nbt/NbtAccounter;)V",
            "comes with the stream visitors",
        );
        golden.check_mut(
            "read(Ljava/io/DataInput;)Lnet/minecraft/nbt/CompoundTag;",
            |input: &mut ByteBuf| NbtIo::read(&mut ByteBufInputStream::new(input)),
        );
        golden.check_mut(
            "read(Ljava/io/DataInput;Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/CompoundTag;",
            |input: &mut ByteBuf, accounter: &mut NbtAccounter| {
                NbtIo::read_accounted(&mut ByteBufInputStream::new(input), accounter)
            },
        );
        golden.check_mut(
            "write(Lnet/minecraft/nbt/CompoundTag;Ljava/io/DataOutput;)V",
            |tag: &mut CompoundTag, output: &mut ByteBuf| {
                NbtIo::write(tag, &mut ByteBufOutputStream::new(output))
            },
        );
        golden.check_mut(
            "readAnyTag(Ljava/io/DataInput;Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/Tag;",
            |input: &mut ByteBuf, accounter: &mut NbtAccounter| {
                NbtIo::read_any_tag(&mut ByteBufInputStream::new(input), accounter)
            },
        );
        golden.check_mut(
            "writeAnyTag(Lnet/minecraft/nbt/Tag;Ljava/io/DataOutput;)V",
            |tag: &mut Tag, output: &mut ByteBuf| {
                NbtIo::write_any_tag(tag, &mut ByteBufOutputStream::new(output))
            },
        );
        golden.check_mut(
            "writeUnnamedTag(Lnet/minecraft/nbt/Tag;Ljava/io/DataOutput;)V",
            |tag: &mut Tag, output: &mut ByteBuf| {
                NbtIo::write_unnamed_tag(tag, &mut ByteBufOutputStream::new(output))
            },
        );
        golden.check_mut(
            "writeUnnamedTagWithFallback(Lnet/minecraft/nbt/Tag;Ljava/io/DataOutput;)V",
            |tag: &mut Tag, output: &mut ByteBuf| {
                NbtIo::write_unnamed_tag_with_fallback(tag, &mut ByteBufOutputStream::new(output))
            },
        );
        golden.check_mut(
            "readUnnamedTag(Ljava/io/DataInput;Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/Tag;",
            |input: &mut ByteBuf, accounter: &mut NbtAccounter| {
                NbtIo::read_unnamed_tag(&mut ByteBufInputStream::new(input), accounter)
            },
        );
        golden.finish();
    }
}
