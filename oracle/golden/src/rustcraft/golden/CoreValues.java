package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import java.util.ArrayList;
import java.util.EnumSet;
import java.util.List;
import java.util.Random;
import net.minecraft.core.Direction;
import net.minecraft.core.Position;
import net.minecraft.core.Vec3i;
import net.minecraft.world.phys.Vec2;
import net.minecraft.world.phys.Vec3;

/**
 * Arguments and receivers of the game's core value types. A Vec3i is {@code [x, y, z]} (ints), a Vec3 {@code [x, y, z]}
 * (doubles), a Vec2 {@code [x, y]} (floats), an enum constant its name ({@code "NORTH"}), and an
 * {@code EnumSet<Direction.Axis>} the names of its axes in order.
 */
final class CoreValues {
    static final Values.Generator DIRECTION = new Constants<>(Direction.class);
    static final Values.Generator AXIS = new Constants<>(Direction.Axis.class);
    static final Values.Generator AXIS_DIRECTION = new Constants<>(Direction.AxisDirection.class);
    static final Values.Generator PLANE = new Constants<>(Direction.Plane.class);
    static final Values.Generator AXIS_SET = new AxisSets();

    // Each copy is a new object, so equals never takes its `this == o` shortcut, which only the receiver itself would.
    static final Values.Generator VEC3I = new JomlValues.Mutable<>(
        List.of(
            Vec3i.ZERO, new Vec3i(1, 0, 0), new Vec3i(0, 1, 0), new Vec3i(0, 0, 1), new Vec3i(-1, 0, 0), new Vec3i(0, -1, 0),
            new Vec3i(0, 0, -1), new Vec3i(1, 2, 3), new Vec3i(0, -61, 0), new Vec3i(16, 16, 16), new Vec3i(-30000000, 320, 30000000),
            new Vec3i(33554431, 2047, -33554432), new Vec3i(Integer.MAX_VALUE, Integer.MIN_VALUE, Integer.MAX_VALUE),
            new Vec3i(Integer.MIN_VALUE, Integer.MIN_VALUE, Integer.MIN_VALUE)
        ),
        CoreValues::randomVec3i,
        vec -> new Vec3i(vec.getX(), vec.getY(), vec.getZ()),
        vec -> ints(vec.getX(), vec.getY(), vec.getZ())
    );

    // The walk scenario's spawn and its deltas standing and walking, unit vectors, and vectors whose length overflows, is
    // below normalize's threshold, or isn't a number (x86's own NaN is negative, unlike Java's canonical one).
    static final Values.Generator VEC3 = new JomlValues.Mutable<>(
        List.of(
            Vec3.ZERO, Vec3.X_AXIS, Vec3.Y_AXIS, Vec3.Z_AXIS, new Vec3(-0.0, -0.0, -0.0), new Vec3(0.5, -60.0, 0.5),
            new Vec3(0.0, -0.0784000015258789, 0.053508008053839436), new Vec3(0.0, 0.0, -1.0), new Vec3(1.0, 2.0, 3.0),
            new Vec3(1.0E-6, 0.0, 0.0), new Vec3(3.0E7, 320.0, -3.0E7), new Vec3(Double.MAX_VALUE, 1.0, 0.0),
            new Vec3(Double.MIN_VALUE, 0.0, 0.0), new Vec3(Double.POSITIVE_INFINITY, 1.0, 0.0), new Vec3(Double.NaN, 0.0, 1.0),
            new Vec3(0.0, Double.longBitsToDouble(0xfff8000000000000L), 0.0)
        ),
        CoreValues::randomVec3,
        vec -> new Vec3(vec.x, vec.y, vec.z),
        vec -> doubles(vec.x, vec.y, vec.z)
    );

    // Rotations (x up and down, y around), and vectors like Vec3's, NaNs of both signs included.
    static final Values.Generator VEC2 = new JomlValues.Mutable<>(
        List.of(
            Vec2.ZERO, Vec2.ONE, Vec2.UNIT_X, Vec2.NEG_UNIT_Y, new Vec2(-0.0F, -0.0F), new Vec2(30.0F, 90.0F), new Vec2(-90.0F, 180.0F),
            new Vec2(1.0E-5F, 0.0F), Vec2.MAX, Vec2.MIN, new Vec2(Float.POSITIVE_INFINITY, 1.0F), new Vec2(Float.NaN, 0.0F),
            new Vec2(0.0F, Float.intBitsToFloat(0xffc00000))
        ),
        CoreValues::randomVec2,
        vec -> new Vec2(vec.x, vec.y),
        vec -> JomlValues.floats(vec.x, vec.y)
    );

    private CoreValues() {
    }

    static Values.Generator generator(final Class<?> type) {
        if (type == Direction.class) {
            return DIRECTION;
        }
        if (type == Direction.Axis.class) {
            return AXIS;
        }
        if (type == Direction.AxisDirection.class) {
            return AXIS_DIRECTION;
        }
        if (type == Direction.Plane.class) {
            return PLANE;
        }
        if (type == EnumSet.class) {
            return AXIS_SET;
        }
        if (type == Vec3i.class) {
            return VEC3I;
        }
        // Vec3 is the Position the game passes, entities' positions included.
        if (type == Vec3.class || type == Position.class) {
            return VEC3;
        }
        if (type == Vec2.class) {
            return VEC2;
        }
        return null;
    }

    static boolean canEncode(final Class<?> type) {
        return generator(type) != null;
    }

    static JsonElement encode(final Object value) {
        return switch (value) {
            case Enum<?> constant -> new JsonPrimitive(constant.name());
            case EnumSet<?> set -> AXIS_SET.encode(set);
            case Vec3i vec -> VEC3I.encode(vec);
            case Vec3 vec -> VEC3.encode(vec);
            case Vec2 vec -> VEC2.encode(vec);
            default -> throw new IllegalArgumentException("no encoder for " + value.getClass().getName());
        };
    }

    /** An enum's constants, which are all its values, by name. A null (an argument a method lets be null) is null. */
    record Constants<E extends Enum<E>>(Class<E> type) implements Values.Generator {
        @Override
        public List<Object> edges() {
            return List.of((Object[])this.type.getEnumConstants());
        }

        @Override
        public Object random(final Random random) {
            E[] constants = this.type.getEnumConstants();
            return constants[random.nextInt(constants.length)];
        }

        @Override
        public JsonElement encode(final Object spec) {
            return spec == null ? JsonNull.INSTANCE : new JsonPrimitive(((Enum<?>)spec).name());
        }

        @Override
        public boolean exhaustive() {
            return true;
        }
    }

    /** Every set of axes, the only EnumSet a covered method takes (Vec3.align). Each call gets its own copy. */
    private static final class AxisSets implements Values.Generator {
        @Override
        public List<Object> edges() {
            List<Object> sets = new ArrayList<>();
            for (int bits = 0; bits < 8; bits++) {
                sets.add(axisSet(bits));
            }
            return sets;
        }

        @Override
        public Object random(final Random random) {
            return axisSet(random.nextInt(8));
        }

        @Override
        public JsonElement encode(final Object spec) {
            JsonArray array = new JsonArray();
            for (Object axis : (EnumSet<?>)spec) {
                array.add(((Enum<?>)axis).name());
            }
            return array;
        }

        @Override
        public Object instantiate(final Object spec) {
            return EnumSet.copyOf((EnumSet<Direction.Axis>)spec);
        }

        @Override
        public boolean exhaustive() {
            return true;
        }

        private static EnumSet<Direction.Axis> axisSet(final int bits) {
            EnumSet<Direction.Axis> set = EnumSet.noneOf(Direction.Axis.class);
            for (Direction.Axis axis : Direction.Axis.VALUES) {
                if ((bits & 1 << axis.ordinal()) != 0) {
                    set.add(axis);
                }
            }
            return set;
        }
    }

    // Blocks around the walk scenario's spawn within its render distance at any height of the overworld, offsets and
    // steps, anywhere inside the world border at any height a BlockPos packs, and any ints.
    static Vec3i randomVec3i(final Random random) {
        return switch (random.nextInt(4)) {
            case 0 -> new Vec3i(random.nextInt(161) - 80, random.nextInt(385) - 64, random.nextInt(161) - 80);
            case 1 -> new Vec3i(random.nextInt(5) - 2, random.nextInt(5) - 2, random.nextInt(5) - 2);
            case 2 -> new Vec3i(random.nextInt(60000001) - 30000000, random.nextInt(4096) - 2048, random.nextInt(60000001) - 30000000);
            default -> new Vec3i((Integer)Values.INTS.random(random), (Integer)Values.INTS.random(random), (Integer)Values.INTS.random(random));
        };
    }

    // Positions around the walk scenario's spawn, entities' deltas (some components zero), look vectors, block corners
    // and centers, and any doubles.
    static Vec3 randomVec3(final Random random) {
        return switch (random.nextInt(5)) {
            case 0 -> new Vec3((random.nextDouble() * 2.0 - 1.0) * 80.0, random.nextDouble() * 384.0 - 64.0, (random.nextDouble() * 2.0 - 1.0) * 80.0);
            case 1 -> {
                double scale = random.nextBoolean() ? 0.5 : 4.0;
                yield new Vec3(delta(random, scale), random.nextInt(4) == 0 ? -0.0784000015258789 : delta(random, scale), delta(random, scale));
            }
            case 2 -> Vec3.directionFromRotation((random.nextFloat() * 2.0F - 1.0F) * 90.0F, (random.nextFloat() * 2.0F - 1.0F) * 180.0F);
            case 3 -> {
                Vec3i block = randomVec3i(random);
                yield random.nextBoolean() ? Vec3.atCenterOf(block) : Vec3.atLowerCornerOf(block);
            }
            default -> new Vec3((Double)Values.DOUBLES.random(random), (Double)Values.DOUBLES.random(random), (Double)Values.DOUBLES.random(random));
        };
    }

    private static double delta(final Random random, final double scale) {
        return random.nextInt(3) == 0 ? 0.0 : (random.nextDouble() * 2.0 - 1.0) * scale;
    }

    static Vec2 randomVec2(final Random random) {
        return switch (random.nextInt(3)) {
            case 0 -> new Vec2((random.nextFloat() * 2.0F - 1.0F) * 90.0F, (random.nextFloat() * 2.0F - 1.0F) * 180.0F);
            case 1 -> new Vec2(random.nextFloat() * 2.0F - 1.0F, random.nextFloat() * 2.0F - 1.0F);
            default -> new Vec2((Float)Values.FLOATS.random(random), (Float)Values.FLOATS.random(random));
        };
    }

    private static JsonArray ints(final int... values) {
        JsonArray array = new JsonArray();
        for (int value : values) {
            array.add(value);
        }
        return array;
    }

    private static JsonArray doubles(final double... values) {
        JsonArray array = new JsonArray();
        for (double value : values) {
            array.add(new JsonPrimitive(Values.doubleBits(value)));
        }
        return array;
    }
}
