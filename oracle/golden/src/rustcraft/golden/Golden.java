package rustcraft.golden;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.util.ARGB;
import net.minecraft.util.Mth;
import net.minecraft.world.level.levelgen.RandomSupport;

/** Writes vanilla's answers for the Rust golden tests, one JSON Lines file per class. See TESTING.md. */
public final class Golden {
    private static final double SIN_SCALE = 10430.378350470453;

    private static final List<Suite> SUITES = List.of(
        new StaticSuite(Mth.class)
            .extraInputs("sin(D)F", sinTableInputs(0))
            .extraInputs("cos(D)F", sinTableInputs(-16384))
            // Steps by 2π until the difference is in range, which never ends once 2π is below the float's precision.
            .where("rotLerpRad(FFF)F", args -> !(Math.abs((Float)args[2] - (Float)args[1]) >= 0x1p26F)),
        new StaticSuite(ARGB.class),
        new StaticSuite(RandomSupport.class).exclude("generateUniqueSeed()J", "seeded from System.nanoTime"),
        new RandomSourceSuite(),
        new StaticSuite(Math.class).only(
            "abs(I)I", "abs(J)J", "abs(F)F", "abs(D)D",
            "min(FF)F", "max(FF)F", "min(DD)D", "max(DD)D",
            "floorDiv(II)I", "floorDiv(JJ)J", "floorMod(II)I", "floorMod(JJ)J",
            "sin(D)D", "cos(D)D", "asin(D)D", "log(D)D", "sqrt(D)D"
        ),
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
}
