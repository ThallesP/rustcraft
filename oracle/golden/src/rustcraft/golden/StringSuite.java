package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;

/** java.lang.String behaviour the game depends on: hashCode runs over UTF-16 units, not UTF-8 bytes or chars. */
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
    }
}
