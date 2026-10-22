package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonPrimitive;
import com.mojang.math.Axis;

/**
 * One op of a hand-written suite's program, encoded as {@code [name, args…]}. Args are floats, doubles, axes and JOML
 * objects, which suites copy before handing them to a method that writes to them.
 */
record Op(String name, Object... args) {
    JsonArray encode() {
        JsonArray op = new JsonArray();
        op.add(this.name);
        for (Object arg : this.args) {
            op.add(switch (arg) {
                case Float value -> new JsonPrimitive(Values.floatBits(value));
                case Double value -> new JsonPrimitive(Values.doubleBits(value));
                case Boolean value -> new JsonPrimitive(value);
                case Axis axis -> Values.AXES.encode(axis);
                default -> JomlValues.encode(arg);
            });
        }
        return op;
    }

    static JsonArray encode(final Iterable<Op> program) {
        JsonArray ops = new JsonArray();
        program.forEach(op -> ops.add(op.encode()));
        return ops;
    }
}
