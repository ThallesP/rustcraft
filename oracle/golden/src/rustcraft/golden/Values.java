package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import com.mojang.math.Axis;
import java.lang.reflect.Array;
import java.lang.reflect.RecordComponent;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.UUID;
import java.util.function.Function;
import java.util.stream.IntStream;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.levelgen.LegacyRandomSource;

/** Encodes values the way TESTING.md describes, and generates arguments for reflective suites. */
final class Values {
    /** IntStreams can be infinite (or close enough), so only their first elements are compared. */
    static final int STREAM_LIMIT = 512;

    /**
     * Produces arguments of one type. Edges and random draws are immutable specs; mutable arguments such as a
     * RandomSource get a fresh instance per call from their spec.
     */
    interface Generator {
        List<Object> edges();

        Object random(Random random);

        JsonElement encode(Object spec);

        default Object instantiate(final Object spec) {
            return spec;
        }

        /**
         * State of the argument after the call that must also match, e.g. how far a RandomSource advanced, or null
         * when there's nothing to match.
         */
        default JsonElement after(final Object spec, final Object instance) {
            return null;
        }
    }

    // Random draws pick an edge one time in eight, so edges also meet random partners.
    private record Simple(List<Object> edges, Function<Random, Object> fresh, Function<Object, JsonElement> encoder) implements Generator {
        @Override
        public Object random(final Random random) {
            return random.nextInt(8) == 0 ? this.edges.get(random.nextInt(this.edges.size())) : this.fresh.apply(random);
        }

        @Override
        public JsonElement encode(final Object spec) {
            return this.encoder.apply(spec);
        }
    }

    private static final Generator INTS = new Simple(
        List.<Object>of(
            0, 1, -1, 2, -2, 3, 7, 8, 9, 15, 16, 17, 31, 32, 33, 45, 90, -90, 180, -180, 255, 256, 359, 360, 361, -360,
            65535, 65536, 1 << 30, Integer.MIN_VALUE, Integer.MIN_VALUE + 1, Integer.MAX_VALUE, Integer.MAX_VALUE - 1
        ),
        random -> switch (random.nextInt(3)) {
            case 0 -> random.nextInt();
            case 1 -> random.nextInt(2001) - 1000;
            default -> random.nextInt(65) - 32;
        },
        value -> new JsonPrimitive((Integer)value)
    );

    private static final Generator LONGS = new Simple(
        List.<Object>of(
            0L, 1L, -1L, 2L, -2L, 360L, -360L, (long)Integer.MIN_VALUE, (long)Integer.MAX_VALUE, 1L << 32, (1L << 48) - 1,
            Long.MIN_VALUE, Long.MIN_VALUE + 1, Long.MAX_VALUE, Long.MAX_VALUE - 1
        ),
        random -> switch (random.nextInt(3)) {
            case 0 -> random.nextLong();
            case 1 -> (long)random.nextInt(2001) - 1000;
            default -> (long)random.nextInt();
        },
        value -> new JsonPrimitive((Long)value)
    );

    static final Generator FLOATS = new Simple(
        List.<Object>of(
            0.0F, -0.0F, 1.0F, -1.0F, 0.5F, -0.5F, 0.25F, 2.0F, -2.0F, 0.1F, 1.0E-5F, -1.0E-5F, 1.0E-7F, 0.99999994F, 1.0000001F,
            6.0F, 45.0F, 90.0F, -90.0F, 180.0F, -180.0F, 179.99998F, 270.0F, 360.0F, -360.0F, 720.0F, 1.0E6F, -1.0E6F,
            (float)Math.PI, (float)-Math.PI, (float)(Math.PI * 2), Float.MIN_VALUE, -Float.MIN_VALUE, Float.MIN_NORMAL,
            Float.MAX_VALUE, -Float.MAX_VALUE, Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY, Float.NaN
        ),
        random -> switch (random.nextInt(5)) {
            case 0 -> random.nextFloat();
            case 1 -> random.nextFloat() * 2.0F - 1.0F;
            case 2 -> (random.nextFloat() * 2.0F - 1.0F) * 400.0F;
            case 3 -> (random.nextFloat() * 2.0F - 1.0F) * 1.0E6F;
            default -> canonicalNaN(Float.intBitsToFloat(random.nextInt()));
        },
        value -> new JsonPrimitive(floatBits((Float)value))
    );

    private static final Generator DOUBLES = new Simple(
        List.<Object>of(
            0.0, -0.0, 1.0, -1.0, 0.5, -0.5, 0.25, 2.0, -2.0, 0.1, 1.0E-5, -1.0E-5, 1.0E-7, 1.0E-300, 0.9999999999999999,
            1.0000000000000002, 6.0, 45.0, 90.0, -90.0, 180.0, -180.0, 270.0, 360.0, -360.0, 720.0, 3000.0, 1.0E6, -1.0E6, 1.0E15,
            Math.PI, -Math.PI, Math.PI * 2, Double.MIN_VALUE, -Double.MIN_VALUE, Double.MIN_NORMAL, Double.MAX_VALUE,
            -Double.MAX_VALUE, Double.POSITIVE_INFINITY, Double.NEGATIVE_INFINITY, Double.NaN
        ),
        random -> switch (random.nextInt(5)) {
            case 0 -> random.nextDouble();
            case 1 -> random.nextDouble() * 2.0 - 1.0;
            case 2 -> (random.nextDouble() * 2.0 - 1.0) * 400.0;
            case 3 -> (random.nextDouble() * 2.0 - 1.0) * 1.0E6;
            default -> canonicalNaN(Double.longBitsToDouble(random.nextLong()));
        },
        value -> new JsonPrimitive(doubleBits((Double)value))
    );

    private static final Generator BYTES = new Simple(
        IntStream.range(Byte.MIN_VALUE, Byte.MAX_VALUE + 1).mapToObj(value -> (Object)(byte)value).toList(),
        random -> (byte)random.nextInt(),
        value -> new JsonPrimitive((Byte)value)
    );

    private static final Generator SHORTS = new Simple(
        List.<Object>of((short)0, (short)1, (short)-1, Short.MIN_VALUE, Short.MAX_VALUE),
        random -> (short)random.nextInt(),
        value -> new JsonPrimitive((Short)value)
    );

    private static final Generator CHARS = new Simple(
        List.<Object>of('\0', '0', '9', 'a', 'Z', '\u007f', 'ÿ', '￿'),
        random -> (char)random.nextInt(0xd800),
        value -> new JsonPrimitive((int)(Character)value)
    );

    private static final Generator BOOLEANS = new Simple(
        List.<Object>of(true, false),
        Random::nextBoolean,
        value -> new JsonPrimitive((Boolean)value)
    );

    // Java parses Unicode digits ("٣", "１２") as numbers and counts UTF-16 units, so those edges matter.
    static final Generator STRINGS = new Simple(
        List.<Object>of(
            "", "0", "1", "-1", "+1", "-0", "007", "42", " 42", "42 ", "2147483647", "2147483648", "-2147483648", "-2147483649",
            "1.5", "1e3", "abc", "٣", "１２", "😀", "minecraft:stone", "\u0000", "Aa", "BB"
        ),
        random -> switch (random.nextInt(3)) {
            case 0 -> Integer.toString(random.nextInt());
            case 1 -> randomString(random, 0x20, 0x7f);
            default -> randomString(random, 0x20, 0x30000);
        },
        value -> new JsonPrimitive((String)value)
    );

    private static final Generator RANDOM_SOURCES = new Generator() {
        @Override
        public List<Object> edges() {
            return List.of(0L, 1L, -1L, 42L, Long.MIN_VALUE, Long.MAX_VALUE);
        }

        @Override
        public Object random(final Random random) {
            return random.nextLong();
        }

        @Override
        public JsonElement encode(final Object spec) {
            return new JsonPrimitive((Long)spec);
        }

        @Override
        public Object instantiate(final Object spec) {
            return new LegacyRandomSource((Long)spec);
        }

        @Override
        public JsonElement after(final Object spec, final Object instance) {
            return new JsonPrimitive(((RandomSource)instance).nextLong());
        }
    };

    // Axis's six constants, by name. Axis.of isn't covered: nothing in the game calls it.
    private static final Map<Axis, String> AXIS_NAMES = Map.of(
        Axis.XN, "XN", Axis.XP, "XP", Axis.YN, "YN", Axis.YP, "YP", Axis.ZN, "ZN", Axis.ZP, "ZP"
    );
    private static final List<Object> AXIS_CONSTANTS = List.of(Axis.XN, Axis.XP, Axis.YN, Axis.YP, Axis.ZN, Axis.ZP);

    static final Generator AXES = new Simple(
        AXIS_CONSTANTS,
        random -> AXIS_CONSTANTS.get(random.nextInt(AXIS_CONSTANTS.size())),
        value -> new JsonPrimitive(AXIS_NAMES.get((Axis)value))
    );

    private static final Map<Class<?>, Generator> GENERATORS = Map.ofEntries(
        Map.entry(int.class, INTS),
        Map.entry(Integer.class, INTS),
        Map.entry(long.class, LONGS),
        Map.entry(float.class, FLOATS),
        Map.entry(double.class, DOUBLES),
        Map.entry(byte.class, BYTES),
        Map.entry(short.class, SHORTS),
        Map.entry(char.class, CHARS),
        Map.entry(boolean.class, BOOLEANS),
        Map.entry(String.class, STRINGS),
        Map.entry(RandomSource.class, RANDOM_SOURCES),
        Map.entry(Axis.class, AXES)
    );

    private static final List<Class<?>> BOXES = List.of(
        Integer.class, Long.class, Float.class, Double.class, Byte.class, Short.class, Character.class, Boolean.class
    );

    private Values() {
    }

    static Generator generator(final Class<?> type) {
        Generator generator = GENERATORS.get(type);
        return generator != null ? generator : JomlValues.generator(type);
    }

    static boolean canEncode(final Class<?> type) {
        if (type == void.class) {
            return false;
        }
        if (type.isPrimitive() || BOXES.contains(type) || type == String.class || type == UUID.class || type == IntStream.class) {
            return true;
        }
        if (JomlValues.canEncode(type)) {
            return true;
        }
        if (type.isArray()) {
            return canEncode(type.getComponentType());
        }
        if (type.isRecord()) {
            for (RecordComponent component : type.getRecordComponents()) {
                if (!canEncode(component.getType())) {
                    return false;
                }
            }
            return true;
        }
        return false;
    }

    static JsonElement encode(final Object value, final Class<?> type) {
        if (value == null) {
            return JsonNull.INSTANCE;
        }
        if (type == float.class || type == Float.class) {
            return new JsonPrimitive(floatBits((Float)value));
        }
        if (type == double.class || type == Double.class) {
            return new JsonPrimitive(doubleBits((Double)value));
        }
        if (type == char.class || type == Character.class) {
            return new JsonPrimitive((int)(Character)value);
        }
        if (type == boolean.class || type == Boolean.class) {
            return new JsonPrimitive((Boolean)value);
        }
        if (type.isPrimitive() || BOXES.contains(type)) {
            return new JsonPrimitive((Number)value);
        }
        if (type == String.class) {
            return new JsonPrimitive((String)value);
        }
        if (type == UUID.class) {
            UUID uuid = (UUID)value;
            JsonArray array = new JsonArray();
            array.add(uuid.getMostSignificantBits());
            array.add(uuid.getLeastSignificantBits());
            return array;
        }
        if (type == IntStream.class) {
            JsonArray array = new JsonArray();
            ((IntStream)value).limit(STREAM_LIMIT).forEach(array::add);
            return array;
        }
        if (type.isArray()) {
            JsonArray array = new JsonArray();
            for (int i = 0; i < Array.getLength(value); i++) {
                array.add(encode(Array.get(value, i), type.getComponentType()));
            }
            return array;
        }
        if (type.isRecord()) {
            JsonArray array = new JsonArray();
            for (RecordComponent component : type.getRecordComponents()) {
                try {
                    array.add(encode(component.getAccessor().invoke(value), component.getType()));
                } catch (ReflectiveOperationException e) {
                    throw new IllegalStateException(e);
                }
            }
            return array;
        }
        if (JomlValues.canEncode(type)) {
            return JomlValues.encode(value);
        }
        throw new IllegalArgumentException("no encoder for " + type.getName());
    }

    static String floatBits(final float value) {
        return String.format("%08x", Float.floatToRawIntBits(value));
    }

    static String doubleBits(final double value) {
        return String.format("%016x", Double.doubleToRawLongBits(value));
    }

    static double parseDouble(final JsonElement bits) {
        return Double.longBitsToDouble(Long.parseUnsignedLong(bits.getAsString(), 16));
    }

    static float parseFloat(final JsonElement bits) {
        return Float.intBitsToFloat(Integer.parseUnsignedInt(bits.getAsString(), 16));
    }

    // NaN payloads never change game behaviour, so random bit patterns use the canonical NaN.
    private static float canonicalNaN(final float value) {
        return Float.isNaN(value) ? Float.NaN : value;
    }

    private static double canonicalNaN(final double value) {
        return Double.isNaN(value) ? Double.NaN : value;
    }

    // Skips surrogate code points: a lone surrogate can't be written as UTF-8.
    private static String randomString(final Random random, final int fromCodePoint, final int toCodePoint) {
        StringBuilder builder = new StringBuilder();
        int length = random.nextInt(13);
        while (builder.codePointCount(0, builder.length()) < length) {
            int codePoint = fromCodePoint + random.nextInt(toCodePoint - fromCodePoint);
            if (codePoint < 0xd800 || codePoint > 0xdfff) {
                builder.appendCodePoint(codePoint);
            }
        }
        return builder.toString();
    }
}
