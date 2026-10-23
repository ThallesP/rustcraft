package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import com.mojang.datafixers.util.Either;
import com.mojang.datafixers.util.Pair;
import com.mojang.serialization.Codec;
import com.mojang.serialization.DataResult;
import com.mojang.serialization.MapCodec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.Random;
import java.util.function.Function;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.NbtOps;
import net.minecraft.nbt.Tag;

/**
 * DataFixerUpper's codecs on NBT: the primitive codecs, and codecs made with each combinator the game's codecs reach
 * ({@code listOf}, {@code either}, {@code xmap}, {@code comapFlatMap}, {@code flatComapMap}, {@code flatXmap},
 * {@code validate}, {@code withAlternative}, {@code fieldOf}, {@code optionalFieldOf} and its lenient and defaulted
 * forms, {@code RecordCodecBuilder} of 1 to 16 fields, {@code dispatch} and {@code partialDispatch}, a map codec's
 * {@code xmap}, {@code flatXmap} and {@code stable}, and {@code recursive}), each named by how it's made.
 *
 * <p>A codec named N gets {@code N.decode} cases, a tag decoded (the result a {@code DataResult} of {@code [value,
 * rest]}), and {@code N.encode} cases, a value encoded onto a prefix (a {@code DataResult} of the tag). Values are
 * encoded by shape: numbers as {@code Values} encodes them, a list as an array, an {@code Optional} as its value or
 * null, an {@code Either} as {@code ["left", value]} or {@code ["right", value]}, a record as the array of its fields,
 * and a tree (the recursive codec's) as its {@code Either}. The tags decoded are the codec's own encodings of random
 * values (the rest of the cases), those with one tag inside them replaced, removed or added, and any tags; prefixes
 * are the end mostly, then compounds and any tags.
 */
final class DfuCodecSuite implements Suite {
    private static final NbtOps OPS = NbtOps.INSTANCE;
    private static final int VALUES = 150;
    private static final List<String> STRINGS = List.of("", "a", "text", "12", "-7", "2147483648", "é", "int", "pair", "true", " 3");

    @Override
    public String name() {
        return "com.mojang.serialization.Codec";
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Random random = new Random(45);
        for (Named named : codecs()) {
            named.write(writer, random);
        }
    }

    private record Named(String name, Codec<Object> codec, Shape shape) {
        void write(final GoldenWriter writer, final Random random) throws IOException {
            List<Object> values = new ArrayList<>();
            for (int i = 0; i < VALUES; i++) {
                values.add(this.shape.random(random, 0));
            }
            for (Object value : values) {
                Tag prefix = switch (random.nextInt(10)) {
                    case 0 -> NbtValues.randomCompound(random, 1);
                    case 1 -> (Tag)NbtValues.TAGS.random(random);
                    default -> OPS.empty();
                };
                JsonArray args = new JsonArray();
                args.add(this.shape.encode(value));
                args.add(NbtValues.encodeTag(prefix));
                writer.result(this.name + ".encode", args, DfuValues.result(this.codec.encode(value, OPS, prefix), NbtValues::encodeTag), null);
            }
            List<Tag> inputs = new ArrayList<>();
            for (Object value : values) {
                this.codec.encodeStart(OPS, value).resultOrPartial().ifPresent(tag -> {
                    inputs.add(tag);
                    inputs.add(mutate(tag, random));
                });
            }
            for (int i = 0; i < VALUES / 3; i++) {
                inputs.add((Tag)NbtValues.TAGS.random(random));
            }
            for (Tag input : inputs) {
                JsonArray args = new JsonArray();
                args.add(NbtValues.encodeTag(input));
                writer.result(
                    this.name + ".decode", args,
                    DfuValues.result(this.codec.decode(OPS, input), pair -> DfuValues.pair(pair, this.shape::encode, NbtValues::encodeTag)), null
                );
            }
        }
    }

    // The tag with one tag inside it, or itself, replaced by any tag, or a compound's entry removed or another added.
    private static Tag mutate(final Tag tag, final Random random) {
        if (random.nextInt(4) == 0) {
            return (Tag)NbtValues.TAGS.random(random);
        }
        Tag copy = tag.copy();
        if (copy instanceof CompoundTag compound && !compound.isEmpty()) {
            List<String> keys = new ArrayList<>(compound.keySet());
            String key = keys.get(random.nextInt(keys.size()));
            switch (random.nextInt(3)) {
                case 0 -> compound.remove(key);
                case 1 -> compound.put(key, mutate(compound.get(key), random));
                default -> compound.put(STRINGS.get(random.nextInt(STRINGS.size())), (Tag)NbtValues.TAGS.random(random));
            }
            return compound;
        }
        if (copy instanceof ListTag list && !list.isEmpty()) {
            int index = random.nextInt(list.size());
            list.set(index, mutate(list.get(index), random));
            return list;
        }
        return (Tag)NbtValues.TAGS.random(random);
    }

    /** How a codec's values are made and encoded. */
    private interface Shape {
        Object random(Random random, int depth);

        JsonElement encode(Object value);
    }

    private static Shape shape(final Function<Random, Object> random, final Function<Object, JsonElement> encode) {
        return new Shape() {
            @Override
            public Object random(final Random r, final int depth) {
                return random.apply(r);
            }

            @Override
            public JsonElement encode(final Object value) {
                return encode.apply(value);
            }
        };
    }

    private static final Shape BOOL = shape(Random::nextBoolean, value -> new JsonPrimitive((Boolean)value));
    private static final Shape BYTE = shape(r -> Values.generator(byte.class).random(r), value -> Values.encode(value, byte.class));
    private static final Shape SHORT = shape(r -> (short)r.nextInt(), value -> Values.encode(value, short.class));
    private static final Shape INT = shape(DfuCodecSuite::randomInt, value -> Values.encode(value, int.class));
    private static final Shape LONG = shape(r -> Values.generator(long.class).random(r), value -> Values.encode(value, long.class));
    private static final Shape FLOAT = shape(r -> Values.generator(float.class).random(r), value -> Values.encode(value, float.class));
    private static final Shape DOUBLE = shape(r -> Values.generator(double.class).random(r), value -> Values.encode(value, double.class));
    private static final Shape STRING = shape(r -> STRINGS.get(r.nextInt(STRINGS.size())), value -> new JsonPrimitive((String)value));

    // Small ints mostly, with the edges now and then.
    private static int randomInt(final Random random) {
        return switch (random.nextInt(5)) {
            case 0 -> (Integer)Values.generator(int.class).random(random);
            case 1 -> -random.nextInt(10);
            default -> random.nextInt(10);
        };
    }

    private static Shape list(final Shape element, final int maxSize) {
        return new Shape() {
            @Override
            public Object random(final Random random, final int depth) {
                List<Object> list = new ArrayList<>();
                for (int i = random.nextInt(maxSize + 1); i > 0; i--) {
                    list.add(element.random(random, depth + 1));
                }
                return list;
            }

            @Override
            public JsonElement encode(final Object value) {
                JsonArray array = new JsonArray();
                ((List<?>)value).forEach(element1 -> array.add(element.encode(element1)));
                return array;
            }
        };
    }

    private static Shape optional(final Shape element) {
        return new Shape() {
            @Override
            public Object random(final Random random, final int depth) {
                return random.nextInt(3) == 0 ? Optional.empty() : Optional.of(element.random(random, depth));
            }

            @Override
            public JsonElement encode(final Object value) {
                return ((Optional<?>)value).map(element::encode).orElse(JsonNull.INSTANCE);
            }
        };
    }

    private static Shape either(final Shape left, final Shape right) {
        return new Shape() {
            @Override
            public Object random(final Random random, final int depth) {
                return random.nextBoolean() ? Either.left(left.random(random, depth)) : Either.right(right.random(random, depth));
            }

            @Override
            public JsonElement encode(final Object value) {
                JsonArray array = new JsonArray();
                Either<?, ?> either = (Either<?, ?>)value;
                either.ifLeft(l -> {
                    array.add("left");
                    array.add(left.encode(l));
                }).ifRight(r -> {
                    array.add("right");
                    array.add(right.encode(r));
                });
                return array;
            }
        };
    }

    private static Shape record(final Shape... fields) {
        return new Shape() {
            @Override
            public Object random(final Random random, final int depth) {
                List<Object> record = new ArrayList<>();
                for (Shape field : fields) {
                    record.add(field.random(random, depth));
                }
                return record;
            }

            @Override
            public JsonElement encode(final Object value) {
                JsonArray array = new JsonArray();
                List<?> record = (List<?>)value;
                for (int i = 0; i < fields.length; i++) {
                    array.add(fields[i].encode(record.get(i)));
                }
                return array;
            }
        };
    }

    // The recursive codec's trees: an int, or a list of trees, shallower the deeper it is.
    private static final Shape TREE = new Shape() {
        @Override
        public Object random(final Random random, final int depth) {
            if (depth >= 3 || random.nextInt(3) == 0) {
                return Either.left(randomInt(random));
            }
            List<Object> children = new ArrayList<>();
            for (int i = random.nextInt(4); i > 0; i--) {
                children.add(this.random(random, depth + 1));
            }
            return Either.right(children);
        }

        @Override
        public JsonElement encode(final Object value) {
            JsonArray array = new JsonArray();
            Either<?, ?> either = (Either<?, ?>)value;
            either.ifLeft(l -> {
                array.add("left");
                array.add((Integer)l);
            }).ifRight(r -> {
                array.add("right");
                JsonArray children = new JsonArray();
                ((List<?>)r).forEach(child -> children.add(this.encode(child)));
                array.add(children);
            });
            return array;
        }
    };

    @SuppressWarnings("unchecked")
    private static <A> Codec<Object> erase(final Codec<A> codec) {
        return (Codec<Object>)(Codec<?>)codec;
    }

    @SuppressWarnings("unchecked")
    private static <A> A field(final Object record, final int index) {
        return (A)((List<?>)record).get(index);
    }

    private static List<Named> codecs() {
        List<Named> codecs = new ArrayList<>();
        codecs.add(new Named("BOOL", erase(Codec.BOOL), BOOL));
        codecs.add(new Named("BYTE", erase(Codec.BYTE), BYTE));
        codecs.add(new Named("SHORT", erase(Codec.SHORT), SHORT));
        codecs.add(new Named("INT", erase(Codec.INT), INT));
        codecs.add(new Named("LONG", erase(Codec.LONG), LONG));
        codecs.add(new Named("FLOAT", erase(Codec.FLOAT), FLOAT));
        codecs.add(new Named("DOUBLE", erase(Codec.DOUBLE), DOUBLE));
        codecs.add(new Named("STRING", erase(Codec.STRING), STRING));
        codecs.add(new Named("INT.listOf()", erase(Codec.INT.listOf()), list(INT, 6)));
        codecs.add(new Named("STRING.listOf(1, 3)", erase(Codec.STRING.listOf(1, 3)), list(STRING, 5)));
        codecs.add(new Named("INT.listOf().listOf()", erase(Codec.INT.listOf().listOf()), list(list(INT, 3), 3)));
        codecs.add(new Named("either(INT, STRING)", erase(Codec.either(Codec.INT, Codec.STRING)), either(INT, STRING)));
        codecs.add(new Named("either(INT.listOf(), BOOL)", erase(Codec.either(Codec.INT.listOf(), Codec.BOOL)), either(list(INT, 4), BOOL)));
        codecs.add(new Named("INT.xmap(x * 3, x / 3)", erase(Codec.INT.xmap(x -> x * 3, x -> x / 3)), INT));
        codecs.add(new Named(
            "STRING.comapFlatMap(parseInt, toString)",
            erase(Codec.STRING.comapFlatMap(s -> {
                try {
                    return DataResult.success(Integer.parseInt(s));
                } catch (NumberFormatException e) {
                    return DataResult.error(() -> "Not an int: " + s);
                }
            }, Object::toString)),
            INT
        ));
        codecs.add(new Named(
            "INT.flatXmap(half of even, double of non-negative)",
            erase(Codec.INT.flatXmap(
                x -> x % 2 == 0 ? DataResult.success(x / 2) : DataResult.error(() -> "Odd: " + x, x / 2),
                x -> x >= 0 ? DataResult.success(x * 2) : DataResult.error(() -> "Negative: " + x)
            )),
            INT
        ));
        codecs.add(new Named(
            "INT.validate(non-negative)", erase(Codec.INT.validate(x -> x >= 0 ? DataResult.success(x) : DataResult.error(() -> "Negative: " + x))), INT
        ));
        codecs.add(new Named(
            "STRING.flatComapMap(s + !, without !)",
            erase(Codec.STRING.flatComapMap(s -> s + "!", s -> s.endsWith("!") ? DataResult.success(s.substring(0, s.length() - 1)) : DataResult.error(() -> "No !: " + s))),
            shape(r -> STRINGS.get(r.nextInt(STRINGS.size())) + (r.nextInt(4) == 0 ? "" : "!"), value -> new JsonPrimitive((String)value))
        ));
        codecs.add(new Named("withAlternative(INT, STRING, length)", erase(Codec.withAlternative(Codec.INT, Codec.STRING, String::length)), INT));
        codecs.add(new Named(
            "withAlternative(INT, FLOAT.xmap(toInt, toFloat))", erase(Codec.withAlternative(Codec.INT, Codec.FLOAT.xmap(f -> (int)(float)f, i -> (float)(int)i))), INT
        ));
        codecs.add(new Named("INT.fieldOf(a).codec()", erase(Codec.INT.fieldOf("a").codec()), INT));
        codecs.add(new Named("STRING.optionalFieldOf(b).codec()", erase(Codec.STRING.optionalFieldOf("b").codec()), optional(STRING)));
        codecs.add(new Named("INT.lenientOptionalFieldOf(c).codec()", erase(Codec.INT.lenientOptionalFieldOf("c").codec()), optional(INT)));
        codecs.add(new Named("INT.optionalFieldOf(d, 7).codec()", erase(Codec.INT.optionalFieldOf("d", 7).codec()), INT));
        codecs.add(new Named("BOOL.lenientOptionalFieldOf(e, true).codec()", erase(Codec.BOOL.lenientOptionalFieldOf("e", true).codec()), BOOL));
        codecs.add(new Named("INT.listOf().optionalFieldOf(f, []).codec()", erase(Codec.INT.listOf().optionalFieldOf("f", List.of()).codec()), list(INT, 3)));
        codecs.add(new Named(
            "record1",
            erase(RecordCodecBuilder.<List<Object>>create(i -> i.group(Codec.INT.fieldOf("a").forGetter(r -> field(r, 0))).apply(i, a -> List.of(a)))),
            record(INT)
        ));
        codecs.add(new Named(
            "record1 optional",
            erase(RecordCodecBuilder.<List<Object>>create(
                i -> i.group(Codec.STRING.optionalFieldOf("a").forGetter(r -> field(r, 0))).apply(i, a -> List.of(a))
            )),
            record(optional(STRING))
        ));
        codecs.add(new Named(
            "record2",
            erase(RecordCodecBuilder.<List<Object>>create(i -> i.group(
                Codec.INT.fieldOf("a").forGetter(r -> field(r, 0)), Codec.STRING.optionalFieldOf("b").forGetter(r -> field(r, 1))
            ).apply(i, (a, b) -> List.of(a, b)))),
            record(INT, optional(STRING))
        ));
        codecs.add(new Named(
            "record3",
            erase(RecordCodecBuilder.<List<Object>>create(i -> i.group(
                Codec.STRING.fieldOf("a").forGetter(r -> field(r, 0)),
                Codec.INT.optionalFieldOf("b", 0).forGetter(r -> field(r, 1)),
                Codec.BOOL.lenientOptionalFieldOf("c").forGetter(r -> field(r, 2))
            ).apply(i, (a, b, c) -> List.of(a, b, c)))),
            record(STRING, INT, optional(BOOL))
        ));
        codecs.add(new Named(
            "record4",
            erase(RecordCodecBuilder.<List<Object>>create(i -> i.group(
                Codec.INT.fieldOf("a").forGetter(r -> field(r, 0)),
                Codec.INT.fieldOf("b").forGetter(r -> field(r, 1)),
                Codec.STRING.listOf().fieldOf("c").forGetter(r -> field(r, 2)),
                Codec.either(Codec.INT, Codec.STRING).fieldOf("d").forGetter(r -> field(r, 3))
            ).apply(i, (a, b, c, d) -> List.of(a, b, c, d)))),
            record(INT, INT, list(STRING, 3), either(INT, STRING))
        ));
        codecs.add(new Named(
            "record5",
            erase(RecordCodecBuilder.<List<Object>>create(i -> i.group(
                Codec.INT.fieldOf("a").forGetter(r -> field(r, 0)),
                Codec.STRING.fieldOf("b").forGetter(r -> field(r, 1)),
                Codec.BOOL.optionalFieldOf("c").forGetter(r -> field(r, 2)),
                Codec.INT.listOf().fieldOf("d").forGetter(r -> field(r, 3)),
                Codec.STRING.optionalFieldOf("e", "x").forGetter(r -> field(r, 4))
            ).apply(i, (a, b, c, d, e) -> List.of(a, b, c, d, e)))),
            record(INT, STRING, optional(BOOL), list(INT, 3), STRING)
        ));
        codecs.add(new Named(
            "record11",
            erase(RecordCodecBuilder.<List<Object>>create(i -> i.group(
                Codec.INT.optionalFieldOf("color").forGetter(r -> field(r, 0)),
                Codec.INT.optionalFieldOf("shadow_color").forGetter(r -> field(r, 1)),
                Codec.BOOL.optionalFieldOf("bold").forGetter(r -> field(r, 2)),
                Codec.BOOL.optionalFieldOf("italic").forGetter(r -> field(r, 3)),
                Codec.BOOL.optionalFieldOf("underlined").forGetter(r -> field(r, 4)),
                Codec.BOOL.optionalFieldOf("strikethrough").forGetter(r -> field(r, 5)),
                Codec.BOOL.optionalFieldOf("obfuscated").forGetter(r -> field(r, 6)),
                Codec.STRING.fieldOf("text").forGetter(r -> field(r, 7)),
                Codec.INT.listOf().optionalFieldOf("extra").forGetter(r -> field(r, 8)),
                Codec.STRING.optionalFieldOf("insertion").forGetter(r -> field(r, 9)),
                Codec.STRING.fieldOf("font").forGetter(r -> field(r, 10))
            ).apply(i, (a, b, c, d, e, f, g, h, j, k, l) -> List.of(a, b, c, d, e, f, g, h, j, k, l)))),
            record(optional(INT), optional(INT), optional(BOOL), optional(BOOL), optional(BOOL), optional(BOOL), optional(BOOL), STRING, optional(list(INT, 3)), optional(STRING), STRING)
        ));
        codecs.add(new Named(
            "record16",
            erase(RecordCodecBuilder.<List<Object>>create(i -> i.group(
                Codec.INT.fieldOf("a").forGetter(r -> field(r, 0)),
                Codec.INT.fieldOf("b").forGetter(r -> field(r, 1)),
                Codec.INT.fieldOf("c").forGetter(r -> field(r, 2)),
                Codec.INT.fieldOf("d").forGetter(r -> field(r, 3)),
                Codec.INT.fieldOf("e").forGetter(r -> field(r, 4)),
                Codec.INT.fieldOf("f").forGetter(r -> field(r, 5)),
                Codec.INT.fieldOf("g").forGetter(r -> field(r, 6)),
                Codec.INT.fieldOf("h").forGetter(r -> field(r, 7)),
                Codec.STRING.fieldOf("i").forGetter(r -> field(r, 8)),
                Codec.STRING.fieldOf("j").forGetter(r -> field(r, 9)),
                Codec.STRING.fieldOf("k").forGetter(r -> field(r, 10)),
                Codec.STRING.fieldOf("l").forGetter(r -> field(r, 11)),
                Codec.BOOL.fieldOf("m").forGetter(r -> field(r, 12)),
                Codec.BOOL.fieldOf("n").forGetter(r -> field(r, 13)),
                Codec.BOOL.fieldOf("o").forGetter(r -> field(r, 14)),
                Codec.BOOL.fieldOf("p").forGetter(r -> field(r, 15))
            ).apply(i, (a, b, c, d, e, f, g, h, i1, j, k, l, m, n, o, p) -> List.of(a, b, c, d, e, f, g, h, i1, j, k, l, m, n, o, p)))),
            record(INT, INT, INT, INT, INT, INT, INT, INT, STRING, STRING, STRING, STRING, BOOL, BOOL, BOOL, BOOL)
        ));
        MapCodec<List<Object>> record2 = RecordCodecBuilder.mapCodec(i -> i.group(
            Codec.INT.fieldOf("a").forGetter(r -> field(r, 0)), Codec.STRING.optionalFieldOf("b").forGetter(r -> field(r, 1))
        ).apply(i, (a, b) -> List.of(a, b)));
        codecs.add(new Named("record2.stable().codec()", erase(record2.stable().codec()), record(INT, optional(STRING))));
        codecs.add(new Named(
            "record2.xmap(swap).codec()",
            erase(record2.<List<Object>>xmap(r -> List.of(field(r, 1), field(r, 0)), r -> List.of(field(r, 1), field(r, 0))).codec()),
            record(optional(STRING), INT)
        ));
        codecs.add(new Named(
            "record2.flatXmap(a non-negative).codec()",
            erase(record2.flatXmap(
                r -> (Integer)field(r, 0) >= 0 ? DataResult.success(r) : DataResult.error(() -> "Negative a", r),
                r -> (Integer)field(r, 0) >= 0 ? DataResult.success(r) : DataResult.error(() -> "Negative a: " + field(r, 0), List.of(0, field(r, 1)))
            ).codec()),
            record(INT, optional(STRING))
        ));
        MapCodec<Object> intValue = Codec.INT.fieldOf("value").xmap(i -> (Object)Either.left(i), e -> (Integer)((Either<?, ?>)e).left().orElseThrow());
        MapCodec<Object> pair = RecordCodecBuilder.<List<Object>>mapCodec(i -> i.group(
            Codec.STRING.fieldOf("name").forGetter(r -> field(r, 0)), Codec.INT.optionalFieldOf("n", 0).forGetter(r -> field(r, 1))
        ).apply(i, (name, n) -> List.of(name, n))).xmap(r -> (Object)Either.right(r), e -> (List<Object>)((Either<?, ?>)e).right().orElseThrow());
        codecs.add(new Named(
            "STRING.validate(known).dispatch(type)",
            erase(Codec.STRING.validate(s -> s.equals("int") || s.equals("pair") ? DataResult.success(s) : DataResult.error(() -> "Unknown type: " + s))
                .dispatch("type", e -> ((Either<?, ?>)e).map(l -> "int", r -> "pair"), t -> t.equals("int") ? intValue : pair)),
            either(INT, record(STRING, INT))
        ));
        codecs.add(new Named(
            "STRING.partialDispatch(kind)",
            erase(Codec.STRING.partialDispatch(
                "kind",
                e -> ((Either<?, ?>)e).map(l -> (Integer)l < 0 ? DataResult.error(() -> "No kind for " + l) : DataResult.success("int"), r -> DataResult.success("pair")),
                t -> t.equals("int") ? DataResult.success(intValue) : t.equals("pair") ? DataResult.success(pair) : DataResult.error(() -> "Unknown kind: " + t)
            )),
            either(INT, record(STRING, INT))
        ));
        codecs.add(new Named("recursive(tree)", erase(Codec.<Object>recursive("tree", self -> erase(Codec.either(Codec.INT, self.listOf())))), TREE));
        return codecs;
    }
}
