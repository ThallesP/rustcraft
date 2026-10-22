package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import java.io.IOException;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import java.util.Random;
import java.util.function.IntFunction;
import java.util.function.Predicate;
import net.minecraft.network.codec.StreamCodec;

/**
 * Cases for the game's stream codecs and IntFunctions, which are objects rather than methods. A codec named N gets
 * {@code N.decode} cases, which read a buffer ({@code [buffer]} → the value, and the buffer afterwards in {@code after}),
 * and {@code N.encode} cases, which write a value into one ({@code [buffer, value]} → null, and the buffer afterwards).
 * An IntFunction named N gets {@code N.apply} cases ({@code [id]} → the value). A codec or function that a factory makes
 * from arguments that can be generated (ByteBufCodecs.stringUtf8's maximum) takes them first, and a case throws when the
 * factory does.
 *
 * <p>Decoding reads the buffer edges and random buffers, and the codec's own encodings of the value edges and random
 * values: whole, cut short by a byte or anywhere, and with another field after them. Encoding writes each value edge
 * into a random buffer, a random value into each buffer edge, and random values into random buffers.
 */
final class Codecs {
    private static final int RANDOM_CASES = 300;
    private static final int ENCODED_VALUES = 200;

    /** What a codec reads from and writes to, and its values. */
    record Codec(Values.Generator buffer, Values.Generator value, Class<?> valueType) {
    }

    /** Makes the codec or function from the leading arguments. */
    @FunctionalInterface
    interface Factory {
        Object make(Object[] leading) throws Exception;
    }

    /** What a case does with the factory's codec or function and the arguments after the leading ones. */
    @FunctionalInterface
    interface Use {
        Object apply(Object made, Object[] rest);
    }

    private Codecs() {
    }

    /**
     * The codec a field or a method's result is, when its buffer and values have generators, or null. Its values are a
     * class's, since a generic one's generator wouldn't know the type arguments (an Optional of what).
     */
    static Codec codecOf(final Type type) {
        if (!(type instanceof ParameterizedType parameterized) || parameterized.getRawType() != StreamCodec.class) {
            return null;
        }
        Type[] arguments = parameterized.getActualTypeArguments();
        if (!(arguments[0] instanceof Class<?> buffer) || NetworkValues.generator(buffer) == null) {
            return null;
        }
        if (!(arguments[1] instanceof Class<?> valueType) || Values.generator(valueType) == null || !Values.canEncode(valueType)) {
            return null;
        }
        return new Codec(NetworkValues.generator(buffer), Values.generator(valueType), valueType);
    }

    /** The values an IntFunction field or result hands out, when they're a class's and can be encoded, or null. */
    static Class<?> functionOf(final Type type) {
        if (!(type instanceof ParameterizedType parameterized) || parameterized.getRawType() != IntFunction.class) {
            return null;
        }
        return parameterized.getActualTypeArguments()[0] instanceof Class<?> valueType && Values.canEncode(valueType) ? valueType : null;
    }

    /** Why a codec or IntFunction type gets no cases, naming the type argument without a generator, or null for other types. */
    static String unsupportedReason(final Type type) {
        if (!(type instanceof ParameterizedType parameterized)) {
            return null;
        }
        Type[] arguments = parameterized.getActualTypeArguments();
        if (parameterized.getRawType() == StreamCodec.class) {
            if (!(arguments[0] instanceof Class<?> buffer) || NetworkValues.generator(buffer) == null) {
                return "no generator for " + arguments[0].getTypeName();
            }
            return "no generator for " + arguments[1].getTypeName();
        }
        if (parameterized.getRawType() == IntFunction.class) {
            return "no encoder for " + arguments[0].getTypeName();
        }
        return null;
    }

    static void writeCodec(
        final GoldenWriter writer, final String name, final Values.Generator[] leading, final Factory factory, final Codec codec,
        final Predicate<Object[]> decodeDomain
    ) throws IOException {
        writeCodec(writer, name, leading, factory, codec, decodeDomain, List.of());
    }

    /**
     * Like writeCodec, with more buffers to decode (each the leading arguments' specs and the buffer), for what the
     * codec's own encodings never hold: a field longer than its writer allows, but not its reader.
     */
    @SuppressWarnings("unchecked")
    static void writeCodec(
        final GoldenWriter writer, final String name, final Values.Generator[] leading, final Factory factory, final Codec codec,
        final Predicate<Object[]> decodeDomain, final List<Object[]> extraDecodes
    ) throws IOException {
        Random random = new Random(name.hashCode());

        Values.Generator[] decoding = append(leading, codec.buffer());
        List<Object[]> decodes = new ArrayList<>(eachEdge(decoding, random));
        decodes.addAll(encodings(leading, factory, codec, random));
        decodes.addAll(extraDecodes);
        for (int i = 0; i < RANDOM_CASES; i++) {
            decodes.add(StaticSuite.randomTuple(decoding, random));
        }
        writeCases(
            writer, name + ".decode", leading.length, factory, decoding, decodes.stream().filter(decodeDomain).toList(), codec.valueType(),
            (made, rest) -> ((StreamCodec<Object, Object>)made).decode(rest[0])
        );

        Values.Generator[] encoding = append(append(leading, codec.buffer()), codec.value());
        List<Object[]> encodes = new ArrayList<>(eachEdge(encoding, random));
        for (int i = 0; i < RANDOM_CASES; i++) {
            encodes.add(StaticSuite.randomTuple(encoding, random));
        }
        writeCases(writer, name + ".encode", leading.length, factory, encoding, encodes, void.class, (made, rest) -> {
            ((StreamCodec<Object, Object>)made).encode(rest[0], rest[1]);
            return null;
        });
    }

    // Ids around a continuous map's (Direction's 0 to 5), and any ints.
    static void writeFunction(
        final GoldenWriter writer, final String name, final Values.Generator[] leading, final Factory factory, final Class<?> valueType
    ) throws IOException {
        Random random = new Random(name.hashCode());
        Values.Generator ids = Values.generator(int.class);
        Values.Generator[] generators = append(leading, ids);
        List<Object[]> specs = new ArrayList<>(eachEdge(generators, random));
        for (int id = -16; id <= 16; id++) {
            Object[] spec = StaticSuite.randomTuple(generators, random);
            spec[leading.length] = id;
            specs.add(spec);
        }
        for (int i = 0; i < RANDOM_CASES; i++) {
            specs.add(StaticSuite.randomTuple(generators, random));
        }
        writeCases(
            writer, name + ".apply", leading.length, factory, generators, specs, valueType,
            (made, rest) -> ((IntFunction<?>)made).apply((Integer)rest[0])
        );
    }

    /** Values that may be empty, encoded as the value or null. */
    static Values.Generator optionalOf(final Values.Generator element) {
        return new Values.Generator() {
            @Override
            public List<Object> edges() {
                List<Object> edges = new ArrayList<>();
                edges.add(Optional.empty());
                element.edges().forEach(edge -> edges.add(Optional.of(edge)));
                return edges;
            }

            @Override
            public Object random(final Random random) {
                return random.nextInt(4) == 0 ? Optional.empty() : Optional.of(element.random(random));
            }

            @Override
            public JsonElement encode(final Object spec) {
                return ((Optional<?>)spec).map(element::encode).orElse(JsonNull.INSTANCE);
            }

            @Override
            public Object instantiate(final Object spec) {
                return ((Optional<?>)spec).map(element::instantiate);
            }
        };
    }

    /** Values that may be null, like an optional's (null a quarter of the time), for arguments a method lets be null. */
    static Values.Generator nullableOf(final Values.Generator element) {
        return new Values.Generator() {
            @Override
            public List<Object> edges() {
                List<Object> edges = new ArrayList<>();
                edges.add(null);
                edges.addAll(element.edges());
                return edges;
            }

            @Override
            public Object random(final Random random) {
                return random.nextInt(4) == 0 ? null : element.random(random);
            }

            @Override
            public JsonElement encode(final Object spec) {
                return spec == null ? JsonNull.INSTANCE : element.encode(spec);
            }

            @Override
            public Object instantiate(final Object spec) {
                return spec == null ? null : element.instantiate(spec);
            }
        };
    }

    /**
     * Lists of up to 6 elements mostly, and now and then up to 140 (whose count takes a second byte), encoded as arrays.
     * Edges add the empty list, each element edge alone, and all of them.
     */
    static Values.Generator listOf(final Values.Generator element) {
        return new Values.Generator() {
            @Override
            public List<Object> edges() {
                List<Object> edges = new ArrayList<>();
                edges.add(List.of());
                element.edges().forEach(edge -> edges.add(List.of(edge)));
                edges.add(element.edges());
                return edges;
            }

            @Override
            public Object random(final Random random) {
                int size = random.nextInt(4) == 0 ? random.nextInt(141) : random.nextInt(7);
                List<Object> list = new ArrayList<>();
                for (int i = 0; i < size; i++) {
                    list.add(element.random(random));
                }
                return list;
            }

            @Override
            public JsonElement encode(final Object spec) {
                JsonArray array = new JsonArray();
                ((List<?>)spec).forEach(value -> array.add(element.encode(value)));
                return array;
            }

            @Override
            public Object instantiate(final Object spec) {
                return new ArrayList<>(((List<?>)spec).stream().map(element::instantiate).toList());
            }
        };
    }

    /** Lists with one element from each generator, encoded as arrays. Edges put each edge in its place among first edges. */
    static Values.Generator tupleOf(final Values.Generator... elements) {
        return new Values.Generator() {
            @Override
            public List<Object> edges() {
                List<Object> edges = new ArrayList<>();
                for (int i = 0; i < elements.length; i++) {
                    for (Object edge : elements[i].edges()) {
                        Object[] tuple = new Object[elements.length];
                        for (int j = 0; j < elements.length; j++) {
                            tuple[j] = elements[j].edges().get(0);
                        }
                        tuple[i] = edge;
                        edges.add(tuple);
                    }
                }
                return edges;
            }

            @Override
            public Object random(final Random random) {
                Object[] tuple = new Object[elements.length];
                for (int i = 0; i < elements.length; i++) {
                    tuple[i] = elements[i].random(random);
                }
                return tuple;
            }

            @Override
            public JsonElement encode(final Object spec) {
                JsonArray array = new JsonArray();
                Object[] tuple = (Object[])spec;
                for (int i = 0; i < elements.length; i++) {
                    array.add(elements[i].encode(tuple[i]));
                }
                return array;
            }

            @Override
            public Object instantiate(final Object spec) {
                Object[] tuple = (Object[])spec;
                Object[] instances = new Object[elements.length];
                for (int i = 0; i < elements.length; i++) {
                    instances[i] = elements[i].instantiate(tuple[i]);
                }
                return Arrays.asList(instances);
            }
        };
    }

    // The codec's encodings of the value edges and random values, with random leading arguments, as buffers to decode.
    @SuppressWarnings("unchecked")
    private static List<Object[]> encodings(final Values.Generator[] leading, final Factory factory, final Codec codec, final Random random) {
        List<Object[]> specs = new ArrayList<>();
        List<Object> edges = codec.value().edges();
        for (int i = 0; i < ENCODED_VALUES; i++) {
            Object value = i < edges.size() ? edges.get(i) : codec.value().random(random);
            Object[] leadingSpecs = StaticSuite.randomTuple(leading, random);
            byte[] bytes;
            try {
                StreamCodec<Object, Object> made = (StreamCodec<Object, Object>)factory.make(instantiate(leading, leadingSpecs));
                Object buffer = NetworkValues.bufferLike(codec.buffer(), 0, new byte[0]);
                made.encode(buffer, codec.value().instantiate(value));
                bytes = NetworkValues.readable(NetworkValues.unwrap(buffer));
            } catch (Exception e) {
                continue;
            }

            for (byte[] variant : variants(bytes, random)) {
                Object[] spec = Arrays.copyOf(leadingSpecs, leading.length + 1);
                spec[leading.length] = NetworkValues.bufferLike(codec.buffer(), random.nextInt(17), variant);
                specs.add(spec);
            }
        }
        return specs;
    }

    /** An encoding to decode whole, with another field after it, cut short by a byte, and cut anywhere. */
    static List<byte[]> variants(final byte[] bytes, final Random random) {
        List<byte[]> variants = new ArrayList<>(List.of(bytes, NetworkValues.concat(bytes, NetworkValues.randomField(random))));
        if (bytes.length > 0) {
            variants.add(Arrays.copyOf(bytes, bytes.length - 1));
        }
        if (bytes.length > 1) {
            variants.add(Arrays.copyOf(bytes, 1 + random.nextInt(bytes.length - 1)));
        }
        return variants;
    }

    // Each edge once in each position, with random arguments elsewhere.
    private static List<Object[]> eachEdge(final Values.Generator[] generators, final Random random) {
        List<Object[]> tuples = new ArrayList<>();
        for (int i = 0; i < generators.length; i++) {
            for (Object edge : generators[i].edges()) {
                Object[] tuple = StaticSuite.randomTuple(generators, random);
                tuple[i] = edge;
                tuples.add(tuple);
            }
        }
        return tuples;
    }

    /**
     * Writes a case for each spec: the factory makes the codec or function from the leading arguments, and use applies
     * it to the rest. A throw from either is the case's.
     */
    static void writeCases(
        final GoldenWriter writer, final String name, final int leading, final Factory factory, final Values.Generator[] generators,
        final List<Object[]> specs, final Class<?> valueType, final Use use
    ) throws IOException {
        for (Object[] spec : specs) {
            JsonArray args = new JsonArray();
            Object[] instances = new Object[spec.length];
            for (int i = 0; i < spec.length; i++) {
                args.add(generators[i].encode(spec[i]));
                instances[i] = generators[i].instantiate(spec[i]);
            }

            Watchdog.begin(name + " " + args);
            try {
                Object made = factory.make(Arrays.copyOf(instances, leading));
                Object result = use.apply(made, Arrays.copyOfRange(instances, leading, instances.length));
                JsonElement encoded = valueType == void.class ? JsonNull.INSTANCE : Values.encode(result, valueType);
                writer.result(name, args, encoded, StaticSuite.after(generators, spec, instances, result));
            } catch (InvocationTargetException e) {
                writer.threw(name, args, e.getCause(), StaticSuite.after(generators, spec, instances, null));
            } catch (RuntimeException e) {
                writer.threw(name, args, e, StaticSuite.after(generators, spec, instances, null));
            } catch (Exception e) {
                throw new IllegalStateException(e);
            } finally {
                Watchdog.end();
            }
        }
    }

    private static Object[] instantiate(final Values.Generator[] generators, final Object[] specs) {
        Object[] instances = new Object[specs.length];
        for (int i = 0; i < specs.length; i++) {
            instances[i] = generators[i].instantiate(specs[i]);
        }
        return instances;
    }

    private static Values.Generator[] append(final Values.Generator[] generators, final Values.Generator last) {
        Values.Generator[] appended = Arrays.copyOf(generators, generators.length + 1);
        appended[generators.length] = last;
        return appended;
    }
}
