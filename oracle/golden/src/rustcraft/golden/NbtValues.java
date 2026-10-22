package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufInputStream;
import io.netty.buffer.ByteBufOutputStream;
import io.netty.buffer.Unpooled;
import java.io.DataInput;
import java.io.DataOutput;
import java.io.IOException;
import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
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

/**
 * NBT's tags, and what reads and writes them. A tag is {@code [id, payload]} whatever it's declared as: an EndTag
 * {@code [0, null]}, a number {@code [1, byte]} to {@code [6, double]}, an array {@code [7, [byte, …]]},
 * {@code [11, [int, …]]} or {@code [12, [long, …]]}, a string {@code [8, value]}, a list {@code [9, [tag, …]]}, and a
 * compound {@code [10, [capacity, [[key, tag], …]]]}: its HashMap's table length (0 before it's made) and its tags in the
 * map's order, which is the order it writes them in. A TagType is its name ({@code "BYTE"}, {@code "INVALID[13]"}), an
 * NbtAccounter {@code [quota, usage, maxDepth, depth]}, and a DataInput or DataOutput, which is Netty's
 * ByteBufInputStream or ByteBufOutputStream as the game reads and writes a packet's NBT through them, the buffer it reads
 * or writes. A Number is encoded by its own class, and a Map.Entry (a compound's entrySet) is {@code [key, value]}.
 *
 * <p>Tags are the game's kinds: items' and block entities' compounds, lists of one type and of several (which compounds
 * wrap when they're written), lists holding the compounds that wrap, nested ones up to 17 deep (a deeper one would pass
 * serde_json's recursion limit on the port's side, and the depth check is the same at any depth, which accounters with
 * small depths check), strings around modified UTF-8's 65535 bytes, and compounds whose tables are bigger than their size
 * (after removals) or smaller (copied like CompoundTag.shallowCopy). Each argument is its own copy, its compounds' maps
 * made with the same table length and order.
 */
final class NbtValues {
    private static final Constructor<CompoundTag> COMPOUND = compoundConstructor();
    private static final Field COMPOUND_TAGS_FIELD = field(CompoundTag.class, "tags");
    private static final Field QUOTA = field(NbtAccounter.class, "quota");
    private static final Field USAGE = field(NbtAccounter.class, "usage");
    private static final Field MAX_DEPTH = field(NbtAccounter.class, "maxDepth");
    private static final Field DEPTH = field(NbtAccounter.class, "depth");
    private static final Field INPUT_BUFFER = field(ByteBufInputStream.class, "buffer");

    private static final Values.Generator BYTES = Values.generator(byte.class);
    private static final Values.Generator INTS = Values.generator(int.class);
    private static final Values.Generator LONGS = Values.generator(long.class);
    private static final Values.Generator FLOATS = Values.generator(float.class);
    private static final Values.Generator DOUBLES = Values.generator(double.class);
    private static final Values.Generator STRINGS = Values.generator(String.class);
    private static final Values.Generator BYTE_ARRAYS = Values.generator(byte[].class);
    private static final Values.Generator INT_ARRAYS = Values.generator(int[].class);
    private static final Values.Generator LONG_ARRAYS = Values.generator(long[].class);

    // Shorts, and the bounds of ShortTag's and IntTag's caches.
    private static final List<Object> SHORT_EDGES = List.of(
        (short)0, (short)1, (short)-1, (short)-128, (short)-129, (short)1024, (short)1025, Short.MIN_VALUE, Short.MAX_VALUE
    );

    /**
     * Strings around writeUTF's limit of 65535 bytes, in characters of one, two (U+0000 and é) and three bytes, and in
     * supplementary ones, whose two surrogates take three bytes each. They're too big to be among every member's edges,
     * so the writers and readers get them as extra inputs (LONG_TAGS).
     */
    static final List<String> LONG_STRINGS = List.of(
        "a".repeat(65535), "a".repeat(65536), "é".repeat(32767) + "a", "é".repeat(32768), "\u0000".repeat(32767), "\u0000".repeat(32768),
        "€".repeat(21845), "€".repeat(21845) + "a", "😀".repeat(10922) + "aaa", "😀".repeat(10922) + "aaaa"
    );

    // A compound's keys: an item's and a block entity's, a text component's, the empty key a list's wrapping compound has,
    // and two whose hashCodes are equal.
    private static final List<String> COMMON_KEYS = List.of(
        "id", "count", "Count", "Slot", "x", "y", "z", "Name", "Items", "display", "tag", "components", "", "minecraft:custom_data", "Damage",
        "Lore", "CustomName", "text", "color", "extra", "bold", "Aa", "BB"
    );
    // Keys whose hashCodes are all equal, which share a bin in any table: never more than 6 in a compound, so no bin
    // becomes a tree.
    private static final List<String> COLLIDING_KEYS = HashMapSuite.colliding(5).subList(0, 6);

    static final Values.Generator END_TAGS = immutable(List.of(EndTag.INSTANCE), random -> EndTag.INSTANCE);
    static final Values.Generator BYTE_TAGS = immutable(
        BYTES.edges().stream().map(value -> (Object)ByteTag.valueOf((Byte)value)).toList(), random -> ByteTag.valueOf((Byte)BYTES.random(random))
    );
    static final Values.Generator SHORT_TAGS = immutable(
        SHORT_EDGES.stream().map(value -> (Object)ShortTag.valueOf((Short)value)).toList(), random -> ShortTag.valueOf(randomShort(random))
    );
    static final Values.Generator INT_TAGS = immutable(
        concat(INTS.edges(), List.of(-128, -129, 1024, 1025)).stream().map(value -> (Object)IntTag.valueOf((Integer)value)).toList(),
        random -> IntTag.valueOf((Integer)INTS.random(random))
    );
    static final Values.Generator LONG_TAGS = immutable(
        concat(LONGS.edges(), List.of(-128L, -129L, 1024L, 1025L)).stream().map(value -> (Object)LongTag.valueOf((Long)value)).toList(),
        random -> LongTag.valueOf((Long)LONGS.random(random))
    );
    // FloatTag.valueOf makes -0.0 the ZERO constant, so the constructor makes the tag holding it.
    static final Values.Generator FLOAT_TAGS = immutable(
        concat(FLOATS.edges().stream().map(value -> (Object)FloatTag.valueOf((Float)value)).toList(), List.of(negativeZeroFloat())),
        random -> FloatTag.valueOf((Float)FLOATS.random(random))
    );
    static final Values.Generator DOUBLE_TAGS = immutable(
        concat(DOUBLES.edges().stream().map(value -> (Object)DoubleTag.valueOf((Double)value)).toList(), List.of(negativeZeroDouble())),
        random -> DoubleTag.valueOf((Double)DOUBLES.random(random))
    );
    static final Values.Generator STRING_TAGS = immutable(
        STRINGS.edges().stream().map(value -> (Object)StringTag.valueOf((String)value)).toList(),
        random -> StringTag.valueOf((String)STRINGS.random(random))
    );
    static final Values.Generator BYTE_ARRAY_TAGS = mutable(
        BYTE_ARRAYS.edges().stream().map(value -> (Tag)new ByteArrayTag((byte[])value)).toList(), random -> new ByteArrayTag((byte[])BYTE_ARRAYS.random(random))
    );
    static final Values.Generator INT_ARRAY_TAGS = mutable(
        INT_ARRAYS.edges().stream().map(value -> (Tag)new IntArrayTag((int[])value)).toList(), random -> new IntArrayTag((int[])INT_ARRAYS.random(random))
    );
    static final Values.Generator LONG_ARRAY_TAGS = mutable(
        LONG_ARRAYS.edges().stream().map(value -> (Tag)new LongArrayTag((long[])value)).toList(), random -> new LongArrayTag((long[])LONG_ARRAYS.random(random))
    );
    static final Values.Generator LIST_TAGS = mutable(listEdges(), random -> randomList(random, 0));
    static final Values.Generator COMPOUND_TAGS = mutable(compoundEdges(), random -> randomCompound(random, 0));

    // Every kind of tag: a few edges of each kind, and random ones of any kind (an EndTag now and then).
    static final Values.Generator TAGS = mutable(
        anyEdges(
            END_TAGS, BYTE_TAGS, SHORT_TAGS, INT_TAGS, LONG_TAGS, FLOAT_TAGS, DOUBLE_TAGS, BYTE_ARRAY_TAGS, STRING_TAGS, LIST_TAGS, COMPOUND_TAGS,
            INT_ARRAY_TAGS, LONG_ARRAY_TAGS
        ),
        random -> random.nextInt(40) == 0 ? EndTag.INSTANCE : randomTag(random, 0)
    );
    static final Values.Generator NUMERIC_TAGS = mutable(
        anyEdges(BYTE_TAGS, SHORT_TAGS, INT_TAGS, LONG_TAGS, FLOAT_TAGS, DOUBLE_TAGS), random -> randomTagOf(random, 1 + random.nextInt(6), 0)
    );
    static final Values.Generator COLLECTION_TAGS = mutable(
        anyEdges(LIST_TAGS, BYTE_ARRAY_TAGS, INT_ARRAY_TAGS, LONG_ARRAY_TAGS), random -> randomTagOf(random, new int[]{7, 9, 11, 12}[random.nextInt(4)], 0)
    );

    // Each tag's own type, ids next to theirs, and any ints.
    static final Values.Generator TAG_TYPES = new Values.Simple(
        List.of(
            TagTypes.getType(0), TagTypes.getType(1), TagTypes.getType(2), TagTypes.getType(3), TagTypes.getType(4), TagTypes.getType(5),
            TagTypes.getType(6), TagTypes.getType(7), TagTypes.getType(8), TagTypes.getType(9), TagTypes.getType(10), TagTypes.getType(11),
            TagTypes.getType(12), TagTypes.getType(13), TagTypes.getType(-1), TagTypes.getType(255), TagTypes.getType(Integer.MIN_VALUE)
        ),
        random -> TagTypes.getType(random.nextInt(4) == 0 ? (Integer)INTS.random(random) : random.nextInt(15) - 1),
        type -> new JsonPrimitive(((TagType<?>)type).getName())
    );

    static final Values.Generator ACCOUNTERS = new JomlValues.Mutable<>(
        accounterEdges(), NbtValues::randomAccounter, NbtValues::copy, NbtValues::encodeAccounter
    );

    static final Values.Generator DATA_INPUTS = new DataInputs();
    static final Values.Generator DATA_OUTPUTS = new DataOutputs();

    private static final Map<Class<?>, Values.Generator> GENERATORS = Map.ofEntries(
        Map.entry(Tag.class, TAGS), Map.entry(EndTag.class, END_TAGS), Map.entry(ByteTag.class, BYTE_TAGS), Map.entry(ShortTag.class, SHORT_TAGS),
        Map.entry(IntTag.class, INT_TAGS), Map.entry(LongTag.class, LONG_TAGS), Map.entry(FloatTag.class, FLOAT_TAGS),
        Map.entry(DoubleTag.class, DOUBLE_TAGS), Map.entry(ByteArrayTag.class, BYTE_ARRAY_TAGS), Map.entry(StringTag.class, STRING_TAGS),
        Map.entry(ListTag.class, LIST_TAGS), Map.entry(CompoundTag.class, COMPOUND_TAGS), Map.entry(IntArrayTag.class, INT_ARRAY_TAGS),
        Map.entry(LongArrayTag.class, LONG_ARRAY_TAGS), Map.entry(NumericTag.class, NUMERIC_TAGS), Map.entry(CollectionTag.class, COLLECTION_TAGS),
        Map.entry(TagType.class, TAG_TYPES), Map.entry(NbtAccounter.class, ACCOUNTERS), Map.entry(DataInput.class, DATA_INPUTS),
        Map.entry(ByteBufInputStream.class, DATA_INPUTS), Map.entry(DataOutput.class, DATA_OUTPUTS), Map.entry(ByteBufOutputStream.class, DATA_OUTPUTS)
    );

    private NbtValues() {
    }

    static Values.Generator generator(final Class<?> type) {
        return GENERATORS.get(type);
    }

    static boolean canEncode(final Class<?> type) {
        return Tag.class.isAssignableFrom(type) || TagType.class.isAssignableFrom(type) || type == NbtAccounter.class || type == Number.class
            || Map.Entry.class.isAssignableFrom(type) || DataInput.class.isAssignableFrom(type) || DataOutput.class.isAssignableFrom(type);
    }

    static JsonElement encode(final Object value) {
        return switch (value) {
            case Tag tag -> encodeTag(tag);
            case TagType<?> type -> new JsonPrimitive(type.getName());
            case NbtAccounter accounter -> encodeAccounter(accounter);
            case Number number -> Values.encode(number, number.getClass());
            case Map.Entry<?, ?> entry -> {
                JsonArray array = new JsonArray();
                array.add(Values.encode(entry.getKey(), entry.getKey().getClass()));
                array.add(Values.encode(entry.getValue(), entry.getValue().getClass()));
                yield array;
            }
            case ByteBufInputStream input -> NetworkValues.encode(inputBuffer(input));
            case ByteBufOutputStream output -> NetworkValues.encode(output.buffer());
            default -> throw new IllegalArgumentException("no encoder for " + value.getClass().getName());
        };
    }

    static JsonArray encodeTag(final Tag tag) {
        JsonArray array = new JsonArray();
        array.add(tag.getId());
        array.add(switch (tag) {
            case EndTag end -> JsonNull.INSTANCE;
            case ByteTag(byte value) -> new JsonPrimitive(value);
            case ShortTag(short value) -> new JsonPrimitive(value);
            case IntTag(int value) -> new JsonPrimitive(value);
            case LongTag(long value) -> new JsonPrimitive(value);
            case FloatTag(float value) -> new JsonPrimitive(Values.floatBits(value));
            case DoubleTag(double value) -> new JsonPrimitive(Values.doubleBits(value));
            case ByteArrayTag bytes -> Values.encode(bytes.getAsByteArray(), byte[].class);
            case StringTag(String value) -> new JsonPrimitive(value);
            case ListTag list -> {
                JsonArray elements = new JsonArray();
                list.forEach(element -> elements.add(encodeTag(element)));
                yield elements;
            }
            case CompoundTag compound -> {
                JsonArray entries = new JsonArray();
                tags(compound).forEach((key, value) -> {
                    JsonArray entry = new JsonArray();
                    entry.add(key);
                    entry.add(encodeTag(value));
                    entries.add(entry);
                });
                JsonArray map = new JsonArray();
                map.add(HashMapSuite.capacity(tags(compound)));
                map.add(entries);
                yield map;
            }
            case IntArrayTag ints -> Values.encode(ints.getAsIntArray(), int[].class);
            case LongArrayTag longs -> Values.encode(longs.getAsLongArray(), long[].class);
        });
        return array;
    }

    /**
     * A copy of the tag sharing nothing mutable with it: arrays copied, and each compound's map made with the same table
     * length, its tags put in the same order, so it iterates the same. A map made with a capacity has no table until
     * its first put, so an empty one with a table gets a key put and taken out again.
     */
    static Tag copy(final Tag tag) {
        return switch (tag) {
            case ListTag list -> {
                ListTag copy = new ListTag();
                list.forEach(element -> copy.add(copy(element)));
                yield copy;
            }
            case CompoundTag compound -> {
                HashMap<String, Tag> map = tags(compound);
                int capacity = HashMapSuite.capacity(map);
                HashMap<String, Tag> copy = capacity == 0 ? new HashMap<>() : new HashMap<>(capacity);
                if (capacity > 0 && map.isEmpty()) {
                    copy.put("", EndTag.INSTANCE);
                    copy.remove("");
                }
                map.forEach((key, value) -> copy.put(key, copy(value)));
                yield compound(copy);
            }
            case ByteArrayTag bytes -> new ByteArrayTag(bytes.getAsByteArray().clone());
            case IntArrayTag ints -> new IntArrayTag(ints.getAsIntArray().clone());
            case LongArrayTag longs -> new LongArrayTag(longs.getAsLongArray().clone());
            default -> tag;
        };
    }

    /** What writeAnyTag writes of the tag. */
    static byte[] anyTagBytes(final Tag tag) {
        ByteBuf buffer = Unpooled.buffer();
        try {
            NbtIo.writeAnyTag(tag, new ByteBufOutputStream(buffer));
        } catch (IOException e) {
            throw new IllegalStateException(e);
        }
        return NetworkValues.readable(buffer);
    }

    /** What the tag writes of itself: its payload, without its id. */
    static byte[] payloadBytes(final Tag tag) {
        ByteBuf buffer = Unpooled.buffer();
        try {
            tag.write(new ByteBufOutputStream(buffer));
        } catch (IOException e) {
            throw new IllegalStateException(e);
        }
        return NetworkValues.readable(buffer);
    }

    /** What writeUnnamedTag writes of the tag: its id, an empty name and its payload. */
    static byte[] unnamedTagBytes(final Tag tag) {
        ByteBuf buffer = Unpooled.buffer();
        try {
            NbtIo.writeUnnamedTag(tag, new ByteBufOutputStream(buffer));
        } catch (IOException e) {
            throw new IllegalStateException(e);
        }
        return NetworkValues.readable(buffer);
    }

    /** Whether the tag writes without throwing: none of its strings is too long for writeUTF. */
    static boolean writes(final Tag tag) {
        try {
            anyTagBytes(tag);
            return true;
        } catch (IllegalStateException e) {
            return false;
        }
    }

    // A tag of any kind but End, without lists and compounds past a depth of 3.
    static Tag randomTag(final Random random, final int depth) {
        return randomTagOf(random, depth >= 3 ? new int[]{1, 2, 3, 4, 5, 6, 7, 8, 11, 12}[random.nextInt(10)] : 1 + random.nextInt(12), depth);
    }

    static Tag randomTagOf(final Random random, final int id, final int depth) {
        return switch (id) {
            case 1 -> ByteTag.valueOf((Byte)BYTES.random(random));
            case 2 -> ShortTag.valueOf(randomShort(random));
            case 3 -> IntTag.valueOf((Integer)INTS.random(random));
            case 4 -> LongTag.valueOf((Long)LONGS.random(random));
            case 5 -> FloatTag.valueOf((Float)FLOATS.random(random));
            case 6 -> DoubleTag.valueOf((Double)DOUBLES.random(random));
            case 7 -> new ByteArrayTag((byte[])BYTE_ARRAYS.random(random));
            case 8 -> StringTag.valueOf((String)STRINGS.random(random));
            case 9 -> randomList(random, depth);
            case 10 -> randomCompound(random, depth);
            case 11 -> new IntArrayTag((int[])INT_ARRAYS.random(random));
            case 12 -> new LongArrayTag((long[])LONG_ARRAYS.random(random));
            default -> throw new IllegalArgumentException("tag id " + id);
        };
    }

    // Lists of one kind of tag mostly, of several now and then (written as compounds that wrap them), and of compounds,
    // some of which look like the ones that wrap (a single tag under the empty key).
    static ListTag randomList(final Random random, final int depth) {
        ListTag list = new ListTag();
        int size = random.nextInt(4) == 0 ? random.nextInt(20) : random.nextInt(5);
        int kind = depth >= 2 ? random.nextInt(3) : random.nextInt(4);
        int id = depth >= 1 ? 1 + random.nextInt(8) : 1 + random.nextInt(12);
        for (int i = 0; i < size; i++) {
            list.add(switch (kind) {
                case 0, 1 -> randomTagOf(random, id, depth + 1);
                case 2 -> randomTag(random, depth + 1);
                default -> random.nextInt(3) == 0 ? wrapping(randomTag(random, depth + 1)) : randomCompound(random, depth + 1);
            });
        }
        return list;
    }

    // Compounds of a few tags mostly, and now and then of enough numbers and strings that the table grows to 32 or 64,
    // some of them removed after (so the table is bigger than the compound needs), or copied like shallowCopy (so it's as
    // small as can be).
    static CompoundTag randomCompound(final Random random, final int depth) {
        CompoundTag compound = new CompoundTag();
        boolean big = depth == 0 && random.nextInt(5) == 0;
        int size = big ? 10 + random.nextInt(40) : random.nextInt(depth >= 2 ? 3 : 6);
        for (int i = 0; i < size; i++) {
            compound.put(randomKey(random), big ? randomTagOf(random, new int[]{1, 3, 5, 8}[random.nextInt(4)], depth + 1) : randomTag(random, depth + 1));
        }
        if (random.nextInt(4) == 0) {
            List<String> keys = new ArrayList<>(compound.keySet());
            for (int i = random.nextInt(keys.size() + 1); i > 0; i--) {
                compound.remove(keys.get(random.nextInt(keys.size())));
            }
        }
        return random.nextInt(6) == 0 ? shallowCopy(compound) : compound;
    }

    // Mostly a common key, a random string, or one of the keys whose hashCodes are equal.
    static String randomKey(final Random random) {
        return switch (random.nextInt(8)) {
            case 0, 1, 2, 3 -> COMMON_KEYS.get(random.nextInt(COMMON_KEYS.size()));
            case 4, 5 -> (String)STRINGS.random(random);
            default -> COLLIDING_KEYS.get(random.nextInt(COLLIDING_KEYS.size()));
        };
    }

    /** A compound that wraps the tag the way ListTag writes a list of several kinds: the tag under the empty key. */
    static CompoundTag wrapping(final Tag tag) {
        return compound("", tag);
    }

    /** CompoundTag.shallowCopy: a compound of a map copied with new HashMap<>(map), whose table is sized for its size. */
    static CompoundTag shallowCopy(final CompoundTag compound) {
        return compound(new HashMap<>(tags(compound)));
    }

    @SuppressWarnings("unchecked")
    static HashMap<String, Tag> tags(final CompoundTag compound) {
        try {
            return (HashMap<String, Tag>)COMPOUND_TAGS_FIELD.get(compound);
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
    }

    private static CompoundTag compound(final Map<String, Tag> tags) {
        try {
            return COMPOUND.newInstance(tags);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }

    /** Compounds nested the given depth, each holding the next under "a". */
    static Tag nestedCompounds(final int depth) {
        Tag tag = IntTag.valueOf(depth);
        for (int i = 0; i < depth; i++) {
            tag = compound("a", tag);
        }
        return tag;
    }

    /** Lists nested the given depth, each holding the next. */
    static Tag nestedLists(final int depth) {
        Tag tag = IntTag.valueOf(depth);
        for (int i = 0; i < depth; i++) {
            tag = list(tag);
        }
        return tag;
    }

    // Empty, of one kind, of two kinds (written as wrapping compounds), of compounds, a compound that wraps among
    // compounds (written wrapped again) and among others, nested, long, of strings, of a NaN and a double, and nested 17
    // deep.
    private static List<Tag> listEdges() {
        List<Tag> edges = new ArrayList<>();
        edges.add(new ListTag());
        edges.add(list(ByteTag.valueOf((byte)1), ByteTag.valueOf((byte)2)));
        edges.add(list(ByteTag.valueOf((byte)1), StringTag.valueOf("a")));
        edges.add(list(new CompoundTag(), compound("a", IntTag.valueOf(1))));
        edges.add(list(wrapping(IntTag.valueOf(1))));
        edges.add(list(wrapping(IntTag.valueOf(1)), IntTag.valueOf(2)));
        edges.add(list(IntTag.valueOf(1), wrapping(IntTag.valueOf(2))));
        edges.add(list(list(IntTag.valueOf(1)), list(IntTag.valueOf(2), IntTag.valueOf(3))));
        edges.add(list(new ListTag(), new ListTag()));
        ListTag ints = new ListTag();
        for (int i = 0; i < 20; i++) {
            ints.add(IntTag.valueOf(i * 31));
        }
        edges.add(ints);
        edges.add(list(StringTag.valueOf(""), StringTag.valueOf("minecraft:stone"), StringTag.valueOf("😀")));
        edges.add(list(FloatTag.valueOf(Float.NaN), DoubleTag.valueOf(-1.0)));
        edges.add(nestedLists(17));
        return edges;
    }

    // Empty (no table yet), a lone tag under the empty key (like a wrapping compound), an item's, one of every kind of
    // tag, 12 tags (a table of 16), 13 (32), 24 and 25 (64), 13 with 10 removed (a table of 32 for 3 tags), shallow
    // copies of 1, 3 and 6 tags (tables of 2, 4 and 8), keys whose hashCodes are equal, and nested 16 and 17 deep (the
    // click action's accounter takes 16).
    private static List<Tag> compoundEdges() {
        List<Tag> edges = new ArrayList<>();
        edges.add(new CompoundTag());
        edges.add(wrapping(IntTag.valueOf(1)));
        CompoundTag item = new CompoundTag();
        item.putString("id", "minecraft:stone");
        item.putInt("count", 1);
        edges.add(item);
        CompoundTag kinds = new CompoundTag();
        kinds.putByte("byte", (byte)1);
        kinds.putShort("short", (short)2);
        kinds.putInt("int", 3);
        kinds.putLong("long", 4L);
        kinds.putFloat("float", 5.0F);
        kinds.putDouble("double", 6.0);
        kinds.putByteArray("bytes", new byte[]{7});
        kinds.putString("string", "8");
        kinds.put("list", list(IntTag.valueOf(9)));
        kinds.put("compound", item.copy());
        kinds.putIntArray("ints", new int[]{11});
        kinds.putLongArray("longs", new long[]{12L});
        kinds.putBoolean("boolean", true);
        edges.add(kinds);
        for (int size : new int[]{12, 13, 24, 25}) {
            edges.add(numbered(size));
        }
        CompoundTag removed = numbered(13);
        for (int i = 0; i < 10; i++) {
            removed.remove("k" + i);
        }
        edges.add(removed);
        for (int size : new int[]{1, 3, 6}) {
            edges.add(shallowCopy(numbered(size)));
        }
        CompoundTag colliding = new CompoundTag();
        for (String key : COLLIDING_KEYS) {
            colliding.putInt(key, key.length());
        }
        edges.add(colliding);
        edges.add(nestedCompounds(16));
        edges.add(nestedCompounds(17));
        return edges;
    }

    /**
     * Tags holding strings around writeUTF's limit: each long string, as a compound's key and value (the longest that
     * writes and one byte more), and in a list.
     */
    static List<Tag> longTags() {
        List<Tag> tags = new ArrayList<>();
        LONG_STRINGS.forEach(string -> tags.add(StringTag.valueOf(string)));
        tags.add(compound("a".repeat(65535), IntTag.valueOf(1)));
        tags.add(compound("a".repeat(65536), IntTag.valueOf(1)));
        tags.add(compound("a", StringTag.valueOf("😀".repeat(10922) + "aaa")));
        tags.add(compound("a", StringTag.valueOf("😀".repeat(10922) + "aaaa")));
        tags.add(list(StringTag.valueOf("b"), StringTag.valueOf("a".repeat(65536))));
        tags.add(list(compound("b", IntTag.valueOf(1)), compound("a".repeat(65536), IntTag.valueOf(1))));
        return tags;
    }

    /**
     * Whether reading the buffer allocates little: a read with an accounter of the default quota doesn't run out of it.
     * A read only allocates arrays and lists after accounting for them, so one that fits the default quota reads the
     * same with more, and allocates at most that. A count read from random bytes can be in the billions, which a read
     * with a bigger quota (the trusted ones, or a usage whose sum overflows) would allocate an array of.
     */
    static boolean readsLittle(final ByteBuf buffer, final NbtAccounter accounter, final Reader reader) {
        if (accounter != null && usage(accounter) >= 0 && quota(accounter) <= NbtAccounter.DEFAULT_NBT_QUOTA) {
            return true;
        }
        int maxDepth = accounter == null ? 512 : maxDepth(accounter);
        NbtAccounter probe = accounter(NbtAccounter.DEFAULT_NBT_QUOTA, 0L, maxDepth, accounter == null ? 0 : depth(accounter));
        try {
            reader.read(new ByteBufInputStream(NetworkValues.copy(buffer)), probe);
        } catch (RuntimeException | IOException e) {
            return !(e.getMessage() != null && e.getMessage().contains("too big"));
        }
        return true;
    }

    /** A read from a DataInput with an accounter, which readsLittle tries with one of the default quota. */
    @FunctionalInterface
    interface Reader {
        void read(DataInput input, NbtAccounter accounter) throws IOException;
    }

    // Tags "k0" to "k{size - 1}", put in order.
    private static CompoundTag numbered(final int size) {
        CompoundTag compound = new CompoundTag();
        for (int i = 0; i < size; i++) {
            compound.putInt("k" + i, i);
        }
        return compound;
    }

    private static ListTag list(final Tag... tags) {
        ListTag list = new ListTag();
        for (Tag tag : tags) {
            list.add(tag);
        }
        return list;
    }

    private static CompoundTag compound(final String key, final Tag tag) {
        CompoundTag compound = new CompoundTag();
        compound.put(key, tag);
        return compound;
    }

    @SuppressWarnings("removal")
    private static FloatTag negativeZeroFloat() {
        return new FloatTag(-0.0F);
    }

    @SuppressWarnings("removal")
    private static DoubleTag negativeZeroDouble() {
        return new DoubleTag(-0.0);
    }

    // The game's accounters (a packet's default quota, the trusted ones, the click action's 32 KiB and 16 deep), small
    // quotas, small depths, ones already at their quota or their depth or at none, a usage whose next sum overflows,
    // and a negative quota.
    private static List<NbtAccounter> accounterEdges() {
        List<NbtAccounter> edges = new ArrayList<>(List.of(
            NbtAccounter.defaultQuota(), NbtAccounter.unlimitedHeap(), NbtAccounter.uncompressedQuota(), new NbtAccounter(32768L, 16),
            NbtAccounter.create(0L), NbtAccounter.create(8L), NbtAccounter.create(36L), NbtAccounter.create(100L), new NbtAccounter(Long.MAX_VALUE, 0),
            new NbtAccounter(Long.MAX_VALUE, 1), new NbtAccounter(Long.MAX_VALUE, 2), new NbtAccounter(Long.MAX_VALUE, 3), new NbtAccounter(-1L, 512)
        ));
        edges.add(accounter(2097152L, 2097150L, 512, 0));
        edges.add(accounter(2097152L, 0L, 512, 512));
        edges.add(accounter(2097152L, 0L, 512, 511));
        edges.add(accounter(32768L, 0L, 16, 1));
        edges.add(accounter(Long.MAX_VALUE, Long.MAX_VALUE - 4L, 512, 0));
        return edges;
    }

    private static NbtAccounter randomAccounter(final Random random) {
        long quota = switch (random.nextInt(4)) {
            case 0 -> 2097152L;
            case 1 -> random.nextInt(2000);
            case 2 -> Long.MAX_VALUE;
            default -> random.nextLong();
        };
        int maxDepth = random.nextBoolean() ? 512 : random.nextInt(20);
        long usage = random.nextInt(3) == 0 && quota > 0 ? (long)(random.nextDouble() * quota) : 0L;
        int depth = random.nextInt(3) == 0 && maxDepth > 0 ? random.nextInt(maxDepth + 1) : 0;
        return accounter(quota, usage, maxDepth, depth);
    }

    private static NbtAccounter accounter(final long quota, final long usage, final int maxDepth, final int depth) {
        NbtAccounter accounter = new NbtAccounter(quota, maxDepth);
        try {
            USAGE.setLong(accounter, usage);
            DEPTH.setInt(accounter, depth);
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
        return accounter;
    }

    private static long quota(final NbtAccounter accounter) {
        return field(QUOTA, accounter).longValue();
    }

    private static long usage(final NbtAccounter accounter) {
        return field(USAGE, accounter).longValue();
    }

    private static int maxDepth(final NbtAccounter accounter) {
        return field(MAX_DEPTH, accounter).intValue();
    }

    private static int depth(final NbtAccounter accounter) {
        return field(DEPTH, accounter).intValue();
    }

    private static Number field(final Field field, final NbtAccounter accounter) {
        try {
            return (Number)field.get(accounter);
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
    }

    private static NbtAccounter copy(final NbtAccounter accounter) {
        try {
            return accounter(QUOTA.getLong(accounter), USAGE.getLong(accounter), MAX_DEPTH.getInt(accounter), DEPTH.getInt(accounter));
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
    }

    private static JsonArray encodeAccounter(final NbtAccounter accounter) {
        try {
            JsonArray array = new JsonArray();
            array.add(QUOTA.getLong(accounter));
            array.add(USAGE.getLong(accounter));
            array.add(MAX_DEPTH.getInt(accounter));
            array.add(DEPTH.getInt(accounter));
            return array;
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
    }

    static ByteBuf inputBuffer(final ByteBufInputStream input) {
        try {
            return (ByteBuf)INPUT_BUFFER.get(input);
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
    }

    /**
     * Whether no string modified UTF-8 reads from the bytes can hold a lone surrogate, which neither the golden files'
     * UTF-8 nor the port's Rust strings can hold: every encoded surrogate (ED A0 to ED BF) is a high one followed by a
     * low one.
     */
    static boolean noLoneSurrogates(final byte[] bytes) {
        for (int i = 0; i + 1 < bytes.length; i++) {
            if ((bytes[i] & 0xff) != 0xed || (bytes[i + 1] & 0xe0) != 0xa0) {
                continue;
            }
            boolean high = (bytes[i + 1] & 0xf0) == 0xa0;
            boolean paired = high && i + 4 < bytes.length && (bytes[i + 3] & 0xff) == 0xed && (bytes[i + 4] & 0xf0) == 0xb0;
            if (!paired) {
                return false;
            }
            i += 5;
        }
        return true;
    }

    // Buffers a DataInput reads from: the buffer edges and random buffers half the time, and NBT the other half, as
    // writeAnyTag, writeUnnamedTag or a tag's own write writes it, whole, cut short or with a field after.
    private static final class DataInputs implements Values.Generator {
        @Override
        public List<Object> edges() {
            List<Object> edges = new ArrayList<>();
            for (Object buffer : NetworkValues.BUFFER.edges()) {
                if (noLoneSurrogates(NetworkValues.array((ByteBuf)buffer))) {
                    edges.add(buffer);
                }
            }
            edges.add(NetworkValues.buffer(0, 0, anyTagBytes(new CompoundTag())));
            edges.add(NetworkValues.buffer(0, 0, anyTagBytes(compoundEdges().get(3))));
            edges.add(NetworkValues.buffer(0, 0, unnamedTagBytes(compoundEdges().get(2))));
            edges.add(NetworkValues.buffer(0, 0, anyTagBytes(listEdges().get(2))));
            return edges;
        }

        @Override
        public Object random(final Random random) {
            while (true) {
                ByteBuf buffer = random.nextBoolean() ? NetworkValues.randomBuffer(random) : nbtBuffer(random);
                if (noLoneSurrogates(NetworkValues.array(buffer))) {
                    return buffer;
                }
            }
        }

        private static ByteBuf nbtBuffer(final Random random) {
            Tag tag = randomTag(random, 0);
            while (!writes(tag)) {
                tag = randomTag(random, 0);
            }
            byte[] bytes = switch (random.nextInt(3)) {
                case 0 -> anyTagBytes(tag);
                case 1 -> unnamedTagBytes(tag);
                default -> payloadBytes(tag);
            };
            List<byte[]> variants = Codecs.variants(bytes, random);
            return NetworkValues.buffer(random.nextInt(17), 0, variants.get(random.nextInt(variants.size())));
        }

        @Override
        public JsonElement encode(final Object spec) {
            return spec instanceof ByteBufInputStream input ? NetworkValues.encode(inputBuffer(input)) : NetworkValues.encode(spec);
        }

        @Override
        public Object instantiate(final Object spec) {
            return new ByteBufInputStream(NetworkValues.copy((ByteBuf)spec));
        }

        @Override
        public JsonElement after(final Object spec, final Object instance) {
            JsonElement state = this.encode(instance);
            return state.equals(this.encode(spec)) ? null : state;
        }
    }

    // Buffers a DataOutput writes to: the buffer edges and random buffers.
    private static final class DataOutputs implements Values.Generator {
        @Override
        public List<Object> edges() {
            return NetworkValues.BUFFER.edges();
        }

        @Override
        public Object random(final Random random) {
            return NetworkValues.randomBuffer(random);
        }

        @Override
        public JsonElement encode(final Object spec) {
            return spec instanceof ByteBufOutputStream output ? NetworkValues.encode(output.buffer()) : NetworkValues.encode(spec);
        }

        @Override
        public Object instantiate(final Object spec) {
            return new ByteBufOutputStream(NetworkValues.copy((ByteBuf)spec));
        }

        @Override
        public JsonElement after(final Object spec, final Object instance) {
            JsonElement state = this.encode(instance);
            return state.equals(this.encode(spec)) ? null : state;
        }
    }

    // A few edges of each kind, interleaved, so the first of each come first.
    private static List<Tag> anyEdges(final Values.Generator... kinds) {
        List<Tag> edges = new ArrayList<>();
        for (int i = 0; i < 4; i++) {
            for (Values.Generator kind : kinds) {
                if (i < kind.edges().size()) {
                    edges.add((Tag)kind.edges().get(i));
                }
            }
        }
        return edges;
    }

    private static Values.Generator immutable(final List<Object> edges, final Function<Random, Object> fresh) {
        return new Values.Simple(edges, fresh, tag -> encodeTag((Tag)tag));
    }

    // Tags holding arrays, lists or compounds: each argument gets its own copy.
    private static Values.Generator mutable(final List<Tag> edges, final Function<Random, Tag> fresh) {
        return new JomlValues.Mutable<>(edges, fresh, NbtValues::copy, NbtValues::encodeTag);
    }

    private static short randomShort(final Random random) {
        return random.nextInt(4) == 0 ? (Short)SHORT_EDGES.get(random.nextInt(SHORT_EDGES.size())) : (short)random.nextInt();
    }

    private static List<Object> concat(final List<Object> first, final List<?> second) {
        List<Object> all = new ArrayList<>(first);
        all.addAll(second);
        return all;
    }

    @SuppressWarnings("unchecked")
    private static Constructor<CompoundTag> compoundConstructor() {
        try {
            Constructor<CompoundTag> constructor = CompoundTag.class.getDeclaredConstructor(Map.class);
            constructor.setAccessible(true);
            return constructor;
        } catch (NoSuchMethodException e) {
            throw new IllegalStateException(e);
        }
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
}
