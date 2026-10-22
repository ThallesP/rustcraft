package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.mojang.authlib.GameProfile;
import io.netty.buffer.Unpooled;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationHandler;
import java.lang.reflect.Method;
import java.lang.reflect.Proxy;
import java.lang.reflect.RecordComponent;
import java.util.Arrays;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.UUID;
import java.util.function.Function;
import java.util.stream.Stream;
import net.minecraft.network.ConnectionProtocol;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.PacketListener;
import net.minecraft.network.ProtocolInfo;
import net.minecraft.network.protocol.Packet;
import net.minecraft.network.protocol.PacketFlow;
import net.minecraft.network.protocol.PacketType;
import net.minecraft.network.protocol.common.ClientboundStoreCookiePacket;
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

/**
 * Arguments and values of the protocols. A PacketFlow, ConnectionProtocol or ClientIntent is its constant's name, and a
 * PacketType {@code [flow, id]} like any record. A packet is its fields: a record's components, or the fields another
 * class's public constructor sets, in that order (read by reflection), and a value declared as a Packet (a protocol
 * codec's) is {@code [type, fields]}, since it can be any of the protocol's. A protocol's template is its details,
 * {@code [id, flow, [[type, networkId], …]]}, and a bound protocol {@code [id, flow]}: ProtocolSuite gives its codec cases.
 * The custom queries' payloads are records like packets, and a payload declared as its interface is encoded by its own
 * class. Each argument is its own copy of a packet or payload, made of copies of its fields: a record compares an array
 * component by reference, which a JSON value can't show.
 *
 * <p>A packet listener records what's called on it: it starts as {@code []}, and its {@code after} slot is the calls,
 * each {@code [method, argument…]}, so a packet's {@code handle} shows which handler it calls with what.
 */
final class PacketValues {
    static final Values.Generator PACKET_FLOW = new CoreValues.Constants<>(PacketFlow.class);
    static final Values.Generator CONNECTION_PROTOCOL = new CoreValues.Constants<>(ConnectionProtocol.class);
    static final Values.Generator CLIENT_INTENT = new CoreValues.Constants<>(ClientIntent.class);

    private static final Values.Generator INTS = Values.generator(int.class);
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
        LoginPacketTypes.SERVERBOUND_LOGIN_ACKNOWLEDGED, CookiePacketTypes.CLIENTBOUND_COOKIE_REQUEST, CookiePacketTypes.SERVERBOUND_COOKIE_RESPONSE
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

    private static final Map<Class<?>, Values.Generator> PACKETS = new LinkedHashMap<>();
    // The generators of the payloads packets hold, by their class and by their interface.
    private static final Map<Class<?>, Values.Generator> PAYLOADS = new HashMap<>();
    // The fields each packet class that isn't a record is encoded as.
    private static final Map<Class<?>, List<Field>> FIELDS = new LinkedHashMap<>();

    static {
        // The only payloads vanilla reads: what it doesn't know, kept as the query's identifier and nothing else.
        Values.Generator answers = values(DiscardedQueryAnswerPayload.class, fields -> DiscardedQueryAnswerPayload.INSTANCE, List.of());
        Values.Generator queries = values(DiscardedQueryPayload.class, fields -> new DiscardedQueryPayload((Identifier)fields[0]), List.of(), IDENTIFIERS);
        PAYLOADS.put(DiscardedQueryAnswerPayload.class, answers);
        // A packet's answer may be null.
        PAYLOADS.put(CustomQueryAnswerPayload.class, Codecs.nullableOf(answers));
        PAYLOADS.put(DiscardedQueryPayload.class, queries);
        PAYLOADS.put(CustomQueryPayload.class, queries);

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
        if (type == PacketType.class) {
            return PACKET_TYPE;
        }
        if (type.isInterface() && PacketListener.class.isAssignableFrom(type)) {
            return new Listener(type);
        }
        return PACKETS.containsKey(type) ? PACKETS.get(type) : PAYLOADS.get(type);
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
