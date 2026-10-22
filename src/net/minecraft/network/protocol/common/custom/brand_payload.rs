//! net/minecraft/network/protocol/common/custom/BrandPayload.java

use std::any::Any;
use std::fmt;
use std::sync::LazyLock;

use crate::java::lang::object::HashCode;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::custom::custom_packet_payload::{
    self, CustomPacketPayload, Type,
};

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, BrandPayload> = LazyLock::new(|| {
    Box::new(custom_packet_payload::codec(
        BrandPayload::write,
        BrandPayload::from_friendly_byte_buf,
    ))
});
static TYPE: LazyLock<Type> = LazyLock::new(|| custom_packet_payload::create_type("brand"));

/// The name of the client's or the server's build, which each side sends the other after login:
/// `vanilla`, or a mod loader's.
#[derive(Clone, Debug, PartialEq)]
pub struct BrandPayload {
    brand: String,
}

impl BrandPayload {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, BrandPayload> =
        &STREAM_CODEC;
    pub const TYPE: &'static LazyLock<Type> = &TYPE;

    pub fn new(brand: String) -> BrandPayload {
        BrandPayload { brand }
    }

    /// Up to 32767 characters, and panics past them, where Java throws `DecoderException`.
    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> BrandPayload {
        BrandPayload::new(input.read_utf())
    }

    /// Panics past 32767 characters, where Java throws `EncoderException`.
    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_utf(&self.brand);
    }

    pub fn brand(&self) -> &str {
        &self.brand
    }
}

impl CustomPacketPayload for BrandPayload {
    fn r#type(&self) -> Type {
        (*BrandPayload::TYPE).clone()
    }

    /// A record's: another `BrandPayload` with an equal brand.
    fn equals(&self, other: &dyn CustomPacketPayload) -> bool {
        let other: &dyn Any = other;
        other
            .downcast_ref::<BrandPayload>()
            .is_some_and(|other| self == other)
    }

    /// A record's of one component: its hash.
    fn hash_code(&self) -> i32 {
        self.brand.hash_code()
    }
}

/// A record's: `BrandPayload[brand=vanilla]`.
impl fmt::Display for BrandPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BrandPayload[brand={}]", self.brand)
    }
}

#[cfg(test)]
mod tests {
    use super::BrandPayload;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::protocol::common::custom::custom_packet_payload::CustomPacketPayload;

    // A record: its components.
    encode_fields!(BrandPayload { brand: String });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.common.custom.BrandPayload");
        golden.check_codec("STREAM_CODEC", || BrandPayload::STREAM_CODEC);
        golden.field("TYPE", (*BrandPayload::TYPE).clone());
        golden.check("<init>(Ljava/lang/String;)V", BrandPayload::new);
        golden.check_method("brand()Ljava/lang/String;", |this: &mut BrandPayload| {
            this.brand().to_owned()
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/common/custom/CustomPacketPayload$Type;",
            |this: &mut BrandPayload| this.r#type(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut BrandPayload, other: BrandPayload| this.equals(&other),
        );
        golden.check_method("hashCode()I", |this: &mut BrandPayload| this.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut BrandPayload| {
            this.to_string()
        });
        golden.finish();
    }
}
