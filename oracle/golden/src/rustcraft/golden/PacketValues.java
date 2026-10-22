package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.mojang.authlib.GameProfile;
import io.netty.buffer.Unpooled;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationHandler;
import java.lang.reflect.Method;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Proxy;
import java.lang.reflect.RecordComponent;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Random;
import java.util.Set;
import java.util.UUID;
import java.util.function.Function;
import java.util.stream.Stream;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistrySynchronization;
import net.minecraft.nbt.Tag;
import net.minecraft.network.ConnectionProtocol;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.PacketListener;
import net.minecraft.network.ProtocolInfo;
import net.minecraft.network.protocol.Packet;
import net.minecraft.network.protocol.PacketFlow;
import net.minecraft.network.protocol.PacketType;
import net.minecraft.network.protocol.common.ClientboundClearDialogPacket;
import net.minecraft.network.protocol.common.ClientboundCustomPayloadPacket;
import net.minecraft.network.protocol.common.ClientboundCustomReportDetailsPacket;
import net.minecraft.network.protocol.common.ClientboundKeepAlivePacket;
import net.minecraft.network.protocol.common.ClientboundPingPacket;
import net.minecraft.network.protocol.common.ClientboundPostEffectsPacket;
import net.minecraft.network.protocol.common.ClientboundResourcePackPopPacket;
import net.minecraft.network.protocol.common.ClientboundStoreCookiePacket;
import net.minecraft.network.protocol.common.ClientboundTransferPacket;
import net.minecraft.network.protocol.common.CommonPacketTypes;
import net.minecraft.network.protocol.common.ServerboundClientInformationPacket;
import net.minecraft.network.protocol.common.ServerboundCustomClickActionPacket;
import net.minecraft.network.protocol.common.ServerboundCustomPayloadPacket;
import net.minecraft.network.protocol.common.ServerboundKeepAlivePacket;
import net.minecraft.network.protocol.common.ServerboundPongPacket;
import net.minecraft.network.protocol.common.ServerboundResourcePackPacket;
import net.minecraft.network.protocol.common.custom.BrandPayload;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.network.protocol.common.custom.DiscardedPayload;
import net.minecraft.network.protocol.configuration.ClientboundCodeOfConductPacket;
import net.minecraft.network.protocol.configuration.ClientboundFinishConfigurationPacket;
import net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket;
import net.minecraft.network.protocol.configuration.ClientboundResetChatPacket;
import net.minecraft.network.protocol.configuration.ClientboundSelectKnownPacks;
import net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket;
import net.minecraft.network.protocol.configuration.ConfigurationPacketTypes;
import net.minecraft.network.protocol.configuration.ServerboundAcceptCodeOfConductPacket;
import net.minecraft.network.protocol.configuration.ServerboundFinishConfigurationPacket;
import net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks;
import net.minecraft.network.protocol.cookie.ClientboundCookieRequestPacket;
import net.minecraft.network.protocol.cookie.CookiePacketTypes;
import net.minecraft.network.protocol.cookie.ServerboundCookieResponsePacket;
import net.minecraft.network.protocol.handshake.ClientIntent;
import net.minecraft.network.protocol.handshake.ClientIntentionPacket;
import net.minecraft.network.protocol.handshake.HandshakePacketTypes;
import net.minecraft.network.protocol.login.ClientboundCustomQueryPacket;
import net.minecraft.network.protocol.login.ClientboundHelloPacket;
import net.minecraft.network.protocol.login.ClientboundLoginCompressionPacket;
import net.minecraft.network.protocol.login.ClientboundLoginFinishedPacket;
import net.minecraft.network.protocol.login.LoginPacketTypes;
import net.minecraft.network.protocol.login.ServerboundCustomQueryAnswerPacket;
import net.minecraft.network.protocol.login.ServerboundHelloPacket;
import net.minecraft.network.protocol.login.ServerboundKeyPacket;
import net.minecraft.network.protocol.login.ServerboundLoginAcknowledgedPacket;
import net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload;
import net.minecraft.network.protocol.login.custom.CustomQueryPayload;
import net.minecraft.network.protocol.login.custom.DiscardedQueryAnswerPayload;
import net.minecraft.network.protocol.login.custom.DiscardedQueryPayload;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.level.ClientInformation;
import net.minecraft.server.level.ParticleStatus;
import net.minecraft.server.packs.repository.KnownPack;
import net.minecraft.world.entity.HumanoidArm;
import net.minecraft.world.entity.player.ChatVisiblity;
import net.minecraft.world.flag.FeatureFlag;
import net.minecraft.world.flag.FeatureFlagSet;
import net.minecraft.world.flag.FeatureFlags;

/**
 * Arguments and values of the protocols. A PacketFlow, ConnectionProtocol or ClientIntent is its constant's name, and a
 * PacketType {@code [flow, id]} like any record. A packet is its fields: a record's components, or the fields another
 * class's public constructor sets, in that order (read by reflection), and a value declared as a Packet (a protocol
 * codec's) is {@code [type, fields]}, since it can be any of the protocol's. A protocol's template is its details,
 * {@code [id, flow, [[type, networkId], …]]}, and a bound protocol {@code [id, flow]}: ProtocolSuite gives its codec cases.
 * The custom queries' payloads are records like packets, and a payload declared as its interface is encoded by its own
 * class. Each argument is its own copy of a packet or payload, made of copies of its fields: a record compares an array
 * component by reference, which a JSON value can't show. A custom packet payload declared as its interface can be any of
 * several classes, so it's {@code [class, fields]}, the class's simple name first ({@code ["BrandPayload", ["vanilla"]]}).
 * The records packets hold (ClientInformation, KnownPack) are their components like any record.
 *
 * <p>A packet listener records what's called on it: it starts as {@code []}, and its {@code after} slot is the calls,
 * each {@code [method, argument…]}, so a packet's {@code handle} shows which handler it calls with what.
 */
final class PacketValues {
    static final Values.Generator PACKET_FLOW = new CoreValues.Constants<>(PacketFlow.class);
    static final Values.Generator CONNECTION_PROTOCOL = new CoreValues.Constants<>(ConnectionProtocol.class);
    static final Values.Generator CLIENT_INTENT = new CoreValues.Constants<>(ClientIntent.class);
    static final Values.Generator CHAT_VISIBLITY = new CoreValues.Constants<>(ChatVisiblity.class);
    static final Values.Generator HUMANOID_ARM = new CoreValues.Constants<>(HumanoidArm.class);
    static final Values.Generator PARTICLE_STATUS = new CoreValues.Constants<>(ParticleStatus.class);
    static final Values.Generator RESOURCE_PACK_ACTION = new CoreValues.Constants<>(ServerboundResourcePackPacket.Action.class);

    private static final Values.Generator INTS = Values.generator(int.class);
    private static final Values.Generator LONGS = Values.generator(long.class);
    private static final Values.Generator STRINGS = Values.generator(String.class);
    private static final Values.Generator BOOLEANS = Values.generator(boolean.class);
    private static final Values.Generator BYTE_ARRAYS = Values.generator(byte[].class);
    private static final Values.Generator UUIDS = Values.generator(UUID.class);
    private static final Values.Generator IDENTIFIERS = CoreValues.IDENTIFIER;

    // The types the ported protocols register, and each one's id with the other flow.
    private static final List<PacketType<?>> PORTED_TYPES = List.of(
        HandshakePacketTypes.CLIENT_INTENTION, LoginPacketTypes.CLIENTBOUND_CUSTOM_QUERY, LoginPacketTypes.CLIENTBOUND_LOGIN_FINISHED,
        LoginPacketTypes.CLIENTBOUND_HELLO, LoginPacketTypes.CLIENTBOUND_LOGIN_COMPRESSION, LoginPacketTypes.CLIENTBOUND_LOGIN_DISCONNECT,
        LoginPacketTypes.SERVERBOUND_CUSTOM_QUERY_ANSWER, LoginPacketTypes.SERVERBOUND_HELLO, LoginPacketTypes.SERVERBOUND_KEY,
        LoginPacketTypes.SERVERBOUND_LOGIN_ACKNOWLEDGED, CookiePacketTypes.CLIENTBOUND_COOKIE_REQUEST, CookiePacketTypes.SERVERBOUND_COOKIE_RESPONSE,
        CommonPacketTypes.SERVERBOUND_CLIENT_INFORMATION, CommonPacketTypes.SERVERBOUND_CUSTOM_CLICK_ACTION, CommonPacketTypes.SERVERBOUND_CUSTOM_PAYLOAD,
        CommonPacketTypes.SERVERBOUND_KEEP_ALIVE,
        CommonPacketTypes.SERVERBOUND_PONG, CommonPacketTypes.SERVERBOUND_RESOURCE_PACK, ConfigurationPacketTypes.SERVERBOUND_ACCEPT_CODE_OF_CONDUCT,
        ConfigurationPacketTypes.SERVERBOUND_FINISH_CONFIGURATION, ConfigurationPacketTypes.SERVERBOUND_SELECT_KNOWN_PACKS,
        CommonPacketTypes.CLIENTBOUND_CLEAR_DIALOG, CommonPacketTypes.CLIENTBOUND_CUSTOM_PAYLOAD, CommonPacketTypes.CLIENTBOUND_KEEP_ALIVE,
        CommonPacketTypes.CLIENTBOUND_PING, CommonPacketTypes.CLIENTBOUND_POST_EFFECTS, CommonPacketTypes.CLIENTBOUND_RESOURCE_PACK_POP,
        CommonPacketTypes.CLIENTBOUND_STORE_COOKIE, CommonPacketTypes.CLIENTBOUND_TRANSFER, ConfigurationPacketTypes.CLIENTBOUND_CODE_OF_CONDUCT,
        ConfigurationPacketTypes.CLIENTBOUND_FINISH_CONFIGURATION, ConfigurationPacketTypes.CLIENTBOUND_REGISTRY_DATA,
        ConfigurationPacketTypes.CLIENTBOUND_RESET_CHAT, ConfigurationPacketTypes.CLIENTBOUND_SELECT_KNOWN_PACKS,
        CommonPacketTypes.CLIENTBOUND_CUSTOM_REPORT_DETAILS, ConfigurationPacketTypes.CLIENTBOUND_UPDATE_ENABLED_FEATURES
    );
    static final Values.Generator PACKET_TYPE = new Values.Simple(
        PORTED_TYPES.stream()
            .flatMap(type -> Stream.of(type, new PacketType<>(type.flow().getOpposite(), type.id())))
            .map(type -> (Object)type)
            .toList(),
        random -> new PacketType<>((PacketFlow)PACKET_FLOW.random(random), (Identifier)IDENTIFIERS.instantiate(IDENTIFIERS.random(random))),
        type -> Values.encode(type, PacketType.class)
    );

    // The protocol version a release's handshake sends, a snapshot's (and with the snapshot bit), and any ints.
    private static final Values.Generator PROTOCOL_VERSIONS = new Values.Simple(
        List.of(777, 338, 0, -1, 0x40000000 | 338, Integer.MIN_VALUE, Integer.MAX_VALUE), INTS::random, INTS::encode
    );
    // Addresses, and around readUtf(255)'s limit in characters of one, two and four bytes.
    private static final Values.Generator HOST_NAMES = strings(
        "localhost", "127.0.0.1", "", "mc.example.com.", "a".repeat(255), "a".repeat(256), "é".repeat(255), "é".repeat(256),
        "😀".repeat(127) + "a", "😀".repeat(128)
    );
    // The default port, the unsigned short's bounds, and past them, which writeShort cuts to 16 bits.
    private static final Values.Generator PORTS = new Values.Simple(
        List.of(25565, 0, 65535, 32767, 32768, 65536, -1, Integer.MIN_VALUE, Integer.MAX_VALUE), INTS::random, INTS::encode
    );
    // An offline server's empty id, and around readUtf(20)'s limit.
    private static final Values.Generator SERVER_IDS = strings("", "a".repeat(20), "a".repeat(21), "😀".repeat(10), "😀".repeat(10) + "a");
    // Byte arrays, and the length of an RSA public key's encoding and of a challenge.
    private static final Values.Generator KEYS = new JomlValues.Mutable<>(
        Stream.concat(BYTE_ARRAYS.edges().stream().map(bytes -> (byte[])bytes), Stream.of(new byte[162], new byte[4])).toList(),
        random -> (byte[])BYTE_ARRAYS.random(random), byte[]::clone, bytes -> (JsonArray)BYTE_ARRAYS.encode(bytes)
    );

    // Byte arrays, and around a cookie's limit of 5120 bytes.
    private static final Values.Generator COOKIES = new JomlValues.Mutable<>(
        Stream.concat(BYTE_ARRAYS.edges().stream().map(bytes -> (byte[])bytes), Stream.of(new byte[5120], new byte[5121])).toList(),
        random -> (byte[])BYTE_ARRAYS.random(random), byte[]::clone, bytes -> (JsonArray)BYTE_ARRAYS.encode(bytes)
    );

    // A client's language, and around readUtf(16)'s limit in characters of one, two and four bytes.
    private static final Values.Generator LANGUAGES = strings(
        "en_us", "", "pt_br", "a".repeat(16), "a".repeat(17), "é".repeat(16), "😀".repeat(8), "😀".repeat(8) + "a"
    );
    // Render distances, and past a byte, which writeByte cuts to 8 bits and readByte reads signed.
    private static final Values.Generator VIEW_DISTANCES = new Values.Simple(
        List.of(2, 4, 12, 32, 0, -1, 127, 128, 255, 256, Integer.MIN_VALUE, Integer.MAX_VALUE), INTS::random, INTS::encode
    );
    // The skin's parts that show (every part is 127), and past a byte, which readUnsignedByte reads unsigned.
    private static final Values.Generator MODEL_CUSTOMISATIONS = new Values.Simple(
        List.of(0, 127, 1, 64, 128, 255, 256, -1, Integer.MIN_VALUE, Integer.MAX_VALUE), INTS::random, INTS::encode
    );
    // The vanilla pack, a mod's, and around STRING_UTF8's limit of 32767.
    private static final Values.Generator PACK_NAMESPACES = strings("minecraft", "fabric", "", "Minecraft", "a".repeat(32767), "a".repeat(32768));
    private static final Values.Generator PACK_IDS = strings("core", "", "😀".repeat(16383) + "a", "😀".repeat(16384));
    private static final Values.Generator PACK_VERSIONS = strings("26.3", "", "1.0.0", "a".repeat(32767), "a".repeat(32768));
    // A client's brand, and around readUtf()'s limit of 32767.
    private static final Values.Generator BRANDS = strings("vanilla", "fabric", "", "a".repeat(32767), "a".repeat(32768), "😀".repeat(16384));
    // The scenario's marker, the brand's (which a DiscardedPayload can have too), and any identifiers.
    private static final Values.Generator PAYLOAD_IDS = new Values.Simple(
        Stream.concat(
            Stream.of(Identifier.fromNamespaceAndPath("rustcraft", "scenario_start"), Identifier.withDefaultNamespace("brand"), Identifier.withDefaultNamespace("register")),
            IDENTIFIERS.edges().stream()
        ).toList(),
        IDENTIFIERS::random,
        IDENTIFIERS::encode
    );

    // A transfer's host, and around readUtf()'s limit of 32767 in characters of one byte and four.
    private static final Values.Generator TRANSFER_HOSTS = strings(
        "localhost", "127.0.0.1", "", "mc.example.com", "a".repeat(32767), "a".repeat(32768), "😀".repeat(16383) + "a", "😀".repeat(16384)
    );
    // A server's code of conduct, and around STRING_UTF8's limit of 32767.
    private static final Values.Generator CODES_OF_CONDUCT = strings(
        "Be kind.", "", "Line one\nLine two", "a".repeat(32767), "a".repeat(32768), "😀".repeat(16383) + "a", "😀".repeat(16384)
    );
    // A resource pack to pop, or all of them.
    private static final Values.Generator OPTIONAL_UUIDS = Codecs.optionalOf(UUIDS);
    // A ClientboundPostEffectsPacket's shaders.
    private static final Values.Generator IDENTIFIER_LISTS = Codecs.listOf(IDENTIFIERS);
    // A ClientboundUpdateEnabledFeaturesPacket's features (see featureSets).
    static final Values.Generator FEATURE_SETS = Codecs.hashSetOf(IDENTIFIERS, featureSets());
    // A ClientboundCustomReportDetailsPacket's details, with names around stringUtf8(128)'s limit in characters of one and
    // four bytes. Values around 4096 are among reportDetails' maps, not the values' edges, so random maps don't draw them.
    static final Values.Generator REPORT_DETAILS = Codecs.hashMapOf(
        strings("Server", "", "Mod list", "a".repeat(128), "a".repeat(129), "😀".repeat(64), "😀".repeat(63) + "a", "😀".repeat(64) + "a"),
        strings("Example", "", "Line one\nLine two", "1.0.0"),
        reportDetails()
    );

    private static final Map<Class<?>, Values.Generator> PACKETS = new LinkedHashMap<>();
    // The generators of the payloads packets hold, by their class and by their interface, and of the other records they
    // hold.
    private static final Map<Class<?>, Values.Generator> PAYLOADS = new HashMap<>();
    // The fields each packet class that isn't a record is encoded as.
    private static final Map<Class<?>, List<Field>> FIELDS = new LinkedHashMap<>();
    // The select_known_packs packets' packs.
    private static final Values.Generator KNOWN_PACK_LISTS;
    // A ClientboundRegistryDataPacket's entries.
    private static final Values.Generator REGISTRY_ENTRY_LISTS;

    static {
        // The only payloads vanilla reads: what it doesn't know, kept as the query's identifier and nothing else.
        Values.Generator answers = values(DiscardedQueryAnswerPayload.class, fields -> DiscardedQueryAnswerPayload.INSTANCE, List.of());
        Values.Generator queries = values(DiscardedQueryPayload.class, fields -> new DiscardedQueryPayload((Identifier)fields[0]), List.of(), IDENTIFIERS);
        PAYLOADS.put(DiscardedQueryAnswerPayload.class, answers);
        // A packet's answer may be null.
        PAYLOADS.put(CustomQueryAnswerPayload.class, Codecs.nullableOf(answers));
        PAYLOADS.put(DiscardedQueryPayload.class, queries);
        PAYLOADS.put(CustomQueryPayload.class, queries);
        Values.Generator brands = values(BrandPayload.class, fields -> new BrandPayload((String)fields[0]), List.of(), BRANDS);
        Values.Generator discarded = values(DiscardedPayload.class, fields -> new DiscardedPayload((Identifier)fields[0]), List.of(), PAYLOAD_IDS);
        PAYLOADS.put(BrandPayload.class, brands);
        PAYLOADS.put(DiscardedPayload.class, discarded);
        PAYLOADS.put(CustomPacketPayload.class, new AnyPayload(List.of(brands, discarded)));
        PAYLOADS.put(
            CustomPacketPayload.Type.class, values(CustomPacketPayload.Type.class, fields -> new CustomPacketPayload.Type<>((Identifier)fields[0]), List.of(), PAYLOAD_IDS)
        );
        Values.Generator clientInformation = values(
            ClientInformation.class,
            fields -> new ClientInformation(
                (String)fields[0], (Integer)fields[1], (ChatVisiblity)fields[2], (Boolean)fields[3], (Integer)fields[4], (HumanoidArm)fields[5],
                (Boolean)fields[6], (Boolean)fields[7], (ParticleStatus)fields[8]
            ),
            List.of(), LANGUAGES, VIEW_DISTANCES, CHAT_VISIBLITY, BOOLEANS, MODEL_CUSTOMISATIONS, HUMANOID_ARM, BOOLEANS, BOOLEANS, PARTICLE_STATUS
        );
        PAYLOADS.put(ClientInformation.class, clientInformation);
        Values.Generator knownPacks = values(
            KnownPack.class, fields -> new KnownPack((String)fields[0], (String)fields[1], (String)fields[2]), List.of(), PACK_NAMESPACES, PACK_IDS,
            PACK_VERSIONS
        );
        PAYLOADS.put(KnownPack.class, knownPacks);
        KNOWN_PACK_LISTS = knownPackLists(knownPacks);
        // A registry's element: its identifier, and its data, or none when the client has it in a known pack.
        Values.Generator registryEntries = values(
            RegistrySynchronization.PackedRegistryEntry.class,
            fields -> new RegistrySynchronization.PackedRegistryEntry((Identifier)fields[0], tagPayload(fields[1])), List.of(), IDENTIFIERS,
            NbtValues.OPTIONAL_TAGS
        );
        PAYLOADS.put(RegistrySynchronization.PackedRegistryEntry.class, registryEntries);
        REGISTRY_ENTRY_LISTS = Codecs.listOf(registryEntries);

        packet(
            ClientIntentionPacket.class, fields -> new ClientIntentionPacket((Integer)fields[0], (String)fields[1], (Integer)fields[2], (ClientIntent)fields[3]),
            List.of(), PROTOCOL_VERSIONS, HOST_NAMES, PORTS, CLIENT_INTENT
        );
        packet(
            ServerboundHelloPacket.class, fields -> new ServerboundHelloPacket((String)fields[0], (UUID)fields[1]), List.of(),
            ProfileValues.PLAYER_NAMES, UUIDS
        );
        packet(
            ClientboundHelloPacket.class,
            fields -> new ClientboundHelloPacket((String)fields[0], (byte[])fields[1], (byte[])fields[2], (Boolean)fields[3]),
            List.of("serverId", "publicKey", "challenge", "shouldAuthenticate"), SERVER_IDS, KEYS, KEYS, BOOLEANS
        );
        packet(
            ClientboundLoginFinishedPacket.class,
            fields -> new ClientboundLoginFinishedPacket((GameProfile)fields[0], (UUID)fields[1]), List.of(), ProfileValues.GAME_PROFILES, UUIDS
        );
        packet(ServerboundLoginAcknowledgedPacket.class, fields -> ServerboundLoginAcknowledgedPacket.INSTANCE, List.of());
        packet(
            ClientboundLoginCompressionPacket.class, fields -> new ClientboundLoginCompressionPacket((Integer)fields[0]),
            List.of("compressionThreshold"), INTS
        );
        packet(
            ServerboundKeyPacket.class, fields -> keyPacket((byte[])fields[0], (byte[])fields[1]), List.of("keybytes", "encryptedChallenge"),
            KEYS, KEYS
        );
        packet(
            ServerboundCustomQueryAnswerPacket.class,
            fields -> new ServerboundCustomQueryAnswerPacket((Integer)fields[0], (CustomQueryAnswerPayload)fields[1]), List.of(), INTS,
            PAYLOADS.get(CustomQueryAnswerPayload.class)
        );
        packet(
            ClientboundCustomQueryPacket.class, fields -> new ClientboundCustomQueryPacket((Integer)fields[0], (CustomQueryPayload)fields[1]),
            List.of(), INTS, queries
        );
        packet(ClientboundCookieRequestPacket.class, fields -> new ClientboundCookieRequestPacket((Identifier)fields[0]), List.of(), IDENTIFIERS);
        packet(
            ServerboundCookieResponsePacket.class, fields -> new ServerboundCookieResponsePacket((Identifier)fields[0], (byte[])fields[1]),
            List.of(), IDENTIFIERS, Codecs.nullableOf(COOKIES)
        );
        packet(
            ClientboundStoreCookiePacket.class, fields -> new ClientboundStoreCookiePacket((Identifier)fields[0], (byte[])fields[1]), List.of(),
            IDENTIFIERS, COOKIES
        );
        packet(
            ServerboundClientInformationPacket.class, fields -> new ServerboundClientInformationPacket((ClientInformation)fields[0]), List.of(),
            clientInformation
        );
        packet(
            ServerboundCustomPayloadPacket.class, fields -> new ServerboundCustomPayloadPacket((CustomPacketPayload)fields[0]), List.of(),
            PAYLOADS.get(CustomPacketPayload.class)
        );
        packet(ServerboundFinishConfigurationPacket.class, fields -> ServerboundFinishConfigurationPacket.INSTANCE, List.of());
        packet(ServerboundKeepAlivePacket.class, fields -> new ServerboundKeepAlivePacket((Long)fields[0]), List.of("id"), LONGS);
        packet(ServerboundPongPacket.class, fields -> new ServerboundPongPacket((Integer)fields[0]), List.of("id"), INTS);
        packet(
            ServerboundResourcePackPacket.class,
            fields -> new ServerboundResourcePackPacket((UUID)fields[0], (ServerboundResourcePackPacket.Action)fields[1]), List.of(), UUIDS,
            RESOURCE_PACK_ACTION
        );
        packet(
            ServerboundSelectKnownPacks.class, fields -> new ServerboundSelectKnownPacks(knownPackList(fields[0])), List.of(), KNOWN_PACK_LISTS
        );
        packet(ServerboundAcceptCodeOfConductPacket.class, fields -> ServerboundAcceptCodeOfConductPacket.INSTANCE, List.of());
        // A dialog's custom action: its identifier, and any tag or none (an EndTag writes like none).
        packet(
            ServerboundCustomClickActionPacket.class, fields -> new ServerboundCustomClickActionPacket((Identifier)fields[0], tagPayload(fields[1])),
            List.of(), IDENTIFIERS, NbtValues.OPTIONAL_TAGS
        );

        // The configuration protocol's clientbound packets.
        packet(ClientboundFinishConfigurationPacket.class, fields -> ClientboundFinishConfigurationPacket.INSTANCE, List.of());
        packet(ClientboundResetChatPacket.class, fields -> ClientboundResetChatPacket.INSTANCE, List.of());
        packet(ClientboundClearDialogPacket.class, fields -> ClientboundClearDialogPacket.INSTANCE, List.of());
        packet(ClientboundKeepAlivePacket.class, fields -> new ClientboundKeepAlivePacket((Long)fields[0]), List.of("id"), LONGS);
        packet(ClientboundPingPacket.class, fields -> new ClientboundPingPacket((Integer)fields[0]), List.of("id"), INTS);
        packet(
            ClientboundSelectKnownPacks.class, fields -> new ClientboundSelectKnownPacks(knownPackList(fields[0])), List.of(), KNOWN_PACK_LISTS
        );
        packet(ClientboundResourcePackPopPacket.class, fields -> new ClientboundResourcePackPopPacket(uuid(fields[0])), List.of(), OPTIONAL_UUIDS);
        packet(
            ClientboundTransferPacket.class, fields -> new ClientboundTransferPacket((String)fields[0], (Integer)fields[1]), List.of(),
            TRANSFER_HOSTS, PORTS
        );
        packet(ClientboundCodeOfConductPacket.class, fields -> new ClientboundCodeOfConductPacket((String)fields[0]), List.of(), CODES_OF_CONDUCT);
        packet(
            ClientboundPostEffectsPacket.class, fields -> new ClientboundPostEffectsPacket(identifierList(fields[0])), List.of(), IDENTIFIER_LISTS
        );
        packet(
            ClientboundCustomPayloadPacket.class, fields -> new ClientboundCustomPayloadPacket((CustomPacketPayload)fields[0]), List.of(),
            PAYLOADS.get(CustomPacketPayload.class)
        );
        packet(
            ClientboundRegistryDataPacket.class, fields -> new ClientboundRegistryDataPacket(registryKey(fields[0]), registryEntryList(fields[1])),
            List.of(), CoreValues.RESOURCE_KEY, REGISTRY_ENTRY_LISTS
        );
        packet(
            ClientboundUpdateEnabledFeaturesPacket.class, fields -> new ClientboundUpdateEnabledFeaturesPacket(identifierSet(fields[0])), List.of(),
            FEATURE_SETS
        );
        packet(
            ClientboundCustomReportDetailsPacket.class, fields -> new ClientboundCustomReportDetailsPacket(stringMap(fields[0])), List.of(),
            REPORT_DETAILS
        );
    }

    private PacketValues() {
    }

    static Values.Generator generator(final Class<?> type) {
        if (type == PacketFlow.class) {
            return PACKET_FLOW;
        }
        if (type == ConnectionProtocol.class) {
            return CONNECTION_PROTOCOL;
        }
        if (type == ClientIntent.class) {
            return CLIENT_INTENT;
        }
        if (type == ChatVisiblity.class) {
            return CHAT_VISIBLITY;
        }
        if (type == HumanoidArm.class) {
            return HUMANOID_ARM;
        }
        if (type == ParticleStatus.class) {
            return PARTICLE_STATUS;
        }
        if (type == ServerboundResourcePackPacket.Action.class) {
            return RESOURCE_PACK_ACTION;
        }
        if (type == PacketType.class) {
            return PACKET_TYPE;
        }
        if (type.isInterface() && PacketListener.class.isAssignableFrom(type)) {
            return new Listener(type);
        }
        return PACKETS.containsKey(type) ? PACKETS.get(type) : PAYLOADS.get(type);
    }

    /**
     * The parameterized arguments with a generator of their own here: the select_known_packs packets' packs, a
     * ClientboundPostEffectsPacket's shaders, a ClientboundResourcePackPopPacket's pack, a registry's entries, the enabled
     * features and a server's report details.
     */
    static Values.Generator generator(final ParameterizedType type) {
        Object element = type.getActualTypeArguments()[0];
        if (type.getRawType() == Set.class && element == Identifier.class) {
            return FEATURE_SETS;
        }
        if (type.getRawType() == Map.class && element == String.class && type.getActualTypeArguments()[1] == String.class) {
            return REPORT_DETAILS;
        }
        if (type.getRawType() == List.class) {
            if (element == KnownPack.class) {
                return KNOWN_PACK_LISTS;
            }
            if (element == Identifier.class) {
                return IDENTIFIER_LISTS;
            }
            if (element == RegistrySynchronization.PackedRegistryEntry.class) {
                return REGISTRY_ENTRY_LISTS;
            }
        }
        return type.getRawType() == Optional.class && element == UUID.class ? OPTIONAL_UUIDS : null;
    }

    static boolean canEncode(final Class<?> type) {
        return generator(type) != null || type == Packet.class || ProtocolInfo.class.isAssignableFrom(type)
            || ProtocolInfo.DetailsProvider.class.isAssignableFrom(type);
    }

    static JsonElement encode(final Object value, final Class<?> type) {
        if (type == Packet.class) {
            JsonArray array = new JsonArray();
            array.add(Values.encode(((Packet<?>)value).type(), PacketType.class));
            array.add(encodeFields(value));
            return array;
        }
        return switch (value) {
            case Enum<?> constant -> CoreValues.encode(constant);
            case Packet<?> packet -> encodeFields(packet);
            // Declared as their interface, so encoded by their own class.
            case CustomQueryAnswerPayload payload -> Values.encode(payload, payload.getClass());
            case CustomQueryPayload payload -> Values.encode(payload, payload.getClass());
            // Declared as their interface, which several classes implement.
            case CustomPacketPayload payload -> AnyPayload.tagged(payload);
            case ProtocolInfo<?> protocol -> {
                JsonArray array = new JsonArray();
                array.add(Values.encode(protocol.id(), ConnectionProtocol.class));
                array.add(Values.encode(protocol.flow(), PacketFlow.class));
                yield array;
            }
            case ProtocolInfo.DetailsProvider provider -> encodeDetails(provider.details());
            default -> throw new IllegalArgumentException("no encoder for " + value.getClass().getName());
        };
    }

    /** The generator of the packets of this type, or null when its class has none. */
    static Values.Generator packetsOfType(final PacketType<?> type) {
        for (Values.Generator packets : PACKETS.values()) {
            if (((Packet<?>)packets.edges().get(0)).type().equals(type)) {
                return packets;
            }
        }
        return null;
    }

    /** Any of the generators' packets, each declared as a Packet: {@code [type, fields]}. */
    static Values.Generator anyOf(final List<Values.Generator> packets) {
        return new Values.Simple(
            packets.stream().flatMap(generator -> generator.edges().stream()).toList(),
            random -> packets.get(random.nextInt(packets.size())).random(random),
            packet -> encode(packet, Packet.class)
        );
    }

    // A packet class's values, which are the packets themselves.
    private static void packet(
        final Class<?> type, final Function<Object[], Object> constructor, final List<String> fieldNames, final Values.Generator... fields
    ) {
        PACKETS.put(type, values(type, constructor, fieldNames, fields));
    }

    // A class's values: edges put each field's edges in its place among the first edges, and random ones are made of
    // random fields. Each argument is a copy made of copies of the value's fields, so no two share an array. A record's
    // fields are its components; another class names its own, which are read and encoded by reflection.
    private static Values.Generator values(
        final Class<?> type, final Function<Object[], Object> constructor, final List<String> fieldNames, final Values.Generator... fields
    ) {
        Values.Generator tuple = Codecs.tupleOf(fields);
        Function<Object, Object> make = spec -> constructor.apply(((List<?>)tuple.instantiate(spec)).toArray());
        List<Object> edges = fields.length == 0 ? List.of(constructor.apply(new Object[0])) : tuple.edges().stream().map(make).toList();
        List<Field> declared = fieldNames.stream().map(name -> field(type, name)).toList();
        if (!type.isRecord()) {
            FIELDS.put(type, declared);
        }
        Function<Object, Object> copy = value -> {
            Object[] copies = fieldsOf(value);
            for (int i = 0; i < fields.length; i++) {
                copies[i] = fields[i].instantiate(copies[i]);
            }
            return constructor.apply(copies);
        };
        Values.Generator values = new JomlValues.Mutable<>(
            edges, random -> make.apply(tuple.random(random)), copy, value -> (JsonArray)encodeFields(value)
        );
        return fields.length > 0 ? values : new Only(values);
    }

    // A packet without fields, whose one value is every value there is.
    private record Only(Values.Generator values) implements Values.Generator {
        @Override
        public List<Object> edges() {
            return this.values.edges();
        }

        @Override
        public Object random(final Random random) {
            return this.values.edges().get(0);
        }

        @Override
        public JsonElement encode(final Object spec) {
            return this.values.encode(spec);
        }

        @Override
        public boolean exhaustive() {
            return true;
        }
    }

    private static JsonElement encodeFields(final Object packet) {
        Class<?> type = packet.getClass();
        if (type.isRecord()) {
            return Values.encode(packet, type);
        }
        JsonArray array = new JsonArray();
        for (Field field : FIELDS.get(type)) {
            try {
                array.add(Values.encode(field.get(packet), field.getType()));
            } catch (IllegalAccessException e) {
                throw new IllegalStateException(e);
            }
        }
        return array;
    }

    // The values of its fields, in the order its constructor takes them.
    private static Object[] fieldsOf(final Object value) {
        Class<?> type = value.getClass();
        try {
            if (type.isRecord()) {
                RecordComponent[] components = type.getRecordComponents();
                Object[] values = new Object[components.length];
                for (int i = 0; i < components.length; i++) {
                    values[i] = components[i].getAccessor().invoke(value);
                }
                return values;
            }
            List<Field> declared = FIELDS.get(type);
            Object[] values = new Object[declared.size()];
            for (int i = 0; i < values.length; i++) {
                values[i] = declared.get(i).get(value);
            }
            return values;
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }

    // Lists of packs like Codecs.listOf's, and the most ServerboundSelectKnownPacks' codec takes (64) and one more.
    private static Values.Generator knownPackLists(final Values.Generator packs) {
        Values.Generator lists = Codecs.listOf(packs);
        List<Object> edges = new ArrayList<>(lists.edges());
        for (int count : new int[]{64, 65}) {
            List<Object> list = new ArrayList<>();
            for (int i = 0; i < count; i++) {
                list.add(new KnownPack("minecraft", "pack" + i, "26.3"));
            }
            edges.add(list);
        }
        return new Values.Generator() {
            @Override
            public List<Object> edges() {
                return edges;
            }

            @Override
            public Object random(final Random random) {
                return lists.random(random);
            }

            @Override
            public JsonElement encode(final Object spec) {
                return lists.encode(spec);
            }

            @Override
            public Object instantiate(final Object spec) {
                return lists.instantiate(spec);
            }
        };
    }

    @SuppressWarnings("unchecked")
    private static List<KnownPack> knownPackList(final Object list) {
        return (List<KnownPack>)list;
    }

    @SuppressWarnings("unchecked")
    private static Optional<Tag> tagPayload(final Object payload) {
        return (Optional<Tag>)payload;
    }

    @SuppressWarnings("unchecked")
    private static Optional<UUID> uuid(final Object id) {
        return (Optional<UUID>)id;
    }

    @SuppressWarnings("unchecked")
    private static List<Identifier> identifierList(final Object list) {
        return (List<Identifier>)list;
    }

    @SuppressWarnings("unchecked")
    private static Set<Identifier> identifierSet(final Object set) {
        return (Set<Identifier>)set;
    }

    @SuppressWarnings("unchecked")
    private static Map<String, String> stringMap(final Object map) {
        return (Map<String, String>)map;
    }

    /**
     * The server's features (FeatureFlagRegistry.toNames makes a HashSet), every flag's, each flag alone, and none;
     * identifiers whose hashCodes are equal, which share a bin, added either way round (equal sets that write their
     * identifiers in other orders); 12, 13, 24 and 25 of them, where the table grows; and 13 with 10 taken out again.
     */
    @SuppressWarnings("unchecked")
    private static List<HashSet<Object>> featureSets() {
        List<HashSet<Object>> sets = new ArrayList<>();
        sets.add((HashSet<Object>)(Set<?>)FeatureFlags.REGISTRY.toNames(FeatureFlags.VANILLA_SET));
        sets.add((HashSet<Object>)(Set<?>)FeatureFlags.REGISTRY.toNames(FeatureFlags.REGISTRY.allFlags()));
        for (FeatureFlag flag : new FeatureFlag[]{FeatureFlags.TRADE_REBALANCE, FeatureFlags.REDSTONE_EXPERIMENTS, FeatureFlags.MINECART_IMPROVEMENTS}) {
            sets.add((HashSet<Object>)(Set<?>)FeatureFlags.REGISTRY.toNames(FeatureFlagSet.of(flag)));
        }
        sets.add((HashSet<Object>)(Set<?>)FeatureFlags.REGISTRY.toNames(FeatureFlagSet.of()));
        List<Object> colliding = new ArrayList<>();
        for (String path : collidingPaths(3).subList(0, 6)) {
            colliding.add(Identifier.withDefaultNamespace(path));
        }
        sets.add(Codecs.addAll(new HashSet<>(), colliding));
        sets.add(Codecs.addAll(new HashSet<>(), colliding.reversed()));
        sets.add(Codecs.addAll(new HashSet<>(), Stream.concat(Stream.of(Identifier.withDefaultNamespace("vanilla")), colliding.stream()).toList()));
        for (int count : new int[]{12, 13, 24, 25}) {
            sets.add(Codecs.addAll(new HashSet<>(), numbered(count)));
        }
        HashSet<Object> shrunk = Codecs.addAll(new HashSet<>(), numbered(13));
        numbered(10).forEach(shrunk::remove);
        sets.add(shrunk);
        return sets;
    }

    // minecraft:feature_0, minecraft:feature_1, …
    private static List<Object> numbered(final int count) {
        List<Object> identifiers = new ArrayList<>();
        for (int i = 0; i < count; i++) {
            identifiers.add(Identifier.withDefaultNamespace("feature_" + i));
        }
        return identifiers;
    }

    /**
     * Every path of {@code blocks} "ak" and "c-" blocks, whose hashCodes are all equal ({@code 31 * 'a' + 'k'} is
     * {@code 31 * 'c' + '-'}), like HashMapSuite's "Aa" and "BB" ones, which an identifier's path can't hold.
     */
    private static List<String> collidingPaths(final int blocks) {
        List<String> paths = new ArrayList<>(List.of(""));
        for (int i = 0; i < blocks; i++) {
            List<String> longer = new ArrayList<>();
            for (String path : paths) {
                longer.add(path + "ak");
                longer.add(path + "c-");
            }
            paths = longer;
        }
        return paths;
    }

    /**
     * A server's details; values around stringUtf8(4096)'s limit in characters of one and four bytes; 32 and 33 details
     * (the codec's limit); keys whose hashCodes are equal, which share a bin, put either way round (equal maps that write
     * their entries in other orders); 13, where the table grows; and 13 with 10 taken out again.
     */
    private static List<HashMap<Object, Object>> reportDetails() {
        List<HashMap<Object, Object>> maps = new ArrayList<>();
        maps.add(Codecs.putAll(new HashMap<>(), List.of("Server", "Server version", "Mod list"), List.of("Example", "1.0.0", "fabric, lithium")));
        for (String value : new String[]{"a".repeat(4096), "a".repeat(4097), "😀".repeat(2048), "😀".repeat(2047) + "a", "😀".repeat(2048) + "a"}) {
            maps.add(Codecs.putAll(new HashMap<>(), List.of("Server", "Mod list"), List.of(value, "fabric")));
        }
        for (int count : new int[]{13, 32, 33}) {
            maps.add(Codecs.putAll(new HashMap<>(), numberedDetails(count), List.of("value")));
        }
        List<String> colliding = HashMapSuite.colliding(3).subList(0, 6);
        maps.add(Codecs.putAll(new HashMap<>(), colliding, List.of("a", "b")));
        maps.add(Codecs.putAll(new HashMap<>(), colliding.reversed(), List.of("b", "a")));
        HashMap<Object, Object> shrunk = Codecs.putAll(new HashMap<>(), numberedDetails(13), List.of("value"));
        numberedDetails(10).forEach(shrunk::remove);
        maps.add(shrunk);
        return maps;
    }

    // "Detail 0", "Detail 1", …
    private static List<Object> numberedDetails(final int count) {
        List<Object> keys = new ArrayList<>();
        for (int i = 0; i < count; i++) {
            keys.add("Detail " + i);
        }
        return keys;
    }

    @SuppressWarnings("unchecked")
    private static ResourceKey<? extends Registry<?>> registryKey(final Object key) {
        return (ResourceKey<? extends Registry<?>>)key;
    }

    @SuppressWarnings("unchecked")
    private static List<RegistrySynchronization.PackedRegistryEntry> registryEntryList(final Object list) {
        return (List<RegistrySynchronization.PackedRegistryEntry>)list;
    }

    /**
     * Payloads declared as CustomPacketPayload, any of the generators': {@code [class, fields]}, by the class's simple
     * name, since a DiscardedPayload can have a BrandPayload's identifier.
     */
    private record AnyPayload(List<Values.Generator> payloads) implements Values.Generator {
        static JsonElement tagged(final CustomPacketPayload payload) {
            JsonArray array = new JsonArray();
            array.add(payload.getClass().getSimpleName());
            array.add(Values.encode(payload, payload.getClass()));
            return array;
        }

        @Override
        public List<Object> edges() {
            return this.payloads.stream().flatMap(generator -> generator.edges().stream()).toList();
        }

        @Override
        public Object random(final Random random) {
            return this.payloads.get(random.nextInt(this.payloads.size())).random(random);
        }

        @Override
        public JsonElement encode(final Object spec) {
            return tagged((CustomPacketPayload)spec);
        }

        @Override
        public Object instantiate(final Object spec) {
            return PAYLOADS.get(spec.getClass()).instantiate(spec);
        }
    }

    // A key packet as a server reads it, since its public constructor encrypts with Crypt.
    private static ServerboundKeyPacket keyPacket(final byte[] keybytes, final byte[] encryptedChallenge) {
        FriendlyByteBuf buffer = new FriendlyByteBuf(Unpooled.buffer());
        buffer.writeByteArray(keybytes);
        buffer.writeByteArray(encryptedChallenge);
        return ServerboundKeyPacket.STREAM_CODEC.decode(buffer);
    }

    private static JsonArray encodeDetails(final ProtocolInfo.Details details) {
        JsonArray packets = new JsonArray();
        details.listPackets((type, networkId) -> {
            JsonArray packet = new JsonArray();
            packet.add(Values.encode(type, PacketType.class));
            packet.add(networkId);
            packets.add(packet);
        });
        JsonArray array = new JsonArray();
        array.add(Values.encode(details.id(), ConnectionProtocol.class));
        array.add(Values.encode(details.flow(), PacketFlow.class));
        array.add(packets);
        return array;
    }

    private static Field field(final Class<?> type, final String name) {
        try {
            Field field = type.getDeclaredField(name);
            field.setAccessible(true);
            return field;
        } catch (NoSuchFieldException e) {
            throw new IllegalStateException(e);
        }
    }

    private static Values.Generator strings(final String... edges) {
        return new Values.Simple(Arrays.asList((Object[])edges), STRINGS::random, STRINGS::encode);
    }

    /**
     * A listener of the interface that records each call: {@code []} before, and the calls afterwards. A default method
     * runs as it is after it's recorded, and an abstract one returning a value throws, since no packet calls one.
     */
    private record Listener(Class<?> type) implements Values.Generator {
        @Override
        public List<Object> edges() {
            return List.of(this.type);
        }

        @Override
        public Object random(final Random random) {
            return this.type;
        }

        @Override
        public JsonElement encode(final Object spec) {
            return Proxy.isProxyClass(spec.getClass()) ? ((Recorder)Proxy.getInvocationHandler(spec)).calls.deepCopy() : new JsonArray();
        }

        @Override
        public Object instantiate(final Object spec) {
            return Proxy.newProxyInstance(this.type.getClassLoader(), new Class<?>[]{this.type}, new Recorder(new JsonArray()));
        }

        @Override
        public JsonElement after(final Object spec, final Object instance) {
            JsonArray calls = ((Recorder)Proxy.getInvocationHandler(instance)).calls;
            return calls.isEmpty() ? null : calls;
        }

        @Override
        public boolean exhaustive() {
            return true;
        }
    }

    private record Recorder(JsonArray calls) implements InvocationHandler {
        @Override
        public Object invoke(final Object proxy, final Method method, final Object[] args) throws Throwable {
            if (method.getDeclaringClass() == Object.class) {
                return switch (method.getName()) {
                    case "hashCode" -> System.identityHashCode(proxy);
                    case "equals" -> proxy == args[0];
                    default -> "Recorder" + this.calls;
                };
            }

            JsonArray call = new JsonArray();
            call.add(method.getName());
            Class<?>[] parameters = method.getParameterTypes();
            for (int i = 0; i < parameters.length; i++) {
                call.add(Values.encode(args[i], parameters[i]));
            }
            this.calls.add(call);
            if (method.isDefault()) {
                return InvocationHandler.invokeDefault(proxy, method, args);
            }
            if (method.getReturnType() != void.class) {
                throw new UnsupportedOperationException(method.getName());
            }
            return null;
        }
    }
}
