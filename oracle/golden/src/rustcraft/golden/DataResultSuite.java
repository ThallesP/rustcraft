package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import com.mojang.serialization.DataResult;
import com.mojang.serialization.Lifecycle;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import java.util.function.Function;

/**
 * {@code com.mojang.serialization.DataResult}: programs on results of ints. A program starts from a result made one of
 * the six ways ({@code success} with or without a lifecycle, {@code error} with or without a partial value and a
 * lifecycle) and runs ops on it, each step the op's output and the result afterwards. A result is {@code ["success",
 * value, lifecycle]} or {@code ["error", message, partial, lifecycle]}, a lifecycle {@code "Stable"}, {@code
 * "Experimental"} or {@code ["Deprecated", since]}. The functions are fixed ({@code map}'s {@code x * 2 + 1}, {@code
 * apply2}'s {@code a * 3 + b}…), and the results they combine with are made like the first.
 */
final class DataResultSuite implements Suite {
    private static final int PROGRAMS = 2000;
    private static final List<String> MESSAGES = List.of("Not a number", "a", "", "Missing key 'é'", "x; y");
    private static final List<Integer> VALUES = List.of(0, 1, -1, 7, Integer.MAX_VALUE, Integer.MIN_VALUE, 1000);

    @Override
    public String name() {
        return "com.mojang.serialization.DataResult";
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Random random = new Random(43);
        for (int program = 0; program < PROGRAMS; program++) {
            JsonArray start = randomSpec(random);
            DataResult<Integer> result = build(start);
            JsonArray ops = new JsonArray();
            JsonArray outputs = new JsonArray();
            for (int step = random.nextInt(8); step > 0; step--) {
                JsonArray op = new JsonArray();
                JsonElement output = JsonNull.INSTANCE;
                switch (random.nextInt(19)) {
                    case 0 -> {
                        op.add("map");
                        result = result.map(x -> x * 2 + 1);
                    }
                    case 1 -> {
                        JsonArray spec = randomSpec(random);
                        op.add("flatMap");
                        op.add(spec);
                        result = result.flatMap(x -> build(spec).map(y -> x * 31 + y));
                    }
                    case 2 -> {
                        JsonArray spec = randomSpec(random);
                        op.add("ap");
                        op.add(spec);
                        DataResult<Function<Integer, Integer>> function = build(spec).map(y -> x -> x * 7 + y);
                        result = result.ap(function);
                    }
                    case 3 -> {
                        JsonArray spec = randomSpec(random);
                        op.add("apply2");
                        op.add(spec);
                        result = result.apply2((a, b) -> a * 3 + b, build(spec));
                    }
                    case 4 -> {
                        JsonArray spec = randomSpec(random);
                        op.add("apply2stable");
                        op.add(spec);
                        result = result.apply2stable((a, b) -> a * 3 + b, build(spec));
                    }
                    case 5 -> {
                        JsonArray second = randomSpec(random);
                        JsonArray third = randomSpec(random);
                        op.add("apply3");
                        op.add(second);
                        op.add(third);
                        result = result.apply3((a, b, c) -> a * 5 + b * 3 + c, build(second), build(third));
                    }
                    case 6 -> {
                        int value = VALUES.get(random.nextInt(VALUES.size()));
                        op.add("setPartial");
                        op.add(value);
                        result = result.setPartial(value);
                    }
                    case 7 -> {
                        op.add("mapError");
                        result = result.mapError(message -> "[" + message + "]");
                    }
                    case 8 -> {
                        JsonElement lifecycle = randomLifecycle(random);
                        op.add("setLifecycle");
                        op.add(lifecycle);
                        result = result.setLifecycle(lifecycle(lifecycle));
                    }
                    case 9 -> {
                        JsonElement lifecycle = randomLifecycle(random);
                        op.add("addLifecycle");
                        op.add(lifecycle);
                        result = result.addLifecycle(lifecycle(lifecycle));
                    }
                    case 10 -> {
                        op.add("promotePartial");
                        List<String> messages = new ArrayList<>();
                        result = result.promotePartial(messages::add);
                        output = strings(messages);
                    }
                    case 11 -> {
                        op.add("resultOrPartial");
                        List<String> messages = new ArrayList<>();
                        JsonArray answer = new JsonArray();
                        answer.add(result.resultOrPartial(messages::add).<JsonElement>map(JsonPrimitive::new).orElse(JsonNull.INSTANCE));
                        answer.add(strings(messages));
                        output = answer;
                    }
                    case 12 -> {
                        op.add("getOrThrow");
                        try {
                            output = new JsonPrimitive(result.getOrThrow());
                        } catch (IllegalStateException e) {
                            output = new JsonPrimitive("throws " + e.getMessage());
                        }
                    }
                    case 13 -> {
                        op.add("getPartialOrThrow");
                        try {
                            output = new JsonPrimitive(result.getPartialOrThrow());
                        } catch (IllegalStateException e) {
                            output = new JsonPrimitive("throws " + e.getMessage());
                        }
                    }
                    case 14 -> {
                        op.add("hasResultOrPartial");
                        output = new JsonPrimitive(result.hasResultOrPartial());
                    }
                    case 15 -> {
                        op.add("isSuccess");
                        output = new JsonPrimitive(result.isSuccess());
                    }
                    case 16 -> {
                        int value = VALUES.get(random.nextInt(VALUES.size()));
                        op.add("setPartialSupplier");
                        op.add(value);
                        result = result.setPartial(() -> value);
                    }
                    case 17 -> {
                        op.add("mapOrElse");
                        String answer = result.mapOrElse(value -> "success " + value, error -> "error " + error.message());
                        output = new JsonPrimitive(answer);
                    }
                    default -> {
                        op.add("toString");
                        output = new JsonPrimitive(result.toString());
                    }
                }
                ops.add(op);
                JsonArray outputAndResult = new JsonArray();
                outputAndResult.add(output);
                outputAndResult.add(encode(result));
                outputs.add(outputAndResult);
            }
            JsonArray args = new JsonArray();
            args.add(start);
            args.add(ops);
            JsonArray answer = new JsonArray();
            answer.add(encode(build(start)));
            answer.add(outputs);
            writer.result("programs", args, answer, null);
        }
    }

    // One of the six ways to make a result, with a value, a message and a lifecycle drawn at random.
    private static JsonArray randomSpec(final Random random) {
        JsonArray spec = new JsonArray();
        int value = VALUES.get(random.nextInt(VALUES.size()));
        String message = MESSAGES.get(random.nextInt(MESSAGES.size()));
        switch (random.nextInt(6)) {
            case 0 -> {
                spec.add("success");
                spec.add(value);
            }
            case 1 -> {
                spec.add("successLifecycle");
                spec.add(value);
                spec.add(randomLifecycle(random));
            }
            case 2 -> {
                spec.add("error");
                spec.add(message);
            }
            case 3 -> {
                spec.add("errorPartial");
                spec.add(message);
                spec.add(value);
            }
            case 4 -> {
                spec.add("errorPartialLifecycle");
                spec.add(message);
                spec.add(value);
                spec.add(randomLifecycle(random));
            }
            default -> {
                spec.add("errorLifecycle");
                spec.add(message);
                spec.add(randomLifecycle(random));
            }
        }
        return spec;
    }

    private static DataResult<Integer> build(final JsonArray spec) {
        return switch (spec.get(0).getAsString()) {
            case "success" -> DataResult.success(spec.get(1).getAsInt());
            case "successLifecycle" -> DataResult.success(spec.get(1).getAsInt(), lifecycle(spec.get(2)));
            case "error" -> DataResult.error(() -> spec.get(1).getAsString());
            case "errorPartial" -> DataResult.error(() -> spec.get(1).getAsString(), spec.get(2).getAsInt());
            case "errorPartialLifecycle" -> DataResult.error(() -> spec.get(1).getAsString(), spec.get(2).getAsInt(), lifecycle(spec.get(3)));
            default -> DataResult.error(() -> spec.get(1).getAsString(), lifecycle(spec.get(2)));
        };
    }

    private static JsonElement randomLifecycle(final Random random) {
        return switch (random.nextInt(4)) {
            case 0 -> new JsonPrimitive("Stable");
            case 1 -> new JsonPrimitive("Experimental");
            default -> {
                JsonArray deprecated = new JsonArray();
                deprecated.add("Deprecated");
                deprecated.add(1 + random.nextInt(2));
                yield deprecated;
            }
        };
    }

    private static Lifecycle lifecycle(final JsonElement spec) {
        if (spec.isJsonArray()) {
            return Lifecycle.deprecated(spec.getAsJsonArray().get(1).getAsInt());
        }
        return spec.getAsString().equals("Stable") ? Lifecycle.stable() : Lifecycle.experimental();
    }

    private static JsonElement encodeLifecycle(final Lifecycle lifecycle) {
        if (lifecycle instanceof Lifecycle.Deprecated deprecated) {
            JsonArray array = new JsonArray();
            array.add("Deprecated");
            array.add(deprecated.since());
            return array;
        }
        return new JsonPrimitive(lifecycle.toString());
    }

    private static JsonArray encode(final DataResult<Integer> result) {
        JsonArray array = new JsonArray();
        if (result instanceof DataResult.Success<Integer> success) {
            array.add("success");
            array.add(success.value());
        } else {
            DataResult.Error<Integer> error = result.error().orElseThrow();
            array.add("error");
            array.add(error.message());
            array.add(error.partialValue().<JsonElement>map(JsonPrimitive::new).orElse(JsonNull.INSTANCE));
        }
        array.add(encodeLifecycle(result.lifecycle()));
        return array;
    }

    private static JsonArray strings(final List<String> strings) {
        JsonArray array = new JsonArray();
        strings.forEach(array::add);
        return array;
    }
}
