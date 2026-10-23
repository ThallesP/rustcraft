package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonPrimitive;
import com.mojang.math.OctahedralGroup;
import it.unimi.dsi.fastutil.doubles.DoubleList;
import java.io.IOException;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import java.util.function.Supplier;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.Vec3i;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.level.EmptyBlockGetter;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.CrossCollisionBlock;
import net.minecraft.world.level.block.SlabBlock;
import net.minecraft.world.level.block.SoulSandBlock;
import net.minecraft.world.level.block.StairBlock;
import net.minecraft.world.level.block.state.BlockBehaviour;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.phys.shapes.BitSetDiscreteVoxelShape;
import net.minecraft.world.phys.shapes.BooleanOp;
import net.minecraft.world.phys.shapes.CubePointRange;
import net.minecraft.world.phys.shapes.DiscreteVoxelShape;
import net.minecraft.world.phys.shapes.IndexMerger;
import net.minecraft.world.phys.shapes.Shapes;
import net.minecraft.world.phys.shapes.SliceShape;
import net.minecraft.world.phys.shapes.VoxelShape;

/**
 * Voxel shapes and the collisions made of them (TESTING.md). A shape argument is the recipe that makes it, which the
 * port follows with its own functions (`"BLOCK"`, `["box", minX, …]`, `["move", shape, dx, dy, dz]`, `["join", first,
 * second, op]`…), so each argument is a fresh shape and only the constants are the same object twice. A shape result
 * is the constant's name, or everything about it: its class, each axis's coordinates (`["cube", values]` for a
 * `CubePointRange`, which `createIndexMerger` tells apart, `["array", values]` otherwise) and its voxels
 * (`[class, xSize, ySize, zSize, firstFull, lastFull, bits]`, the bits x outermost).
 */
final class ShapeSuites {
    private static final Field SHAPE = field(VoxelShape.class, "shape");
    private static final Method FIND_INDEX = method(VoxelShape.class, "findIndex", Direction.Axis.class, double.class);
    private static final Method FIND_BITS = method(Shapes.class, "findBits", double.class, double.class);
    private static final Method LCM = method(Shapes.class, "lcm", int.class, int.class);
    private static final Method CREATE_INDEX_MERGER = method(
        Shapes.class, "createIndexMerger", int.class, DoubleList.class, DoubleList.class, boolean.class, boolean.class
    );
    private static final Method COLLIDE_WITH_SHAPES = method(Entity.class, "collideWithShapes", Vec3.class, AABB.class, List.class);
    private static final Method COLLECT_CANDIDATE_STEP_UP_HEIGHTS = method(
        Entity.class, "collectCandidateStepUpHeights", AABB.class, List.class, float.class, float.class
    );
    private static final Method GET_INPUT_VECTOR = method(Entity.class, "getInputVector", Vec3.class, float.class, float.class);
    private static final Method COMPUTE_MODIFIED_FRICTION = method(LivingEntity.class, "computeModifiedFriction", float.class, float.class);
    private static final String SHAPE_TYPE = "Lnet/minecraft/world/phys/shapes/VoxelShape;";
    private static final String AXIS_TYPE = "Lnet/minecraft/core/Direction$Axis;";
    private static final String AABB_TYPE = "Lnet/minecraft/world/phys/AABB;";
    private static final String OP_TYPE = "Lnet/minecraft/world/phys/shapes/BooleanOp;";
    private static final String DIRECTION_TYPE = "Lnet/minecraft/core/Direction;";
    private static final String VEC3_TYPE = "Lnet/minecraft/world/phys/Vec3;";
    private static final String OCTAHEDRAL_GROUP_TYPE = "Lcom/mojang/math/OctahedralGroup;";
    private static final List<String> OP_NAMES = List.of(
        "FALSE", "NOT_OR", "ONLY_SECOND", "NOT_FIRST", "ONLY_FIRST", "NOT_SECOND", "NOT_SAME", "NOT_AND", "AND", "SAME", "SECOND", "CAUSES",
        "FIRST", "CAUSED_BY", "OR", "TRUE"
    );
    private static final List<BooleanOp> OPS = OP_NAMES.stream().map(ShapeSuites::op).toList();
    // The ones the game joins with: the rest only differ in which voxels they keep.
    private static final List<String> GAME_OPS = List.of("OR", "AND", "ONLY_FIRST", "ONLY_SECOND", "NOT_SAME", "FIRST", "SECOND");
    // Vanilla blocks' shapes, in Block.box's sixteenths: slabs, carpets, snow layers, soul sand, paths, cacti, chests,
    // fences and walls (taller than a block), panes, campfires, enchanting tables, end portal frames, daylight detectors,
    // beds, hoppers' parts, stairs' quarters, lanterns, doors, trapdoors, lily pads, chains (half sixteenths) and anvils.
    private static final double[][] BLOCK_BOXES = {
        {0, 0, 0, 16, 16, 16}, {0, 0, 0, 16, 8, 16}, {0, 8, 0, 16, 16, 16}, {0, 0, 0, 16, 1, 16}, {0, 0, 0, 16, 2, 16},
        {0, 0, 0, 16, 4, 16}, {0, 0, 0, 16, 6, 16}, {0, 0, 0, 16, 10, 16}, {0, 0, 0, 16, 12, 16}, {0, 0, 0, 16, 14, 16},
        {0, 0, 0, 16, 15, 16}, {0, 0, 0, 16, 13, 16}, {0, 0, 0, 16, 7, 16}, {0, 0, 0, 16, 3, 16}, {0, 3, 0, 16, 9, 16},
        {1, 0, 1, 15, 15, 15}, {1, 0, 1, 15, 14, 15}, {1, 0, 1, 15, 16, 15}, {6, 0, 6, 10, 24, 10}, {6, 0, 6, 10, 16, 10},
        {7, 0, 0, 9, 24, 9}, {4, 0, 4, 12, 24, 12}, {5, 0, 0, 11, 24, 8}, {7, 0, 7, 9, 16, 9}, {7, 0, 0, 9, 16, 9},
        {0, 10, 0, 16, 16, 16}, {4, 4, 4, 12, 10, 12}, {6, 0, 6, 10, 4, 10}, {0, 8, 0, 8, 16, 8}, {8, 8, 0, 16, 16, 8},
        {0, 8, 8, 8, 16, 16}, {8, 8, 8, 16, 16, 16}, {5, 0, 5, 11, 7, 11}, {6, 7, 6, 10, 9, 10}, {0, 0, 0, 16, 16, 3},
        {0, 0, 13, 16, 16, 16}, {0, 0, 0, 3, 16, 16}, {13, 0, 0, 16, 16, 16}, {0, 13, 0, 16, 16, 16}, {1, 0, 1, 15, 1.5, 15},
        {6.5, 0, 6.5, 9.5, 16, 9.5}, {2, 0, 2, 14, 4, 14}, {4, 4, 3, 12, 5, 13}, {6, 5, 4, 10, 10, 12}, {3, 10, 0, 13, 16, 16},
        {0, 0, 0, 16, 9, 16}, {2, 0, 2, 14, 16, 14}, {0, 0, 0, 16, 0, 16}, {8, 0, 8, 8, 16, 8}, {4, 0, 4, 12, 16, 12},
        {0, 0, 0, 0.001, 16, 16}, {3, 0, 3, 13, 16, 13}, {0, 0, 0, 16, 16, 15.999999}, {0, -8, 0, 16, 8, 16}
    };
    // Entities' sizes: the player standing, crouching and swimming, an item, an arrow and a minecart.
    private static final float[][] ENTITY_SIZES = {{0.6F, 1.8F}, {0.6F, 1.5F}, {0.6F, 0.6F}, {0.25F, 0.25F}, {0.5F, 0.5F}, {0.98F, 0.7F}};
    // Coordinates around a side, where a shape's grid lines and the collision margins meet: on it, 5.0E-8, 1.0E-7 and
    // 2.0E-7 either way, and an ulp either side of 1.0E-7.
    private static final double[] NUDGES = {
        0.0, 5.0E-8, -5.0E-8, 1.0E-7, -1.0E-7, Math.nextUp(1.0E-7), Math.nextDown(1.0E-7), -Math.nextUp(1.0E-7), -Math.nextDown(1.0E-7),
        2.0E-7, -2.0E-7, 1.0E-6, -1.0E-6
    };

    private ShapeSuites() {
    }

    static List<Suite> suites() {
        return List.of(
            new ShapesSuite(), new VoxelShapeSuite(), new DiscreteVoxelShapeSuite(), new EntitySuite(), new LivingEntitySuite(), new BlockStateShapeSuite()
        );
    }

    /** A shape argument: its recipe, and how to make it afresh. */
    record Recipe(JsonElement json, Supplier<VoxelShape> make) {
    }

    static Recipe constant(final String name, final Supplier<VoxelShape> shape) {
        return new Recipe(new JsonPrimitive(name), shape);
    }

    static Recipe box(final String kind, final double... sides) {
        JsonArray json = new JsonArray();
        json.add(kind);
        for (double side : sides) {
            json.add(d(side));
        }
        return new Recipe(json, () -> switch (kind) {
            case "box" -> Shapes.box(sides[0], sides[1], sides[2], sides[3], sides[4], sides[5]);
            case "create" -> Shapes.create(sides[0], sides[1], sides[2], sides[3], sides[4], sides[5]);
            default -> throw new IllegalArgumentException(kind);
        });
    }

    static Recipe move(final Recipe shape, final double dx, final double dy, final double dz) {
        return new Recipe(BlockSuites.array(new JsonPrimitive("move"), shape.json, d(dx), d(dy), d(dz)), () -> shape.make.get().move(dx, dy, dz));
    }

    static Recipe join(final String kind, final Recipe first, final Recipe second, final String op) {
        return new Recipe(BlockSuites.array(new JsonPrimitive(kind), first.json, second.json, new JsonPrimitive(op)), () -> switch (kind) {
            case "join" -> Shapes.join(first.make.get(), second.make.get(), op(op));
            case "joinUnoptimized" -> Shapes.joinUnoptimized(first.make.get(), second.make.get(), op(op));
            default -> throw new IllegalArgumentException(kind);
        });
    }

    static Recipe face(final Recipe shape, final Direction direction) {
        return new Recipe(
            BlockSuites.array(new JsonPrimitive("face"), shape.json, new JsonPrimitive(direction.name())),
            () -> shape.make.get().getFaceShape(direction)
        );
    }

    static Recipe slice(final Recipe shape, final Direction.Axis axis, final int point) {
        return new Recipe(
            BlockSuites.array(new JsonPrimitive("slice"), shape.json, new JsonPrimitive(axis.name()), new JsonPrimitive(point)),
            () -> new SliceShape(shape.make.get(), axis, point)
        );
    }

    static Recipe optimize(final Recipe shape) {
        return new Recipe(BlockSuites.array(new JsonPrimitive("optimize"), shape.json), () -> shape.make.get().optimize());
    }

    static BooleanOp op(final String name) {
        try {
            return (BooleanOp)BooleanOp.class.getField(name).get(null);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }

    static JsonPrimitive d(final double value) {
        return new JsonPrimitive(Values.doubleBits(value));
    }

    static JsonPrimitive f(final float value) {
        return new JsonPrimitive(Values.floatBits(value));
    }

    // Block.box's shapes, at the block and moved to blocks around the spawn.
    static Recipe blockBox(final double[] box) {
        return box("box", box[0] / 16.0, box[1] / 16.0, box[2] / 16.0, box[3] / 16.0, box[4] / 16.0, box[5] / 16.0);
    }

    /** Shapes the game makes: blocks' shapes and their unions (stairs, fences, hoppers, cauldrons), entities' boxes. */
    static List<Recipe> baseShapes() {
        List<Recipe> shapes = new ArrayList<>();
        shapes.add(constant("BLOCK", Shapes::block));
        shapes.add(constant("EMPTY", Shapes::empty));
        shapes.add(constant("INFINITY", () -> Shapes.INFINITY));
        for (double[] box : BLOCK_BOXES) {
            shapes.add(blockBox(box));
        }
        // Stairs: a bottom slab and a top quarter, either way round.
        Recipe slab = blockBox(BLOCK_BOXES[1]);
        shapes.add(join("join", slab, blockBox(BLOCK_BOXES[28]), "OR"));
        shapes.add(join("join", blockBox(BLOCK_BOXES[29]), slab, "OR"));
        // A hopper, a fence post with two arms, and a cauldron (a block with its inside taken out).
        shapes.add(join("join", join("join", blockBox(BLOCK_BOXES[25]), blockBox(BLOCK_BOXES[26]), "OR"), blockBox(BLOCK_BOXES[27]), "OR"));
        shapes.add(join("join", join("join", blockBox(BLOCK_BOXES[19]), blockBox(BLOCK_BOXES[24]), "OR"), blockBox(new double[]{7, 0, 7, 16, 16, 9}), "OR"));
        shapes.add(join("join", constant("BLOCK", Shapes::block), join("join", blockBox(new double[]{2, 4, 2, 14, 16, 14}), blockBox(new double[]{0, 0, 4, 16, 3, 12}), "OR"), "ONLY_FIRST"));
        // The player's boxes at the spawn, and an item's.
        for (float[] size : ENTITY_SIZES) {
            AABB box = CoreValues.entityBox(size[0], size[1], new Vec3(0.5, -60.0, 0.5));
            shapes.add(box("create", box.minX, box.minY, box.minZ, box.maxX, box.maxY, box.maxZ));
        }
        // Boxes whose sides are around eighths (where findBits gives up) and around the block's sides.
        for (double nudge : NUDGES) {
            shapes.add(box("box", 0.125 + nudge, 0.0, 0.0, 0.5, 1.0, 1.0));
            shapes.add(box("box", 0.0, 0.0, 0.0, 1.0 + nudge, 0.5, 1.0));
            shapes.add(box("create", nudge, 0.25, 0.0, 1.0, 0.75, 1.0 - nudge));
        }
        return shapes;
    }

    /** Base shapes, moved to blocks, joined, sliced, faced and optimized, a few levels deep. */
    static Recipe randomShape(final Random random, final List<Recipe> base, final int depth) {
        if (depth <= 0 || random.nextInt(3) == 0) {
            return random.nextInt(4) == 0 ? randomBox(random) : base.get(random.nextInt(base.size()));
        }
        return switch (random.nextInt(6)) {
            case 0, 1 -> join(
                random.nextBoolean() ? "join" : "joinUnoptimized",
                randomShape(random, base, depth - 1),
                randomShape(random, base, depth - 1),
                GAME_OPS.get(random.nextInt(GAME_OPS.size()))
            );
            case 2 -> {
                Vec3i pos = randomBlock(random);
                yield random.nextBoolean()
                    ? move(randomShape(random, base, depth - 1), pos.getX(), pos.getY(), pos.getZ())
                    : move(randomShape(random, base, depth - 1), random.nextDouble() - 0.5, random.nextDouble() - 0.5, random.nextDouble() - 0.5);
            }
            case 3 -> face(randomShape(random, base, depth - 1), Direction.values()[random.nextInt(6)]);
            case 4 -> {
                Recipe shape = randomShape(random, base, depth - 1);
                Direction.Axis axis = Direction.Axis.values()[random.nextInt(3)];
                int size = ((DiscreteVoxelShape)get(SHAPE, shape.make.get())).getSize(axis);
                yield size == 0 ? shape : slice(shape, axis, random.nextInt(size));
            }
            default -> optimize(randomShape(random, base, depth - 1));
        };
    }

    static Recipe randomBox(final Random random) {
        return switch (random.nextInt(4)) {
            case 0 -> {
                AABB box = CoreValues.blockShape(random);
                yield box(random.nextBoolean() ? "box" : "create", box.minX, box.minY, box.minZ, box.maxX, box.maxY, box.maxZ);
            }
            case 1 -> {
                AABB box = CoreValues.randomEntityBox(random, new Vec3(random.nextDouble() * 4.0 - 2.0, -61.0 + random.nextDouble() * 3.0, random.nextDouble() * 4.0 - 2.0));
                yield box("create", box.minX, box.minY, box.minZ, box.maxX, box.maxY, box.maxZ);
            }
            case 2 -> {
                double[] sides = new double[6];
                for (int axis = 0; axis < 3; axis++) {
                    double a = random.nextInt(9) / 8.0 + NUDGES[random.nextInt(NUDGES.length)];
                    double b = random.nextInt(9) / 8.0 + NUDGES[random.nextInt(NUDGES.length)];
                    sides[axis] = Math.min(a, b);
                    sides[axis + 3] = Math.max(a, b);
                }
                yield box("create", sides);
            }
            default -> {
                double[] sides = new double[6];
                for (int axis = 0; axis < 3; axis++) {
                    double a = random.nextDouble() * 3.0 - 1.0;
                    double b = random.nextDouble() * 3.0 - 1.0;
                    sides[axis] = Math.min(a, b);
                    sides[axis + 3] = Math.max(a, b);
                }
                yield box("create", sides);
            }
        };
    }

    static Vec3i randomBlock(final Random random) {
        return new Vec3i(random.nextInt(5) - 2, random.nextInt(7) - 64, random.nextInt(5) - 2);
    }

    static JsonElement shape(final VoxelShape shape) {
        if (shape == Shapes.block()) {
            return new JsonPrimitive("BLOCK");
        }
        if (shape == Shapes.empty()) {
            return new JsonPrimitive("EMPTY");
        }
        if (shape == Shapes.INFINITY) {
            return new JsonPrimitive("INFINITY");
        }
        return structure(shape);
    }

    static JsonArray structure(final VoxelShape shape) {
        return BlockSuites.array(
            new JsonPrimitive(shape.getClass().getSimpleName()),
            BlockSuites.array(list(shape.getCoords(Direction.Axis.X)), list(shape.getCoords(Direction.Axis.Y)), list(shape.getCoords(Direction.Axis.Z))),
            discrete((DiscreteVoxelShape)get(SHAPE, shape))
        );
    }

    static JsonArray list(final DoubleList list) {
        JsonArray values = new JsonArray();
        for (int i = 0; i < list.size(); i++) {
            values.add(d(list.getDouble(i)));
        }
        return BlockSuites.array(new JsonPrimitive(list instanceof CubePointRange ? "cube" : "array"), values);
    }

    static JsonArray discrete(final DiscreteVoxelShape shape) {
        StringBuilder bits = new StringBuilder();
        for (int x = 0; x < shape.getXSize(); x++) {
            for (int y = 0; y < shape.getYSize(); y++) {
                for (int z = 0; z < shape.getZSize(); z++) {
                    bits.append(shape.isFull(x, y, z) ? '1' : '0');
                }
            }
        }
        JsonArray first = new JsonArray();
        JsonArray last = new JsonArray();
        for (Direction.Axis axis : Direction.Axis.values()) {
            first.add(shape.firstFull(axis));
            last.add(shape.lastFull(axis));
        }
        return BlockSuites.array(
            new JsonPrimitive(shape.getClass().getSimpleName()), new JsonPrimitive(shape.getXSize()), new JsonPrimitive(shape.getYSize()),
            new JsonPrimitive(shape.getZSize()), first, last, new JsonPrimitive(bits.toString())
        );
    }

    static JsonArray recipes(final List<Recipe> shapes) {
        JsonArray array = new JsonArray();
        shapes.forEach(shape -> array.add(shape.json));
        return array;
    }

    static List<VoxelShape> make(final List<Recipe> shapes) {
        return shapes.stream().map(shape -> shape.make.get()).toList();
    }

    static JsonArray floats(final float[] values) {
        JsonArray array = new JsonArray();
        for (float value : values) {
            array.add(f(value));
        }
        return array;
    }

    // Shapes' side lists, like createIndexMerger gets them: cubes' points, moved and joined shapes' points.
    static DoubleList coords(final Recipe shape, final Direction.Axis axis) {
        return shape.make.get().getCoords(axis);
    }

    static JsonArray merger(final IndexMerger merger) {
        JsonArray triples = new JsonArray();
        merger.forMergedIndexes((first, second, result) -> {
            triples.add(BlockSuites.array(new JsonPrimitive(first), new JsonPrimitive(second), new JsonPrimitive(result)));
            return true;
        });
        return BlockSuites.array(new JsonPrimitive(merger.getClass().getSimpleName()), list(merger.getList()), new JsonPrimitive(merger.size()), triples);
    }

    static Field field(final Class<?> owner, final String name) {
        try {
            Field field = owner.getDeclaredField(name);
            field.setAccessible(true);
            return field;
        } catch (NoSuchFieldException e) {
            throw new IllegalStateException(e);
        }
    }

    static Method method(final Class<?> owner, final String name, final Class<?>... parameters) {
        try {
            Method method = owner.getDeclaredMethod(name, parameters);
            method.setAccessible(true);
            return method;
        } catch (NoSuchMethodException e) {
            throw new IllegalStateException(e);
        }
    }

    static Object get(final Field field, final Object owner) {
        try {
            return field.get(owner);
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
    }

    /** Calls a method by reflection, rethrowing what it threw. */
    static Object invoke(final Method method, final Object receiver, final Object... args) {
        try {
            return method.invoke(receiver, args);
        } catch (InvocationTargetException e) {
            if (e.getCause() instanceof RuntimeException runtime) {
                throw runtime;
            }
            throw new IllegalStateException(e.getCause());
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
    }

    /** Pairs of shapes for the joins: every pair of a few base shapes, then random ones. */
    static List<Recipe[]> shapePairs(final Random random, final List<Recipe> base, final int count) {
        List<Recipe[]> pairs = new ArrayList<>();
        List<Recipe> few = base.subList(0, 12);
        for (Recipe first : few) {
            for (Recipe second : few) {
                pairs.add(new Recipe[]{first, second});
            }
        }
        for (int i = 0; i < count; i++) {
            pairs.add(new Recipe[]{randomShape(random, base, 2), randomShape(random, base, 2)});
        }
        return pairs;
    }

    /** Moving boxes and how far they move, for collide: an entity's box with a side on, or around, a shape's side. */
    record Collision(Direction.Axis axis, AABB moving, double distance) {
    }

    static Collision collision(final Random random, final VoxelShape shape) {
        Direction.Axis axis = Direction.Axis.values()[random.nextInt(3)];
        float[] size = ENTITY_SIZES[random.nextInt(ENTITY_SIZES.length)];
        AABB box;
        double distance;
        if (shape.isEmpty() || random.nextInt(5) == 0) {
            box = CoreValues.randomEntityBox(random, new Vec3(random.nextDouble() * 4.0 - 2.0, random.nextDouble() * 6.0 - 64.0, random.nextDouble() * 4.0 - 2.0));
            distance = random.nextDouble() * 4.0 - 2.0;
        } else {
            AABB bounds = shape.bounds();
            boolean positive = random.nextBoolean();
            double nudge = NUDGES[random.nextInt(NUDGES.length)];
            double width = size[0];
            double height = size[1];
            // The box's leading side at the shape's side, nudged; across the other axes over a part of the shape, or with a
            // side on one of its sides, nudged.
            double[] min = new double[3];
            double[] max = new double[3];
            for (Direction.Axis other : Direction.Axis.values()) {
                int i = other.ordinal();
                double extent = other == Direction.Axis.Y ? height : width;
                double lo = bounds.min(other);
                double hi = bounds.max(other);
                if (other == axis) {
                    double gap = random.nextInt(3) == 0 ? random.nextDouble() * 0.5 : 0.0;
                    if (positive) {
                        max[i] = Double.isInfinite(lo) ? 0.0 : lo - gap + nudge;
                        min[i] = max[i] - extent;
                    } else {
                        min[i] = Double.isInfinite(hi) ? 0.0 : hi + gap + nudge;
                        max[i] = min[i] + extent;
                    }
                } else {
                    double start = switch (random.nextInt(3)) {
                        case 0 -> (Double.isInfinite(hi) ? 0.0 : hi) + NUDGES[random.nextInt(NUDGES.length)];
                        case 1 -> (Double.isInfinite(lo) ? 0.0 : lo) - extent + NUDGES[random.nextInt(NUDGES.length)];
                        default -> (Double.isInfinite(lo) ? 0.0 : lo) + random.nextDouble() - extent * 0.5;
                    };
                    min[i] = start;
                    max[i] = start + extent;
                }
            }
            box = new AABB(min[0], min[1], min[2], max[0], max[1], max[2]);
            double reach = random.nextInt(4) == 0 ? NUDGES[random.nextInt(NUDGES.length)] : random.nextDouble() * (random.nextBoolean() ? 0.3 : 3.0);
            distance = positive ? reach : -reach;
        }
        return new Collision(axis, box, distance);
    }

    /** Shapes' static methods, Block.box, and the fields. */
    static final class ShapesSuite implements Suite {
        @Override
        public String name() {
            return Shapes.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0x5348415045L);
            List<Recipe> base = baseShapes();
            writer.field("EPSILON", d(Shapes.EPSILON));
            writer.field("BIG_EPSILON", d(Shapes.BIG_EPSILON));
            writer.field("INFINITY", structure(Shapes.INFINITY));
            writer.result("block()" + SHAPE_TYPE, new JsonArray(), structure(Shapes.block()), null);
            writer.result("empty()" + SHAPE_TYPE, new JsonArray(), structure(Shapes.empty()), null);

            List<double[]> boxes = new ArrayList<>();
            for (double[] box : BLOCK_BOXES) {
                boxes.add(new double[]{box[0] / 16.0, box[1] / 16.0, box[2] / 16.0, box[3] / 16.0, box[4] / 16.0, box[5] / 16.0});
            }
            for (double nudge : NUDGES) {
                for (double side : new double[]{0.0, 0.125, 0.25, 0.375, 0.5, 1.0 / 3.0, 0.0625, 1.0}) {
                    boxes.add(new double[]{side + nudge, 0.0, 0.0, 1.0, 1.0, 1.0});
                    boxes.add(new double[]{0.0, side, 0.0, 1.0, side + nudge + 1.0E-6, 1.0});
                    boxes.add(new double[]{0.0, 0.0, side - nudge, 1.0, 1.0, side + 0.5 + nudge});
                }
            }
            double[] odd = {Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY, -1.0, 2.0, 1.0000001, -1.0E-7, Math.nextUp(1.0000001), -Math.nextUp(1.0E-7)};
            for (double value : odd) {
                boxes.add(new double[]{value, 0.0, 0.0, 1.0, 1.0, 1.0});
                boxes.add(new double[]{0.0, 0.0, 0.0, 1.0, value, 1.0});
            }
            boxes.add(new double[]{0.5, 0.0, 0.0, 0.25, 1.0, 1.0});
            boxes.add(new double[]{0.0, 0.5, 0.0, 1.0, 0.5, 1.0});
            boxes.add(new double[]{0.0, 0.0, 0.0, 1.0, 1.0, 9.9E-8});
            for (int i = 0; i < 1500; i++) {
                Recipe recipe = randomBox(random);
                double[] sides = new double[6];
                for (int j = 0; j < 6; j++) {
                    sides[j] = Values.parseDouble(recipe.json.getAsJsonArray().get(j + 1));
                }
                boxes.add(sides);
            }
            for (double[] b : boxes) {
                JsonArray args = BlockSuites.array(d(b[0]), d(b[1]), d(b[2]), d(b[3]), d(b[4]), d(b[5]));
                BlockSuites.call(writer, "box(DDDDDD)" + SHAPE_TYPE, args, () -> shape(Shapes.box(b[0], b[1], b[2], b[3], b[4], b[5])));
                BlockSuites.call(writer, "create(DDDDDD)" + SHAPE_TYPE, args, () -> shape(Shapes.create(b[0], b[1], b[2], b[3], b[4], b[5])));
                AABB aabb = new AABB(b[0], b[1], b[2], b[3], b[4], b[5]);
                BlockSuites.call(writer, "create(" + AABB_TYPE + ")" + SHAPE_TYPE, BlockSuites.array(CoreValues.encode(aabb)), () -> shape(Shapes.create(aabb)));
                BlockSuites.call(
                    writer, "Block.box(DDDDDD)" + SHAPE_TYPE,
                    BlockSuites.array(d(b[0] * 16.0), d(b[1] * 16.0), d(b[2] * 16.0), d(b[3] * 16.0), d(b[4] * 16.0), d(b[5] * 16.0)),
                    () -> shape(Block.box(b[0] * 16.0, b[1] * 16.0, b[2] * 16.0, b[3] * 16.0, b[4] * 16.0, b[5] * 16.0))
                );
                for (int axis = 0; axis < 3; axis++) {
                    double min = b[axis];
                    double max = b[axis + 3];
                    BlockSuites.call(writer, "findBits(DD)I", BlockSuites.array(d(min), d(max)), () -> new JsonPrimitive((Integer)invoke(FIND_BITS, null, min, max)));
                }
            }

            for (int first = -2; first <= 20; first++) {
                for (int second = -2; second <= 20; second++) {
                    int a = first;
                    int b = second;
                    BlockSuites.call(writer, "lcm(II)J", BlockSuites.array(new JsonPrimitive(a), new JsonPrimitive(b)), () -> new JsonPrimitive((Long)invoke(LCM, null, a, b)));
                }
            }

            // Every op for the pairs of base shapes; past those, only the ops the joins don't refuse (an op that holds
            // of two empty voxels throws, whatever the shapes).
            List<Recipe[]> pairs = shapePairs(random, base, 700);
            for (int index = 0; index < pairs.size(); index++) {
                Recipe[] pair = pairs.get(index);
                boolean every = index < 144;
                JsonArray args = BlockSuites.array(pair[0].json, pair[1].json);
                BlockSuites.call(writer, "or(" + SHAPE_TYPE + SHAPE_TYPE + ")" + SHAPE_TYPE, args, () -> shape(Shapes.or(pair[0].make.get(), pair[1].make.get())));
                BlockSuites.call(writer, "equal(" + SHAPE_TYPE + SHAPE_TYPE + ")Z", args, () -> new JsonPrimitive(Shapes.equal(pair[0].make.get(), pair[1].make.get())));
                for (int i = 0; i < OPS.size(); i++) {
                    BooleanOp op = OPS.get(i);
                    if (!every && op.apply(false, false)) {
                        continue;
                    }
                    JsonArray opArgs = BlockSuites.array(pair[0].json, pair[1].json, new JsonPrimitive(OP_NAMES.get(i)));
                    BlockSuites.call(writer, "joinIsNotEmpty(" + SHAPE_TYPE + SHAPE_TYPE + OP_TYPE + ")Z", opArgs, () -> new JsonPrimitive(Shapes.joinIsNotEmpty(pair[0].make.get(), pair[1].make.get(), op)));
                    if (i % 4 == random.nextInt(4) || GAME_OPS.contains(OP_NAMES.get(i))) {
                        BlockSuites.call(writer, "joinUnoptimized(" + SHAPE_TYPE + SHAPE_TYPE + OP_TYPE + ")" + SHAPE_TYPE, opArgs, () -> shape(Shapes.joinUnoptimized(pair[0].make.get(), pair[1].make.get(), op)));
                        BlockSuites.call(writer, "join(" + SHAPE_TYPE + SHAPE_TYPE + OP_TYPE + ")" + SHAPE_TYPE, opArgs, () -> shape(Shapes.join(pair[0].make.get(), pair[1].make.get(), op)));
                    }
                }
                for (Direction direction : Direction.values()) {
                    JsonArray directionArgs = BlockSuites.array(pair[0].json, pair[1].json, new JsonPrimitive(direction.name()));
                    BlockSuites.call(writer, "blockOccludes(" + SHAPE_TYPE + SHAPE_TYPE + DIRECTION_TYPE + ")Z", directionArgs, () -> new JsonPrimitive(Shapes.blockOccludes(pair[0].make.get(), pair[1].make.get(), direction)));
                    BlockSuites.call(writer, "mergedFaceOccludes(" + SHAPE_TYPE + SHAPE_TYPE + DIRECTION_TYPE + ")Z", directionArgs, () -> new JsonPrimitive(Shapes.mergedFaceOccludes(pair[0].make.get(), pair[1].make.get(), direction)));
                }
                BlockSuites.call(writer, "faceShapeOccludes(" + SHAPE_TYPE + SHAPE_TYPE + ")Z", args, () -> new JsonPrimitive(Shapes.faceShapeOccludes(pair[0].make.get(), pair[1].make.get())));
            }

            // A shape with itself, which joinUnoptimized and joinIsNotEmpty tell by reference.
            for (int i = 0; i < 200; i++) {
                Recipe shape = i < base.size() ? base.get(i) : randomShape(random, base, 2);
                for (int j = 0; j < OPS.size(); j++) {
                    BooleanOp op = OPS.get(j);
                    JsonArray opArgs = BlockSuites.array(shape.json, new JsonPrimitive(OP_NAMES.get(j)));
                    BlockSuites.call(writer, "joinUnoptimized(itself)", opArgs, () -> {
                        VoxelShape made = shape.make.get();
                        return shape(Shapes.joinUnoptimized(made, made, op));
                    });
                    BlockSuites.call(writer, "joinIsNotEmpty(itself)", opArgs, () -> {
                        VoxelShape made = shape.make.get();
                        return new JsonPrimitive(Shapes.joinIsNotEmpty(made, made, op));
                    });
                }
            }

            for (int i = 0; i < 300; i++) {
                List<Recipe> tail = new ArrayList<>();
                for (int count = random.nextInt(4); count > 0; count--) {
                    tail.add(randomShape(random, base, 1));
                }
                Recipe first = randomShape(random, base, 1);
                BlockSuites.call(
                    writer, "or(" + SHAPE_TYPE + "[" + SHAPE_TYPE + ")" + SHAPE_TYPE, BlockSuites.array(first.json, recipes(tail)),
                    () -> shape(Shapes.or(first.make.get(), make(tail).toArray(VoxelShape[]::new)))
                );
            }

            for (int i = 0; i < 3000; i++) {
                Recipe first = randomShape(random, base, 2);
                Recipe second = random.nextInt(4) == 0 ? first : randomShape(random, base, 2);
                Direction.Axis axis = Direction.Axis.values()[random.nextInt(3)];
                int cost = switch (random.nextInt(4)) {
                    case 0 -> 1;
                    case 1 -> 1 + random.nextInt(64);
                    case 2 -> 128 + random.nextInt(3) - 1;
                    default -> 256 + random.nextInt(3) - 1;
                };
                boolean firstOnly = random.nextBoolean();
                boolean secondOnly = random.nextBoolean();
                JsonArray args = BlockSuites.array(
                    new JsonPrimitive(cost), first.json, second.json, new JsonPrimitive(axis.name()), new JsonPrimitive(firstOnly), new JsonPrimitive(secondOnly)
                );
                BlockSuites.call(writer, "createIndexMerger(ILit/unimi/dsi/fastutil/doubles/DoubleList;Lit/unimi/dsi/fastutil/doubles/DoubleList;ZZ)", args, () -> merger(
                    (IndexMerger)invoke(CREATE_INDEX_MERGER, null, cost, coords(first, axis), coords(second, axis), firstOnly, secondOnly)
                ));
            }

            for (int i = 0; i < 4000; i++) {
                List<Recipe> shapes = new ArrayList<>();
                Vec3i around = randomBlock(random);
                for (int count = random.nextInt(4); count >= 0; count--) {
                    Recipe shape = random.nextInt(3) == 0 ? randomShape(random, base, 1) : base.get(random.nextInt(base.size()));
                    shapes.add(move(shape, around.getX() + random.nextInt(3) - 1, around.getY() + random.nextInt(3) - 1, around.getZ() + random.nextInt(3) - 1));
                }
                VoxelShape target = shapes.get(random.nextInt(shapes.size())).make.get();
                Collision collision = collision(random, target);
                JsonArray args = BlockSuites.array(
                    new JsonPrimitive(collision.axis.name()), CoreValues.encode(collision.moving), recipes(shapes), d(collision.distance)
                );
                BlockSuites.call(
                    writer, "collide(" + AXIS_TYPE + AABB_TYPE + "Ljava/lang/Iterable;D)D", args,
                    () -> d(Shapes.collide(collision.axis, collision.moving, make(shapes), collision.distance))
                );
            }

            // Every rotation of the group, of the base shapes and of a few made of them, about the block's center and about
            // other points; and the maps of rotated shapes blocks build their shapes from, in their keys' order.
            Random rotations = new Random(0x524f54L);
            List<Recipe> rotated = new ArrayList<>(base);
            for (int i = 0; i < 40; i++) {
                rotated.add(randomShape(rotations, base, 2));
            }
            Vec3[] points = {new Vec3(0.5, 0.5, 0.5), Vec3.ZERO, new Vec3(0.5, 0.25, 0.75), new Vec3(-1.0, 3.0, 0.125)};
            for (Recipe shape : rotated) {
                for (OctahedralGroup rotation : OctahedralGroup.values()) {
                    BlockSuites.call(
                        writer, "rotate(" + SHAPE_TYPE + OCTAHEDRAL_GROUP_TYPE + ")" + SHAPE_TYPE,
                        BlockSuites.array(shape.json, new JsonPrimitive(rotation.name())), () -> shape(Shapes.rotate(shape.make.get(), rotation))
                    );
                }
                for (Vec3 point : points) {
                    OctahedralGroup rotation = OctahedralGroup.values()[rotations.nextInt(OctahedralGroup.values().length)];
                    BlockSuites.call(
                        writer, "rotate(" + SHAPE_TYPE + OCTAHEDRAL_GROUP_TYPE + VEC3_TYPE + ")" + SHAPE_TYPE,
                        BlockSuites.array(shape.json, new JsonPrimitive(rotation.name()), CoreValues.encode(point)),
                        () -> shape(Shapes.rotate(shape.make.get(), rotation, point))
                    );
                }
                JsonArray args = BlockSuites.array(shape.json);
                BlockSuites.call(writer, "rotateHorizontal(" + SHAPE_TYPE + ")Ljava/util/Map;", args, () -> shapes(Shapes.rotateHorizontal(shape.make.get())));
                BlockSuites.call(writer, "rotateAll(" + SHAPE_TYPE + ")Ljava/util/Map;", args, () -> shapes(Shapes.rotateAll(shape.make.get())));
                BlockSuites.call(writer, "rotateHorizontalAxis(" + SHAPE_TYPE + ")Ljava/util/Map;", args, () -> shapes(Shapes.rotateHorizontalAxis(shape.make.get())));
                BlockSuites.call(writer, "rotateAllAxis(" + SHAPE_TYPE + ")Ljava/util/Map;", args, () -> shapes(Shapes.rotateAllAxis(shape.make.get())));
                for (OctahedralGroup initial : List.of(OctahedralGroup.INVERT_Y, OctahedralGroup.BLOCK_ROT_X_90, OctahedralGroup.SWAP_XZ)) {
                    BlockSuites.call(
                        writer, "rotateHorizontal(" + SHAPE_TYPE + OCTAHEDRAL_GROUP_TYPE + ")Ljava/util/Map;",
                        BlockSuites.array(shape.json, new JsonPrimitive(initial.name())), () -> shapes(Shapes.rotateHorizontal(shape.make.get(), initial))
                    );
                }
            }
            writer.skipped("rotateAttachFace(" + SHAPE_TYPE + ")Ljava/util/Map;", "AttachFace isn't ported: no block needing it is");
        }
    }

    // A map of shapes by enum constants, in its iteration order: [[name, shape], …].
    static JsonArray shapes(final java.util.Map<? extends Enum<?>, VoxelShape> shapes) {
        JsonArray entries = new JsonArray();
        shapes.forEach((key, shape) -> entries.add(BlockSuites.array(new JsonPrimitive(key.name()), shape(shape))));
        return entries;
    }

    /**
     * The shapes of the blocks whose classes' shapes are ported, for every state, as a state reports them with no level
     * around it (`EmptyBlockGetter`, `BlockPos.ZERO`), which is what the state's cache holds: `getShape` and
     * `getCollisionShape`. A state is its id.
     */
    static final class BlockStateShapeSuite implements Suite {
        private static final List<Class<? extends Block>> CLASSES = List.of(
            SoulSandBlock.class, SlabBlock.class, StairBlock.class, CrossCollisionBlock.class
        );

        @Override
        public String name() {
            return BlockBehaviour.BlockStateBase.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            for (BlockState state : Block.BLOCK_STATE_REGISTRY) {
                if (CLASSES.stream().noneMatch(type -> type.isInstance(state.getBlock()))) {
                    continue;
                }
                JsonArray args = BlockSuites.array(new JsonPrimitive(Block.BLOCK_STATE_REGISTRY.getId(state)));
                BlockSuites.call(
                    writer, "getShape(Lnet/minecraft/world/level/BlockGetter;Lnet/minecraft/core/BlockPos;)" + SHAPE_TYPE, args,
                    () -> shape(state.getShape(EmptyBlockGetter.INSTANCE, BlockPos.ZERO))
                );
                BlockSuites.call(
                    writer, "getCollisionShape(Lnet/minecraft/world/level/BlockGetter;Lnet/minecraft/core/BlockPos;)" + SHAPE_TYPE, args,
                    () -> shape(state.getCollisionShape(EmptyBlockGetter.INSTANCE, BlockPos.ZERO))
                );
            }
        }
    }

    /** VoxelShape's instance methods: `args[0]` is the receiver's recipe. */
    static final class VoxelShapeSuite implements Suite {
        @Override
        public String name() {
            return VoxelShape.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0x564f58454cL);
            List<Recipe> shapes = new ArrayList<>(baseShapes());
            int baseCount = shapes.size();
            for (int i = 0; i < 1200; i++) {
                shapes.add(randomShape(random, shapes.subList(0, baseCount), 3));
            }
            for (Recipe recipe : shapes) {
                JsonArray receiver = BlockSuites.array(recipe.json);
                VoxelShape shape = recipe.make.get();
                for (Direction.Axis axis : Direction.Axis.values()) {
                    JsonArray args = BlockSuites.array(recipe.json, new JsonPrimitive(axis.name()));
                    BlockSuites.call(writer, "min(" + AXIS_TYPE + ")D", args, () -> d(shape.min(axis)));
                    BlockSuites.call(writer, "max(" + AXIS_TYPE + ")D", args, () -> d(shape.max(axis)));
                    BlockSuites.call(writer, "getCoords(" + AXIS_TYPE + ")Lit/unimi/dsi/fastutil/doubles/DoubleList;", args, () -> list(shape.getCoords(axis)));
                    for (int i = 0; i < 4; i++) {
                        double b = coordAround(random, shape, AxisCycleShim.forward(axis));
                        double c = coordAround(random, shape, AxisCycleShim.backward(axis));
                        JsonArray rowArgs = BlockSuites.array(recipe.json, new JsonPrimitive(axis.name()), d(b), d(c));
                        BlockSuites.call(writer, "min(" + AXIS_TYPE + "DD)D", rowArgs, () -> d(shape.min(axis, b, c)));
                        BlockSuites.call(writer, "max(" + AXIS_TYPE + "DD)D", rowArgs, () -> d(shape.max(axis, b, c)));
                        double coord = coordAround(random, shape, axis);
                        BlockSuites.call(
                            writer, "findIndex(" + AXIS_TYPE + "D)I", BlockSuites.array(recipe.json, new JsonPrimitive(axis.name()), d(coord)),
                            () -> new JsonPrimitive((Integer)invoke(FIND_INDEX, recipe.make.get(), axis, coord))
                        );
                    }
                }
                BlockSuites.call(writer, "bounds()" + AABB_TYPE, receiver, () -> CoreValues.encode(shape.bounds()));
                BlockSuites.call(writer, "singleEncompassing()" + SHAPE_TYPE, receiver, () -> shape(recipe.make.get().singleEncompassing()));
                BlockSuites.call(writer, "isEmpty()Z", receiver, () -> new JsonPrimitive(shape.isEmpty()));
                BlockSuites.call(writer, "optimize()" + SHAPE_TYPE, receiver, () -> shape(recipe.make.get().optimize()));
                BlockSuites.call(writer, "toAabbs()Ljava/util/List;", receiver, () -> BlockSuites.list(shape.toAabbs(), CoreValues::encode));
                BlockSuites.call(writer, "forAllEdges(Lnet/minecraft/world/phys/shapes/Shapes$DoubleLineConsumer;)V", receiver, () -> {
                    JsonArray lines = new JsonArray();
                    shape.forAllEdges((x1, y1, z1, x2, y2, z2) -> lines.add(BlockSuites.array(d(x1), d(y1), d(z1), d(x2), d(y2), d(z2))));
                    return lines;
                });
                BlockSuites.call(writer, "forAllBoxes(Lnet/minecraft/world/phys/shapes/Shapes$DoubleLineConsumer;)V", receiver, () -> {
                    JsonArray lines = new JsonArray();
                    shape.forAllBoxes((x1, y1, z1, x2, y2, z2) -> lines.add(BlockSuites.array(d(x1), d(y1), d(z1), d(x2), d(y2), d(z2))));
                    return lines;
                });
                for (Direction direction : Direction.values()) {
                    BlockSuites.call(
                        writer, "getFaceShape(" + DIRECTION_TYPE + ")" + SHAPE_TYPE, BlockSuites.array(recipe.json, new JsonPrimitive(direction.name())),
                        () -> shape(recipe.make.get().getFaceShape(direction))
                    );
                }
                double dx = random.nextBoolean() ? random.nextInt(5) - 2 : random.nextDouble() * 4.0 - 2.0;
                double dy = random.nextBoolean() ? random.nextInt(7) - 64 : random.nextDouble() * 4.0 - 2.0;
                double dz = random.nextInt(4) == 0 ? -0.0 : random.nextDouble() * 4.0 - 2.0;
                BlockSuites.call(writer, "move(DDD)" + SHAPE_TYPE, BlockSuites.array(recipe.json, d(dx), d(dy), d(dz)), () -> shape(recipe.make.get().move(dx, dy, dz)));
                Vec3 delta = new Vec3(dx, dy, dz);
                BlockSuites.call(writer, "move(" + VEC3_TYPE + ")" + SHAPE_TYPE, BlockSuites.array(recipe.json, CoreValues.encode(delta)), () -> shape(recipe.make.get().move(delta)));
                Vec3i block = randomBlock(random);
                BlockSuites.call(
                    writer, "move(Lnet/minecraft/core/Vec3i;)" + SHAPE_TYPE, BlockSuites.array(recipe.json, CoreValues.encode(block)),
                    () -> shape(recipe.make.get().move(block))
                );
                for (int i = 0; i < 4; i++) {
                    Vec3 point = new Vec3(coordAround(random, shape, Direction.Axis.X), coordAround(random, shape, Direction.Axis.Y), coordAround(random, shape, Direction.Axis.Z));
                    BlockSuites.call(
                        writer, "closestPointTo(" + VEC3_TYPE + ")Ljava/util/Optional;", BlockSuites.array(recipe.json, CoreValues.encode(point)),
                        () -> shape.closestPointTo(point).map(CoreValues::encode).orElse(com.google.gson.JsonNull.INSTANCE)
                    );
                    Vec3i block3 = randomBlock(random);
                    BlockPos pos = new BlockPos(block3.getX(), block3.getY(), block3.getZ());
                    Vec3 from = Vec3.atLowerCornerOf(pos).add(random.nextDouble() * 3.0 - 1.0, random.nextDouble() * 3.0 - 1.0, random.nextDouble() * 3.0 - 1.0);
                    Vec3 target = random.nextBoolean()
                        ? Vec3.atLowerCornerOf(pos).add(point)
                        : from.add(random.nextDouble() * 2.0 - 1.0, random.nextDouble() * 2.0 - 1.0, random.nextDouble() * 2.0 - 1.0);
                    Vec3 to = random.nextInt(8) == 0 ? from.add(1.0E-4, 0.0, 0.0) : from.add(target.subtract(from).scale(1.0 + random.nextDouble()));
                    BlockSuites.call(
                        writer, "clip(" + VEC3_TYPE + VEC3_TYPE + "Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/phys/BlockHitResult;",
                        BlockSuites.array(recipe.json, CoreValues.encode(from), CoreValues.encode(to), CoreValues.encode(pos)),
                        () -> {
                            var hit = shape.clip(from, to, pos);
                            return hit == null ? com.google.gson.JsonNull.INSTANCE : CoreValues.encode(hit);
                        }
                    );
                    Collision collision = collision(random, shape);
                    BlockSuites.call(
                        writer, "collide(" + AXIS_TYPE + AABB_TYPE + "D)D",
                        BlockSuites.array(recipe.json, new JsonPrimitive(collision.axis.name()), CoreValues.encode(collision.moving), d(collision.distance)),
                        () -> d(shape.collide(collision.axis, collision.moving, collision.distance))
                    );
                }
            }
            // collide's margins: a box whose leading side is 1.0E-7 past one of the shape's grid lines, give or take a few
            // ulps, where findIndex's rounding and the distance's decide whether the line stops it. The shapes are the base
            // ones and the first random ones, at the block and moved to the walk scenario's ground, where ulps are bigger.
            for (int index = 0; index < 160; index++) {
                Recipe recipe = index % 2 == 0 ? shapes.get(index / 2) : move(shapes.get(index / 2), 1.0, -61.0, -2.0);
                VoxelShape shape = recipe.make.get();
                if (shape.isEmpty()) {
                    continue;
                }
                AABB bounds = shape.bounds();
                for (Direction.Axis axis : Direction.Axis.values()) {
                    DoubleList lines = shape.getCoords(axis);
                    for (int line = 0; line < lines.size(); line++) {
                        double value = lines.getDouble(line);
                        if (Double.isInfinite(value)) {
                            continue;
                        }
                        for (int ulps = -2; ulps <= 2; ulps++) {
                            for (boolean positive : new boolean[]{true, false}) {
                                double side = positive ? value + 1.0E-7 : value - 1.0E-7;
                                for (int step = 0; step < Math.abs(ulps); step++) {
                                    side = ulps > 0 ? Math.nextUp(side) : Math.nextDown(side);
                                }
                                double[] min = new double[3];
                                double[] max = new double[3];
                                for (Direction.Axis other : Direction.Axis.values()) {
                                    int i = other.ordinal();
                                    if (other == axis) {
                                        min[i] = positive ? side - 0.6 : side;
                                        max[i] = positive ? side : side + 0.6;
                                    } else {
                                        double center = Double.isInfinite(bounds.min(other)) ? 0.0 : (bounds.min(other) + bounds.max(other)) / 2.0;
                                        min[i] = center - 0.3;
                                        max[i] = center + 0.3;
                                    }
                                }
                                AABB moving = new AABB(min[0], min[1], min[2], max[0], max[1], max[2]);
                                double distance = positive ? 0.5 : -0.5;
                                BlockSuites.call(
                                    writer, "collide(" + AXIS_TYPE + AABB_TYPE + "D)D",
                                    BlockSuites.array(recipe.json, new JsonPrimitive(axis.name()), CoreValues.encode(moving), d(distance)),
                                    () -> d(shape.collide(axis, moving, distance))
                                );
                            }
                        }
                    }
                }
            }
            writer.skipped("toString()Ljava/lang/String;", "prints AABB.toString, which needs Java's Double.toString");
            writer.skipped("equals(Ljava/lang/Object;)Z", "Object's, by reference");
        }

        // A coordinate on one of the shape's grid lines along the axis, nudged, or anywhere around the shape.
        private static double coordAround(final Random random, final VoxelShape shape, final Direction.Axis axis) {
            DoubleList coords = shape.getCoords(axis);
            if (random.nextInt(3) != 0 && coords.size() > 0) {
                double line = coords.getDouble(random.nextInt(coords.size()));
                return (Double.isInfinite(line) ? 0.0 : line) + NUDGES[random.nextInt(NUDGES.length)];
            }
            double low = Double.isInfinite(shape.isEmpty() ? 0.0 : shape.min(axis)) || shape.isEmpty() ? -1.0 : shape.min(axis) - 1.0;
            return low + random.nextDouble() * 3.0;
        }
    }

    /** AxisCycle.FORWARD and BACKWARD, which min(Axis, double, double) reads its row's axes with. */
    static final class AxisCycleShim {
        static Direction.Axis forward(final Direction.Axis axis) {
            return Direction.Axis.values()[(axis.ordinal() + 1) % 3];
        }

        static Direction.Axis backward(final Direction.Axis axis) {
            return Direction.Axis.values()[(axis.ordinal() + 2) % 3];
        }
    }

    /** DiscreteVoxelShape's methods on a shape's voxels: `args[0]` is the shape's recipe. */
    static final class DiscreteVoxelShapeSuite implements Suite {
        @Override
        public String name() {
            return DiscreteVoxelShape.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0x4449534352L);
            List<Recipe> shapes = new ArrayList<>(baseShapes());
            int baseCount = shapes.size();
            for (int i = 0; i < 800; i++) {
                shapes.add(randomShape(random, shapes.subList(0, baseCount), 3));
            }
            for (Recipe recipe : shapes) {
                DiscreteVoxelShape shape = (DiscreteVoxelShape)get(SHAPE, recipe.make.get());
                JsonArray receiver = BlockSuites.array(recipe.json);
                BlockSuites.call(writer, "isEmpty()Z", receiver, () -> new JsonPrimitive(shape.isEmpty()));
                BlockSuites.call(writer, "forAllFaces(Lnet/minecraft/world/phys/shapes/DiscreteVoxelShape$IntFaceConsumer;)V", receiver, () -> {
                    JsonArray faces = new JsonArray();
                    shape.forAllFaces((direction, x, y, z) -> faces.add(BlockSuites.array(
                        new JsonPrimitive(direction.name()), new JsonPrimitive(x), new JsonPrimitive(y), new JsonPrimitive(z)
                    )));
                    return faces;
                });
                for (boolean merge : new boolean[]{false, true}) {
                    JsonArray args = BlockSuites.array(recipe.json, new JsonPrimitive(merge));
                    BlockSuites.call(writer, "forAllEdges(Lnet/minecraft/world/phys/shapes/DiscreteVoxelShape$IntLineConsumer;Z)V", args, () -> {
                        JsonArray lines = new JsonArray();
                        shape.forAllEdges((x1, y1, z1, x2, y2, z2) -> lines.add(ints(x1, y1, z1, x2, y2, z2)), merge);
                        return lines;
                    });
                    BlockSuites.call(writer, "forAllBoxes(Lnet/minecraft/world/phys/shapes/DiscreteVoxelShape$IntLineConsumer;Z)V", args, () -> {
                        JsonArray lines = new JsonArray();
                        shape.forAllBoxes((x1, y1, z1, x2, y2, z2) -> lines.add(ints(x1, y1, z1, x2, y2, z2)), merge);
                        return lines;
                    });
                }
                for (Direction.Axis axis : Direction.Axis.values()) {
                    for (int i = 0; i < 4; i++) {
                        int b = random.nextInt(shape.getSize(AxisCycleShim.forward(axis)) + 3) - 1;
                        int c = random.nextInt(shape.getSize(AxisCycleShim.backward(axis)) + 3) - 1;
                        JsonArray args = BlockSuites.array(recipe.json, new JsonPrimitive(axis.name()), new JsonPrimitive(b), new JsonPrimitive(c));
                        BlockSuites.call(writer, "firstFull(" + AXIS_TYPE + "II)I", args, () -> new JsonPrimitive(shape.firstFull(axis, b, c)));
                        BlockSuites.call(writer, "lastFull(" + AXIS_TYPE + "II)I", args, () -> new JsonPrimitive(shape.lastFull(axis, b, c)));
                    }
                }
                for (int i = 0; i < 6; i++) {
                    int x = random.nextInt(shape.getXSize() + 3) - 1;
                    int y = random.nextInt(shape.getYSize() + 3) - 1;
                    int z = random.nextInt(shape.getZSize() + 3) - 1;
                    JsonArray args = BlockSuites.array(recipe.json, new JsonPrimitive(x), new JsonPrimitive(y), new JsonPrimitive(z));
                    BlockSuites.call(writer, "isFullWide(III)Z", args, () -> new JsonPrimitive(shape.isFullWide(x, y, z)));
                    if (shape instanceof BitSetDiscreteVoxelShape bitSet) {
                        BlockSuites.call(writer, "BitSetDiscreteVoxelShape.isInterior(III)Z", args, () -> new JsonPrimitive(bitSet.isInterior(x, y, z)));
                    }
                }
            }
            writer.skipped("rotate(Lcom/mojang/math/OctahedralGroup;)Lnet/minecraft/world/phys/shapes/DiscreteVoxelShape;", "no generator for com.mojang.math.OctahedralGroup");
        }

        private static JsonArray ints(final int... values) {
            JsonArray array = new JsonArray();
            for (int value : values) {
                array.add(value);
            }
            return array;
        }
    }

    /**
     * The pure parts of an entity's movement: collideWithShapes (a box moving through shapes, axis by axis),
     * collectCandidateStepUpHeights and getInputVector (an input turned by the look and scaled). They're private or
     * protected statics, called by reflection.
     */
    static final class EntitySuite implements Suite {
        @Override
        public String name() {
            return Entity.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0x434f4c4cL);
            List<Recipe> base = baseShapes();
            // The walk scenario's ground: a row of full blocks under the player, and blocks' shapes around it.
            for (int i = 0; i < 6000; i++) {
                Vec3 feet = new Vec3(random.nextDouble() * 4.0 - 2.0, random.nextInt(3) == 0 ? -60.0 : -61.0 + random.nextDouble() * 3.0, random.nextDouble() * 4.0 - 2.0);
                float[] size = ENTITY_SIZES[random.nextInt(3)];
                AABB box = CoreValues.entityBox(size[0], size[1], feet);
                List<Recipe> shapes = new ArrayList<>();
                BlockPos below = BlockPos.containing(feet).below();
                for (int count = random.nextInt(6); count >= 0; count--) {
                    BlockPos pos = below.offset(random.nextInt(3) - 1, random.nextInt(4) - 1, random.nextInt(3) - 1);
                    Recipe shape = random.nextInt(3) == 0 ? constant("BLOCK", Shapes::block) : base.get(random.nextInt(base.size()));
                    shapes.add(move(shape, pos.getX(), pos.getY(), pos.getZ()));
                }
                if (random.nextInt(10) == 0) {
                    shapes.clear();
                }
                Vec3 movement = switch (random.nextInt(5)) {
                    case 0 -> new Vec3(random.nextDouble() * 0.6 - 0.3, -0.0784000015258789, random.nextDouble() * 0.6 - 0.3);
                    case 1 -> new Vec3(random.nextDouble() * 0.6 - 0.3, random.nextDouble() * 1.2 - 0.8, random.nextDouble() * 0.6 - 0.3);
                    case 2 -> new Vec3(random.nextDouble() * 0.6 - 0.3, 0.6F, random.nextDouble() * 0.6 - 0.3);
                    case 3 -> new Vec3(random.nextBoolean() ? 0.0 : random.nextDouble() - 0.5, random.nextBoolean() ? -0.0 : random.nextDouble() * 8.0 - 4.0, random.nextBoolean() ? 0.0 : random.nextDouble() - 0.5);
                    default -> new Vec3(NUDGES[random.nextInt(NUDGES.length)], NUDGES[random.nextInt(NUDGES.length)], NUDGES[random.nextInt(NUDGES.length)]);
                };
                JsonArray args = BlockSuites.array(CoreValues.encode(movement), CoreValues.encode(box), recipes(shapes));
                BlockSuites.call(
                    writer, "collideWithShapes(" + VEC3_TYPE + AABB_TYPE + "Ljava/util/List;)" + VEC3_TYPE, args,
                    () -> CoreValues.encode((Vec3)invoke(COLLIDE_WITH_SHAPES, null, movement, box, make(shapes)))
                );
                float maxStep = switch (random.nextInt(4)) {
                    case 0, 1 -> 0.6F;
                    case 2 -> 1.0F;
                    default -> random.nextFloat() * 1.5F;
                };
                float skip = switch (random.nextInt(4)) {
                    case 0 -> 0.0F;
                    case 1 -> -0.0F;
                    case 2 -> (float)(-0.0784000015258789);
                    default -> (float)(Math.floor(feet.y) + random.nextInt(3) * 0.5 - feet.y);
                };
                AABB grounded = random.nextBoolean() ? box : box.move(0.0, Math.floor(feet.y) - feet.y, 0.0);
                JsonArray stepArgs = BlockSuites.array(CoreValues.encode(grounded), recipes(shapes), f(maxStep), f(skip));
                BlockSuites.call(
                    writer, "collectCandidateStepUpHeights(" + AABB_TYPE + "Ljava/util/List;FF)[F", stepArgs,
                    () -> floats((float[])invoke(COLLECT_CANDIDATE_STEP_UP_HEIGHTS, null, grounded, make(shapes), maxStep, skip))
                );
            }

            float[] impulses = {0.0F, 0.98F, -0.98F, 0.294F, -0.294F, 0.6929646F, -0.6929646F, 0.20788938F, 1.0F, 0.98F * 0.3F};
            float[] speeds = {0.1F, 0.13F, 0.02F, 0.025999999F, 0.1F * (0.21600002F / (0.6F * 0.6F * 0.6F)), 0.13F * (0.21600002F / (0.6F * 0.6F * 0.6F)), 0.05F, 0.0F};
            float[] yRots = {0.0F, 90.0F, -90.0F, 180.0F, 45.0F, 30.0F, -135.5F, 359.9F, 720.0F, -1.0E-5F};
            for (int i = 0; i < 4000; i++) {
                double x = i < 500 ? impulses[i % impulses.length] : random.nextInt(3) == 0 ? impulses[random.nextInt(impulses.length)] : random.nextDouble() * 2.0 - 1.0;
                double y = random.nextInt(4) == 0 ? random.nextDouble() * 2.0 - 1.0 : 0.0;
                double z = i < 500 ? impulses[(i / impulses.length) % impulses.length] : random.nextInt(3) == 0 ? impulses[random.nextInt(impulses.length)] : random.nextDouble() * 2.0 - 1.0;
                if (random.nextInt(50) == 0) {
                    x = 3.0E-4;
                    z = 1.0E-4 * random.nextDouble();
                }
                float speed = random.nextInt(3) == 0 ? random.nextFloat() : speeds[random.nextInt(speeds.length)];
                float yRot = random.nextInt(3) == 0 ? (random.nextFloat() * 2.0F - 1.0F) * 360.0F : yRots[random.nextInt(yRots.length)];
                Vec3 input = new Vec3(x, y, z);
                BlockSuites.call(
                    writer, "getInputVector(" + VEC3_TYPE + "FF)" + VEC3_TYPE, BlockSuites.array(CoreValues.encode(input), f(speed), f(yRot)),
                    () -> CoreValues.encode((Vec3)invoke(GET_INPUT_VECTOR, null, input, speed, yRot))
                );
            }
        }
    }

    /** LivingEntity's computeModifiedFriction, a private static called by reflection: blocks' frictions and modifiers. */
    static final class LivingEntitySuite implements Suite {
        @Override
        public String name() {
            return LivingEntity.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0x46524943L);
            float[] frictions = {0.6F, 0.8F, 0.98F, 0.989F, 1.0F, 0.0F, 0.5F};
            float[] modifiers = {1.0F, 0.0F, 0.5F, 2.0F, 2048.0F, 1.5F, 0.1F};
            for (float friction : frictions) {
                for (float modifier : modifiers) {
                    BlockSuites.call(
                        writer, "computeModifiedFriction(FF)F", BlockSuites.array(f(friction), f(modifier)),
                        () -> f((Float)invoke(COMPUTE_MODIFIED_FRICTION, null, friction, modifier))
                    );
                }
            }
            for (int i = 0; i < 500; i++) {
                float friction = random.nextFloat() * 1.2F;
                float modifier = random.nextFloat() * 4.0F;
                BlockSuites.call(
                    writer, "computeModifiedFriction(FF)F", BlockSuites.array(f(friction), f(modifier)),
                    () -> f((Float)invoke(COMPUTE_MODIFIED_FRICTION, null, friction, modifier))
                );
            }
        }
    }
}
