//! net/minecraft/nbt/TagTypes.java

use crate::net::minecraft::nbt::tag_type::{self, TagType};

pub struct TagTypes;

impl TagTypes {
    /// The type of a tag's id, from `END` (0) to `LONG[]` (12), or a new invalid one.
    pub fn get_type(type_id: i32) -> TagType {
        match type_id {
            0 => TagType::End,
            1 => TagType::Byte,
            2 => TagType::Short,
            3 => TagType::Int,
            4 => TagType::Long,
            5 => TagType::Float,
            6 => TagType::Double,
            7 => TagType::ByteArray,
            8 => TagType::String,
            9 => TagType::List,
            10 => TagType::Compound,
            11 => TagType::IntArray,
            12 => TagType::LongArray,
            _ => tag_type::create_invalid(type_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TagTypes;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.TagTypes");
        golden.check("getType(I)Lnet/minecraft/nbt/TagType;", TagTypes::get_type);
        golden.finish();
    }
}
