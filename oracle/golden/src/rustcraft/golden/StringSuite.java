package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;

/**
 * java.lang.String behaviour the game depends on: hashCode runs over UTF-16 units, not UTF-8 bytes or chars, and compareTo
 * returns the difference of the first UTF-16 units that differ, or of the lengths.
 */
final class StringSuite implements Suite {
    private static final int RANDOM_CASES = 500;

    @Override
    public String name() {
        return String.class.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Random random = new Random(31);
        List<Object> inputs = new ArrayList<>(Values.STRINGS.edges());
        for (int i = 0; i < RANDOM_CASES; i++) {
            inputs.add(Values.STRINGS.random(random));
        }

        for (Object input : inputs) {
            JsonArray args = new JsonArray();
            args.add((String)input);
            writer.result("hashCode()I", args, new JsonPrimitive(input.hashCode()), null);
        }

        // Every pair of edges, random pairs, and strings against their prefixes and one-unit changes.
        List<String[]> pairs = new ArrayList<>();
        for (Object a : Values.STRINGS.edges()) {
            for (Object b : Values.STRINGS.edges()) {
                pairs.add(new String[]{(String)a, (String)b});
            }
        }
        for (int i = 0; i < RANDOM_CASES; i++) {
            String a = (String)Values.STRINGS.random(random);
            String b = switch (random.nextInt(3)) {
                case 0 -> (String)Values.STRINGS.random(random);
                // Whole code points: a lone surrogate can't be written as UTF-8.
                case 1 -> a.substring(0, a.offsetByCodePoints(0, random.nextInt(a.codePointCount(0, a.length()) + 1)));
                default -> a + (char)('a' + random.nextInt(26));
            };
            pairs.add(random.nextBoolean() ? new String[]{a, b} : new String[]{b, a});
        }
        for (String[] pair : pairs) {
            JsonArray args = new JsonArray();
            args.add(pair[0]);
            args.add(pair[1]);
            writer.result("compareTo(Ljava/lang/String;)I", args, new JsonPrimitive(pair[0].compareTo(pair[1])), null);
        }
    }
}
