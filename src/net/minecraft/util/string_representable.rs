//! net/minecraft/util/StringRepresentable.java
//!
//! Only `getSerializedName`, the name a block state property writes its values by. Not ported
//! yet: the codecs (`fromEnum`, `fromValues`, `EnumCodec`, `keys`), which wait for
//! DataFixerUpper's.

/// A value with a name of its own for saving and sending, `north` for `Direction.NORTH`.
pub trait StringRepresentable {
    fn get_serialized_name(&self) -> &'static str;
}
