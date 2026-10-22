package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.function.LongFunction;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.levelgen.LegacyRandomSource;
import net.minecraft.world.level.levelgen.SingleThreadedRandomSource;
import net.minecraft.world.level.levelgen.ThreadSafeLegacyRandomSource;
import net.minecraft.world.level.levelgen.XoroshiroRandomSource;

/**
 * Runs random programs against every RandomSource implementation. A case is one source seeded with
 * {@code args[0]} running the ops in {@code args[1]}; the result is each op's output. Order matters, since
 * every op advances the source. Programs that draw gaussians are filed under {@code <source>+nextGaussian},
 * because nextGaussian goes through Math.log, which differs between CPUs.
 */
final class RandomSourceSuite implements Suite {
    private static final int PROGRAMS_PER_SOURCE = 64;
    private static final int GAUSSIAN_PROGRAMS_PER_SOURCE = 16;
    private static final int OPS_PER_PROGRAM = 96;
    private static final List<Long> SEED_EDGES = List.of(0L, 1L, -1L, 42L, Long.MIN_VALUE, Long.MAX_VALUE, 25214903917L);
    private static final int[] BOUNDS = {1, 2, 3, 5, 7, 10, 16, 64, 100, 1000, 1 << 30, (1 << 30) + 1, Integer.MAX_VALUE};
    private static final List<JsonArray> INVALID_OPS = List.of(
        op("nextInt", 0),
        op("nextInt", -1),
        op("nextInt", Integer.MIN_VALUE),
        op("nextIntRange", 5, 5),
        op("nextIntRange", 5, 1),
        op("nextIntBetweenInclusive", 5, 3),
        op("nextIntBetweenInclusive", Integer.MIN_VALUE, Integer.MAX_VALUE)
    );

    private static final Map<String, LongFunction<RandomSource>> SOURCES = new LinkedHashMap<>();

    static {
        SOURCES.put("LegacyRandomSource", LegacyRandomSource::new);
        SOURCES.put("XoroshiroRandomSource", XoroshiroRandomSource::new);
        SOURCES.put("SingleThreadedRandomSource", SingleThreadedRandomSource::new);
        SOURCES.put("ThreadSafeLegacyRandomSource", ThreadSafeLegacyRandomSource::new);
    }

    @Override
    public String name() {
        return RandomSource.class.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Random random = new Random(0x5eed);
        for (Map.Entry<String, LongFunction<RandomSource>> source : SOURCES.entrySet()) {
            for (int i = 0; i < PROGRAMS_PER_SOURCE; i++) {
                long seed = i < SEED_EDGES.size() ? SEED_EDGES.get(i) : random.nextLong();
                run(writer, source.getKey(), source.getValue(), seed, randomProgram(random, false));
            }

            for (int i = 0; i < GAUSSIAN_PROGRAMS_PER_SOURCE; i++) {
                run(writer, source.getKey() + "+nextGaussian", source.getValue(), random.nextLong(), randomProgram(random, true));
            }

            for (JsonArray op : INVALID_OPS) {
                JsonArray program = new JsonArray();
                program.add(op);
                run(writer, source.getKey(), source.getValue(), 0L, program);
            }
        }
    }

    // A thrown op ends the program: the source's state after an exception isn't worth matching.
    private static void run(
        final GoldenWriter writer, final String name, final LongFunction<RandomSource> factory, final long seed, final JsonArray program
    ) throws IOException {
        RandomSource source = factory.apply(seed);
        JsonArray outputs = new JsonArray();
        for (JsonElement op : program) {
            try {
                outputs.add(apply(source, op.getAsJsonArray()));
            } catch (RuntimeException e) {
                JsonObject thrown = new JsonObject();
                thrown.addProperty("throws", e.getClass().getName());
                outputs.add(thrown);
                break;
            }
        }

        JsonArray args = new JsonArray();
        args.add(seed);
        args.add(program);
        writer.result(name, args, outputs, null);
    }

    private static JsonElement apply(final RandomSource source, final JsonArray op) {
        return switch (op.get(0).getAsString()) {
            case "nextInt" -> new JsonPrimitive(op.size() == 1 ? source.nextInt() : source.nextInt(op.get(1).getAsInt()));
            case "nextIntRange" -> new JsonPrimitive(source.nextInt(op.get(1).getAsInt(), op.get(2).getAsInt()));
            case "nextIntBetweenInclusive" -> new JsonPrimitive(source.nextIntBetweenInclusive(op.get(1).getAsInt(), op.get(2).getAsInt()));
            case "nextLong" -> new JsonPrimitive(source.nextLong());
            case "nextBoolean" -> new JsonPrimitive(source.nextBoolean());
            case "nextFloat" -> new JsonPrimitive(Values.floatBits(source.nextFloat()));
            case "nextDouble" -> new JsonPrimitive(Values.doubleBits(source.nextDouble()));
            case "nextGaussian" -> new JsonPrimitive(Values.doubleBits(source.nextGaussian()));
            case "triangleDouble" -> new JsonPrimitive(Values.doubleBits(source.triangle(Values.parseDouble(op.get(1)), Values.parseDouble(op.get(2)))));
            case "triangleFloat" -> new JsonPrimitive(Values.floatBits(source.triangle(Values.parseFloat(op.get(1)), Values.parseFloat(op.get(2)))));
            case "consumeCount" -> {
                source.consumeCount(op.get(1).getAsInt());
                yield JsonNull.INSTANCE;
            }
            case "setSeed" -> {
                source.setSeed(op.get(1).getAsLong());
                yield JsonNull.INSTANCE;
            }
            case "fork" -> probe(source.fork());
            case "forkPositionalAt" -> probe(source.forkPositional().at(op.get(1).getAsInt(), op.get(2).getAsInt(), op.get(3).getAsInt()));
            case "forkPositionalFromHashOf" -> probe(source.forkPositional().fromHashOf(op.get(1).getAsString()));
            case "forkPositionalFromSeed" -> probe(source.forkPositional().fromSeed(op.get(1).getAsLong()));
            case "forkPositionalParity" -> {
                StringBuilder builder = new StringBuilder();
                source.forkPositional().parityConfigString(builder);
                yield new JsonPrimitive(builder.toString());
            }
            default -> throw new IllegalArgumentException(op.toString());
        };
    }

    // A few draws from a derived source tell apart both its seed and its implementation.
    private static JsonArray probe(final RandomSource source) {
        JsonArray probe = new JsonArray();
        probe.add(source.nextLong());
        probe.add(source.nextInt(100));
        probe.add(Values.floatBits(source.nextFloat()));
        probe.add(Values.doubleBits(source.nextDouble()));
        return probe;
    }

    private static JsonArray randomProgram(final Random random, final boolean gaussians) {
        JsonArray program = new JsonArray();
        while (program.size() < OPS_PER_PROGRAM) {
            JsonArray op = randomOp(random);
            if (gaussians || !op.get(0).getAsString().equals("nextGaussian")) {
                program.add(op);
            }
        }
        return program;
    }

    private static JsonArray randomOp(final Random random) {
        return switch (random.nextInt(19)) {
            case 0 -> op("nextInt");
            case 1, 2 -> op("nextInt", random.nextBoolean() ? BOUNDS[random.nextInt(BOUNDS.length)] : 1 + random.nextInt(Integer.MAX_VALUE));
            case 3 -> {
                int origin = random.nextInt(2001) - 1000;
                yield op("nextIntRange", origin, origin + 1 + random.nextInt(1000));
            }
            case 4 -> {
                int min = random.nextInt(2001) - 1000;
                yield op("nextIntBetweenInclusive", min, min + random.nextInt(1000));
            }
            case 5 -> op("nextLong");
            case 6 -> op("nextBoolean");
            case 7 -> op("nextFloat");
            case 8 -> op("nextDouble");
            case 9, 10 -> op("nextGaussian");
            case 11 -> op("triangleDouble", Values.doubleBits(random.nextDouble() * 20.0 - 10.0), Values.doubleBits(random.nextDouble() * 5.0));
            case 12 -> op("triangleFloat", Values.floatBits(random.nextFloat() * 20.0F - 10.0F), Values.floatBits(random.nextFloat() * 5.0F));
            case 13 -> op("consumeCount", random.nextInt(11));
            case 14 -> op("setSeed", random.nextLong());
            case 15 -> op("fork");
            case 16 -> op("forkPositionalAt", random.nextInt(), random.nextInt(4096) - 2048, random.nextInt());
            case 17 -> op("forkPositionalFromHashOf", (String)Values.STRINGS.random(random));
            default -> random.nextBoolean() ? op("forkPositionalFromSeed", random.nextLong()) : op("forkPositionalParity");
        };
    }

    private static JsonArray op(final String name, final Object... args) {
        JsonArray op = new JsonArray();
        op.add(name);
        for (Object arg : args) {
            switch (arg) {
                case Integer value -> op.add(value);
                case Long value -> op.add(value);
                case String value -> op.add(value);
                default -> throw new IllegalArgumentException(arg.toString());
            }
        }
        return op;
    }
}
