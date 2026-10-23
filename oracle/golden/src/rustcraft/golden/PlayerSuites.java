package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Random;
import net.minecraft.core.Holder;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.Identifier;
import net.minecraft.tags.TagKey;
import net.minecraft.world.entity.EntityDimensions;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.EntityTypes;
import net.minecraft.world.entity.MobCategory;
import net.minecraft.world.entity.ai.attributes.Attribute;
import net.minecraft.world.entity.ai.attributes.AttributeInstance;
import net.minecraft.world.entity.ai.attributes.AttributeModifier;
import net.minecraft.world.entity.ai.attributes.Attributes;

/**
 * The player's movement's own suites (TESTING.md). An attribute is its registered name, a modifier operation its
 * constant's name.
 */
final class PlayerSuites {
    // The attributes the player's movement reads, by registered name.
    private static final Map<String, Holder<Attribute>> ATTRIBUTES = Map.ofEntries(
        Map.entry("movement_speed", Attributes.MOVEMENT_SPEED),
        Map.entry("sneaking_speed", Attributes.SNEAKING_SPEED),
        Map.entry("jump_strength", Attributes.JUMP_STRENGTH),
        Map.entry("gravity", Attributes.GRAVITY),
        Map.entry("step_height", Attributes.STEP_HEIGHT),
        Map.entry("scale", Attributes.SCALE),
        Map.entry("friction_modifier", Attributes.FRICTION_MODIFIER),
        Map.entry("air_drag_modifier", Attributes.AIR_DRAG_MODIFIER),
        Map.entry("bounciness", Attributes.BOUNCINESS),
        Map.entry("movement_efficiency", Attributes.MOVEMENT_EFFICIENCY),
        Map.entry("water_movement_efficiency", Attributes.WATER_MOVEMENT_EFFICIENCY),
        Map.entry("max_health", Attributes.MAX_HEALTH)
    );
    private static final List<String> NAMES = ATTRIBUTES.keySet().stream().sorted().toList();
    // Each operation's modifiers come from one id, so an instance never holds two of one operation: their order is an
    // Object2ObjectOpenHashMap's, which the port doesn't follow yet.
    private static final List<String> IDS = List.of("minecraft:added", "minecraft:base_multiple", "minecraft:sprinting");
    private static final double[] AMOUNTS = {
        0.3F, 0.2, 0.4, -0.15, 1.0, -1.0, 0.0, -0.0, 0.5, 2.0, 1.0E-7, 100.0, -100.0, 3000.0, Double.NaN, 0.1F, -0.3F, 0.42F, 0.08
    };
    private static final double[] BASES = {
        0.1F, 0.10000000149011612, 0.7, 0.0, -0.0, 1.0, 20.0, 0.42F, 0.08, 0.6, 0.3, 1024.0, 1025.0, -1.0, 2048.0, 0.0625, 16.0,
        Double.NaN, Double.POSITIVE_INFINITY, Double.NEGATIVE_INFINITY, 1.0E-300
    };

    private PlayerSuites() {
    }

    static List<Suite> suites() {
        return List.of(new AttributeInstanceSuite(), new MobCategorySuite(), new EntityTypeSuite(), new EntityTypesSuite());
    }

    /**
     * MobCategory: its constants in order, and each one's {@code [ordinal, getName, getDebugAbbreviation,
     * getSerializedName, getMaxInstancesPerChunk, isFriendly, isPersistent, getDespawnDistance, getNoDespawnDistance]}.
     */
    static final class MobCategorySuite implements Suite {
        @Override
        public String name() {
            return MobCategory.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            JsonArray values = new JsonArray();
            for (MobCategory category : MobCategory.values()) {
                values.add(category.name());
            }
            writer.field("values", values);
            for (MobCategory category : MobCategory.values()) {
                writer.field(category.name(), ChunkSuites.array(
                    category.ordinal(), category.getName(), category.getDebugAbbreviation(), category.getSerializedName(),
                    category.getMaxInstancesPerChunk(), category.isFriendly(), category.isPersistent(), category.getDespawnDistance(),
                    category.getNoDespawnDistance()
                ));
            }
        }
    }

    /**
     * EntityType: NO_UPDATE_INTERVAL, and STREAM_CODEC's cases (Codecs) over RegistryFriendlyByteBufs, its values every
     * type, each its registered name. BuiltInRegistries.ENTITY_TYPE is a defaulted registry, so an id it doesn't have
     * decodes as the pig.
     */
    static final class EntityTypeSuite implements Suite {
        @Override
        public String name() {
            return EntityType.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            writer.field("NO_UPDATE_INTERVAL", new JsonPrimitive(EntityType.NO_UPDATE_INTERVAL));
            List<Object> types = new ArrayList<>();
            for (int id = 0; id < BuiltInRegistries.ENTITY_TYPE.size(); id++) {
                types.add(BuiltInRegistries.ENTITY_TYPE.byId(id));
            }
            Values.Generator values = new Values.Generator() {
                @Override
                public List<Object> edges() {
                    return types;
                }

                @Override
                public Object random(final Random random) {
                    return types.get(random.nextInt(types.size()));
                }

                @Override
                public JsonElement encode(final Object spec) {
                    return RegistryValues.encode(spec);
                }

                @Override
                public boolean exhaustive() {
                    return true;
                }
            };
            Codecs.writeCodec(
                writer, "STREAM_CODEC", new Values.Generator[0], leading -> EntityType.STREAM_CODEC,
                new Codecs.Codec(RegistryValues.REGISTRY_FRIENDLY_BUFFER, values, EntityType.class), spec -> true
            );
        }
    }

    /**
     * EntityTypes' fields, each {@code [id, key, category, [canSerialize, canSummon, fireImmune, canSpawnFarFromPlayer,
     * isAllowedInPeaceful, trackDeltas, hasUpdateInterval, onlyOpCanSetNbt], immuneTo, [width, height, eyeHeight, fixed],
     * spawnDimensionsScale, clientTrackingRange, updateInterval, descriptionId, toShortString, lootTable]} (the private
     * immuneTo and spawnDimensionsScale read by reflection, the loot table's identifier or null), then
     * BuiltInRegistries.ENTITY_TYPE's size, its default key, and byId, getValue and getOptional cases, which fall back to
     * the default (but getOptional doesn't).
     */
    static final class EntityTypesSuite implements Suite {
        private static final List<String> KEYS = List.of(
            "minecraft:pig", "minecraft:player", "minecraft:acacia_boat", "minecraft:fishing_bobber", "minecraft:zombie_horse",
            "minecraft:nope", "minecraft:entity/pig", "rustcraft:pig", "minecraft:"
        );

        @Override
        public String name() {
            return EntityTypes.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            try {
                Field immuneTo = EntityType.class.getDeclaredField("immuneTo");
                Field spawnDimensionsScale = EntityType.class.getDeclaredField("spawnDimensionsScale");
                immuneTo.setAccessible(true);
                spawnDimensionsScale.setAccessible(true);
                for (Field field : EntityTypes.class.getFields()) {
                    if (Modifier.isStatic(field.getModifiers()) && field.getType() == EntityType.class) {
                        EntityType<?> type = (EntityType<?>)field.get(null);
                        EntityDimensions dimensions = type.getDimensions();
                        writer.field(field.getName(), ChunkSuites.array(
                            BuiltInRegistries.ENTITY_TYPE.getId(type), EntityType.getKey(type).toString(), type.getCategory().name(),
                            ChunkSuites.array(
                                type.canSerialize(), type.canSummon(), type.fireImmune(), type.canSpawnFarFromPlayer(),
                                type.isAllowedInPeaceful(), type.trackDeltas(), type.hasUpdateInterval(), type.onlyOpCanSetNbt()
                            ),
                            ((TagKey<?>)immuneTo.get(type)).location().toString(),
                            ChunkSuites.array(
                                Values.floatBits(dimensions.width()), Values.floatBits(dimensions.height()),
                                Values.floatBits(dimensions.eyeHeight()), dimensions.fixed()
                            ),
                            Values.floatBits(spawnDimensionsScale.getFloat(type)), type.clientTrackingRange(), type.updateInterval(),
                            type.getDescriptionId(), type.toShortString(),
                            type.getDefaultLootTable().map(table -> table.identifier().toString()).orElse(null)
                        ));
                    }
                }
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
            writer.field("size", new JsonPrimitive(BuiltInRegistries.ENTITY_TYPE.size()));
            writer.field("defaultKey", new JsonPrimitive(BuiltInRegistries.ENTITY_TYPE.getDefaultKey().toString()));
            int size = BuiltInRegistries.ENTITY_TYPE.size();
            for (int id : new int[]{Integer.MIN_VALUE, -2, -1, 0, 1, 2, size / 2, size - 2, size - 1, size, size + 1, 1000, Integer.MAX_VALUE}) {
                writer.result("byId", ChunkSuites.array(id), RegistryValues.encode(BuiltInRegistries.ENTITY_TYPE.byId(id)), null);
            }
            for (String key : KEYS) {
                Identifier identifier = Identifier.parse(key);
                writer.result("getValue", ChunkSuites.array(key), RegistryValues.encode(BuiltInRegistries.ENTITY_TYPE.getValue(identifier)), null);
                writer.result(
                    "getOptional", ChunkSuites.array(key),
                    BuiltInRegistries.ENTITY_TYPE.getOptional(identifier).map(RegistryValues::encode).orElse(JsonNull.INSTANCE), null
                );
            }
        }
    }

    /**
     * Programs on an AttributeInstance of one of the attributes, `[attribute, ops]`, each step its output and the value
     * afterwards: `["base", value]` (setBaseValue), `["add", id, amount, operation]` (addTransientModifier, which throws
     * for an id that's on already), `["remove", id]` (removeModifier, whether there was one), `["removeAll"]` and
     * `["value"]`.
     */
    static final class AttributeInstanceSuite implements Suite {
        @Override
        public String name() {
            return AttributeInstance.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0x41545452L);
            for (int program = 0; program < 1500; program++) {
                String name = NAMES.get(program < NAMES.size() * 4 ? program % NAMES.size() : random.nextInt(NAMES.size()));
                AttributeInstance instance = new AttributeInstance(ATTRIBUTES.get(name), changed -> {
                });
                JsonArray ops = new JsonArray();
                JsonArray steps = new JsonArray();
                for (int count = 1 + random.nextInt(12); count > 0; count--) {
                    JsonArray op = new JsonArray();
                    Runnable run;
                    JsonElement[] output = {JsonNull.INSTANCE};
                    switch (random.nextInt(5)) {
                        case 0 -> {
                            double base = random.nextInt(3) == 0 ? random.nextDouble() * 4.0 - 2.0 : BASES[random.nextInt(BASES.length)];
                            op.add("base");
                            op.add(ShapeSuites.d(base));
                            run = () -> instance.setBaseValue(base);
                        }
                        case 1, 2 -> {
                            int which = random.nextInt(3);
                            double amount = random.nextInt(3) == 0 ? random.nextDouble() * 2.0 - 1.0 : AMOUNTS[random.nextInt(AMOUNTS.length)];
                            AttributeModifier.Operation operation = AttributeModifier.Operation.values()[which];
                            op.add("add");
                            op.add(IDS.get(which));
                            op.add(ShapeSuites.d(amount));
                            op.add(operation.name());
                            run = () -> instance.addTransientModifier(new AttributeModifier(Identifier.parse(IDS.get(which)), amount, operation));
                        }
                        case 3 -> {
                            if (random.nextInt(4) == 0) {
                                op.add("removeAll");
                                run = instance::removeModifiers;
                            } else {
                                String id = IDS.get(random.nextInt(IDS.size()));
                                op.add("remove");
                                op.add(id);
                                run = () -> output[0] = new JsonPrimitive(instance.removeModifier(Identifier.parse(id)));
                            }
                        }
                        default -> {
                            op.add("value");
                            run = () -> output[0] = ShapeSuites.d(instance.getValue());
                        }
                    }
                    ops.add(op);
                    try {
                        run.run();
                    } catch (RuntimeException e) {
                        JsonObject thrown = new JsonObject();
                        thrown.addProperty("throws", e.getClass().getName());
                        steps.add(thrown);
                        break;
                    }
                    steps.add(BlockSuites.array(output[0], ShapeSuites.d(instance.getValue())));
                }
                writer.result("programs", BlockSuites.array(new JsonPrimitive(name), ops), steps, null);
            }
        }
    }
}
