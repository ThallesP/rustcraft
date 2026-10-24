//! net/minecraft/server/ServerLinks.java
//!
//! Not ported yet: `findKnownType` and `untrust`, which only the server and the pause screen's
//! links read.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::datafixers::util::either::Either;
use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::net::uri::URI;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::util::by_id_map::{ByIdMap, OutOfBoundsStrategy};

/// The links a server lists for its players (the pause screen's "server links").
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ServerLinks {
    entries: Vec<Entry>,
}

static TYPE_STREAM_CODEC: StaticStreamCodec<ByteBuf, Either<KnownLinkType, Component>> =
    LazyLock::new(|| {
        Box::new(ByteBufCodecs::either(
            KnownLinkType::STREAM_CODEC,
            ComponentSerialization::TRUSTED_CONTEXT_FREE_STREAM_CODEC,
        ))
    });

static UNTRUSTED_LINKS_STREAM_CODEC: StaticStreamCodec<ByteBuf, Vec<UntrustedEntry>> =
    LazyLock::new(|| Box::new(UntrustedEntry::STREAM_CODEC.apply(ByteBufCodecs::list())));

impl ServerLinks {
    pub const TYPE_STREAM_CODEC: &'static StaticStreamCodec<
        ByteBuf,
        Either<KnownLinkType, Component>,
    > = &TYPE_STREAM_CODEC;
    pub const UNTRUSTED_LINKS_STREAM_CODEC: &'static StaticStreamCodec<
        ByteBuf,
        Vec<UntrustedEntry>,
    > = &UNTRUSTED_LINKS_STREAM_CODEC;

    /// `EMPTY`.
    pub fn empty() -> ServerLinks {
        ServerLinks::new(Vec::new())
    }

    pub fn new(entries: Vec<Entry>) -> ServerLinks {
        ServerLinks { entries }
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// `ServerLinks.Entry`: a link of a known kind or named by the server, checked as a URI.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub r#type: Either<KnownLinkType, Component>,
    pub link: URI,
}

impl Entry {
    /// The link's name: its kind's translation, or the server's text.
    pub fn display_name(&self) -> Component {
        match &self.r#type {
            Either::Left(known) => known.display_name(),
            Either::Right(name) => name.clone(),
        }
    }
}

/// `ServerLinks.KnownLinkType`: the kinds of link the client names itself.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KnownLinkType {
    BUG_REPORT,
    COMMUNITY_GUIDELINES,
    SUPPORT,
    STATUS,
    FEEDBACK,
    COMMUNITY,
    WEBSITE,
    FORUMS,
    NEWS,
    ANNOUNCEMENTS,
}

// By id, the first for any other.
static BY_ID: LazyLock<Box<dyn Fn(i32) -> KnownLinkType + Send + Sync>> = LazyLock::new(|| {
    Box::new(ByIdMap::continuous(
        KnownLinkType::id,
        &KnownLinkType::VALUES,
        OutOfBoundsStrategy::ZERO,
    ))
});

static KNOWN_LINK_TYPE_STREAM_CODEC: StaticStreamCodec<ByteBuf, KnownLinkType> =
    LazyLock::new(|| Box::new(ByteBufCodecs::id_mapper(BY_ID.as_ref(), KnownLinkType::id)));

impl KnownLinkType {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, KnownLinkType> =
        &KNOWN_LINK_TYPE_STREAM_CODEC;

    pub const VALUES: [KnownLinkType; 10] = [
        KnownLinkType::BUG_REPORT,
        KnownLinkType::COMMUNITY_GUIDELINES,
        KnownLinkType::SUPPORT,
        KnownLinkType::STATUS,
        KnownLinkType::FEEDBACK,
        KnownLinkType::COMMUNITY,
        KnownLinkType::WEBSITE,
        KnownLinkType::FORUMS,
        KnownLinkType::NEWS,
        KnownLinkType::ANNOUNCEMENTS,
    ];

    // The id and name each constant is made with.
    fn id(&self) -> i32 {
        *self as i32
    }

    fn name(&self) -> &'static str {
        [
            "report_bug",
            "community_guidelines",
            "support",
            "status",
            "feedback",
            "community",
            "website",
            "forums",
            "news",
            "announcements",
        ][*self as usize]
    }

    fn display_name(&self) -> Component {
        Component::translatable(&format!("known_server_link.{}", self.name())).into()
    }

    pub fn create(self, link: URI) -> Entry {
        Entry {
            r#type: Either::Left(self),
            link,
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for KnownLinkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// `ServerLinks.UntrustedEntry`: a link as the server sent it, not checked yet.
#[derive(Clone, Debug, PartialEq)]
pub struct UntrustedEntry {
    pub r#type: Either<KnownLinkType, Component>,
    pub link: String,
}

static UNTRUSTED_ENTRY_STREAM_CODEC: StaticStreamCodec<ByteBuf, UntrustedEntry> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite2(
            ServerLinks::TYPE_STREAM_CODEC,
            |entry: &UntrustedEntry| entry.r#type.clone(),
            ByteBufCodecs::STRING_UTF8,
            |entry: &UntrustedEntry| entry.link.clone(),
            |r#type, link| UntrustedEntry { r#type, link },
        ))
    });

impl UntrustedEntry {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, UntrustedEntry> =
        &UNTRUSTED_ENTRY_STREAM_CODEC;
}

/// `UntrustedEntry[type=…, link=…]`.
impl fmt::Display for UntrustedEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UntrustedEntry[type={}, link={}]",
            self.r#type, self.link
        )
    }
}
