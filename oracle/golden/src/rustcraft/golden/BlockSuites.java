package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.IdentityHashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.function.Function;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.level.EmptyBlockGetter;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.ColorCollection;
import net.minecraft.world.level.block.WeatheringCopperCollection;
import net.minecraft.world.level.block.state.BlockBehaviour;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.StateDefinition;
import net.minecraft.world.level.block.state.properties.BlockStateProperties;
import net.minecraft.world.level.block.state.properties.BooleanProperty;
import net.minecraft.world.level.block.state.properties.EnumProperty;
import net.minecraft.world.level.block.state.properties.IntegerProperty;
import net.minecraft.world.level.block.state.properties.Property;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.phys.AABB;

/**
 * The block state registry: every property in BlockStateProperties, every block Blocks registers, and every state in
 * Block.BLOCK_STATE_REGISTRY, by id. See TESTING.md for the encodings: a property is the name of its field in
 * BlockStateProperties (every block's properties are among them), a value is a boolean, an int or its enum constant's
 * name, a block is its registered name, and a state is its id.
 */
final class BlockSuites {
    private static final String PROPERTY = "Lnet/minecraft/world/level/block/state/properties/Property;";
    private static final String BLOCK_GETTER = "Lnet/minecraft/world/level/BlockGetter;";
    private static final String BLOCK_POS = "Lnet/minecraft/core/BlockPos;";
    // Where predicates taking a level are asked: no block entity there, nothing around. Not the origin, so that a
    // post-process position at the block and one above it differ from (0, 0, 0).
    private static final BlockPos POS = new BlockPos(1, 2, 3);
    // Properties a block's default state is asked about whether it has them or not: ones blocks often have, and others
    // with the same names.
    private static final List<String> PROBES = List.of(
        "WATERLOGGED", "FACING", "HORIZONTAL_FACING", "FACING_HOPPER", "AXIS", "HORIZONTAL_AXIS", "LIT", "POWERED", "AGE_7", "AGE_3",
        "LEVEL", "HALF", "DOUBLE_BLOCK_HALF", "NORTH", "NORTH_WALL", "NORTH_REDSTONE"
    );
    // Strings a property is asked for a value by: besides its values' names, ones that look like other values, and what
    // Integer.parseInt also takes (a sign, leading zeros, any Unicode decimal digit) or not (spaces, overflow).
    private static final List<String> VALUE_NAMES = List.of(
        "", "x", "NORTH", "north", "up", "true", "false", "TRUE", "0", "1", "2", "3", "4", "7", "8", "15", "16", "24", "25", "26", "-1", "+1",
        "+0", "-0", "01", "007", " 1", "1 ", "1.0", "0x1", "2147483647", "2147483648", "-2147483648", "٣", "３", "๓", "1٠",
        "१०", "floor", "wall", "none", "low", "tall", "side", "single", "bottom", "top", "x ", "X", "y", "z"
    );

    private BlockSuites() {
    }

    static List<Suite> suites() {
        return List.of(new PropertySuite(), new StateDefinitionSuite(), new BlockStatePropertiesSuite(), new BlocksSuite(), new BlockSuite(), new BlockStateSuite());
    }

    /** BlockStateProperties' properties by their fields, in declaration order. */
    static final Map<String, Property<?>> PROPERTIES = properties();
    private static final Map<Property<?>, String> FIELD_NAMES = fieldNames();

    private static Map<String, Property<?>> properties() {
        Map<String, Property<?>> properties = new LinkedHashMap<>();
        for (Field field : BlockStateProperties.class.getFields()) {
            if (Modifier.isStatic(field.getModifiers()) && Property.class.isAssignableFrom(field.getType())) {
                try {
                    properties.put(field.getName(), (Property<?>)field.get(null));
                } catch (IllegalAccessException e) {
                    throw new IllegalStateException(e);
                }
            }
        }
        return properties;
    }

    private static Map<Property<?>, String> fieldNames() {
        Map<Property<?>, String> names = new IdentityHashMap<>();
        PROPERTIES.forEach((name, property) -> {
            if (names.put(property, name) != null) {
                throw new IllegalStateException("two fields hold " + name);
            }
        });
        return names;
    }

    static JsonElement property(final Property<?> property) {
        String name = FIELD_NAMES.get(property);
        if (name == null) {
            throw new IllegalStateException("not a BlockStateProperties field: " + property);
        }
        return new JsonPrimitive(name);
    }

    static JsonElement value(final Object value) {
        return switch (value) {
            case null -> JsonNull.INSTANCE;
            case Boolean b -> new JsonPrimitive(b);
            case Integer i -> new JsonPrimitive(i);
            case Enum<?> e -> new JsonPrimitive(e.name());
            default -> throw new IllegalArgumentException("not a property value: " + value);
        };
    }

    static JsonElement block(final Block block) {
        return new JsonPrimitive(BuiltInRegistries.BLOCK.getKey(block).toString());
    }

    static JsonElement state(final BlockState state) {
        return new JsonPrimitive(Block.BLOCK_STATE_REGISTRY.getId(state));
    }

    /**
     * A fluid state: {@code [fluid, values, isEmpty, isRandomlyTicking, isSource, getAmount]}, its fluid's registered name
     * and its values like a block state's, {@code [[field, value], …]}.
     */
    static JsonElement fluidState(final FluidState state) {
        JsonArray values = new JsonArray();
        state.getValues().forEach(value -> values.add(array(property(value.property()), value(value.value()))));
        return array(
            new JsonPrimitive(BuiltInRegistries.FLUID.getKey(state.getType()).toString()), values, new JsonPrimitive(state.isEmpty()),
            new JsonPrimitive(state.isRandomlyTicking()), new JsonPrimitive(state.isSource()), new JsonPrimitive(state.getAmount())
        );
    }

    static JsonElement f32(final float value) {
        return new JsonPrimitive(String.format("%08x", Float.floatToRawIntBits(value)));
    }

    static JsonArray array(final JsonElement... elements) {
        JsonArray array = new JsonArray();
        for (JsonElement element : elements) {
            array.add(element);
        }
        return array;
    }

    static <T> JsonArray list(final Iterable<T> values, final Function<T, JsonElement> encode) {
        JsonArray array = new JsonArray();
        values.forEach(value -> array.add(encode.apply(value)));
        return array;
    }

    /** Writes the call's result, or what it threw. */
    static void call(final GoldenWriter writer, final String method, final JsonArray args, final ThrowingSupplier<JsonElement> call) throws IOException {
        JsonElement result;
        try {
            result = call.get();
        } catch (RuntimeException e) {
            writer.threw(method, args, e, null);
            return;
        }
        writer.result(method, args, result, null);
    }

    @FunctionalInterface
    interface ThrowingSupplier<T> {
        T get();
    }

    private static List<Object> enumConstantsAndValues(final Property<?> property) {
        List<Object> values = new ArrayList<>(property.getPossibleValues());
        if (property instanceof EnumProperty<?>) {
            for (Object constant : property.getValueClass().getEnumConstants()) {
                if (!values.contains(constant)) {
                    values.add(constant);
                }
            }
        } else if (property instanceof IntegerProperty integer) {
            int min = (Integer)integer.getPossibleValues().getFirst();
            int max = (Integer)integer.getPossibleValues().getLast();
            for (int value : new int[]{min - 2, min - 1, max + 1, max + 2, Integer.MIN_VALUE, Integer.MAX_VALUE}) {
                values.add(value);
            }
        }
        return values;
    }

    @SuppressWarnings({"unchecked", "rawtypes"})
    private static String nameOf(final Property property, final Object value) {
        return property.getName((Comparable)value);
    }

    @SuppressWarnings({"unchecked", "rawtypes"})
    private static int internalIndex(final Property property, final Object value) {
        return property.getInternalIndex((Comparable)value);
    }

    /**
     * Property's methods on every property in BlockStateProperties (`args[0]` is the field), and the factories' checks.
     * getName(T) and getInternalIndex(T) also get every constant of an enum property's class, and ints around an integer
     * property's range.
     */
    static final class PropertySuite implements Suite {
        @Override
        public String name() {
            return Property.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            for (Map.Entry<String, Property<?>> entry : PROPERTIES.entrySet()) {
                Property<?> property = entry.getValue();
                JsonArray receiver = array(new JsonPrimitive(entry.getKey()));
                writer.result("getName()Ljava/lang/String;", receiver, new JsonPrimitive(property.getName()), null);
                writer.result("getPossibleValues()Ljava/util/List;", receiver, list(property.getPossibleValues(), BlockSuites::value), null);
                writer.result("getValueClass()Ljava/lang/Class;", receiver, new JsonPrimitive(property.getValueClass().getName()), null);
                writer.result("toString()Ljava/lang/String;", receiver, new JsonPrimitive(property.toString()), null);
                for (Object value : property.getPossibleValues()) {
                    JsonArray args = array(new JsonPrimitive(entry.getKey()), value(value));
                    call(writer, "getName(Ljava/lang/Comparable;)Ljava/lang/String;", args, () -> new JsonPrimitive(nameOf(property, value)));
                }
                for (Object value : enumConstantsAndValues(property)) {
                    JsonArray args = array(new JsonPrimitive(entry.getKey()), value(value));
                    call(writer, "getInternalIndex(Ljava/lang/Comparable;)I", args, () -> new JsonPrimitive(internalIndex(property, value)));
                }
                List<String> names = new ArrayList<>();
                property.getPossibleValues().forEach(value -> names.add(nameOf(property, value)));
                names.addAll(VALUE_NAMES);
                for (String name : names) {
                    JsonArray args = array(new JsonPrimitive(entry.getKey()), new JsonPrimitive(name));
                    call(writer, "getValue(Ljava/lang/String;)Ljava/util/Optional;", args, () -> value(property.getValue(name).orElse(null)));
                }
                for (Map.Entry<String, Property<?>> other : PROPERTIES.entrySet()) {
                    JsonArray args = array(new JsonPrimitive(entry.getKey()), new JsonPrimitive(other.getKey()));
                    writer.result("equals(Ljava/lang/Object;)Z", args, new JsonPrimitive(property.equals(other.getValue())), null);
                }
            }
            writer.skipped("hashCode()I", "hashes its value class's identity hash, which changes from run to run");
            writer.skipped("codec()Lcom/mojang/serialization/Codec;", "no generator for com.mojang.serialization.Codec");
            writer.skipped("valueCodec()Lcom/mojang/serialization/Codec;", "no generator for com.mojang.serialization.Codec");
            writer.skipped(
                "parseValue(Lcom/mojang/serialization/DynamicOps;Lnet/minecraft/world/level/block/state/StateHolder;Ljava/lang/Object;)Lcom/mojang/serialization/DataResult;",
                "no generator for com.mojang.serialization.DynamicOps"
            );
            writer.skipped("value(Ljava/lang/Comparable;)Lnet/minecraft/world/level/block/state/properties/Property$Value;", "no encoder for Property.Value");
            writer.skipped(
                "value(Lnet/minecraft/world/level/block/state/StateHolder;)Lnet/minecraft/world/level/block/state/properties/Property$Value;",
                "no encoder for Property.Value"
            );
            writer.skipped("getAllValues()Ljava/util/stream/Stream;", "no encoder for Property.Value");

            // The factories refuse a negative minimum, a maximum not above the minimum, and no values.
            int[][] ranges = {{0, 1}, {0, 15}, {1, 4}, {-1, 3}, {3, 3}, {4, 3}, {0, 0}, {Integer.MAX_VALUE - 1, Integer.MAX_VALUE}};
            for (int[] range : ranges) {
                JsonArray args = array(new JsonPrimitive("age"), new JsonPrimitive(range[0]), new JsonPrimitive(range[1]));
                call(
                    writer, "IntegerProperty.create(Ljava/lang/String;II)Lnet/minecraft/world/level/block/state/properties/IntegerProperty;", args,
                    () -> {
                        IntegerProperty created = IntegerProperty.create("age", range[0], range[1]);
                        return array(new JsonPrimitive(created.getName()), new JsonPrimitive(created.getPossibleValues().size()));
                    }
                );
            }
            call(
                writer, "EnumProperty.create(Ljava/lang/String;Ljava/lang/Class;Ljava/util/List;)Lnet/minecraft/world/level/block/state/properties/EnumProperty;",
                array(new JsonPrimitive("facing"), new JsonArray()),
                () -> property(EnumProperty.create("facing", net.minecraft.core.Direction.class, List.of()))
            );
            writer.result(
                "BooleanProperty.create(Ljava/lang/String;)Lnet/minecraft/world/level/block/state/properties/BooleanProperty;",
                array(new JsonPrimitive("lit")), list(BooleanProperty.create("lit").getPossibleValues(), BlockSuites::value), null
            );
        }
    }

    /**
     * StateDefinition.Builder on lists of properties: the definition it makes is its properties sorted by name and the
     * number of states, unless two properties share a name. A block's own definition is in the Block suite.
     */
    static final class StateDefinitionSuite implements Suite {
        @Override
        public String name() {
            return StateDefinition.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            List<List<String>> lists = List.of(
                List.of(), List.of("WATERLOGGED"), List.of("FACING", "WATERLOGGED"), List.of("WATERLOGGED", "FACING"),
                List.of("FACING", "HALF", "STAIRS_SHAPE", "WATERLOGGED"), List.of("NORTH", "EAST", "SOUTH", "WEST", "UP", "DOWN"),
                List.of("NORTH_REDSTONE", "EAST_REDSTONE", "SOUTH_REDSTONE", "WEST_REDSTONE", "POWER"), List.of("FACING", "HORIZONTAL_FACING"),
                List.of("AGE_1", "AGE_2"), List.of("LIT", "LIT"), List.of("NORTH", "NORTH_WALL"), List.of("AXIS", "HORIZONTAL_AXIS"),
                List.of("DISTANCE", "STABILITY_DISTANCE"), List.of("LEVEL", "LEVEL_CAULDRON"), List.of("LEVEL_HONEY", "FACING"),
                List.of("SLOT_0_OCCUPIED", "SLOT_1_OCCUPIED", "SLOT_2_OCCUPIED", "SLOT_3_OCCUPIED", "SLOT_4_OCCUPIED", "SLOT_5_OCCUPIED", "HORIZONTAL_FACING"),
                List.of("NOTE", "NOTEBLOCK_INSTRUMENT", "POWERED"), List.of("ORIENTATION", "TRIGGERED", "CRAFTING")
            );
            for (List<String> names : lists) {
                JsonArray args = array(list(names, JsonPrimitive::new));
                call(writer, "Builder.add([Lnet/minecraft/world/level/block/state/properties/Property;).create", args, () -> {
                    StateDefinition.Builder<Block, BlockState> builder = new StateDefinition.Builder<>(Blocks.STONE);
                    for (String name : names) {
                        builder.add(PROPERTIES.get(name));
                    }
                    StateDefinition<Block, BlockState> definition = builder.create(Block::defaultBlockState, BlockState::new);
                    return array(list(definition.getProperties(), BlockSuites::property), new JsonPrimitive(definition.getPossibleStates().size()));
                });
            }
        }
    }

    /** BlockStateProperties' constants: a property is `[class, name, values]`. */
    static final class BlockStatePropertiesSuite implements Suite {
        @Override
        public String name() {
            return BlockStateProperties.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            for (Field field : BlockStateProperties.class.getFields()) {
                try {
                    Object value = field.get(null);
                    if (value instanceof Property<?> property) {
                        writer.field(
                            field.getName(),
                            array(
                                new JsonPrimitive(property.getClass().getSimpleName()), new JsonPrimitive(property.getName()),
                                list(property.getPossibleValues(), BlockSuites::value)
                            )
                        );
                    } else if (value instanceof Integer i) {
                        writer.field(field.getName(), new JsonPrimitive(i));
                    } else {
                        throw new IllegalStateException("unexpected field " + field);
                    }
                } catch (IllegalAccessException e) {
                    throw new IllegalStateException(e);
                }
            }
        }
    }

    /** Blocks' constants: a block is its registered name, a collection the list of its blocks. */
    static final class BlocksSuite implements Suite {
        @Override
        public String name() {
            return Blocks.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            for (Field field : Blocks.class.getFields()) {
                try {
                    Object value = field.get(null);
                    JsonElement encoded = switch (value) {
                        case Block block -> block(block);
                        case ColorCollection<?> collection -> list(collection.asList(), element -> block((Block)element));
                        case WeatheringCopperCollection<?> collection -> array(
                            list(byState(collection.weathering()), element -> block((Block)element)),
                            list(byState(collection.waxed()), element -> block((Block)element))
                        );
                        default -> throw new IllegalStateException("unexpected field " + field);
                    };
                    writer.field(field.getName(), encoded);
                } catch (IllegalAccessException e) {
                    throw new IllegalStateException(e);
                }
            }
        }

        private static List<Object> byState(final WeatheringCopperCollection.ByState<?> byState) {
            List<Object> elements = new ArrayList<>();
            byState.forEach(elements::add);
            return elements;
        }
    }

    // A predicate's kind: Properties' default (which needs the block's shapes, or its tags), one of Blocks' named
    // predicates, or another the block was given.
    private static final Map<String, Field> PREDICATE_FIELDS = new LinkedHashMap<>();
    private static final BlockBehaviour.Properties DEFAULT_PROPERTIES = BlockBehaviour.Properties.of();
    private static final Map<Object, String> NAMED_PREDICATES = new IdentityHashMap<>();

    static {
        try {
            for (String name : List.of("isRedstoneConductor", "isSuffocating", "isViewBlocking", "forceSolidOn", "forceSolidOff")) {
                Field field = BlockBehaviour.Properties.class.getDeclaredField(name);
                field.setAccessible(true);
                PREDICATE_FIELDS.put(name, field);
            }
            for (String name : List.of("NOT_CLOSED_SHULKER", "NOT_EXTENDED_PISTON", "NEAR_PLANE_INTERSECTS_OUTLINE")) {
                Field field = Blocks.class.getDeclaredField(name);
                field.setAccessible(true);
                NAMED_PREDICATES.put(field.get(null), name);
            }
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }

    private static Object propertiesField(final Block block, final String name) {
        try {
            return PREDICATE_FIELDS.get(name).get(block.properties());
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
    }

    static String predicateKind(final Block block, final String name) {
        Object predicate = propertiesField(block, name);
        String named = NAMED_PREDICATES.get(predicate);
        if (named != null) {
            return named;
        }
        // A lambda's class stands for where it's written: Properties' default isViewBlocking captures the Properties it
        // was made for, so each Properties has its own, of the same class.
        Object fallback;
        try {
            fallback = PREDICATE_FIELDS.get(name).get(DEFAULT_PROPERTIES);
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
        return predicate.getClass() == fallback.getClass() ? "default" : "custom";
    }

    private static Object blockField(final Block block, final String name) {
        try {
            Field field = net.minecraft.world.level.block.state.BlockBehaviour.class.getDeclaredField(name);
            field.setAccessible(true);
            return field.get(block);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }

    /** Each block, by its registered name in `args[0]`: its registry id and class, its state definition and what Properties gave it. */
    static final class BlockSuite implements Suite {
        @Override
        public String name() {
            return Block.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            writer.field("BLOCK_STATE_REGISTRY.size()", new JsonPrimitive(Block.BLOCK_STATE_REGISTRY.size()));
            for (int id : new int[]{-1, 0, 1, 2, Block.BLOCK_STATE_REGISTRY.size() - 1, Block.BLOCK_STATE_REGISTRY.size(), Integer.MAX_VALUE, Integer.MIN_VALUE}) {
                writer.result("stateById(I)Lnet/minecraft/world/level/block/state/BlockState;", array(new JsonPrimitive(id)), state(Block.stateById(id)), null);
            }
            writer.result("getId(Lnet/minecraft/world/level/block/state/BlockState;)I", array(JsonNull.INSTANCE), new JsonPrimitive(Block.getId(null)), null);
            for (Block block : BuiltInRegistries.BLOCK) {
                JsonArray args = array(block(block));
                StateDefinition<Block, BlockState> definition = block.getStateDefinition();
                writer.result("BuiltInRegistries.BLOCK.getId", args, new JsonPrimitive(BuiltInRegistries.BLOCK.getId(block)), null);
                writer.result("getClass()", args, new JsonPrimitive(block.getClass().getName()), null);
                writer.result("toString()Ljava/lang/String;", args, new JsonPrimitive(block.toString()), null);
                writer.result("defaultBlockState()Lnet/minecraft/world/level/block/state/BlockState;", args, state(block.defaultBlockState()), null);
                writer.result("getStateDefinition().getProperties()", args, list(definition.getProperties(), BlockSuites::property), null);
                writer.result("getStateDefinition().getPossibleStates()", args, list(definition.getPossibleStates(), BlockSuites::state), null);
                writer.result("getStateDefinition().any()", args, state(definition.any()), null);
                writer.result("getStateDefinition().isSingletonState()", args, new JsonPrimitive(definition.isSingletonState()), null);
                writer.result("getStateDefinition().toString()", args, new JsonPrimitive(definition.toString()), null);
                List<String> names = new ArrayList<>();
                definition.getProperties().forEach(property -> names.add(property.getName()));
                names.addAll(List.of("facing", "waterlogged", "age", "", "FACING", "power", "axis"));
                for (String name : names) {
                    Property<?> property = definition.getProperty(name);
                    writer.result(
                        "getStateDefinition().getProperty(Ljava/lang/String;)", array(block(block), new JsonPrimitive(name)),
                        property == null ? JsonNull.INSTANCE : property(property), null
                    );
                }
                writer.result("getDescriptionId()Ljava/lang/String;", args, new JsonPrimitive(block.getDescriptionId()), null);
                writer.result(
                    "getLootTable()Ljava/util/Optional;", args,
                    block.getLootTable().<JsonElement>map(key -> new JsonPrimitive(key.identifier().toString())).orElse(JsonNull.INSTANCE), null
                );
                writer.result("getExplosionResistance()F", args, f32(block.getExplosionResistance()), null);
                writer.result("getFriction()F", args, f32(block.getFriction()), null);
                writer.result("getSpeedFactor()F", args, f32(block.getSpeedFactor()), null);
                writer.result("getJumpFactor()F", args, f32(block.getJumpFactor()), null);
                writer.result("getBounceRestitution()F", args, f32(block.getBounceRestitution()), null);
                writer.result("getFallDistanceReduction()F", args, f32(block.getFallDistanceReduction()), null);
                writer.result("hasDynamicShape()Z", args, new JsonPrimitive(block.hasDynamicShape()), null);
                writer.result("defaultDestroyTime()F", args, f32(block.defaultDestroyTime()), null);
                writer.result("defaultMapColor()Lnet/minecraft/world/level/material/MapColor;", args, new JsonPrimitive(block.defaultMapColor().id), null);
                writer.result("hasCollision", args, new JsonPrimitive((Boolean)blockField(block, "hasCollision")), null);
                writer.result("isRandomlyTicking", args, new JsonPrimitive((Boolean)blockField(block, "isRandomlyTicking")), null);
                writer.result("properties().forceSolidOn", args, new JsonPrimitive((Boolean)propertiesField(block, "forceSolidOn")), null);
                writer.result("properties().forceSolidOff", args, new JsonPrimitive((Boolean)propertiesField(block, "forceSolidOff")), null);
                for (String predicate : List.of("isRedstoneConductor", "isSuffocating", "isViewBlocking")) {
                    writer.result("properties()." + predicate, args, new JsonPrimitive(predicateKind(block, predicate)), null);
                }
            }
        }
    }

    /**
     * Each state in Block.BLOCK_STATE_REGISTRY, by its id in `args[0]`: its block and values, what it took from its
     * block's Properties, and its lookups. The functions taking a level are asked at (1, 2, 3) of EmptyBlockGetter.
     */
    static final class BlockStateSuite implements Suite {
        private static final String GET_VALUE = "getValue(" + PROPERTY + ")Ljava/lang/Comparable;";
        private static final String SET_VALUE = "setValue(" + PROPERTY + "Ljava/lang/Comparable;)Ljava/lang/Object;";
        private static final String GETTER_AND_POS = "(" + BLOCK_GETTER + BLOCK_POS + ")";

        @Override
        public String name() {
            return BlockState.class.getName();
        }

        @Override
        @SuppressWarnings({"unchecked", "rawtypes"})
        public void write(final GoldenWriter writer) throws IOException {
            EmptyBlockGetter level = EmptyBlockGetter.INSTANCE;
            AABB nearPlane = new AABB(POS);
            for (BlockState state : Block.BLOCK_STATE_REGISTRY) {
                JsonArray args = array(state(state));
                Block block = state.getBlock();
                writer.result("getBlock()Lnet/minecraft/world/level/block/Block;", args, block(block), null);
                writer.result(
                    "getValues()Ljava/util/stream/Stream;", args,
                    list(state.getValues().toList(), value -> array(property(value.property()), value(value.value()))), null
                );
                writer.result("toString()Ljava/lang/String;", args, new JsonPrimitive(state.toString()), null);
                writer.result("getLightEmission()I", args, new JsonPrimitive(state.getLightEmission()), null);
                writer.result("isAir()Z", args, new JsonPrimitive(state.isAir()), null);
                writer.result("ignitedByLava()Z", args, new JsonPrimitive(state.ignitedByLava()), null);
                writer.result("liquid()Z", args, new JsonPrimitive(state.liquid()), null);
                writer.result("getPistonPushReaction()Lnet/minecraft/world/level/material/PushReaction;", args, value(state.getPistonPushReaction()), null);
                writer.result("getDestroySpeed" + GETTER_AND_POS + "F", args, f32(state.getDestroySpeed(level, POS)), null);
                writer.result("requiresCorrectToolForDrops()Z", args, new JsonPrimitive(state.requiresCorrectToolForDrops()), null);
                writer.result("canOcclude()Z", args, new JsonPrimitive(state.canOcclude()), null);
                writer.result("emissiveRendering()Z", args, new JsonPrimitive(state.emissiveRendering()), null);
                writer.result("canBeReplaced()Z", args, new JsonPrimitive(state.canBeReplaced()), null);
                writer.result("shouldSpawnTerrainParticles()Z", args, new JsonPrimitive(state.shouldSpawnTerrainParticles()), null);
                writer.result(
                    "instrument()Lnet/minecraft/world/level/block/state/properties/NoteBlockInstrument;", args, value(state.instrument()), null
                );
                writer.result("isRandomlyTicking()Z", args, new JsonPrimitive(state.isRandomlyTicking()), null);
                writer.result("hasOffsetFunction()Z", args, new JsonPrimitive(state.hasOffsetFunction()), null);
                writer.result("hasBlockEntity()Z", args, new JsonPrimitive(state.hasBlockEntity()), null);
                writer.result("getFluidState()Lnet/minecraft/world/level/material/FluidState;", args, fluidState(state.getFluidState()), null);
                writer.result(
                    "getMapColor" + GETTER_AND_POS + "Lnet/minecraft/world/level/material/MapColor;", args,
                    new JsonPrimitive(state.getMapColor(level, POS).id), null
                );
                BlockPos postProcess = state.getPostProcessPos(level, POS);
                writer.result(
                    "getPostProcessPos" + GETTER_AND_POS + BLOCK_POS, args,
                    postProcess == null ? JsonNull.INSTANCE : array(new JsonPrimitive(postProcess.getX()), new JsonPrimitive(postProcess.getY()), new JsonPrimitive(postProcess.getZ())),
                    null
                );
                // Only predicates that answer from the state alone: Properties' defaults and the named ones that aren't need
                // the block's shapes, tags or block entity, which the Block suite records the kinds of.
                String redstoneConductor = predicateKind(block, "isRedstoneConductor");
                if (redstoneConductor.equals("custom")) {
                    writer.result("isRedstoneConductor" + GETTER_AND_POS + "Z", args, new JsonPrimitive(state.isRedstoneConductor(level, POS)), null);
                }
                String suffocating = predicateKind(block, "isSuffocating");
                if (suffocating.equals("custom") || suffocating.equals("NOT_EXTENDED_PISTON")) {
                    writer.result("isSuffocating" + GETTER_AND_POS + "Z", args, new JsonPrimitive(state.isSuffocating(level, POS)), null);
                }
                if (predicateKind(block, "isViewBlocking").equals("custom")) {
                    writer.result(
                        "isViewBlocking(" + BLOCK_GETTER + BLOCK_POS + "Lnet/minecraft/world/phys/AABB;)Z", args,
                        new JsonPrimitive(state.isViewBlocking(level, POS, nearPlane)), null
                    );
                }

                for (Property property : sampled(state) ? state.getProperties() : List.<Property>of()) {
                    JsonArray propertyArgs = array(state(state), property(property));
                    writer.result("cycle(" + PROPERTY + ")Ljava/lang/Object;", propertyArgs, state(state.cycle(property)), null);
                    writer.result(
                        SET_VALUE + "[every value]", propertyArgs,
                        list((List<Comparable>)property.getPossibleValues(), value -> state((BlockState)state.setValue(property, value))), null
                    );
                }

                if (state == block.defaultBlockState()) {
                    for (String probe : PROBES) {
                        Property property = PROPERTIES.get(probe);
                        Comparable first = (Comparable)property.getPossibleValues().getFirst();
                        Comparable last = (Comparable)property.getPossibleValues().getLast();
                        JsonArray probeArgs = array(state(state), new JsonPrimitive(probe));
                        call(writer, GET_VALUE, probeArgs, () -> value(state.getValue(property)));
                        writer.result("hasProperty(" + PROPERTY + ")Z", probeArgs, new JsonPrimitive(state.hasProperty(property)), null);
                        writer.result(
                            "getOptionalValue(" + PROPERTY + ")Ljava/util/Optional;", probeArgs, value(state.getOptionalValue(property).orElse(null)), null
                        );
                        JsonArray withFirst = array(state(state), new JsonPrimitive(probe), value(first));
                        writer.result(
                            "getValueOrElse(" + PROPERTY + "Ljava/lang/Comparable;)Ljava/lang/Comparable;", withFirst,
                            value(state.getValueOrElse(property, first)), null
                        );
                        JsonArray withLast = array(state(state), new JsonPrimitive(probe), value(last));
                        call(writer, SET_VALUE, withLast, () -> state((BlockState)state.setValue(property, last)));
                        writer.result(
                            "trySetValue(" + PROPERTY + "Ljava/lang/Comparable;)Ljava/lang/Object;", withLast,
                            state((BlockState)state.trySetValue(property, last)), null
                        );
                        call(writer, "cycle(" + PROPERTY + ")Ljava/lang/Object;[probe]", probeArgs, () -> state((BlockState)state.cycle(property)));
                    }
                    // Values the state's own properties don't take: ints past their range, constants their list leaves out.
                    for (Property property : state.getProperties()) {
                        for (Object value : enumConstantsAndValues(property)) {
                            if (!property.getPossibleValues().contains(value)) {
                                JsonArray valueArgs = array(state(state), property(property), value(value));
                                call(writer, SET_VALUE, valueArgs, () -> state((BlockState)state.setValue(property, (Comparable)value)));
                            }
                        }
                    }
                }
            }
        }

        // Every state of a block with up to 64, and 64 spread over the others' (the first, the last and the default
        // among them): the lookups go by the block's definition, the same for each of its states.
        private static boolean sampled(final BlockState state) {
            List<BlockState> states = state.getBlock().getStateDefinition().getPossibleStates();
            int index = states.indexOf(state);
            int count = states.size();
            return count <= 64 || index == count - 1 || state == state.getBlock().defaultBlockState() || index % ((count + 63) / 64) == 0;
        }
    }
}
