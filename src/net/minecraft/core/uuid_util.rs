//! net/minecraft/core/UUIDUtil.java
//!
//! Not ported yet: the DataFixerUpper codecs (`CODEC`, `CODEC_SET`, `CODEC_LINKED_SET`,
//! `STRING_CODEC`, `AUTHLIB_CODEC` and `LENIENT_CODEC`) and `readUUID(Dynamic)`, which wait for
//! DataFixerUpper.

use std::sync::LazyLock;

use crate::com::mojang::authlib::game_profile::GameProfile;
use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::util::uuid::UUID;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, UUID> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        FriendlyByteBuf::write_uuid_to,
        FriendlyByteBuf::read_uuid_from,
    ))
});

pub struct UUIDUtil;

impl UUIDUtil {
    /// A UUID's two longs.
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, UUID> = &STREAM_CODEC;
    pub const UUID_BYTES: i32 = 16;
    const UUID_PREFIX_OFFLINE_PLAYER: &str = "OfflinePlayer:";

    /// The first two ints are the most significant long, high half first, and the last two the
    /// least. Panics for fewer than four, where Java throws `ArrayIndexOutOfBoundsException`.
    pub fn uuid_from_int_array(int_array: &[i32]) -> UUID {
        UUID::new(
            (i64::from(int_array[0]) << 32) | (i64::from(int_array[1]) & 4294967295),
            (i64::from(int_array[2]) << 32) | (i64::from(int_array[3]) & 4294967295),
        )
    }

    pub fn uuid_to_int_array(uuid: &UUID) -> Vec<i32> {
        let most_significant_bits = uuid.get_most_significant_bits();
        let least_significant_bits = uuid.get_least_significant_bits();
        UUIDUtil::least_most_to_int_array(most_significant_bits, least_significant_bits)
    }

    fn least_most_to_int_array(
        most_significant_bits: i64,
        least_significant_bits: i64,
    ) -> Vec<i32> {
        vec![
            (most_significant_bits >> 32) as i32,
            most_significant_bits as i32,
            (least_significant_bits >> 32) as i32,
            least_significant_bits as i32,
        ]
    }

    /// Both longs, big-endian.
    pub fn uuid_to_byte_array(uuid: &UUID) -> Vec<i8> {
        let mut bytes = uuid.get_most_significant_bits().to_be_bytes().to_vec();
        bytes.extend(uuid.get_least_significant_bits().to_be_bytes());
        bytes.into_iter().map(|byte| byte as i8).collect()
    }

    /// The UUID an offline server gives a player: version 3, from `OfflinePlayer:` and the name.
    pub fn create_offline_player_uuid(player_name: &str) -> UUID {
        let name = format!("{}{player_name}", UUIDUtil::UUID_PREFIX_OFFLINE_PLAYER);
        let bytes: Vec<i8> = name.bytes().map(|byte| byte as i8).collect();
        UUID::name_uuid_from_bytes(&bytes)
    }

    pub fn create_offline_profile(player_name: &str) -> GameProfile {
        let id = UUIDUtil::create_offline_player_uuid(player_name);
        GameProfile::new(id, player_name.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::UUIDUtil;
    use crate::golden::Golden;
    use crate::java::util::uuid::UUID;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.UUIDUtil");
        for name in [
            "CODEC",
            "CODEC_SET",
            "CODEC_LINKED_SET",
            "STRING_CODEC",
            "AUTHLIB_CODEC",
            "LENIENT_CODEC",
            "readUUID(Lcom/mojang/serialization/Dynamic;)Ljava/util/UUID;",
        ] {
            golden.pending(name, "waits for DataFixerUpper");
        }
        golden.check_codec("STREAM_CODEC", || UUIDUtil::STREAM_CODEC);
        golden.field("UUID_BYTES", UUIDUtil::UUID_BYTES);
        golden.check(
            "uuidFromIntArray([I)Ljava/util/UUID;",
            |int_array: Vec<i32>| UUIDUtil::uuid_from_int_array(&int_array),
        );
        golden.check("uuidToIntArray(Ljava/util/UUID;)[I", |uuid: UUID| {
            UUIDUtil::uuid_to_int_array(&uuid)
        });
        golden.check("uuidToByteArray(Ljava/util/UUID;)[B", |uuid: UUID| {
            UUIDUtil::uuid_to_byte_array(&uuid)
        });
        golden.check(
            "createOfflinePlayerUUID(Ljava/lang/String;)Ljava/util/UUID;",
            |player_name: String| UUIDUtil::create_offline_player_uuid(&player_name),
        );
        golden.check(
            "createOfflineProfile(Ljava/lang/String;)Lcom/mojang/authlib/GameProfile;",
            |player_name: String| UUIDUtil::create_offline_profile(&player_name),
        );
        golden.finish();
    }
}
