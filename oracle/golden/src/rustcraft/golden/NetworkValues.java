package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import io.netty.buffer.AbstractByteBuf;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import java.io.ByteArrayOutputStream;
import java.lang.reflect.Field;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.List;
import java.util.Random;
import java.util.UUID;
import net.minecraft.core.BlockPos;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.Utf8String;
import net.minecraft.network.VarInt;
import net.minecraft.network.VarLong;

/**
 * Arguments and receivers of the network buffers. A ByteBuf is {@code [array, readerIndex, writerIndex,
 * markedReaderIndex, markedWriterIndex]}: its whole backing array as hex, whose length is the capacity, then its indexes.
 * A FriendlyByteBuf is the buffer it wraps. Each call gets a fresh heap buffer like Unpooled.buffer's with the same
 * array and indexes, holding fields the way packets write them: VarInts and VarLongs of every length, too long and cut
 * short, strings with their length (valid UTF-8 or not, longer than what's left, negative), and fixed-size fields.
 */
final class NetworkValues {
    private static final HexFormat HEX = HexFormat.of();
    private static final Field READER_INDEX = indexField("readerIndex");
    private static final Field WRITER_INDEX = indexField("writerIndex");
    private static final Field MARKED_READER_INDEX = indexField("markedReaderIndex");
    private static final Field MARKED_WRITER_INDEX = indexField("markedWriterIndex");

    static final Values.Generator BUFFER = new JomlValues.Mutable<>(
        bufferEdges(), NetworkValues::randomBuffer, NetworkValues::copy, NetworkValues::encodeBuffer
    );
    static final Values.Generator FRIENDLY_BUFFER = new JomlValues.Mutable<>(
        bufferEdges().stream().map(FriendlyByteBuf::new).toList(),
        random -> new FriendlyByteBuf(randomBuffer(random)),
        buffer -> new FriendlyByteBuf(copy(buffer.unwrap())),
        buffer -> encodeBuffer(buffer.unwrap())
    );

    private NetworkValues() {
    }

    static Values.Generator generator(final Class<?> type) {
        if (type == ByteBuf.class) {
            return BUFFER;
        }
        if (type == FriendlyByteBuf.class) {
            return FRIENDLY_BUFFER;
        }
        return null;
    }

    static boolean canEncode(final Class<?> type) {
        return generator(type) != null;
    }

    static JsonElement encode(final Object value) {
        return switch (value) {
            case FriendlyByteBuf buffer -> encodeBuffer(buffer.unwrap());
            case ByteBuf buffer -> encodeBuffer(buffer);
            default -> throw new IllegalArgumentException("no encoder for " + value.getClass().getName());
        };
    }

    // An empty buffer of no capacity and the game's default one (256 bytes), single bytes, every field below on its own
    // and filling its buffer, so writes have to grow it, a handshake packet with its id read, and buffers read to the
    // end, with marks, and cleared after marking, which leaves bytes past the writer index.
    private static List<ByteBuf> bufferEdges() {
        List<ByteBuf> edges = new ArrayList<>();
        edges.add(Unpooled.buffer(0));
        edges.add(Unpooled.buffer());
        for (byte[] field : fieldEdges()) {
            edges.add(buffer(0, 0, field));
        }
        edges.add(buffer(1, 0, bytes(0x80, 0x01, 0x02)));
        edges.add(buffer(0, 4, bytes(0x01, 0x02, 0x03, 0x04)));
        edges.add(buffer(240, 1, handshake()));

        ByteBuf marked = buffer(4, 0, bytes(1, 2, 3, 4, 5, 6, 7, 8));
        marked.readerIndex(2).markReaderIndex().readerIndex(5);
        edges.add(marked);

        ByteBuf cleared = buffer(0, 0, bytes(0xff, 0x7f, 0x80, 0x01, 0x05, 0x06, 0x07, 0x08));
        cleared.readerIndex(5).markReaderIndex().markWriterIndex().clear();
        edges.add(cleared);
        return edges;
    }

    private static List<byte[]> fieldEdges() {
        List<byte[]> fields = new ArrayList<>(List.of(
            bytes(0x00), bytes(0x01), bytes(0x7f), bytes(0x80), bytes(0xff),
            // VarInts of each length, the largest of each, MAX_VALUE, MIN_VALUE and -1, a fifth byte whose high bits
            // don't fit, an overlong zero, six bytes, and ones cut short.
            bytes(0x80, 0x01), bytes(0xff, 0x7f), bytes(0x80, 0x80, 0x01), bytes(0xff, 0xff, 0x7f), bytes(0x80, 0x80, 0x80, 0x01),
            bytes(0xff, 0xff, 0xff, 0x7f), bytes(0xff, 0xff, 0xff, 0xff, 0x07), bytes(0x80, 0x80, 0x80, 0x80, 0x08),
            bytes(0xff, 0xff, 0xff, 0xff, 0x0f), bytes(0xff, 0xff, 0xff, 0xff, 0x7f), bytes(0x80, 0x00),
            bytes(0x80, 0x80, 0x80, 0x80, 0x80, 0x01), bytes(0x80, 0x80, 0x80, 0x80), bytes(0xff, 0xff, 0xff, 0xff, 0xff),
            // VarLongs: MAX_VALUE, -1, MIN_VALUE, a tenth byte whose high bits don't fit, eleven bytes, and one cut short.
            varLong(Long.MAX_VALUE), varLong(-1L), varLong(Long.MIN_VALUE),
            bytes(0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f),
            bytes(0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01),
            bytes(0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80),
            // Strings: valid ones, a length past the end, a negative one, readUtf's longest and one more with nothing
            // after them, and malformed UTF-8 (a lone continuation, an overlong NUL, an encoded surrogate, a sequence
            // cut short, a code point past U+10FFFF, a byte UTF-8 never uses).
            utf(""), utf("minecraft:stone"), utf("é"), utf("😀"), utf("١٢"), bytes(0x05, 0x41, 0x42),
            bytes(0xff, 0xff, 0xff, 0xff, 0x0f, 0x41), varInt(98301), varInt(98302),
            lengthPrefixed(bytes(0x80)), lengthPrefixed(bytes(0xc0, 0x80)), lengthPrefixed(bytes(0xed, 0xa0, 0x80)),
            lengthPrefixed(bytes(0xe2, 0x82)), lengthPrefixed(bytes(0xf4, 0x90, 0x80, 0x80)), lengthPrefixed(bytes(0x41, 0xff, 0x42)),
            // Fixed-size fields: a long, the offline player's UUID, NaNs as a float and a double, the walk scenario's
            // ground block, a Vector3f and a Quaternionf.
            longBytes(0x0123456789abcdefL), uuid(offlinePlayer()), bytes(0x7f, 0xc0, 0x00, 0x00), longBytes(0x7ff8000000000000L),
            longBytes(new BlockPos(0, -61, 0).asLong()), floats(0.5F, -60.0F, 0.5F), floats(0.0F, 0.0F, 0.0F, 1.0F)
        ));
        return fields;
    }

    // Fields one after another, read from the start (half the time), the end of the first field or anywhere, with
    // spare capacity of none, a little, or up to the game's default 256 bytes, and marks now and then.
    static ByteBuf randomBuffer(final Random random) {
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        int firstField = 0;
        for (int fields = random.nextInt(5); fields > 0; fields--) {
            out.writeBytes(randomField(random));
            if (firstField == 0) {
                firstField = out.size();
            }
        }
        byte[] bytes = out.toByteArray();
        int slack = switch (random.nextInt(3)) {
            case 0 -> 0;
            case 1 -> random.nextInt(17);
            default -> Math.max(0, 256 - bytes.length);
        };
        int readerIndex = switch (random.nextInt(4)) {
            case 0, 1 -> 0;
            case 2 -> firstField;
            default -> random.nextInt(bytes.length + 1);
        };
        ByteBuf buffer = buffer(slack, readerIndex, bytes);
        if (random.nextInt(4) == 0) {
            setIndex(MARKED_READER_INDEX, buffer, random.nextInt(bytes.length + 1));
            setIndex(MARKED_WRITER_INDEX, buffer, random.nextInt(buffer.capacity() + 1));
        }
        return buffer;
    }

    static byte[] randomField(final Random random) {
        return switch (random.nextInt(8)) {
            case 0 -> varInt((Integer)Values.INTS.random(random));
            case 1 -> varLong(random.nextBoolean() ? random.nextLong() : random.nextInt(1 << 16));
            case 2 -> utf((String)Values.STRINGS.random(random));
            // A length prefix that's right, or off by a little or a lot.
            case 3 -> {
                byte[] payload = randomBytes(random, random.nextInt(12));
                int length = switch (random.nextInt(3)) {
                    case 0 -> payload.length;
                    case 1 -> payload.length + random.nextInt(5) - 2;
                    default -> random.nextInt();
                };
                yield concat(varInt(length), payload);
            }
            // A VarInt or VarLong cut short: continuation bits up to the end.
            case 4 -> {
                byte[] continuations = new byte[1 + random.nextInt(10)];
                for (int i = 0; i < continuations.length; i++) {
                    continuations[i] = (byte)(0x80 | random.nextInt(0x80));
                }
                yield continuations;
            }
            case 5 -> randomBytes(random, new int[]{1, 2, 4, 8, 12, 16}[random.nextInt(6)]);
            case 6 -> lengthPrefixed(randomUtf8(random));
            default -> randomBytes(random, 1 + random.nextInt(8));
        };
    }

    // Bytes that are mostly UTF-8's: lead bytes followed by continuation bytes, and anything now and then.
    static byte[] randomUtf8(final Random random) {
        int[] interesting = {0x00, 0x41, 0x7f, 0x80, 0x8f, 0x90, 0x9f, 0xa0, 0xbf, 0xc0, 0xc1, 0xc2, 0xdf, 0xe0, 0xe1, 0xed, 0xee, 0xef, 0xf0, 0xf1, 0xf4, 0xf5, 0xf8, 0xff};
        byte[] bytes = new byte[random.nextInt(13)];
        for (int i = 0; i < bytes.length; i++) {
            bytes[i] = (byte)(random.nextBoolean() ? interesting[random.nextInt(interesting.length)] : random.nextInt(256));
        }
        return bytes;
    }

    /** A heap buffer like Unpooled.buffer's, holding the bytes with that much capacity to spare, read up to readerIndex. */
    static ByteBuf buffer(final int slack, final int readerIndex, final byte[] bytes) {
        ByteBuf buffer = Unpooled.buffer(bytes.length + slack);
        buffer.writeBytes(bytes);
        buffer.readerIndex(readerIndex);
        return buffer;
    }

    // A fresh heap buffer with the same array, indexes and marks.
    private static ByteBuf copy(final ByteBuf buffer) {
        byte[] array = array(buffer);
        ByteBuf copy = Unpooled.buffer(array.length);
        copy.writeBytes(array);
        setIndex(READER_INDEX, copy, buffer.readerIndex());
        setIndex(WRITER_INDEX, copy, buffer.writerIndex());
        setIndex(MARKED_READER_INDEX, copy, getIndex(MARKED_READER_INDEX, buffer));
        setIndex(MARKED_WRITER_INDEX, copy, getIndex(MARKED_WRITER_INDEX, buffer));
        return copy;
    }

    // Unpooled.EMPTY_BUFFER (which readBytes(0) returns) isn't an AbstractByteBuf and keeps no marks.
    private static JsonArray encodeBuffer(final ByteBuf buffer) {
        JsonArray array = new JsonArray();
        array.add(HEX.formatHex(array(buffer)));
        array.add(buffer.readerIndex());
        array.add(buffer.writerIndex());
        array.add(buffer instanceof AbstractByteBuf ? getIndex(MARKED_READER_INDEX, buffer) : 0);
        array.add(buffer instanceof AbstractByteBuf ? getIndex(MARKED_WRITER_INDEX, buffer) : 0);
        return array;
    }

    private static byte[] array(final ByteBuf buffer) {
        byte[] array = new byte[buffer.capacity()];
        buffer.getBytes(0, array);
        return array;
    }

    /** The VarInt at the buffer's reader index, or null when there's no whole one, without moving the buffer. */
    static Integer leadingVarInt(final Object buffer) {
        try {
            return VarInt.read(unwrap(buffer).duplicate());
        } catch (RuntimeException e) {
            return null;
        }
    }

    static byte[] bytes(final int... values) {
        byte[] bytes = new byte[values.length];
        for (int i = 0; i < values.length; i++) {
            bytes[i] = (byte)values[i];
        }
        return bytes;
    }

    static byte[] concat(final byte[]... parts) {
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        for (byte[] part : parts) {
            out.writeBytes(part);
        }
        return out.toByteArray();
    }

    static byte[] varInt(final int value) {
        ByteBuf buffer = Unpooled.buffer();
        VarInt.write(buffer, value);
        return readable(buffer);
    }

    static byte[] varLong(final long value) {
        ByteBuf buffer = Unpooled.buffer();
        VarLong.write(buffer, value);
        return readable(buffer);
    }

    static byte[] utf(final String value) {
        ByteBuf buffer = Unpooled.buffer();
        Utf8String.write(buffer, value, Integer.MAX_VALUE);
        return readable(buffer);
    }

    static byte[] lengthPrefixed(final byte[] payload) {
        return concat(varInt(payload.length), payload);
    }

    static byte[] longBytes(final long value) {
        return readable(Unpooled.buffer().writeLong(value));
    }

    static byte[] floats(final float... values) {
        ByteBuf buffer = Unpooled.buffer();
        for (float value : values) {
            buffer.writeFloat(value);
        }
        return readable(buffer);
    }

    static byte[] uuid(final UUID uuid) {
        return concat(longBytes(uuid.getMostSignificantBits()), longBytes(uuid.getLeastSignificantBits()));
    }

    /** The UUID the server gives the offline player `rustcraft`. */
    static UUID offlinePlayer() {
        return UUID.nameUUIDFromBytes("OfflinePlayer:rustcraft".getBytes(StandardCharsets.UTF_8));
    }

    // A handshake: packet id 0, protocol version, address, port and intent (login).
    private static byte[] handshake() {
        return concat(varInt(0), varInt(777), utf("localhost"), bytes(0x63, 0xdd), varInt(2));
    }

    private static byte[] randomBytes(final Random random, final int length) {
        byte[] bytes = new byte[length];
        random.nextBytes(bytes);
        return bytes;
    }

    /** The readable bytes, read. */
    static byte[] readable(final ByteBuf buffer) {
        byte[] bytes = new byte[buffer.readableBytes()];
        buffer.readBytes(bytes);
        return bytes;
    }

    /** A buffer of the generator's kind (a ByteBuf, or a FriendlyByteBuf around one), holding the bytes. */
    static Object bufferLike(final Values.Generator generator, final int slack, final byte[] bytes) {
        ByteBuf buffer = buffer(slack, 0, bytes);
        return generator == FRIENDLY_BUFFER ? new FriendlyByteBuf(buffer) : buffer;
    }

    static ByteBuf unwrap(final Object buffer) {
        return buffer instanceof FriendlyByteBuf friendly ? friendly.unwrap() : (ByteBuf)buffer;
    }

    private static Field indexField(final String name) {
        try {
            Field field = AbstractByteBuf.class.getDeclaredField(name);
            field.setAccessible(true);
            return field;
        } catch (NoSuchFieldException e) {
            throw new IllegalStateException(e);
        }
    }

    private static int getIndex(final Field field, final ByteBuf buffer) {
        try {
            return field.getInt(buffer);
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
    }

    private static void setIndex(final Field field, final ByteBuf buffer, final int index) {
        try {
            field.setInt(buffer, index);
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
    }
}
