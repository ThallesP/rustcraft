package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonPrimitive;
import com.mojang.datafixers.util.Pair;
import com.mojang.serialization.DataResult;
import com.mojang.serialization.Lifecycle;
import java.util.function.Function;

/**
 * DataFixerUpper's values, as the codec suites record them. A {@code DataResult} is {@code ["success", value,
 * lifecycle]} or {@code ["error", message, partial, lifecycle]}, its partial value {@code []} when there's none and
 * {@code [value]} when there is (a value can encode as null); a lifecycle {@code "Stable"}, {@code "Experimental"} or
 * {@code ["Deprecated", since]}; a {@code Number} {@code [box, value]}, the box's simple name ({@code "Byte"}) and the
 * number encoded by its class; and a {@code Pair} {@code [first, second]}.
 */
final class DfuValues {
    private DfuValues() {
    }

    static JsonElement lifecycle(final Lifecycle lifecycle) {
        if (lifecycle instanceof Lifecycle.Deprecated deprecated) {
            JsonArray array = new JsonArray();
            array.add("Deprecated");
            array.add(deprecated.since());
            return array;
        }
        return new JsonPrimitive(lifecycle.toString());
    }

    static <R> JsonArray result(final DataResult<R> result, final Function<? super R, JsonElement> value) {
        JsonArray array = new JsonArray();
        if (result instanceof DataResult.Success<R> success) {
            array.add("success");
            array.add(value.apply(success.value()));
        } else {
            DataResult.Error<R> error = result.error().orElseThrow();
            array.add("error");
            array.add(error.message());
            JsonArray partial = new JsonArray();
            error.partialValue().ifPresent(partialValue -> partial.add(value.apply(partialValue)));
            array.add(partial);
        }
        array.add(lifecycle(result.lifecycle()));
        return array;
    }

    static JsonArray number(final Number number) {
        JsonArray array = new JsonArray();
        array.add(number.getClass().getSimpleName());
        array.add(Values.encode(number, number.getClass()));
        return array;
    }

    static <F, S> JsonArray pair(final Pair<F, S> pair, final Function<? super F, JsonElement> first, final Function<? super S, JsonElement> second) {
        JsonArray array = new JsonArray();
        array.add(first.apply(pair.getFirst()));
        array.add(second.apply(pair.getSecond()));
        return array;
    }
}
