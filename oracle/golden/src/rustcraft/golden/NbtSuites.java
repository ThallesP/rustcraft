package rustcraft.golden;

import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufInputStream;
import io.netty.buffer.ByteBufOutputStream;
import io.netty.buffer.Unpooled;
import java.io.IOException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Random;
import java.util.function.Function;
import net.minecraft.nbt.ByteArrayTag;
import net.minecraft.nbt.ByteTag;
import net.minecraft.nbt.CollectionTag;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.DoubleTag;
import net.minecraft.nbt.EndTag;
import net.minecraft.nbt.FloatTag;
import net.minecraft.nbt.IntArrayTag;
import net.minecraft.nbt.IntTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.LongArrayTag;
import net.minecraft.nbt.LongTag;
import net.minecraft.nbt.NbtAccounter;
import net.minecraft.nbt.NbtIo;
import net.minecraft.nbt.NumericTag;
import net.minecraft.nbt.ShortTag;
import net.minecraft.nbt.StringTag;
import net.minecraft.nbt.Tag;
import net.minecraft.nbt.TagType;
import net.minecraft.nbt.TagTypes;
import net.minecraft.network.FriendlyByteBuf;

/**
 * The NBT suites: java.util.HashMap's programs, every tag class and the interfaces they share, TagType, TagTypes,
 * NbtAccounter and NbtIo, and the Netty streams the game reads and writes a packet's NBT through. See NbtValues for how
 * they're encoded. A member taking a compound's key also gets the receiver's own keys, and one taking an index the indexes
 * around the receiver's size, since random ones would almost never find anything. Readers also read what the writers
 * write, whole, cut short and with a field after.
 */
final class NbtSuites {
    private static final int KEYED_INPUTS = 300;
    private static final String READ_ANY_TAG = "readAnyTag(Ljava/io/DataInput;Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/Tag;";
    private static final String READ_UNNAMED_TAG = "readUnnamedTag(Ljava/io/DataInput;Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/Tag;";
    private static final String READ = "read(Ljava/io/DataInput;)Lnet/minecraft/nbt/CompoundTag;";
    private static final String READ_ACCOUNTED = "read(Ljava/io/DataInput;Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/CompoundTag;";
    private static final String LOAD = "load(Ljava/io/DataInput;Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/Tag;";
    private static final String SKIP = "skip(Ljava/io/DataInput;Lnet/minecraft/nbt/NbtAccounter;)V";
    private static final String SKIP_COUNT = "skip(Ljava/io/DataInput;ILnet/minecraft/nbt/NbtAccounter;)V";
    private static final String WRITE = "write(Ljava/io/DataOutput;)V";

    private NbtSuites() {
    }

    static List<Suite> suites() {
        return List.of(
            new HashMapSuite(),
            InstanceSuite.every(Tag.class).extraInputs(WRITE, longWrites(Tag.class)),
            InstanceSuite.every(NumericTag.class),
            indexed(InstanceSuite.every(CollectionTag.class), CollectionTag.class, NbtValues.COLLECTION_TAGS),
            InstanceSuite.every(EndTag.class),
            InstanceSuite.every(ByteTag.class),
            InstanceSuite.every(ShortTag.class),
            InstanceSuite.every(IntTag.class),
            InstanceSuite.every(LongTag.class),
            InstanceSuite.every(FloatTag.class),
            InstanceSuite.every(DoubleTag.class),
            indexed(InstanceSuite.every(ByteArrayTag.class), ByteArrayTag.class, NbtValues.BYTE_ARRAY_TAGS),
            InstanceSuite.every(StringTag.class)
                .extraInputs(
                    "skipString(Ljava/io/DataInput;)V",
                    inputs(concat(written(201, NbtValues.STRING_TAGS, NbtValues::payloadBytes), longPayloads()))
                )
                .extraInputs(WRITE, longWrites(StringTag.class)),
            indexed(InstanceSuite.every(ListTag.class), ListTag.class, NbtValues.LIST_TAGS)
                .extraInputs(WRITE, concat(longWrites(ListTag.class), endWrites())),
            keyed(InstanceSuite.every(CompoundTag.class))
                .extraInputs(WRITE, longWrites(CompoundTag.class))
                .extraInputs("merge(Lnet/minecraft/nbt/CompoundTag;)Lnet/minecraft/nbt/CompoundTag;", merges()),
            indexed(InstanceSuite.every(IntArrayTag.class), IntArrayTag.class, NbtValues.INT_ARRAY_TAGS),
            indexed(InstanceSuite.every(LongArrayTag.class), LongArrayTag.class, NbtValues.LONG_ARRAY_TAGS),
            // A type loads and skips what a tag of its kind writes, and what several write one after another.
            InstanceSuite.every(TagType.class)
                .extraInputs(LOAD, concat(concat(typed(202, 1), longStringLoads()), duplicateNames(true)))
                .where(LOAD, args -> NbtValues.readsLittle((ByteBuf)args[1], (NbtAccounter)args[2], ((TagType<?>)args[0])::load))
                .extraInputs(SKIP, typed(203, 1))
                .extraInputs(SKIP_COUNT, typed(204, 3)),
            new StaticSuite(TagTypes.class).extraInputs("getType(I)Lnet/minecraft/nbt/TagType;", ids()),
            InstanceSuite.every(NbtAccounter.class),
            // What NbtIo writes, and a named root tag, whose name readUnnamedTag skips.
            new StaticSuite(NbtIo.class)
                .extraInputs(READ_ANY_TAG, concat(accounted(written(205, NbtValues.TAGS, NbtValues::anyTagBytes), 206), duplicateNames(false)))
                .extraInputs(READ_UNNAMED_TAG, accounted(named(207), 208))
                .extraInputs(READ, inputs(named(209)))
                .extraInputs(READ_ACCOUNTED, accounted(named(210), 211))
                .extraInputs("writeAnyTag(Lnet/minecraft/nbt/Tag;Ljava/io/DataOutput;)V", longStaticWrites(false))
                .extraInputs("writeUnnamedTag(Lnet/minecraft/nbt/Tag;Ljava/io/DataOutput;)V", longStaticWrites(false))
                .extraInputs("writeUnnamedTagWithFallback(Lnet/minecraft/nbt/Tag;Ljava/io/DataOutput;)V", longStaticWrites(false))
                .extraInputs("write(Lnet/minecraft/nbt/CompoundTag;Ljava/io/DataOutput;)V", longStaticWrites(true))
                // An accounter with more than the default quota reading a count in the billions would allocate an array
                // that long.
                .where(READ_ANY_TAG, args -> NbtValues.readsLittle((ByteBuf)args[0], (NbtAccounter)args[1], NbtIo::readAnyTag))
                .where(READ_UNNAMED_TAG, args -> NbtValues.readsLittle((ByteBuf)args[0], (NbtAccounter)args[1], NbtIo::readUnnamedTag))
                .where(READ, args -> NbtValues.readsLittle((ByteBuf)args[0], null, NbtIo::read))
                .where(READ_ACCOUNTED, args -> NbtValues.readsLittle((ByteBuf)args[0], (NbtAccounter)args[1], NbtIo::read)),
            // The methods of a DataInput and a DataOutput that NBT and modified UTF-8 call.
            new InstanceSuite(
                ByteBufInputStream.class,
                "readByte()B", "readUnsignedByte()I", "readShort()S", "readUnsignedShort()I", "readInt()I", "readLong()J", "readFloat()F",
                "readDouble()D", "readFully([B)V", "readFully([BII)V", "skipBytes(I)I", "readUTF()Ljava/lang/String;", "available()I"
            )
                .extraInputs("readUTF()Ljava/lang/String;", inputs(modifiedUtf8()))
                .extraInputs("skipBytes(I)I", skips()),
            new InstanceSuite(
                ByteBufOutputStream.class,
                "writeByte(I)V", "writeShort(I)V", "writeInt(I)V", "writeLong(J)V", "writeFloat(F)V", "writeDouble(D)V", "write([B)V", "write([BII)V",
                "writeUTF(Ljava/lang/String;)V"
            )
                .extraInputs("writeUTF(Ljava/lang/String;)V", longStrings())
                .where("write([BII)V", args -> ((Integer)args[3]) <= 1 << 16)
        );
    }

    /** What FriendlyByteBuf's readNbt methods also read: what writeNbt writes of tags, whole, cut short and with a field after. */
    static List<Object[]> nbtBuffers(final long seed) {
        return inputs(written(seed, NbtValues.TAGS, NbtValues::anyTagBytes));
    }

    /** Like nbtBuffers, each with an accounter: the game's default one mostly, and any now and then. */
    static List<Object[]> accountedNbtBuffers(final long seed) {
        return accounted(written(seed, NbtValues.TAGS, NbtValues::anyTagBytes), seed + 1);
    }

    // Each member whose first argument is a key gets receivers with the keys they hold, other keys now and then, and the
    // rest of its arguments generated.
    private static InstanceSuite keyed(final InstanceSuite suite) {
        for (Method method : publicMethods(CompoundTag.class)) {
            Class<?>[] parameters = method.getParameterTypes();
            if (parameters.length == 0 || parameters[0] != String.class || !generated(parameters)) {
                continue;
            }
            Random random = new Random(Descriptors.of(method).hashCode());
            List<Object[]> inputs = new ArrayList<>();
            for (int i = 0; i < KEYED_INPUTS; i++) {
                CompoundTag receiver = (CompoundTag)NbtValues.COMPOUND_TAGS.random(random);
                List<String> keys = new ArrayList<>(NbtValues.tags(receiver).keySet());
                Object[] input = new Object[parameters.length + 1];
                input[0] = receiver;
                input[1] = keys.isEmpty() || random.nextInt(3) == 0 ? NbtValues.randomKey(random) : keys.get(random.nextInt(keys.size()));
                for (int j = 1; j < parameters.length; j++) {
                    input[j + 1] = Values.generator(parameters[j]).random(random);
                }
                inputs.add(input);
            }
            suite.extraInputs(Descriptors.of(method), inputs);
        }
        return suite;
    }

    // Each member whose first argument is an index gets receivers with indexes from one before the first to one past the
    // last, and the rest of its arguments generated.
    private static InstanceSuite indexed(final InstanceSuite suite, final Class<?> type, final Values.Generator receivers) {
        for (Method method : publicMethods(type)) {
            Class<?>[] parameters = method.getParameterTypes();
            if (parameters.length == 0 || parameters[0] != int.class || !generated(parameters)) {
                continue;
            }
            Random random = new Random(Descriptors.of(method).hashCode());
            List<Object[]> inputs = new ArrayList<>();
            for (int i = 0; i < KEYED_INPUTS; i++) {
                Tag receiver = (Tag)receivers.random(random);
                int size = ((CollectionTag)receiver).size();
                Object[] input = new Object[parameters.length + 1];
                input[0] = receiver;
                input[1] = random.nextInt(size + 3) - 1;
                for (int j = 1; j < parameters.length; j++) {
                    input[j + 1] = Values.generator(parameters[j]).random(random);
                }
                inputs.add(input);
            }
            suite.extraInputs(Descriptors.of(method), inputs);
        }
        return suite;
    }

    private static List<Method> publicMethods(final Class<?> type) {
        return Arrays.stream(type.getDeclaredMethods())
            .filter(method -> Modifier.isPublic(method.getModifiers()) && !Modifier.isStatic(method.getModifiers()))
            .filter(method -> !method.isSynthetic() && !method.isBridge())
            .toList();
    }

    private static boolean generated(final Class<?>[] parameters) {
        return Arrays.stream(parameters).allMatch(parameter -> parameter == Object.class || Values.generator(parameter) != null);
    }

    // The tags' encodings by the writer, for the value edges and random values: whole, with another field after, cut
    // short by a byte and cut anywhere. Only tags that write, since a string too long for writeUTF throws.
    private static List<byte[]> written(final long seed, final Values.Generator tags, final Function<Tag, byte[]> writer) {
        Random random = new Random(seed);
        List<Object> edges = tags.edges();
        List<byte[]> written = new ArrayList<>();
        for (int i = 0; i < 150; i++) {
            Tag tag = (Tag)(i < edges.size() ? edges.get(i) : tags.random(random));
            if (NbtValues.writes(tag)) {
                written.addAll(Codecs.variants(writer.apply(tag), random));
            }
        }
        return written;
    }

    // Root tags as a file has them, with a name: empty (as NbtIo writes them), a few characters, and as long as
    // writeUTF allows. Compounds mostly, which read and read(…, NbtAccounter) want, and other tags.
    private static List<byte[]> named(final long seed) {
        Random random = new Random(seed);
        List<byte[]> named = new ArrayList<>();
        List<Object> edges = NbtValues.COMPOUND_TAGS.edges();
        for (int i = 0; i < 100; i++) {
            Tag tag = (Tag)(i < edges.size() ? edges.get(i) : random.nextInt(4) == 0 ? NbtValues.TAGS.random(random) : NbtValues.COMPOUND_TAGS.random(random));
            if (!NbtValues.writes(tag)) {
                continue;
            }
            String name = switch (random.nextInt(4)) {
                case 0, 1 -> "";
                case 2 -> "root";
                default -> random.nextBoolean() ? "a".repeat(65535) : (String)Values.STRINGS.random(random);
            };
            ByteBuf buffer = Unpooled.buffer();
            buffer.writeByte(tag.getId());
            if (tag.getId() != 0) {
                try {
                    new ByteBufOutputStream(buffer).writeUTF(name);
                } catch (IOException e) {
                    throw new IllegalStateException(e);
                }
                buffer.writeBytes(NbtValues.payloadBytes(tag));
            }
            named.addAll(Codecs.variants(NetworkValues.readable(buffer), random));
        }
        return named;
    }

    // Each type with what tags of its kind write, `count` of them one after another, and an accounter.
    private static List<Object[]> typed(final long seed, final int maxCount) {
        Random random = new Random(seed);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 400; i++) {
            int id = i < 13 ? i : random.nextInt(13);
            int count = maxCount == 1 ? 1 : random.nextInt(maxCount + 1);
            ByteBuf bytes = Unpooled.buffer();
            for (int j = 0; j < count; j++) {
                Tag tag = id == 0 ? EndTag.INSTANCE : NbtValues.randomTagOf(random, id, 0);
                if (NbtValues.writes(tag)) {
                    bytes.writeBytes(NbtValues.payloadBytes(tag));
                }
            }
            List<byte[]> variants = Codecs.variants(NetworkValues.readable(bytes), random);
            TagType<?> type = TagTypes.getType(id);
            ByteBuf buffer = buffer(variants.get(random.nextInt(variants.size())));
            inputs.add(maxCount == 1 ? new Object[]{type, buffer, accounter(random)} : new Object[]{type, buffer, count, accounter(random)});
        }
        return inputs;
    }

    // The buffers with an accounter each: the game's default one mostly, and any now and then.
    private static List<Object[]> accounted(final List<byte[]> bytes, final long seed) {
        Random random = new Random(seed);
        return bytes.stream().map(written -> new Object[]{buffer(written), accounter(random)}).toList();
    }

    private static Object accounter(final Random random) {
        return random.nextInt(3) == 0 ? NbtValues.ACCOUNTERS.random(random) : NbtAccounter.defaultQuota();
    }

    private static List<Object[]> inputs(final List<byte[]> bytes) {
        return bytes.stream().map(written -> new Object[]{buffer(written)}).toList();
    }

    // A buffer holding the bytes, unless a string modified UTF-8 reads from them could hold a lone surrogate.
    private static ByteBuf buffer(final byte[] bytes) {
        return NetworkValues.buffer(0, 0, NbtValues.noLoneSurrogates(bytes) ? bytes : new byte[0]);
    }

    // Lists holding an EndTag, which no list the game reads has, alone, first, last and among others: a list's kind
    // ignores them, and one written as compounds writes an EndTag's wrapper as its id alone, which ends it.
    private static List<Object[]> endWrites() {
        List<Object[]> inputs = new ArrayList<>();
        Tag end = EndTag.INSTANCE;
        Tag one = ByteTag.valueOf((byte)1);
        for (Tag[] elements : new Tag[][]{{end}, {end, end}, {end, one}, {one, end}, {one, end, one}, {end, StringTag.valueOf("a"), one}}) {
            ListTag list = new ListTag();
            for (Tag element : elements) {
                list.add(element);
            }
            inputs.add(new Object[]{list, Unpooled.buffer()});
        }
        return inputs;
    }

    // Compounds merged with copies of others holding compounds under some of their names, nested two deep, and
    // other tags under others.
    private static List<Object[]> merges() {
        Random random = new Random(219);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 100; i++) {
            CompoundTag receiver = NbtValues.randomCompound(random, 1);
            CompoundTag other = NbtValues.randomCompound(random, 1);
            List<String> keys = new ArrayList<>(NbtValues.tags(receiver).keySet());
            for (String key : keys) {
                int roll = random.nextInt(3);
                if (roll == 0) {
                    receiver.put(key, NbtValues.randomCompound(random, 2));
                    other.put(key, NbtValues.randomCompound(random, 2));
                } else if (roll == 1) {
                    other.put(key, NbtValues.randomTag(random, 2));
                }
            }
            inputs.add(new Object[]{receiver, other});
        }
        return inputs;
    }

    // Compounds holding a name twice, which NBT written by the game never has: the second tag replaces the first where
    // it was, and the name is accounted once. As a compound's payload for its type's load, or with its id for
    // readAnyTag, with the default accounter and ones that run out at either name.
    private static List<Object[]> duplicateNames(final boolean payload) {
        List<Object[]> inputs = new ArrayList<>();
        for (byte[] entries : new byte[][]{
            NetworkValues.concat(namedInt("a", 1), namedInt("a", 2)),
            NetworkValues.concat(namedInt("a", 1), namedInt("b", 2), namedInt("a", 3)),
            NetworkValues.concat(namedInt("x", 1), namedString("x", "y"), namedInt("x", 2))
        }) {
            byte[] compound = NetworkValues.concat(entries, NetworkValues.bytes(0));
            byte[] bytes = payload ? compound : NetworkValues.concat(NetworkValues.bytes(10), compound);
            for (long quota : new long[]{NbtAccounter.DEFAULT_NBT_QUOTA, 48 + 28 + 2 + 12 + 36, 48 + 2 * (28 + 2 + 12) + 36}) {
                NbtAccounter accounter = NbtAccounter.create(quota);
                inputs.add(payload ? new Object[]{CompoundTag.TYPE, buffer(bytes), accounter} : new Object[]{buffer(bytes), accounter});
            }
        }
        return inputs;
    }

    private static byte[] namedInt(final String name, final int value) {
        return NetworkValues.concat(NetworkValues.bytes(3), utf(name), NetworkValues.bytes(value >> 24, value >> 16, value >> 8, value));
    }

    private static byte[] namedString(final String name, final String value) {
        return NetworkValues.concat(NetworkValues.bytes(8), utf(name), utf(value));
    }

    // writeUTF's modified UTF-8 of an ASCII string: its length as a short, then its bytes.
    private static byte[] utf(final String string) {
        return NetworkValues.concat(NetworkValues.bytes(string.length() >> 8, string.length()), string.getBytes(StandardCharsets.UTF_8));
    }

    // The long tags of the class, each written into the default buffer and an empty one.
    private static List<Object[]> longWrites(final Class<?> type) {
        List<Object[]> inputs = new ArrayList<>();
        for (Tag tag : NbtValues.longTags()) {
            if (type.isInstance(tag)) {
                inputs.add(new Object[]{tag, Unpooled.buffer()});
                inputs.add(new Object[]{tag, Unpooled.buffer(0)});
            }
        }
        return inputs;
    }

    // The long tags (compounds only, for write(CompoundTag, DataOutput)) as the first argument, written into the default
    // buffer and an empty one.
    private static List<Object[]> longStaticWrites(final boolean compounds) {
        List<Object[]> inputs = new ArrayList<>();
        for (Tag tag : NbtValues.longTags()) {
            if (!compounds || tag instanceof CompoundTag) {
                inputs.add(new Object[]{tag, Unpooled.buffer()});
                inputs.add(new Object[]{tag, Unpooled.buffer(0)});
            }
        }
        return inputs;
    }

    // What the long strings that writeUTF writes write, whole and cut short.
    private static List<byte[]> longPayloads() {
        List<byte[]> payloads = new ArrayList<>();
        for (Tag tag : NbtValues.longTags()) {
            if (tag instanceof StringTag && NbtValues.writes(tag)) {
                byte[] bytes = NbtValues.payloadBytes(tag);
                payloads.add(bytes);
                payloads.add(Arrays.copyOf(bytes, bytes.length - 1));
            }
        }
        return payloads;
    }

    // The string type loading the long strings' payloads, with the default accounter and one whose quota is a string's
    // 36 bytes and its characters' 2 each, for the longest, and a byte less.
    private static List<Object[]> longStringLoads() {
        List<Object[]> inputs = new ArrayList<>();
        for (byte[] payload : longPayloads()) {
            inputs.add(new Object[]{StringTag.TYPE, buffer(payload), NbtAccounter.defaultQuota()});
        }
        inputs.add(new Object[]{StringTag.TYPE, buffer(NbtValues.payloadBytes(StringTag.valueOf("a".repeat(65535)))), NbtAccounter.create(36L + 2L * 65535L)});
        inputs.add(new Object[]{StringTag.TYPE, buffer(NbtValues.payloadBytes(StringTag.valueOf("a".repeat(65535)))), NbtAccounter.create(36L + 2L * 65535L - 1L)});
        return inputs;
    }

    private static <T> List<T> concat(final List<T> first, final List<T> second) {
        List<T> all = new ArrayList<>(first);
        all.addAll(second);
        return all;
    }

    // Ids from one before End's to one past the last tag's.
    private static List<Object[]> ids() {
        List<Object[]> ids = new ArrayList<>();
        for (int id = -1; id <= 14; id++) {
            ids.add(new Object[]{id});
        }
        return ids;
    }

    // Modified UTF-8 with its length: ASCII, two and three bytes, a supplementary character as two surrogates, U+0000 as
    // two bytes and as one, overlong two and three bytes, a continuation that isn't one (ASCII, or a lead byte) in each
    // place, a character cut
    // short by the length, a lone continuation, four-byte leads (which modified UTF-8 never has), alone and followed
    // by continuations as a three-byte character would be, ASCII before a malformed character, a length past the end,
    // and the longest.
    private static List<byte[]> modifiedUtf8() {
        List<byte[]> strings = new ArrayList<>();
        for (int[] bytes : new int[][]{
            {}, {0x61, 0x62, 0x63}, {0xc3, 0xa9}, {0xe2, 0x82, 0xac}, {0xed, 0xa0, 0xbd, 0xed, 0xb8, 0x80}, {0xc0, 0x80}, {0x00}, {0xc1, 0x81},
            {0xe0, 0x80, 0x80}, {0xc2, 0x41}, {0xc2, 0xc0}, {0xe2, 0x41, 0x82}, {0xe2, 0xc0, 0x82}, {0xe2, 0x82, 0x41}, {0xe2, 0x82, 0xff}, {0xc2}, {0xe2, 0x82}, {0x80}, {0xbf, 0x41},
            {0xf0, 0x9f, 0x98, 0x80}, {0xf0, 0x9f, 0x98}, {0xf8, 0x80, 0x80}, {0xf8}, {0xff}, {0x61, 0x62, 0xc2}, {0x61, 0xe2, 0x82, 0xac, 0x62}
        }) {
            byte[] utf = NetworkValues.bytes(bytes);
            strings.add(NetworkValues.concat(NetworkValues.bytes(utf.length >> 8, utf.length), utf));
        }
        strings.add(NetworkValues.bytes(0x00, 0x05, 0x61, 0x62));
        strings.add(NetworkValues.bytes(0xff, 0xff));
        byte[] longest = new byte[65537];
        Arrays.fill(longest, (byte)0x61);
        longest[0] = (byte)0xff;
        longest[1] = (byte)0xff;
        strings.add(longest);
        strings.add(NetworkValues.concat(NetworkValues.bytes(0x00, 0x03), "abcdef".getBytes(StandardCharsets.UTF_8)));
        return strings;
    }

    // Counts from -1 to two past what's left, for buffers read from the start and partway.
    private static List<Object[]> skips() {
        List<Object[]> inputs = new ArrayList<>();
        Random random = new Random(212);
        for (int i = 0; i < 40; i++) {
            ByteBuf buffer = NetworkValues.buffer(random.nextInt(5), random.nextInt(3), new byte[2 + random.nextInt(10)]);
            for (int count = -1; count <= buffer.readableBytes() + 2; count++) {
                inputs.add(new Object[]{NetworkValues.copy(buffer), count});
            }
        }
        inputs.add(new Object[]{NetworkValues.buffer(0, 0, new byte[4]), Integer.MAX_VALUE});
        inputs.add(new Object[]{NetworkValues.buffer(0, 0, new byte[4]), Integer.MIN_VALUE});
        return inputs;
    }

    // Strings around writeUTF's limit, into the default buffer and an empty one.
    private static List<Object[]> longStrings() {
        List<Object[]> inputs = new ArrayList<>();
        for (String string : NbtValues.LONG_STRINGS) {
            inputs.add(new Object[]{Unpooled.buffer(), string});
            inputs.add(new Object[]{Unpooled.buffer(0), string});
        }
        return inputs;
    }

    /** The same inputs with each buffer wrapped in a FriendlyByteBuf, for its instance methods. */
    static List<Object[]> friendly(final List<Object[]> inputs) {
        List<Object[]> wrapped = new ArrayList<>();
        for (Object[] input : inputs) {
            Object[] copy = input.clone();
            copy[0] = new FriendlyByteBuf((ByteBuf)input[0]);
            wrapped.add(copy);
        }
        return wrapped;
    }
}
