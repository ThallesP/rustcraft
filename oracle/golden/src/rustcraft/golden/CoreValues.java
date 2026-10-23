package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import java.lang.reflect.Field;
import java.lang.reflect.ParameterizedType;
import java.util.ArrayList;
import java.util.EnumSet;
import java.util.List;
import java.util.Random;
import java.util.stream.Stream;
import com.mojang.math.OctahedralGroup;
import com.mojang.math.Quadrant;
import com.mojang.math.SymmetricGroup3;
import net.minecraft.core.AxisCycle;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.FrontAndTop;
import net.minecraft.core.Position;
import net.minecraft.core.Vec3i;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.util.ByIdMap;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.block.Rotation;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.BlockHitResult;
import net.minecraft.world.phys.HitResult;
import net.minecraft.world.phys.Vec2;
import net.minecraft.world.phys.Vec3;
import org.joml.Vector3f;

/**
 * Arguments and receivers of the game's core value types. A Vec3i is {@code [x, y, z]} (ints), and so are a BlockPos and
 * a MutableBlockPos, a Vec3 {@code [x, y, z]} (doubles), a Vec2 {@code [x, y]} (floats), an enum constant its name
 * ({@code "NORTH"}), and an {@code EnumSet<Direction.Axis>} the names of its axes in order. An AABB is
 * {@code [minX, minY, minZ, maxX, maxY, maxZ]} (doubles), a list of them an array, an AABB.Builder its fields
 * {@code [minX, minY, minZ, maxX, maxY, maxZ, defined]} (floats and a boolean), and a BlockHitResult its fields
 * {@code [location, direction, blockPos, miss, inside, worldBorderHit]}, HitResult's first. An Identifier is
 * {@code [namespace, path]}.
 */
final class CoreValues {
    static final Values.Generator DIRECTION = new Constants<>(Direction.class);
    static final Values.Generator AXIS = new Constants<>(Direction.Axis.class);
    static final Values.Generator AXIS_DIRECTION = new Constants<>(Direction.AxisDirection.class);
    static final Values.Generator PLANE = new Constants<>(Direction.Plane.class);
    static final Values.Generator AXIS_SET = new AxisSets();
    static final Values.Generator AXIS_CYCLE = new Constants<>(AxisCycle.class);
    static final Values.Generator ROTATION = new Constants<>(Rotation.class);
    static final Values.Generator TRAVERSAL_NODE_STATUS = new Constants<>(BlockPos.TraversalNodeStatus.class);
    static final Values.Generator HIT_RESULT_TYPE = new Constants<>(HitResult.Type.class);
    static final Values.Generator OUT_OF_BOUNDS_STRATEGY = new Constants<>(ByIdMap.OutOfBoundsStrategy.class);
    static final Values.Generator SYMMETRIC_GROUP3 = new Constants<>(SymmetricGroup3.class);
    static final Values.Generator OCTAHEDRAL_GROUP = new Constants<>(OctahedralGroup.class);
    static final Values.Generator QUADRANT = new Constants<>(Quadrant.class);
    static final Values.Generator FRONT_AND_TOP = new Constants<>(FrontAndTop.class);

    // Width and height of entities: the player standing, crouching and swimming, an item, an arrow, a minecart.
    private static final float[][] ENTITY_SIZES = {
        {0.6F, 1.8F}, {0.6F, 1.5F}, {0.6F, 0.6F}, {0.25F, 0.25F}, {0.5F, 0.5F}, {0.98F, 0.7F}
    };
    private static final List<Field> BUILDER_FIELDS = builderFields();

    private static final List<Vec3i> VEC3I_EDGES = List.of(
        Vec3i.ZERO, new Vec3i(1, 0, 0), new Vec3i(0, 1, 0), new Vec3i(0, 0, 1), new Vec3i(-1, 0, 0), new Vec3i(0, -1, 0),
        new Vec3i(0, 0, -1), new Vec3i(1, 2, 3), new Vec3i(0, -61, 0), new Vec3i(16, 16, 16), new Vec3i(-30000000, 320, 30000000),
        new Vec3i(33554431, 2047, -33554432), new Vec3i(Integer.MAX_VALUE, Integer.MIN_VALUE, Integer.MAX_VALUE),
        new Vec3i(Integer.MIN_VALUE, Integer.MIN_VALUE, Integer.MIN_VALUE)
    );

    // BlockPos.asLong packs x and z into 26 bits and y into 12, so a block's edges also include the largest and smallest
    // coordinates that fit, and one past them, which wrap.
    private static final List<Vec3i> BLOCK_POS_EDGES = Stream.concat(
        VEC3I_EDGES.stream(),
        Stream.of(
            new Vec3i(33554431, 2047, 33554431), new Vec3i(-33554432, -2048, -33554432), new Vec3i(33554432, 2048, -33554433),
            new Vec3i(-33554433, -2049, 33554432)
        )
    ).toList();

    // Each copy is a new object, so equals never takes its `this == o` shortcut, which only the receiver itself would.
    static final Values.Generator VEC3I = new JomlValues.Mutable<>(
        VEC3I_EDGES,
        CoreValues::randomVec3i,
        vec -> new Vec3i(vec.getX(), vec.getY(), vec.getZ()),
        vec -> ints(vec.getX(), vec.getY(), vec.getZ())
    );

    // A BlockPos is [x, y, z] like a Vec3i, and so is a MutableBlockPos, whose methods get a fresh one to change.
    // Chunks around the walk scenario's spawn, at ChunkPyramid's limit and past it, INVALID_CHUNK_POS's, and any ints.
    static final Values.Generator CHUNK_POS = new Values.Simple(
        List.<Object>of(
            ChunkPos.ZERO, new ChunkPos(-1, -1), new ChunkPos(1, -2), new ChunkPos(-3, 3), ChunkPos.unpack(ChunkPos.INVALID_CHUNK_POS),
            new ChunkPos(2097061, -2097061), new ChunkPos(2097062, 0), new ChunkPos(Integer.MAX_VALUE, Integer.MIN_VALUE),
            new ChunkPos(Integer.MIN_VALUE, -1)
        ),
        random -> random.nextInt(4) == 0
            ? new ChunkPos(random.nextInt(), random.nextInt())
            : new ChunkPos(random.nextInt(65) - 32, random.nextInt(65) - 32),
        value -> Values.encode(value, ChunkPos.class)
    );

    static final Values.Generator BLOCK_POS = new JomlValues.Mutable<>(
        BLOCK_POS_EDGES.stream().map(vec -> new BlockPos(vec.getX(), vec.getY(), vec.getZ())).toList(),
        CoreValues::randomBlockPos,
        pos -> new BlockPos(pos.getX(), pos.getY(), pos.getZ()),
        pos -> ints(pos.getX(), pos.getY(), pos.getZ())
    );
    static final Values.Generator MUTABLE_BLOCK_POS = new JomlValues.Mutable<>(
        BLOCK_POS_EDGES.stream().map(vec -> new BlockPos.MutableBlockPos(vec.getX(), vec.getY(), vec.getZ())).toList(),
        random -> randomBlockPos(random).mutable(),
        pos -> new BlockPos.MutableBlockPos(pos.getX(), pos.getY(), pos.getZ()),
        pos -> ints(pos.getX(), pos.getY(), pos.getZ())
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

    // Whole blocks and blocks' shapes (Block.box's sixteenths: a slab, a carpet, a fence post), the player's box at the
    // walk scenario's spawn and the block it stands on, the empty box an item model without extents gets, and boxes that
    // are flat, endless, as wide as doubles go, a subnormal across, or NaN (of both signs). AABB is immutable, but each
    // call gets its own copy, so equals never takes its `this == o` shortcut.
    static final Values.Generator BOX = new JomlValues.Mutable<>(
        List.of(
            new AABB(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), new AABB(new BlockPos(0, -61, 0)), entityBox(0.6F, 1.8F, new Vec3(0.5, -60.0, 0.5)),
            new AABB(0.0, 0.0, 0.0, 1.0, 0.5, 1.0), new AABB(0.0, 0.0, 0.0, 1.0, 0.0625, 1.0), new AABB(0.375, 0.0, 0.375, 0.625, 1.0, 0.625),
            new AABB(0.0, 0.0, 0.0, 0.0, 0.0, 0.0), new AABB(-0.0, -0.0, -0.0, 0.0, 0.0, 0.0), new AABB(0.0, -60.0, 0.0, 1.0, -60.0, 1.0),
            new AABB(2.9999999E7, 319.0, -3.0E7, 3.0E7, 320.0, -2.9999999E7),
            new AABB(Double.NEGATIVE_INFINITY, Double.NEGATIVE_INFINITY, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY, Double.POSITIVE_INFINITY, Double.POSITIVE_INFINITY),
            new AABB(-Double.MAX_VALUE, 0.0, 0.0, Double.MAX_VALUE, 1.0, 1.0), new AABB(0.0, 0.0, 0.0, Double.MIN_VALUE, Double.MIN_VALUE, Double.MIN_VALUE),
            new AABB(Double.NaN, 0.0, 0.0, 1.0, 1.0, 1.0), new AABB(0.0, 0.0, Double.longBitsToDouble(0xfff8000000000000L), 1.0, 1.0, 1.0)
        ),
        CoreValues::randomBox,
        CoreValues::copyBox,
        box -> doubles(box.minX, box.minY, box.minZ, box.maxX, box.maxY, box.maxZ)
    );

    // A section's box as the octree and the frustum make it, a single block, the world's height at the spawn, boxes at the
    // ints' ends (where maxX + 1 wraps, and a side's length wraps to 0 or below), and any corners, which the constructor
    // puts in order. BoundingBox's move and encapsulate change it, so each call gets its own copy.
    static final Values.Generator BOUNDING_BOX = new JomlValues.Mutable<>(
        List.of(
            new BoundingBox(0, -64, 0, 15, -49, 15), new BoundingBox(BlockPos.ZERO), new BoundingBox(-1, -64, -1, 1, 319, 1),
            new BoundingBox(-16, 0, -16, -1, 15, -1), new BoundingBox(0, 0, 0, Integer.MAX_VALUE, 0, 0),
            new BoundingBox(Integer.MIN_VALUE, 0, 0, Integer.MAX_VALUE, 1, 1), new BoundingBox(Integer.MIN_VALUE, Integer.MIN_VALUE, Integer.MIN_VALUE, Integer.MIN_VALUE, Integer.MIN_VALUE, Integer.MIN_VALUE),
            new BoundingBox(Integer.MAX_VALUE, Integer.MAX_VALUE, Integer.MAX_VALUE, Integer.MAX_VALUE, Integer.MAX_VALUE, Integer.MAX_VALUE)
        ),
        CoreValues::randomBoundingBox,
        box -> new BoundingBox(box.minX(), box.minY(), box.minZ(), box.maxX(), box.maxY(), box.maxZ()),
        box -> ints(box.minX(), box.minY(), box.minZ(), box.maxX(), box.maxY(), box.maxZ())
    );

    // A block's shape as VoxelShape.toAabbs gives it (none, a whole block, a slab, stairs, a fence with two arms), relative
    // to the block, or any boxes. A List<AABB> and an Iterable<AABB> are both lists.
    static final Values.Generator BOX_LIST = new JomlValues.Mutable<List<AABB>>(
        List.of(
            List.of(), List.of(new AABB(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)), List.of(new AABB(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)),
            List.of(new AABB(0.0, 0.0, 0.0, 1.0, 0.5, 1.0), new AABB(0.0, 0.5, 0.5, 1.0, 1.0, 1.0)),
            List.of(new AABB(0.375, 0.0, 0.375, 0.625, 1.0, 0.625), new AABB(0.4375, 0.375, 0.0, 0.5625, 0.9375, 0.375), new AABB(0.625, 0.375, 0.4375, 1.0, 0.9375, 0.5625))
        ),
        CoreValues::randomBoxes,
        boxes -> boxes.stream().map(CoreValues::copyBox).toList(),
        boxes -> {
            JsonArray array = new JsonArray();
            boxes.forEach(box -> array.add(BOX.encode(box)));
            return array;
        }
    );

    // Empty (an item model without extents), one point, and the corners of a camera's near plane, the way
    // Frustum.getNearPlaneBounds and ItemStackRenderState.getModelBoundingBox use it, and any points. Each call gets
    // its own builder with the same fields.
    static final Values.Generator BOX_BUILDER = new JomlValues.Mutable<>(
        List.of(
            builder(), builder(new Vector3f()), builder(new Vector3f(-0.0F, -0.0F, -0.0F)), builder(new Vector3f(0.5F, -60.0F, 0.5F), new Vector3f(-0.5F, 1.0F, 2.0F)),
            builder(new Vector3f(Float.NaN, 0.0F, 1.0F)), builder(new Vector3f(1.0F, 2.0F, 3.0F), new Vector3f(Float.NaN, 0.0F, 1.0F)),
            builder(new Vector3f(Float.NEGATIVE_INFINITY, 0.0F, 0.0F), new Vector3f(Float.POSITIVE_INFINITY, 0.0F, 0.0F))
        ),
        CoreValues::randomBuilder,
        CoreValues::copyBuilder,
        CoreValues::encodeBuilder
    );

    // Picks of blocks, misses, hits from inside a block and on the world border, like the ones the game makes and
    // BlockHitResult's STREAM_CODEC reads.
    static final Values.Generator BLOCK_HIT_RESULT = new JomlValues.Mutable<>(
        List.of(
            new BlockHitResult(new Vec3(0.5, -60.0, 0.5), Direction.UP, new BlockPos(0, -61, 0), false),
            BlockHitResult.miss(new Vec3(0.5, -55.0, 4.5), Direction.SOUTH, new BlockPos(0, -55, 4)),
            new BlockHitResult(new Vec3(1.0, -59.5, 0.25), Direction.WEST, new BlockPos(1, -60, 0), true),
            new BlockHitResult(new Vec3(3.0E7, 64.0, 0.5), Direction.EAST, new BlockPos(29999999, 64, 0), false, true),
            BlockHitResult.miss(new Vec3(Double.NaN, 0.0, 0.0), Direction.DOWN, BlockPos.ZERO).hitBorder()
        ),
        CoreValues::randomBlockHitResult,
        CoreValues::copyBlockHitResult,
        hit -> {
            JsonArray array = new JsonArray();
            array.add(VEC3.encode(hit.getLocation()));
            array.add(hit.getDirection().name());
            array.add(BLOCK_POS.encode(hit.getBlockPos()));
            array.add(hit.getType() == HitResult.Type.MISS);
            array.add(hit.isInside());
            array.add(hit.isWorldBorderHit());
            return array;
        }
    );

    // The game's identifiers: blocks, models and textures, a tag's, the scenario's marker and the realms namespace, and
    // ones with every character a part allows, empty parts, and ".." as a path. Each call gets its own copy.
    static final Values.Generator IDENTIFIER = new JomlValues.Mutable<>(
        List.of(
            Identifier.withDefaultNamespace("stone"), Identifier.withDefaultNamespace("block/stone"),
            Identifier.withDefaultNamespace("textures/atlas/blocks.png"), Identifier.fromNamespaceAndPath("rustcraft", "scenario_start"),
            Identifier.fromNamespaceAndPath("realms", "x"), Identifier.fromNamespaceAndPath("c", "ingots/iron"),
            Identifier.fromNamespaceAndPath("a.b-c_d0", "e/f.g-h_i9"), Identifier.fromNamespaceAndPath("", ""),
            Identifier.withDefaultNamespace(""), Identifier.fromNamespaceAndPath(".", "../..")
        ),
        CoreValues::randomIdentifier,
        id -> Identifier.fromNamespaceAndPath(id.getNamespace(), id.getPath()),
        id -> {
            JsonArray array = new JsonArray();
            array.add(id.getNamespace());
            array.add(id.getPath());
            return array;
        }
    );

    // The registries the server sends in the configuration protocol (and the dimensions', which it doesn't), the root
    // registry's own key, keys of their elements, and keys with the odd identifiers. Keys are interned, so each call gets
    // the same one, like the game's. A key is [registry, identifier].
    static final Values.Generator RESOURCE_KEY = new Values.Simple(
        List.of(
            Registries.BIOME, Registries.DIMENSION_TYPE, Registries.CHAT_TYPE, Registries.DAMAGE_TYPE, Registries.ENCHANTMENT,
            Registries.WOLF_VARIANT, Registries.PAINTING_VARIANT, Registries.DIALOG, Registries.DIMENSION,
            ResourceKey.createRegistryKey(Registries.ROOT_REGISTRY_NAME),
            ResourceKey.create(Registries.DIMENSION, Identifier.withDefaultNamespace("overworld")),
            ResourceKey.create(Registries.BIOME, Identifier.withDefaultNamespace("plains")),
            ResourceKey.create(Registries.DIMENSION_TYPE, Identifier.withDefaultNamespace("overworld")),
            ResourceKey.create(Registries.BIOME, Identifier.fromNamespaceAndPath("", "")),
            ResourceKey.createRegistryKey(Identifier.fromNamespaceAndPath(".", "../..")),
            ResourceKey.create(
                ResourceKey.createRegistryKey(Identifier.fromNamespaceAndPath("rustcraft", "scenario_start")), Identifier.withDefaultNamespace("stone")
            )
        ),
        CoreValues::randomResourceKey,
        key -> {
            JsonArray array = new JsonArray();
            array.add(IDENTIFIER.encode(((ResourceKey<?>)key).registry()));
            array.add(IDENTIFIER.encode(((ResourceKey<?>)key).identifier()));
            return array;
        }
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
        if (type == AxisCycle.class) {
            return AXIS_CYCLE;
        }
        if (type == Rotation.class) {
            return ROTATION;
        }
        if (type == SymmetricGroup3.class) {
            return SYMMETRIC_GROUP3;
        }
        if (type == OctahedralGroup.class) {
            return OCTAHEDRAL_GROUP;
        }
        if (type == Quadrant.class) {
            return QUADRANT;
        }
        if (type == FrontAndTop.class) {
            return FRONT_AND_TOP;
        }
        if (type == BlockPos.TraversalNodeStatus.class) {
            return TRAVERSAL_NODE_STATUS;
        }
        if (type == Vec3i.class) {
            return VEC3I;
        }
        if (type == BlockPos.class) {
            return BLOCK_POS;
        }
        if (type == ChunkPos.class) {
            return CHUNK_POS;
        }
        if (type == BlockPos.MutableBlockPos.class) {
            return MUTABLE_BLOCK_POS;
        }
        // Vec3 is the Position the game passes, entities' positions included.
        if (type == Vec3.class || type == Position.class) {
            return VEC3;
        }
        if (type == Vec2.class) {
            return VEC2;
        }
        if (type == AABB.class) {
            return BOX;
        }
        if (type == AABB.Builder.class) {
            return BOX_BUILDER;
        }
        if (type == BoundingBox.class) {
            return BOUNDING_BOX;
        }
        if (type == HitResult.Type.class) {
            return HIT_RESULT_TYPE;
        }
        if (type == ByIdMap.OutOfBoundsStrategy.class) {
            return OUT_OF_BOUNDS_STRATEGY;
        }
        if (type == Identifier.class) {
            return IDENTIFIER;
        }
        if (type == ResourceKey.class) {
            return RESOURCE_KEY;
        }
        // Entities aren't ported, so a BlockHitResult is the only HitResult.
        if (type == BlockHitResult.class || type == HitResult.class) {
            return BLOCK_HIT_RESULT;
        }
        return null;
    }

    /** The only parameterized arguments with a generator of their own: AABB.collidedAlongVector's and AABB.clip's boxes. */
    static Values.Generator generator(final ParameterizedType type) {
        boolean boxes = type.getActualTypeArguments().length == 1 && type.getActualTypeArguments()[0] == AABB.class;
        return boxes && (type.getRawType() == List.class || type.getRawType() == Iterable.class) ? BOX_LIST : null;
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
            case AABB box -> BOX.encode(box);
            case AABB.Builder builder -> BOX_BUILDER.encode(builder);
            case BlockHitResult hit -> BLOCK_HIT_RESULT.encode(hit);
            case Identifier id -> IDENTIFIER.encode(id);
            case ResourceKey<?> key -> RESOURCE_KEY.encode(key);
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

    static BlockPos randomBlockPos(final Random random) {
        Vec3i vec = randomVec3i(random);
        return new BlockPos(vec.getX(), vec.getY(), vec.getZ());
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

    // Entities around the walk scenario's spawn, standing on a block or anywhere, blocks' shapes at blocks, whole blocks,
    // boxes between any two Vec3s, and any doubles.
    static AABB randomBox(final Random random) {
        return switch (random.nextInt(5)) {
            case 0 -> {
                float[] size = ENTITY_SIZES[random.nextInt(ENTITY_SIZES.length)];
                double y = random.nextDouble() * 384.0 - 64.0;
                Vec3 feet = new Vec3((random.nextDouble() * 2.0 - 1.0) * 80.0, random.nextBoolean() ? Math.floor(y) : y, (random.nextDouble() * 2.0 - 1.0) * 80.0);
                yield entityBox(size[0], size[1], feet);
            }
            case 1 -> blockShape(random).move(randomBlockPos(random));
            case 2 -> new AABB(randomBlockPos(random));
            case 3 -> new AABB(randomVec3(random), randomVec3(random));
            default -> new AABB(
                (Double)Values.DOUBLES.random(random), (Double)Values.DOUBLES.random(random), (Double)Values.DOUBLES.random(random),
                (Double)Values.DOUBLES.random(random), (Double)Values.DOUBLES.random(random), (Double)Values.DOUBLES.random(random)
            );
        };
    }

    /** EntityDimensions.makeBoundingBox: the half width and the height are floats, added to the feet's doubles. */
    static AABB entityBox(final float width, final float height, final Vec3 feet) {
        float w = width / 2.0F;
        return new AABB(feet.x - w, feet.y, feet.z - w, feet.x + w, feet.y + height, feet.z + w);
    }

    static AABB randomEntityBox(final Random random, final Vec3 feet) {
        float[] size = ENTITY_SIZES[random.nextInt(ENTITY_SIZES.length)];
        return entityBox(size[0], size[1], feet);
    }

    /** One of a block's shapes relative to the block, in sixteenths like Block.box's, which doubles hold exactly. */
    static AABB blockShape(final Random random) {
        int[] from = new int[3];
        int[] to = new int[3];
        for (int axis = 0; axis < 3; axis++) {
            from[axis] = random.nextInt(16);
            to[axis] = from[axis] + 1 + random.nextInt(16 - from[axis]);
        }
        return new AABB(from[0] / 16.0, from[1] / 16.0, from[2] / 16.0, to[0] / 16.0, to[1] / 16.0, to[2] / 16.0);
    }

    private static List<AABB> randomBoxes(final Random random) {
        List<AABB> boxes = new ArrayList<>();
        boolean shape = random.nextBoolean();
        for (int count = random.nextInt(5); count > 0; count--) {
            boxes.add(shape ? blockShape(random) : randomBox(random));
        }
        return List.copyOf(boxes);
    }

    private static BoundingBox randomBoundingBox(final Random random) {
        if (random.nextInt(4) == 0) {
            return new BoundingBox(random.nextInt(), random.nextInt(), random.nextInt(), random.nextInt(), random.nextInt(), random.nextInt());
        }
        BlockPos corner = randomBlockPos(random);
        return new BoundingBox(
            corner.getX(), corner.getY(), corner.getZ(),
            corner.getX() + random.nextInt(9) - 4, corner.getY() + random.nextInt(9) - 4, corner.getZ() + random.nextInt(9) - 4
        );
    }

    private static AABB copyBox(final AABB box) {
        return new AABB(box.minX, box.minY, box.minZ, box.maxX, box.maxY, box.maxZ);
    }

    static AABB.Builder builder(final Vector3f... points) {
        AABB.Builder builder = new AABB.Builder();
        for (Vector3f point : points) {
            builder.include(point);
        }
        return builder;
    }

    private static AABB.Builder randomBuilder(final Random random) {
        Vector3f[] points = new Vector3f[random.nextInt(9)];
        for (int i = 0; i < points.length; i++) {
            points[i] = random.nextBoolean() ? JomlValues.randomVector3f(random) : (Vector3f)JomlValues.VECTOR3F.random(random);
        }
        return builder(points);
    }

    private static AABB.Builder copyBuilder(final AABB.Builder builder) {
        AABB.Builder copy = new AABB.Builder();
        try {
            for (Field field : BUILDER_FIELDS) {
                field.set(copy, field.get(builder));
            }
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
        return copy;
    }

    private static JsonArray encodeBuilder(final AABB.Builder builder) {
        JsonArray array = new JsonArray();
        try {
            for (Field field : BUILDER_FIELDS) {
                array.add(field.getType() == boolean.class ? new JsonPrimitive(field.getBoolean(builder)) : new JsonPrimitive(Values.floatBits(field.getFloat(builder))));
            }
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
        return array;
    }

    // Its private fields in declaration order.
    private static List<Field> builderFields() {
        List<Field> fields = new ArrayList<>();
        for (String name : new String[]{"minX", "minY", "minZ", "maxX", "maxY", "maxZ", "defined"}) {
            try {
                Field field = AABB.Builder.class.getDeclaredField(name);
                field.setAccessible(true);
                fields.add(field);
            } catch (NoSuchFieldException e) {
                throw new IllegalStateException(e);
            }
        }
        return fields;
    }

    private static BlockHitResult randomBlockHitResult(final Random random) {
        Vec3 location = randomVec3(random);
        Direction direction = Direction.values()[random.nextInt(6)];
        BlockPos pos = random.nextBoolean() ? BlockPos.containing(location) : randomBlockPos(random);
        return switch (random.nextInt(4)) {
            case 0 -> random.nextBoolean() ? BlockHitResult.miss(location, direction, pos) : BlockHitResult.miss(location, direction, pos).hitBorder();
            case 1 -> new BlockHitResult(location, direction, pos, random.nextBoolean());
            default -> new BlockHitResult(location, direction, pos, random.nextBoolean(), random.nextBoolean());
        };
    }

    // A miss is never inside a block: only the private constructor could make one.
    private static BlockHitResult copyBlockHitResult(final BlockHitResult hit) {
        if (hit.getType() == HitResult.Type.MISS) {
            BlockHitResult miss = BlockHitResult.miss(hit.getLocation(), hit.getDirection(), hit.getBlockPos());
            return hit.isWorldBorderHit() ? miss.hitBorder() : miss;
        }
        return new BlockHitResult(hit.getLocation(), hit.getDirection(), hit.getBlockPos(), hit.isInside(), hit.isWorldBorderHit());
    }

    // A registry's key half the time, and otherwise an element's, in one of the edges' registries or any.
    private static ResourceKey<?> randomResourceKey(final Random random) {
        if (random.nextBoolean()) {
            return ResourceKey.createRegistryKey(randomIdentifier(random));
        }
        Identifier registry = random.nextBoolean()
            ? ((ResourceKey<?>)RESOURCE_KEY.edges().get(random.nextInt(RESOURCE_KEY.edges().size()))).identifier()
            : randomIdentifier(random);
        return ResourceKey.create(ResourceKey.createRegistryKey(registry), randomIdentifier(random));
    }

    // Mostly the default namespace, else a short one of any allowed characters, and a path of up to three such segments.
    static Identifier randomIdentifier(final Random random) {
        String namespace = random.nextBoolean() ? Identifier.DEFAULT_NAMESPACE : randomName(random, "abcdefghijklmnopqrstuvwxyz0123456789_.-");
        StringBuilder path = new StringBuilder(randomName(random, "abcdefghijklmnopqrstuvwxyz0123456789_.-"));
        for (int segments = random.nextInt(3); segments > 0; segments--) {
            path.append('/').append(randomName(random, "abcdefghijklmnopqrstuvwxyz0123456789_.-"));
        }
        return namespace.equals("..") ? Identifier.withDefaultNamespace(path.toString()) : Identifier.fromNamespaceAndPath(namespace, path.toString());
    }

    private static String randomName(final Random random, final String alphabet) {
        StringBuilder name = new StringBuilder();
        for (int length = random.nextInt(9); length > 0; length--) {
            name.append(alphabet.charAt(random.nextInt(alphabet.length())));
        }
        return name.toString();
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
