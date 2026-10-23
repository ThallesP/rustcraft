package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import com.mojang.datafixers.util.Pair;
import com.mojang.serialization.DataResult;
import com.mojang.serialization.Lifecycle;
import com.mojang.serialization.ListBuilder;
import com.mojang.serialization.MapLike;
import com.mojang.serialization.RecordBuilder;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import net.minecraft.nbt.ByteArrayTag;
import net.minecraft.nbt.ByteTag;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.IntArrayTag;
import net.minecraft.nbt.IntTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.LongArrayTag;
import net.minecraft.nbt.LongTag;
import net.minecraft.nbt.NbtOps;
import net.minecraft.nbt.StringTag;
import net.minecraft.nbt.Tag;

/**
 * {@code net.minecraft.nbt.NbtOps.INSTANCE}: each method on tags as {@code NbtValues} makes them, with results and
 * numbers as {@code DfuValues} encodes them, a stream or a map's entries as an array, and the list and record builders
 * as programs. A case's arguments are the tags it was given; one that throws is recorded as throwing.
 *
 * <p>Merging into a list gets lists and arrays merged with tags of their elements' kind half the time, so arrays stay
 * arrays and turn into lists. Merging into a map gets compounds, ends and other tags, with string keys mostly and other
 * tags now and then. A {@code Map} argument is a {@code LinkedHashMap} of the entries in order (a key twice keeps its
 * first place and its last value), and a {@code MapLike} argument is {@code getMap} of the compound given.
 */
final class NbtOpsSuite implements Suite {
    private static final NbtOps OPS = NbtOps.INSTANCE;
    private static final int RANDOM = 300;
    private static final List<String> KEYS = List.of("a", "b", "text", "color", "", "extra", "é");

    @Override
    public String name() {
        return "net.minecraft.nbt.NbtOps";
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Random random = new Random(44);
        List<Tag> tags = new ArrayList<>();
        for (Object edge : NbtValues.TAGS.edges()) {
            tags.add((Tag)edge);
        }
        for (int i = 0; i < RANDOM; i++) {
            tags.add((Tag)NbtValues.TAGS.random(random));
        }

        writer.result("toString()Ljava/lang/String;", new JsonArray(), new JsonPrimitive(OPS.toString()), null);
        writer.result("empty()Lnet/minecraft/nbt/Tag;", new JsonArray(), NbtValues.encodeTag(OPS.empty()), null);
        writer.result("emptyList()Lnet/minecraft/nbt/Tag;", new JsonArray(), NbtValues.encodeTag(OPS.emptyList()), null);
        writer.result("emptyMap()Lnet/minecraft/nbt/Tag;", new JsonArray(), NbtValues.encodeTag(OPS.emptyMap()), null);

        for (Tag tag : tags) {
            JsonArray args = args(tag);
            writer.result("getNumberValue(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", args, DfuValues.result(OPS.getNumberValue(tag), DfuValues::number), null);
            writer.result("getBooleanValue(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", args, DfuValues.result(OPS.getBooleanValue(tag), JsonPrimitive::new), null);
            writer.result("getStringValue(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", args, DfuValues.result(OPS.getStringValue(tag), JsonPrimitive::new), null);
            writer.result("getMapValues(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", args, DfuValues.result(OPS.getMapValues(tag), stream -> pairs(stream.toList())), null);
            List<Pair<Tag, Tag>> entries = new ArrayList<>();
            writer.result(
                "getMapEntries(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", args,
                DfuValues.result(OPS.getMapEntries(tag), consumer -> {
                    consumer.accept((key, value) -> entries.add(Pair.of(key, value)));
                    return pairs(entries);
                }), null
            );
            writer.result("getMap(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", args, DfuValues.result(OPS.getMap(tag), NbtOpsSuite::mapLike), null);
            writer.result("getStream(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", args, DfuValues.result(OPS.getStream(tag), stream -> tags(stream.toList())), null);
            List<Tag> elements = new ArrayList<>();
            writer.result(
                "getList(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", args,
                DfuValues.result(OPS.getList(tag), consumer -> {
                    consumer.accept(elements::add);
                    return tags(elements);
                }), null
            );
            writer.result("getByteBuffer(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", args, DfuValues.result(OPS.getByteBuffer(tag), buffer -> Values.encode(buffer.array(), byte[].class)), null);
            writer.result("getIntStream(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", args, DfuValues.result(OPS.getIntStream(tag), stream -> Values.encode(stream.toArray(), int[].class)), null);
            writer.result("getLongStream(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", args, DfuValues.result(OPS.getLongStream(tag), stream -> Values.encode(stream.toArray(), long[].class)), null);
            writer.result("convertTo(Lcom/mojang/serialization/DynamicOps;Lnet/minecraft/nbt/Tag;)Ljava/lang/Object;", args, NbtValues.encodeTag(OPS.convertTo(OPS, tag)), null);
            writer.result("createString(Ljava/lang/String;)Lnet/minecraft/nbt/Tag;", args, NbtValues.encodeTag(OPS.createString(tag.toString())), null);

            String key = KEYS.get(random.nextInt(KEYS.size()));
            JsonArray keyArgs = args(tag);
            keyArgs.add(key);
            writer.result("remove(Lnet/minecraft/nbt/Tag;Ljava/lang/String;)Lnet/minecraft/nbt/Tag;", keyArgs, NbtValues.encodeTag(OPS.remove(tag, key)), null);
            writer.result("get(Lnet/minecraft/nbt/Tag;Ljava/lang/String;)Lcom/mojang/serialization/DataResult;", keyArgs, DfuValues.result(OPS.get(tag, key), NbtValues::encodeTag), null);

            Tag value = (Tag)NbtValues.TAGS.random(random);
            JsonArray setArgs = args(tag, value);
            setArgs.add(key);
            writer.result("set(Lnet/minecraft/nbt/Tag;Ljava/lang/String;Lnet/minecraft/nbt/Tag;)Lnet/minecraft/nbt/Tag;", setArgs, NbtValues.encodeTag(OPS.set(tag, key, value)), null);
            writer.result("mergeToPrimitive(Lnet/minecraft/nbt/Tag;Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", args(tag, value), DfuValues.result(OPS.mergeToPrimitive(tag, value), NbtValues::encodeTag), null);

            Tag genericKey = random.nextInt(4) == 0 ? (Tag)NbtValues.TAGS.random(random) : StringTag.valueOf(key);
            JsonArray genericArgs = args(tag, genericKey);
            try {
                writer.result("getGeneric(Lnet/minecraft/nbt/Tag;Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", genericArgs, DfuValues.result(OPS.getGeneric(tag, genericKey), NbtValues::encodeTag), null);
            } catch (UnsupportedOperationException e) {
                writer.threw("getGeneric(Lnet/minecraft/nbt/Tag;Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", genericArgs, e, null);
            }

            Number fallback = randomNumber(random);
            JsonArray fallbackArgs = args(tag);
            fallbackArgs.add(DfuValues.number(fallback));
            writer.result("getNumberValue(Lnet/minecraft/nbt/Tag;Ljava/lang/Number;)Ljava/lang/Number;", fallbackArgs, DfuValues.number(OPS.getNumberValue(tag, fallback)), null);
        }

        for (int i = 0; i < 200; i++) {
            Number number = randomNumber(random);
            JsonArray args = new JsonArray();
            args.add(DfuValues.number(number));
            writer.result("createNumeric(Ljava/lang/Number;)Lnet/minecraft/nbt/Tag;", args, NbtValues.encodeTag(OPS.createNumeric(number)), null);
            JsonElement value = Values.encode(number, number.getClass());
            JsonArray valueArgs = new JsonArray();
            valueArgs.add(value);
            switch (number) {
                case Byte b -> writer.result("createByte(B)Lnet/minecraft/nbt/Tag;", valueArgs, NbtValues.encodeTag(OPS.createByte(b)), null);
                case Short s -> writer.result("createShort(S)Lnet/minecraft/nbt/Tag;", valueArgs, NbtValues.encodeTag(OPS.createShort(s)), null);
                case Integer n -> writer.result("createInt(I)Lnet/minecraft/nbt/Tag;", valueArgs, NbtValues.encodeTag(OPS.createInt(n)), null);
                case Long l -> writer.result("createLong(J)Lnet/minecraft/nbt/Tag;", valueArgs, NbtValues.encodeTag(OPS.createLong(l)), null);
                case Float f -> writer.result("createFloat(F)Lnet/minecraft/nbt/Tag;", valueArgs, NbtValues.encodeTag(OPS.createFloat(f)), null);
                default -> writer.result("createDouble(D)Lnet/minecraft/nbt/Tag;", valueArgs, NbtValues.encodeTag(OPS.createDouble((Double)number)), null);
            }
        }
        for (boolean value : new boolean[]{false, true}) {
            JsonArray args = new JsonArray();
            args.add(value);
            writer.result("createBoolean(Z)Lnet/minecraft/nbt/Tag;", args, NbtValues.encodeTag(OPS.createBoolean(value)), null);
        }

        for (int i = 0; i < 400; i++) {
            Tag list = randomListLike(random);
            Tag value = randomElementFor(random, list);
            writer.result("mergeToList(Lnet/minecraft/nbt/Tag;Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", args(list, value), DfuValues.result(OPS.mergeToList(list, value), NbtValues::encodeTag), null);

            List<Tag> values = new ArrayList<>();
            for (int n = random.nextInt(5); n > 0; n--) {
                values.add(randomElementFor(random, list));
            }
            JsonArray listArgs = args(list);
            listArgs.add(tags(values));
            writer.result("mergeToList(Lnet/minecraft/nbt/Tag;Ljava/util/List;)Lcom/mojang/serialization/DataResult;", listArgs, DfuValues.result(OPS.mergeToList(list, values), NbtValues::encodeTag), null);

            List<Tag> created = new ArrayList<>();
            for (int n = random.nextInt(5); n > 0; n--) {
                created.add((Tag)NbtValues.TAGS.random(random));
            }
            JsonArray createArgs = new JsonArray();
            createArgs.add(tags(created));
            writer.result("createList(Ljava/util/stream/Stream;)Lnet/minecraft/nbt/Tag;", createArgs, NbtValues.encodeTag(OPS.createList(created.stream())), null);
        }

        for (int i = 0; i < 400; i++) {
            Tag map = randomMapLike(random);
            Tag key = randomKey(random);
            Tag value = (Tag)NbtValues.TAGS.random(random);
            writer.result(
                "mergeToMap(Lnet/minecraft/nbt/Tag;Lnet/minecraft/nbt/Tag;Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;", args(map, key, value),
                DfuValues.result(OPS.mergeToMap(map, key, value), NbtValues::encodeTag), null
            );

            Map<Tag, Tag> entries = new LinkedHashMap<>();
            for (int n = random.nextInt(5); n > 0; n--) {
                entries.put(randomKey(random), (Tag)NbtValues.TAGS.random(random));
            }
            JsonArray mapArgs = args(map);
            mapArgs.add(pairs(entries.entrySet().stream().map(entry -> Pair.of(entry.getKey(), entry.getValue())).toList()));
            writer.result("mergeToMap(Lnet/minecraft/nbt/Tag;Ljava/util/Map;)Lcom/mojang/serialization/DataResult;", mapArgs, DfuValues.result(OPS.mergeToMap(map, entries), NbtValues::encodeTag), null);

            CompoundTag source = NbtValues.randomCompound(random, 1);
            JsonArray mapLikeArgs = args(map, source);
            writer.result(
                "mergeToMap(Lnet/minecraft/nbt/Tag;Lcom/mojang/serialization/MapLike;)Lcom/mojang/serialization/DataResult;", mapLikeArgs,
                DfuValues.result(OPS.mergeToMap(map, OPS.getMap(source).getOrThrow()), NbtValues::encodeTag), null
            );

            List<Pair<Tag, Tag>> created = new ArrayList<>();
            for (int n = random.nextInt(5); n > 0; n--) {
                created.add(Pair.of(random.nextInt(12) == 0 ? randomKey(random) : StringTag.valueOf(KEYS.get(random.nextInt(KEYS.size()))), (Tag)NbtValues.TAGS.random(random)));
            }
            JsonArray createArgs = new JsonArray();
            createArgs.add(pairs(created));
            try {
                writer.result("createMap(Ljava/util/stream/Stream;)Lnet/minecraft/nbt/Tag;", createArgs, NbtValues.encodeTag(OPS.createMap(created.stream())), null);
            } catch (UnsupportedOperationException e) {
                writer.threw("createMap(Ljava/util/stream/Stream;)Lnet/minecraft/nbt/Tag;", createArgs, e, null);
            }
        }

        for (int i = 0; i < 100; i++) {
            byte[] bytes = (byte[])Values.generator(byte[].class).random(random);
            JsonArray byteArgs = new JsonArray();
            byteArgs.add(Values.encode(bytes, byte[].class));
            writer.result("createByteList(Ljava/nio/ByteBuffer;)Lnet/minecraft/nbt/Tag;", byteArgs, NbtValues.encodeTag(OPS.createByteList(ByteBuffer.wrap(bytes))), null);
            int[] ints = (int[])Values.generator(int[].class).random(random);
            JsonArray intArgs = new JsonArray();
            intArgs.add(Values.encode(ints, int[].class));
            writer.result("createIntList(Ljava/util/stream/IntStream;)Lnet/minecraft/nbt/Tag;", intArgs, NbtValues.encodeTag(OPS.createIntList(Arrays.stream(ints))), null);
            long[] longs = (long[])Values.generator(long[].class).random(random);
            JsonArray longArgs = new JsonArray();
            longArgs.add(Values.encode(longs, long[].class));
            writer.result("createLongList(Ljava/util/stream/LongStream;)Lnet/minecraft/nbt/Tag;", longArgs, NbtValues.encodeTag(OPS.createLongList(Arrays.stream(longs))), null);
        }

        for (int i = 0; i < 300; i++) {
            listBuilderProgram(writer, random);
            mapBuilderProgram(writer, random);
        }
    }

    // A list builder's ops in turn, each step the builder's build of a copy of the ops so far (a builder can't be
    // looked into without building it, which resets it), then the program's own build.
    private static void listBuilderProgram(final GoldenWriter writer, final Random random) throws IOException {
        JsonArray ops = new JsonArray();
        ListBuilder<Tag> builder = OPS.listBuilder();
        for (int step = random.nextInt(6); step > 0; step--) {
            JsonArray op = new JsonArray();
            switch (random.nextInt(4)) {
                case 0 -> {
                    Tag value = (Tag)NbtValues.TAGS.random(random);
                    op.add("add");
                    op.add(NbtValues.encodeTag(value));
                    builder.add(value);
                }
                case 1 -> {
                    DataResult<Tag> value = randomResult(random);
                    op.add("addResult");
                    op.add(DfuValues.result(value, NbtValues::encodeTag));
                    builder.add(value);
                }
                case 2 -> {
                    DataResult<Tag> value = randomResult(random);
                    op.add("withErrorsFrom");
                    op.add(DfuValues.result(value, NbtValues::encodeTag));
                    builder.withErrorsFrom(value);
                }
                default -> {
                    op.add("mapError");
                    builder.mapError(message -> "<" + message + ">");
                }
            }
            ops.add(op);
        }
        Tag prefix = random.nextInt(3) == 0 ? (Tag)NbtValues.TAGS.random(random) : random.nextBoolean() ? OPS.empty() : NbtValues.randomList(random, 1);
        JsonArray args = new JsonArray();
        args.add(ops);
        args.add(NbtValues.encodeTag(prefix));
        JsonArray result = new JsonArray();
        result.add(DfuValues.result(builder.build(prefix), NbtValues::encodeTag));
        result.add(DfuValues.result(builder.build(OPS.empty()), NbtValues::encodeTag));
        writer.result("listBuilder programs", args, result, null);
    }

    // A record builder's ops in turn, then its build of the prefix, and a second build, of what's left after the first.
    private static void mapBuilderProgram(final GoldenWriter writer, final Random random) throws IOException {
        JsonArray ops = new JsonArray();
        RecordBuilder<Tag> builder = OPS.mapBuilder();
        for (int step = random.nextInt(7); step > 0; step--) {
            JsonArray op = new JsonArray();
            switch (random.nextInt(8)) {
                case 0 -> {
                    String key = KEYS.get(random.nextInt(KEYS.size()));
                    Tag value = (Tag)NbtValues.TAGS.random(random);
                    op.add("addString");
                    op.add(key);
                    op.add(NbtValues.encodeTag(value));
                    builder.add(key, value);
                }
                case 1 -> {
                    String key = KEYS.get(random.nextInt(KEYS.size()));
                    DataResult<Tag> value = randomResult(random);
                    op.add("addStringResult");
                    op.add(key);
                    op.add(DfuValues.result(value, NbtValues::encodeTag));
                    builder.add(key, value);
                }
                case 2 -> {
                    Tag key = randomKey(random);
                    Tag value = (Tag)NbtValues.TAGS.random(random);
                    op.add("add");
                    op.add(NbtValues.encodeTag(key));
                    op.add(NbtValues.encodeTag(value));
                    builder.add(key, value);
                }
                case 3 -> {
                    Tag key = randomKey(random);
                    DataResult<Tag> value = randomResult(random);
                    op.add("addResult");
                    op.add(NbtValues.encodeTag(key));
                    op.add(DfuValues.result(value, NbtValues::encodeTag));
                    builder.add(key, value);
                }
                case 4 -> {
                    DataResult<Tag> key = random.nextBoolean() ? DataResult.success(randomKey(random)) : randomResult(random);
                    DataResult<Tag> value = randomResult(random);
                    op.add("addResults");
                    op.add(DfuValues.result(key, NbtValues::encodeTag));
                    op.add(DfuValues.result(value, NbtValues::encodeTag));
                    builder.add(key, value);
                }
                case 5 -> {
                    DataResult<Tag> value = randomResult(random);
                    op.add("withErrorsFrom");
                    op.add(DfuValues.result(value, NbtValues::encodeTag));
                    builder.withErrorsFrom(value);
                }
                case 6 -> {
                    Lifecycle lifecycle = randomLifecycle(random);
                    op.add("setLifecycle");
                    op.add(DfuValues.lifecycle(lifecycle));
                    builder.setLifecycle(lifecycle);
                }
                default -> {
                    op.add("mapError");
                    builder.mapError(message -> "<" + message + ">");
                }
            }
            ops.add(op);
        }
        Tag prefix = random.nextInt(3) == 0 ? (Tag)NbtValues.TAGS.random(random) : random.nextBoolean() ? OPS.empty() : NbtValues.randomCompound(random, 1);
        JsonArray args = new JsonArray();
        args.add(ops);
        args.add(NbtValues.encodeTag(prefix));
        JsonArray result = new JsonArray();
        result.add(DfuValues.result(builder.build(prefix), NbtValues::encodeTag));
        result.add(DfuValues.result(builder.build(OPS.empty()), NbtValues::encodeTag));
        writer.result("mapBuilder programs", args, result, null);
    }

    // A success or an error, with or without a partial tag, with any lifecycle.
    private static DataResult<Tag> randomResult(final Random random) {
        Tag value = (Tag)NbtValues.TAGS.random(random);
        Lifecycle lifecycle = randomLifecycle(random);
        String message = List.of("Not a number", "x", "Missing", "a; b").get(random.nextInt(4));
        return switch (random.nextInt(4)) {
            case 0, 1 -> DataResult.success(value, lifecycle);
            case 2 -> DataResult.error(() -> message, value, lifecycle);
            default -> DataResult.error(() -> message, lifecycle);
        };
    }

    private static Lifecycle randomLifecycle(final Random random) {
        return switch (random.nextInt(4)) {
            case 0 -> Lifecycle.stable();
            case 1 -> Lifecycle.experimental();
            default -> Lifecycle.deprecated(1 + random.nextInt(2));
        };
    }

    private static Number randomNumber(final Random random) {
        return switch (random.nextInt(6)) {
            case 0 -> (Byte)Values.generator(byte.class).random(random);
            case 1 -> (short)random.nextInt();
            case 2 -> (Integer)Values.generator(int.class).random(random);
            case 3 -> (Long)Values.generator(long.class).random(random);
            case 4 -> (Float)Values.generator(float.class).random(random);
            default -> (Double)Values.generator(double.class).random(random);
        };
    }

    // A list, an array, an empty one, an end, or another tag.
    private static Tag randomListLike(final Random random) {
        return switch (random.nextInt(8)) {
            case 0 -> OPS.empty();
            case 1 -> (Tag)NbtValues.TAGS.random(random);
            case 2 -> new ListTag();
            default -> (Tag)NbtValues.COLLECTION_TAGS.random(random);
        };
    }

    // A tag of the list's elements' kind half the time, any tag otherwise.
    private static Tag randomElementFor(final Random random, final Tag list) {
        if (random.nextBoolean()) {
            switch (list) {
                case ByteArrayTag array -> {
                    return ByteTag.valueOf((Byte)Values.generator(byte.class).random(random));
                }
                case IntArrayTag array -> {
                    return IntTag.valueOf((Integer)Values.generator(int.class).random(random));
                }
                case LongArrayTag array -> {
                    return LongTag.valueOf((Long)Values.generator(long.class).random(random));
                }
                case ListTag listTag when !listTag.isEmpty() -> {
                    return NbtValues.randomTagOf(random, listTag.get(0).getId() == 0 ? 1 : listTag.get(0).getId(), 1);
                }
                default -> {
                }
            }
        }
        return (Tag)NbtValues.TAGS.random(random);
    }

    // A compound, an end, an empty compound, or another tag.
    private static Tag randomMapLike(final Random random) {
        return switch (random.nextInt(8)) {
            case 0 -> OPS.empty();
            case 1 -> (Tag)NbtValues.TAGS.random(random);
            case 2 -> new CompoundTag();
            default -> NbtValues.randomCompound(random, 1);
        };
    }

    // A string key mostly, another tag now and then.
    private static Tag randomKey(final Random random) {
        return random.nextInt(10) == 0 ? (Tag)NbtValues.TAGS.random(random) : StringTag.valueOf(KEYS.get(random.nextInt(KEYS.size())));
    }

    // The MapLike's toString, its entries, and what it holds under each key KEYS has.
    private static JsonElement mapLike(final MapLike<Tag> map) {
        JsonArray array = new JsonArray();
        array.add(map.toString());
        array.add(pairs(map.entries().toList()));
        JsonArray lookups = new JsonArray();
        for (String key : KEYS) {
            Tag byString = map.get(key);
            Tag byTag = map.get(StringTag.valueOf(key));
            lookups.add(byString == null ? JsonNull.INSTANCE : NbtValues.encodeTag(byString));
            lookups.add(byTag == null ? JsonNull.INSTANCE : NbtValues.encodeTag(byTag));
        }
        array.add(lookups);
        return array;
    }

    private static JsonArray args(final Tag... tags) {
        JsonArray args = new JsonArray();
        for (Tag tag : tags) {
            args.add(NbtValues.encodeTag(tag));
        }
        return args;
    }

    private static JsonArray tags(final List<Tag> tags) {
        JsonArray array = new JsonArray();
        tags.forEach(tag -> array.add(NbtValues.encodeTag(tag)));
        return array;
    }

    private static JsonArray pairs(final List<Pair<Tag, Tag>> pairs) {
        JsonArray array = new JsonArray();
        pairs.forEach(pair -> array.add(DfuValues.pair(pair, NbtValues::encodeTag, NbtValues::encodeTag)));
        return array;
    }
}
