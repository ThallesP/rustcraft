package rustcraft.golden;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import net.minecraft.util.ARGB;
import net.minecraft.util.Mth;
import net.minecraft.world.level.levelgen.RandomSupport;

/** Writes vanilla's answers for the Rust golden tests, one JSON Lines file per class. See TESTING.md. */
public final class Golden {
    private static final double SIN_SCALE = 10430.378350470453;
    // Last-bit disagreements between libms are rare, so the methods HotSpot computes with its own
    // assembly get this many extra inputs each, spread over every path through the assembly.
    private static final int INTRINSIC_INPUTS = 20000;

    private static final List<Suite> SUITES = List.of(
        new StaticSuite(Mth.class)
            .extraInputs("sin(D)F", sinTableInputs(0))
            .extraInputs("cos(D)F", sinTableInputs(-16384))
            .extraInputs("atan2(DD)D", atan2Inputs())
            // Steps by 2π until the difference is in range, which never ends once 2π is below the float's precision.
            .where("rotLerpRad(FFF)F", args -> !(Math.abs((Float)args[2] - (Float)args[1]) >= 0x1p26F)),
        new StaticSuite(ARGB.class),
        new StaticSuite(RandomSupport.class).exclude("generateUniqueSeed()J", "seeded from System.nanoTime"),
        new RandomSourceSuite(),
        new StaticSuite(Math.class)
            .only(
                "abs(I)I", "abs(J)J", "abs(F)F", "abs(D)D",
                "min(FF)F", "max(FF)F", "min(DD)D", "max(DD)D",
                "floorDiv(II)I", "floorDiv(JJ)J", "floorMod(II)I", "floorMod(JJ)J",
                "sin(D)D", "cos(D)D", "tan(D)D", "asin(D)D", "log(D)D", "pow(DD)D", "sqrt(D)D"
            )
            .extraInputs("sin(D)D", trigInputs(1, -253, 16))
            .extraInputs("cos(D)D", trigInputs(2, -253, 16))
            .extraInputs("tan(D)D", trigInputs(5, -40, 13))
            .extraInputs("log(D)D", logInputs())
            .extraInputs("pow(DD)D", powInputs()),
        new StringSuite()
    );

    public static void main(final String[] args) throws IOException {
        Path outputDir = Path.of(args[0]);
        Files.createDirectories(outputDir);
        for (Suite suite : SUITES) {
            Path file = outputDir.resolve(suite.name() + ".jsonl");
            try (GoldenWriter writer = new GoldenWriter(file)) {
                suite.write(writer);
                System.out.printf("%s: %d lines%n", file.getFileName(), writer.lines());
            }
        }
    }

    // One input per entry of Mth's 65536-entry sine table, offset by `shift` entries (cos reads the table a quarter turn ahead).
    private static List<Object[]> sinTableInputs(final int shift) {
        List<Object[]> inputs = new ArrayList<>();
        for (int index = 0; index < 65536; index++) {
            inputs.add(new Object[]{(index + shift + 0.5) / SIN_SCALE});
        }
        return inputs;
    }

    // Angles game code passes, near multiples of pi/64 (where the reduction cancels the most, and
    // tan's poles), every magnitude from 2^low to 2^high, which spans the main path and its
    // short-cuts for small inputs (2^-252 to 90112 for sin and cos, 2^-29 to 12800 for tan), the
    // large-argument path up to 2^1024, tiny and subnormal inputs, and arbitrary bit patterns.
    private static List<Object[]> trigInputs(final long seed, final int low, final int high) {
        Random random = new Random(seed);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < INTRINSIC_INPUTS; i++) {
            double x = switch (i % 6) {
                case 0 -> (random.nextDouble() * 2.0 - 1.0) * 8.0 * Math.PI;
                case 1 -> {
                    double multiple = random.nextInt(1 << 20) * (Math.PI / 64.0);
                    yield multiple + (random.nextInt(9) - 4) * Math.ulp(multiple);
                }
                case 2 -> magnitude(random, low, high);
                case 3 -> magnitude(random, high, 1023);
                case 4 -> magnitude(random, -1074, low);
                default -> randomBits(random);
            };
            inputs.add(new Object[]{random.nextBoolean() ? x : -x});
        }
        return inputs;
    }

    // Around 1, where the result is smallest, (0, 1) like the squared radii nextGaussian takes,
    // every exponent from the subnormals up, and arbitrary bit patterns, negatives included.
    private static List<Object[]> logInputs() {
        Random random = new Random(3);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < INTRINSIC_INPUTS; i++) {
            double x = switch (i % 5) {
                case 0 -> nearOne(random);
                case 1 -> random.nextDouble();
                case 2 -> random.nextDouble() * 1000.0;
                case 3 -> magnitude(random, -1074, 1023);
                default -> randomBits(random);
            };
            inputs.add(new Object[]{x});
        }
        return inputs;
    }

    // Bases and exponents like game code's, small integer exponents of any base, bases near 1
    // (where log2 takes a more accurate path) with large exponents, results across the whole range
    // (subnormal, overflowing and underflowing ones included), negative bases with integer and
    // other exponents, tiny exponents, the exponents with special cases, and arbitrary bit patterns.
    private static List<Object[]> powInputs() {
        Random random = new Random(6);
        double[] specialExponents = {
            0.5, -0.5, 2.0, 1.0, -1.0, 3.0, 0.0, -0.0, Double.POSITIVE_INFINITY, Double.NEGATIVE_INFINITY, Double.NaN
        };
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < INTRINSIC_INPUTS; i++) {
            Object[] input = switch (i % 10) {
                case 0 -> new Object[]{random.nextDouble() * 1000.0, (random.nextDouble() * 2.0 - 1.0) * 8.0};
                case 1 -> new Object[]{sign(random) * magnitude(random, -32, 31), (double)(random.nextInt(41) - 20)};
                case 2 -> new Object[]{sign(random) * nearOne(random), sign(random) * Math.rint(magnitude(random, 0, 63))};
                case 3 -> new Object[]{nearOne(random), sign(random) * magnitude(random, -20, 63)};
                case 4 -> {
                    double x = magnitude(random, -1074, 1023);
                    double log2 = Math.log(x) / Math.log(2.0);
                    yield new Object[]{x, (random.nextDouble() * 2200.0 - 1120.0) / log2};
                }
                case 5 -> new Object[]{-magnitude(random, -1074, 1023), (double)(random.nextInt(41) - 20)};
                case 6 -> {
                    double y = sign(random) * magnitude(random, -4, 59);
                    yield new Object[]{-magnitude(random, -64, 63), random.nextBoolean() ? Math.rint(y) : y};
                }
                case 7 -> new Object[]{magnitude(random, -1074, 1023), sign(random) * magnitude(random, -1074, 0)};
                case 8 -> new Object[]{
                    random.nextBoolean() ? randomBits(random) : sign(random) * nearOne(random),
                    specialExponents[random.nextInt(specialExponents.length)]
                };
                default -> new Object[]{randomBits(random), randomBits(random)};
            };
            inputs.add(input);
        }
        return inputs;
    }

    // Every angle in every octant often enough to read each entry of Mth.atan2's tables, whose
    // COS_TAB comes from Math.cos, at radii from 2^-20 to 2^20.
    private static List<Object[]> atan2Inputs() {
        Random random = new Random(4);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 4096; i++) {
            double angle = (i + random.nextDouble()) * (2.0 * Math.PI / 4096.0);
            double radius = Math.scalb(1.0 + random.nextDouble(), random.nextInt(41) - 20);
            inputs.add(new Object[]{radius * Math.sin(angle), radius * Math.cos(angle)});
        }
        return inputs;
    }

    private static double sign(final Random random) {
        return random.nextBoolean() ? 1.0 : -1.0;
    }

    // 1 to 2 times a power of two from 2^low to 2^high. Below 2^-1022 it rounds to a subnormal.
    private static double magnitude(final Random random, final int low, final int high) {
        return Math.scalb(1.0 + random.nextDouble(), random.nextInt(high - low + 1) + low);
    }

    // 1 plus or minus up to 2^-k, for k from 0 to 52.
    private static double nearOne(final Random random) {
        return 1.0 + (random.nextDouble() * 2.0 - 1.0) * Math.scalb(1.0, -random.nextInt(53));
    }

    // NaN payloads never change game behaviour, so a random NaN becomes the canonical one, like Values does.
    private static double randomBits(final Random random) {
        double value = Double.longBitsToDouble(random.nextLong());
        return Double.isNaN(value) ? Double.NaN : value;
    }
}
