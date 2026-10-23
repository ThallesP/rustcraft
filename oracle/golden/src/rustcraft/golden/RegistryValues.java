package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonPrimitive;
import io.netty.buffer.ByteBuf;
import java.io.IOException;
import java.lang.reflect.ParameterizedType;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.OptionalInt;
import java.util.Random;
import java.util.function.Supplier;
import java.util.stream.Stream;
import net.minecraft.client.ClientClockManager;
import net.minecraft.core.BlockPos;
import net.minecraft.core.GlobalPos;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.LayeredRegistryAccess;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.Rotations;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.syncher.EntityDataSerializer;
import net.minecraft.network.syncher.EntityDataSerializers;
import net.minecraft.network.syncher.SynchedEntityData;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.RegistryDataLoader;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.RegistryLayer;
import net.minecraft.server.packs.PackType;
import net.minecraft.server.packs.repository.ServerPacksSource;
import net.minecraft.server.packs.resources.CloseableResourceManager;
import net.minecraft.server.packs.resources.MultiPackResourceManager;
import net.minecraft.tags.TagLoader;
import net.minecraft.world.clock.ClockNetworkState;
import net.minecraft.world.clock.WorldClock;
import net.minecraft.world.entity.HumanoidArm;
import net.minecraft.world.entity.Pose;
import net.minecraft.world.entity.ai.attributes.Attribute;
import net.minecraft.world.entity.animal.chicken.ChickenSoundVariant;
import net.minecraft.world.entity.animal.chicken.ChickenVariant;
import net.minecraft.world.entity.animal.cow.CowSoundVariant;
import net.minecraft.world.entity.animal.cow.CowVariant;
import net.minecraft.world.entity.animal.feline.CatSoundVariant;
import net.minecraft.world.entity.animal.feline.CatVariant;
import net.minecraft.world.entity.animal.frog.FrogVariant;
import net.minecraft.world.entity.animal.nautilus.ZombieNautilusVariant;
import net.minecraft.world.entity.animal.pig.PigSoundVariant;
import net.minecraft.world.entity.animal.pig.PigVariant;
import net.minecraft.world.entity.animal.wolf.WolfSoundVariant;
import net.minecraft.world.entity.animal.wolf.WolfVariant;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;

/**
 * What a RegistryFriendlyByteBuf carries: the registries a client has once it has joined a vanilla server. Those are
 * the client's static registries (BuiltInRegistries, ClientRegistryLayer.STATIC) and the synchronized ones, whose ids
 * are the order the server sends their elements in. The server loads them from the vanilla data pack the way
 * WorldLoader does (the pack in client.jar, its static registries' tags, then RegistryDataLoader.WORLD_REGISTRIES), and
 * sends each registry's elements in its order, which the client registers them in. The walk capture's registry data
 * came in this order. Values of those registries are their holders, encoded as their keys.
 *
 * <p>The suite {@code net.minecraft.core.RegistryAccess} writes the synchronized registries, which the port's tests
 * build the buffers' registries from, and the static registries the port has ported, which its own registries must
 * equal.
 */
final class RegistryValues {
    // The static registries the port has, which the suite writes for its test to compare.
    private static final List<ResourceKey<? extends Registry<?>>> PORTED_STATIC = List.of(Registries.ATTRIBUTE);

    // The registry a Holder<T> argument's elements come from, by T.
    private static final Map<Class<?>, ResourceKey<? extends Registry<?>>> HOLDER_REGISTRIES = Map.ofEntries(
        Map.entry(Attribute.class, Registries.ATTRIBUTE),
        Map.entry(WorldClock.class, Registries.WORLD_CLOCK),
        Map.entry(CatVariant.class, Registries.CAT_VARIANT),
        Map.entry(CatSoundVariant.class, Registries.CAT_SOUND_VARIANT),
        Map.entry(CowVariant.class, Registries.COW_VARIANT),
        Map.entry(CowSoundVariant.class, Registries.COW_SOUND_VARIANT),
        Map.entry(WolfVariant.class, Registries.WOLF_VARIANT),
        Map.entry(WolfSoundVariant.class, Registries.WOLF_SOUND_VARIANT),
        Map.entry(FrogVariant.class, Registries.FROG_VARIANT),
        Map.entry(PigVariant.class, Registries.PIG_VARIANT),
        Map.entry(PigSoundVariant.class, Registries.PIG_SOUND_VARIANT),
        Map.entry(ChickenVariant.class, Registries.CHICKEN_VARIANT),
        Map.entry(ChickenSoundVariant.class, Registries.CHICKEN_SOUND_VARIANT),
        Map.entry(ZombieNautilusVariant.class, Registries.ZOMBIE_NAUTILUS_VARIANT)
    );

    private static final Map<ResourceKey<?>, Values.Generator> HOLDERS = new HashMap<>();

    // Loading the server's registries takes a second or two, so only suites that read them do.
    private static final class Loaded {
        static final RegistryAccess.Frozen REGISTRIES = load();
    }

    /**
     * RegistryFriendlyByteBufs: FriendlyByteBuf's buffers, which each call gets with the registries. Their specs are
     * FriendlyByteBufs, so making them needs no registries.
     */
    static final Values.Generator REGISTRY_FRIENDLY_BUFFER = new Values.Generator() {
        @Override
        public List<Object> edges() {
            return NetworkValues.FRIENDLY_BUFFER.edges();
        }

        @Override
        public Object random(final Random random) {
            return NetworkValues.FRIENDLY_BUFFER.random(random);
        }

        @Override
        public JsonElement encode(final Object spec) {
            return NetworkValues.FRIENDLY_BUFFER.encode(spec);
        }

        @Override
        public Object instantiate(final Object spec) {
            return wrap(NetworkValues.copy(((FriendlyByteBuf)spec).unwrap()));
        }

        @Override
        public JsonElement after(final Object spec, final Object instance) {
            JsonElement state = this.encode(instance);
            return state.equals(this.encode(spec)) ? null : state;
        }
    };

    private RegistryValues() {
    }

    /**
     * A generator made the first time it's used, for values that need the game bootstrapped (Golden's constants ask for
     * generators before main bootstraps it) or the registries loaded.
     */
    static Values.Generator lazy(final Supplier<Values.Generator> make) {
        return new Values.Generator() {
            private Values.Generator made;

            private Values.Generator made() {
                if (this.made == null) {
                    this.made = make.get();
                }
                return this.made;
            }

            @Override
            public List<Object> edges() {
                return this.made().edges();
            }

            @Override
            public Object random(final Random random) {
                return this.made().random(random);
            }

            @Override
            public JsonElement encode(final Object spec) {
                return this.made().encode(spec);
            }

            @Override
            public Object instantiate(final Object spec) {
                return this.made().instantiate(spec);
            }

            @Override
            public JsonElement after(final Object spec, final Object instance) {
                return this.made().after(spec, instance);
            }

            @Override
            public boolean exhaustive() {
                return this.made().exhaustive();
            }
        };
    }

    static RegistryAccess.Frozen registries() {
        return Loaded.REGISTRIES;
    }

    static RegistryFriendlyByteBuf wrap(final ByteBuf buffer) {
        return new RegistryFriendlyByteBuf(buffer, registries());
    }

    /** A buffer holding the bytes, which a RegistryFriendlyByteBuf argument gets with the registries. */
    static FriendlyByteBuf buffer(final byte[] bytes) {
        return new FriendlyByteBuf(NetworkValues.buffer(0, 0, bytes));
    }

    private static RegistryAccess.Frozen load() {
        CloseableResourceManager resources = new MultiPackResourceManager(
            PackType.SERVER_DATA, ServerPacksSource.createVanillaPackSource().asResourcesSupplier().openResources(null, null).toList()
        );
        LayeredRegistryAccess<RegistryLayer> initialLayers = RegistryLayer.createRegistryAccess();
        List<Registry.PendingTags<?>> staticLayerTags = TagLoader.loadTagsForExistingRegistries(resources, initialLayers.getLayer(RegistryLayer.STATIC));
        List<HolderLookup.RegistryLookup<?>> worldContextRegistries = TagLoader.buildUpdatedLookups(
            initialLayers.getAccessForLoading(RegistryLayer.WORLD), staticLayerTags
        );
        RegistryAccess.Frozen loaded = RegistryDataLoader.load(resources, worldContextRegistries, RegistryDataLoader.WORLD_REGISTRIES, Runnable::run).join();
        List<Registry<?>> registries = new ArrayList<>();
        BuiltInRegistries.REGISTRY.forEach(registries::add);
        for (RegistryDataLoader.RegistryData<?> data : RegistryDataLoader.SYNCHRONIZED_REGISTRIES) {
            registries.add(loaded.lookupOrThrow(data.key()));
        }
        return new RegistryAccess.ImmutableRegistryAccess(registries).freeze();
    }

    static Values.Generator generator(final Class<?> type) {
        return type == RegistryFriendlyByteBuf.class ? REGISTRY_FRIENDLY_BUFFER : null;
    }

    /** A Holder of one of HOLDER_REGISTRIES' classes is an element of its registry, and a map of world clocks a clock map. */
    static Values.Generator generator(final ParameterizedType type) {
        Object element = type.getActualTypeArguments()[0];
        if (type.getRawType() == Holder.class && element instanceof Class<?> value && HOLDER_REGISTRIES.containsKey(value)) {
            return holders(HOLDER_REGISTRIES.get(value));
        }
        if (
            type.getRawType() == Map.class && element instanceof ParameterizedType holder && holder.getRawType() == Holder.class
                && holder.getActualTypeArguments()[0] == WorldClock.class
        ) {
            return CLOCK_UPDATES;
        }
        return null;
    }

    static boolean canEncode(final Class<?> type) {
        return Holder.class.isAssignableFrom(type) || type == SynchedEntityData.DataValue.class;
    }

    static JsonElement encode(final Object value) {
        return switch (value) {
            case Holder<?> holder -> CoreValues.RESOURCE_KEY.encode(holder.unwrapKey().orElseThrow());
            case SynchedEntityData.DataValue<?> item -> DATA_VALUES.encode(item);
            default -> throw new IllegalArgumentException("no encoder for " + value.getClass().getName());
        };
    }

    /** Every element of the registry, in id order, and a random one. */
    static Values.Generator holders(final ResourceKey<? extends Registry<?>> key) {
        return HOLDERS.computeIfAbsent(key, k -> {
            List<Object> elements = holdersOf(registries().lookupOrThrow(key));
            return new Values.Generator() {
                @Override
                public List<Object> edges() {
                    return elements;
                }

                @Override
                public Object random(final Random random) {
                    return elements.get(random.nextInt(elements.size()));
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
        });
    }

    private static <T> List<Object> holdersOf(final Registry<T> registry) {
        List<Object> holders = new ArrayList<>();
        for (int id = 0; id < registry.size(); id++) {
            holders.add(registry.get(id).orElseThrow());
        }
        return holders;
    }

    /**
     * A ClockNetworkState: a clock's ticks, from the start and the overworld's day time (6000 at the walk's noon), and
     * its partial tick and rate, 0 when time is stopped like the walk's.
     */
    static final Values.Generator CLOCK_STATES = new Values.Simple(
        List.of(
            new ClockNetworkState(6000L, 0.0F, 0.0F), new ClockNetworkState(1L, 0.0F, 0.0F), new ClockNetworkState(0L, 0.0F, 1.0F),
            new ClockNetworkState(24000L, 0.5F, 1.0F), new ClockNetworkState(-1L, -0.0F, -1.0F),
            new ClockNetworkState(Long.MAX_VALUE, Float.NaN, Float.POSITIVE_INFINITY), new ClockNetworkState(Long.MIN_VALUE, 1.0F, 20.0F)
        ),
        random -> new ClockNetworkState(
            (Long)Values.generator(long.class).random(random), (Float)Values.FLOATS.random(random), (Float)Values.FLOATS.random(random)
        ),
        state -> Values.encode(state, ClockNetworkState.class)
    );

    /**
     * A ClientboundSetTimePacket's clock updates, a HashMap of world clocks' holders, which hash by identity: its order
     * changes from run to run, so the encoding sorts its entries by key (HashMapSuite.encode), and a codec writing one
     * only gets maps of one clock or none. Two clocks in a row, the way the walk's server sends them, are decoding's extra
     * inputs.
     */
    static final Values.Generator CLOCK_UPDATES = new Values.Generator() {
        @Override
        public List<Object> edges() {
            List<Object> edges = new ArrayList<>();
            edges.add(new HashMap<>());
            for (Object clock : holders(Registries.WORLD_CLOCK).edges()) {
                for (Object state : CLOCK_STATES.edges()) {
                    HashMap<Object, Object> map = new HashMap<>();
                    map.put(clock, state);
                    edges.add(map);
                }
            }
            return edges;
        }

        @Override
        public Object random(final Random random) {
            HashMap<Object, Object> map = new HashMap<>();
            if (random.nextInt(4) != 0) {
                map.put(holders(Registries.WORLD_CLOCK).random(random), CLOCK_STATES.random(random));
            }
            return map;
        }

        @Override
        public JsonElement encode(final Object spec) {
            return HashMapSuite.encode((HashMap<?, ?>)spec);
        }

        @Override
        @SuppressWarnings("unchecked")
        public Object instantiate(final Object spec) {
            return HashMapSuite.copy((HashMap<Object, Object>)spec);
        }
    };

    /** Holders' entries sorted by their keys, for a map whose order changes from run to run (see CLOCK_UPDATES). */
    static List<Map.Entry<?, ?>> sortedByHolder(final Map<?, ?> map) {
        List<Map.Entry<?, ?>> entries = new ArrayList<>(map.entrySet());
        entries.sort(Comparator.comparing(entry -> ((Holder<?>)entry.getKey()).unwrapKey().orElseThrow().identifier()));
        return entries;
    }

    static boolean keyedByHolders(final Map<?, ?> map) {
        return !map.isEmpty() && map.keySet().stream().allMatch(key -> key instanceof Holder<?>);
    }

    // A Pose by its id, like any enum's constant.
    static final Values.Generator POSE = new CoreValues.Constants<>(Pose.class);
    static final Values.Generator HUMANOID_ARM = new CoreValues.Constants<>(HumanoidArm.class);

    // Rotations (an armor stand's parts), which its constructor takes modulo 360, NaN and infinities to 0.
    static final Values.Generator ROTATIONS = new Values.Simple(
        List.of(
            new Rotations(0.0F, 0.0F, 0.0F), new Rotations(-10.0F, 0.0F, -10.0F), new Rotations(-0.0F, 360.0F, -360.0F),
            new Rotations(720.5F, -1.0E6F, Float.MIN_VALUE), new Rotations(Float.NaN, Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY)
        ),
        random -> new Rotations((Float)Values.FLOATS.random(random), (Float)Values.FLOATS.random(random), (Float)Values.FLOATS.random(random)),
        rotations -> Values.encode(rotations, Rotations.class)
    );

    static final Values.Generator GLOBAL_POS = lazy(() -> Bootstrapped.GLOBAL_POS);

    // What needs the game bootstrapped: dimensions' keys, block states and the entity data serializers.
    private static final class Bootstrapped {
        // The dimensions' keys, one with odd parts, and any in the dimension registry, which is all GlobalPos's codec writes.
        static final Values.Generator DIMENSIONS = new Values.Simple(
            List.of(Level.OVERWORLD, Level.NETHER, Level.END, ResourceKey.create(Registries.DIMENSION, Identifier.fromNamespaceAndPath("", ""))),
            random -> ResourceKey.create(Registries.DIMENSION, (Identifier)CoreValues.IDENTIFIER.instantiate(CoreValues.IDENTIFIER.random(random))),
            key -> CoreValues.RESOURCE_KEY.encode(key)
        );

        // A block in a dimension: the walk's spawn in the overworld first, then each dimension's and each block's edges.
        @SuppressWarnings("unchecked")
        static final Values.Generator GLOBAL_POS = new Values.Simple(
            Stream.concat(
                Stream.of((Object)GlobalPos.of(Level.OVERWORLD, new BlockPos(0, -60, 0))),
                Stream.concat(
                    DIMENSIONS.edges().stream().map(key -> (Object)GlobalPos.of((ResourceKey<Level>)key, BlockPos.ZERO)),
                    CoreValues.BLOCK_POS.edges().stream().map(pos -> (Object)GlobalPos.of(Level.OVERWORLD, (BlockPos)pos))
                )
            ).toList(),
            random -> GlobalPos.of(
                (ResourceKey<Level>)DIMENSIONS.random(random), (BlockPos)CoreValues.BLOCK_POS.instantiate(CoreValues.BLOCK_POS.random(random))
            ),
            pos -> Values.encode(pos, GlobalPos.class)
        );

        // Air, stone, the walk's grass and dirt, the last state, and any state.
        static final Values.Generator BLOCK_STATES = new Values.Simple(
            Stream.of(0, 1, 8, 10, Block.BLOCK_STATE_REGISTRY.size() - 1).map(id -> (Object)Block.stateById(id)).toList(),
            random -> Block.stateById(random.nextInt(Block.BLOCK_STATE_REGISTRY.size())),
            state -> new JsonPrimitive(Block.getId((BlockState)state))
        );

        // An OptionalInt, a parrot's variant: none, 0, and any non-negative int (its codec writes one more).
        static Values.Generator optionalInts() {
            return new Values.Simple(
                List.of(OptionalInt.empty(), OptionalInt.of(0), OptionalInt.of(1), OptionalInt.of(Integer.MAX_VALUE - 1), OptionalInt.of(Integer.MAX_VALUE)),
                random -> random.nextInt(4) == 0 ? OptionalInt.empty() : OptionalInt.of(random.nextInt(2) == 0 ? random.nextInt(5) : random.nextInt() >>> 1),
                value -> Values.encode(value, OptionalInt.class)
            );
        }

        // The serializers the port has, each with its values. The others (components, item stacks, particles, villager data,
        // entity references, the enums of armadillos, sniffers, copper and copper golems, dye colours, painting variants and
        // profiles) are what a buffer must not reach to be decoded (reachesUnported).
        static final Map<EntityDataSerializer<?>, Values.Generator> SERIALIZERS = new LinkedHashMap<>();

        static {
            SERIALIZERS.put(EntityDataSerializers.BYTE, Values.generator(byte.class));
            SERIALIZERS.put(EntityDataSerializers.INT, Values.INTS);
            SERIALIZERS.put(EntityDataSerializers.LONG, Values.generator(long.class));
            SERIALIZERS.put(EntityDataSerializers.FLOAT, Values.FLOATS);
            SERIALIZERS.put(EntityDataSerializers.STRING, Values.STRINGS);
            SERIALIZERS.put(EntityDataSerializers.BOOLEAN, Values.generator(boolean.class));
            SERIALIZERS.put(EntityDataSerializers.ROTATIONS, ROTATIONS);
            SERIALIZERS.put(EntityDataSerializers.BLOCK_POS, CoreValues.BLOCK_POS);
            SERIALIZERS.put(EntityDataSerializers.OPTIONAL_BLOCK_POS, Codecs.optionalOf(CoreValues.BLOCK_POS));
            SERIALIZERS.put(EntityDataSerializers.DIRECTION, CoreValues.DIRECTION);
            SERIALIZERS.put(EntityDataSerializers.BLOCK_STATE, BLOCK_STATES);
            SERIALIZERS.put(EntityDataSerializers.OPTIONAL_BLOCK_STATE, Codecs.optionalOf(BLOCK_STATES));
            SERIALIZERS.put(EntityDataSerializers.OPTIONAL_UNSIGNED_INT, optionalInts());
            SERIALIZERS.put(EntityDataSerializers.POSE, POSE);
            SERIALIZERS.put(EntityDataSerializers.CAT_VARIANT, null);
            SERIALIZERS.put(EntityDataSerializers.CAT_SOUND_VARIANT, null);
            SERIALIZERS.put(EntityDataSerializers.COW_VARIANT, null);
            SERIALIZERS.put(EntityDataSerializers.COW_SOUND_VARIANT, null);
            SERIALIZERS.put(EntityDataSerializers.WOLF_VARIANT, null);
            SERIALIZERS.put(EntityDataSerializers.WOLF_SOUND_VARIANT, null);
            SERIALIZERS.put(EntityDataSerializers.FROG_VARIANT, null);
            SERIALIZERS.put(EntityDataSerializers.PIG_VARIANT, null);
            SERIALIZERS.put(EntityDataSerializers.PIG_SOUND_VARIANT, null);
            SERIALIZERS.put(EntityDataSerializers.CHICKEN_VARIANT, null);
            SERIALIZERS.put(EntityDataSerializers.CHICKEN_SOUND_VARIANT, null);
            SERIALIZERS.put(EntityDataSerializers.ZOMBIE_NAUTILUS_VARIANT, null);
            SERIALIZERS.put(EntityDataSerializers.OPTIONAL_GLOBAL_POS, Codecs.optionalOf(GLOBAL_POS));
            SERIALIZERS.put(EntityDataSerializers.VECTOR3, JomlValues.VECTOR3F);
            SERIALIZERS.put(EntityDataSerializers.QUATERNION, JomlValues.QUATERNIONF);
            SERIALIZERS.put(EntityDataSerializers.HUMANOID_ARM, HUMANOID_ARM);
        }
    }

    // The variants' holders, each from its registry, made when the registries are first needed.
    private static Values.Generator serializerValues(final EntityDataSerializer<?> serializer) {
        Values.Generator values = Bootstrapped.SERIALIZERS.get(serializer);
        if (values != null) {
            return values;
        }
        ResourceKey<? extends Registry<?>> registry = variantRegistry(serializer);
        return holders(registry);
    }

    private static ResourceKey<? extends Registry<?>> variantRegistry(final EntityDataSerializer<?> serializer) {
        if (serializer == EntityDataSerializers.CAT_VARIANT) {
            return Registries.CAT_VARIANT;
        }
        if (serializer == EntityDataSerializers.CAT_SOUND_VARIANT) {
            return Registries.CAT_SOUND_VARIANT;
        }
        if (serializer == EntityDataSerializers.COW_VARIANT) {
            return Registries.COW_VARIANT;
        }
        if (serializer == EntityDataSerializers.COW_SOUND_VARIANT) {
            return Registries.COW_SOUND_VARIANT;
        }
        if (serializer == EntityDataSerializers.WOLF_VARIANT) {
            return Registries.WOLF_VARIANT;
        }
        if (serializer == EntityDataSerializers.WOLF_SOUND_VARIANT) {
            return Registries.WOLF_SOUND_VARIANT;
        }
        if (serializer == EntityDataSerializers.FROG_VARIANT) {
            return Registries.FROG_VARIANT;
        }
        if (serializer == EntityDataSerializers.PIG_VARIANT) {
            return Registries.PIG_VARIANT;
        }
        if (serializer == EntityDataSerializers.PIG_SOUND_VARIANT) {
            return Registries.PIG_SOUND_VARIANT;
        }
        if (serializer == EntityDataSerializers.CHICKEN_VARIANT) {
            return Registries.CHICKEN_VARIANT;
        }
        if (serializer == EntityDataSerializers.CHICKEN_SOUND_VARIANT) {
            return Registries.CHICKEN_SOUND_VARIANT;
        }
        if (serializer == EntityDataSerializers.ZOMBIE_NAUTILUS_VARIANT) {
            return Registries.ZOMBIE_NAUTILUS_VARIANT;
        }
        throw new IllegalArgumentException("no values for serializer " + EntityDataSerializers.getSerializedId(serializer));
    }

    /**
     * An entity's data value, {@code [id, serializer, value]}: its id in the entity's data, its serializer's id in
     * EntityDataSerializers, and the value as that serializer's values are encoded. Ids are the local player's (0, its
     * flags; 6, its pose; 9, its health; 16, its skin's parts) and the byte's limits, 254 being the last before the end
     * marker.
     */
    static final Values.Generator DATA_VALUES = new Values.Generator() {
        private static final List<Integer> IDS = List.of(0, 6, 9, 16, 254);

        @Override
        public List<Object> edges() {
            List<Object> edges = new ArrayList<>();
            for (EntityDataSerializer<?> serializer : Bootstrapped.SERIALIZERS.keySet()) {
                for (Object value : serializerValues(serializer).edges()) {
                    edges.add(dataValue(0, serializer, value));
                }
            }
            for (int id : IDS) {
                edges.add(dataValue(id, EntityDataSerializers.BYTE, (byte)0));
            }
            return edges;
        }

        @Override
        public Object random(final Random random) {
            List<EntityDataSerializer<?>> serializers = new ArrayList<>(Bootstrapped.SERIALIZERS.keySet());
            EntityDataSerializer<?> serializer = serializers.get(random.nextInt(serializers.size()));
            int id = random.nextBoolean() ? IDS.get(random.nextInt(IDS.size())) : random.nextInt(255);
            return dataValue(id, serializer, serializerValues(serializer).random(random));
        }

        @Override
        public JsonElement encode(final Object spec) {
            SynchedEntityData.DataValue<?> item = (SynchedEntityData.DataValue<?>)spec;
            JsonArray array = new JsonArray();
            array.add(item.id());
            array.add(EntityDataSerializers.getSerializedId(item.serializer()));
            array.add(serializerValues(item.serializer()).encode(item.value()));
            return array;
        }

        @Override
        public Object instantiate(final Object spec) {
            SynchedEntityData.DataValue<?> item = (SynchedEntityData.DataValue<?>)spec;
            return dataValue(item.id(), item.serializer(), serializerValues(item.serializer()).instantiate(item.value()));
        }
    };

    @SuppressWarnings("unchecked")
    private static SynchedEntityData.DataValue<?> dataValue(final int id, final EntityDataSerializer<?> serializer, final Object value) {
        return new SynchedEntityData.DataValue<>(id, (EntityDataSerializer<Object>)serializer, value);
    }

    /**
     * Whether reading the buffer the way ClientboundSetEntityDataPacket does (a packet's), or DataValue.read does (one
     * value whose id is read already), reaches a value whose serializer the port hasn't got before it ends or throws.
     * Such buffers aren't decoded.
     */
    static boolean reachesUnported(final Object spec, final boolean packet) {
        RegistryFriendlyByteBuf input = wrap(NetworkValues.copy(((FriendlyByteBuf)spec).unwrap()));
        try {
            if (packet) {
                input.readVarInt();
            }
            while (!packet || input.readUnsignedByte() != 255) {
                int type = input.readVarInt();
                EntityDataSerializer<?> serializer = EntityDataSerializers.getSerializer(type);
                if (serializer == null) {
                    return false;
                }
                if (!Bootstrapped.SERIALIZERS.containsKey(serializer)) {
                    return true;
                }
                serializer.codec().decode(input);
                if (!packet) {
                    return false;
                }
            }
            return false;
        } catch (RuntimeException e) {
            return false;
        }
    }

    /** Buffers holding data values written like the server writes them, for DataValue.read and the packet. */
    static List<byte[]> dataValueBuffers() {
        List<byte[]> buffers = new ArrayList<>();
        // The local player's own data while it joins and walks in the walk capture: its health and skin's parts, its
        // flags (sprinting), its pose (crouching and back).
        buffers.add(NetworkValues.bytes(0x01, 0x09, 0x03, 0x41, 0xa0, 0x00, 0x00, 0x10, 0x00, 0x7f, 0xff));
        buffers.add(NetworkValues.bytes(0x01, 0x00, 0x00, 0x08, 0xff));
        buffers.add(NetworkValues.bytes(0x01, 0x06, 0x14, 0x05, 0xff));
        buffers.add(NetworkValues.bytes(0x01, 0x06, 0x14, 0x00, 0xff));
        // No values, a serializer id past the last, and an end marker missing.
        buffers.add(NetworkValues.bytes(0x01, 0xff));
        buffers.add(NetworkValues.bytes(0x01, 0x00, 0x2c, 0x00, 0xff));
        buffers.add(NetworkValues.bytes(0x01, 0x00, 0x00, 0x08));
        return buffers;
    }

    /** {@code net.minecraft.core.RegistryAccess}: the synchronized registries, and the static ones the port has. */
    static final class RegistriesSuite implements Suite {
        @Override
        public String name() {
            return "net.minecraft.core.RegistryAccess";
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            JsonArray synchronizedRegistries = new JsonArray();
            for (RegistryDataLoader.RegistryData<?> data : RegistryDataLoader.SYNCHRONIZED_REGISTRIES) {
                JsonArray registry = new JsonArray();
                registry.add(CoreValues.IDENTIFIER.encode(data.key().identifier()));
                registry.add(identifiers(registries().lookupOrThrow(data.key())));
                synchronizedRegistries.add(registry);
            }
            writer.field("synchronized", synchronizedRegistries);
            for (ResourceKey<? extends Registry<?>> key : PORTED_STATIC) {
                writer.field(key.identifier().toString(), identifiers(registries().lookupOrThrow(key)));
            }
        }

        private static <T> JsonArray identifiers(final Registry<T> registry) {
            JsonArray ids = new JsonArray();
            for (int id = 0; id < registry.size(); id++) {
                Identifier identifier = registry.getKey(registry.byId(id));
                ids.add(CoreValues.IDENTIFIER.encode(identifier));
            }
            return ids;
        }
    }

    /**
     * {@code net.minecraft.client.ClientClockManager}: programs of {@code tick}, {@code handleUpdates} (of one clock or
     * none, a HashMap's order being its holders' identity hashes') and {@code getInstance}, whose output is the clock's
     * {@code [totalTicks, partialTick, rate, isPaused]}. Game times mostly go up a tick or a few at a time, sometimes
     * jump or go back; rates are 0 (time stopped), 1, fractions and more; partial ticks are in [0, 1) or anything.
     */
    static final class ClockManagerSuite implements Suite {
        private static final int PROGRAMS = 400;

        @Override
        public String name() {
            return "net.minecraft.client.ClientClockManager";
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(19);
            List<Object> clocks = holders(Registries.WORLD_CLOCK).edges();
            float[] rates = {0.0F, 1.0F, 0.5F, 2.0F, 0.1F, 3.0F, 1.0E-4F, -1.0F, 72.0F};
            for (int program = 0; program < PROGRAMS; program++) {
                ClientClockManager manager = new ClientClockManager();
                JsonArray ops = new JsonArray();
                JsonArray outputs = new JsonArray();
                long gameTime = random.nextInt(4) == 0 ? random.nextLong() : random.nextInt(100000);
                for (int step = random.nextInt(40); step > 0; step--) {
                    JsonArray op = new JsonArray();
                    gameTime += switch (random.nextInt(8)) {
                        case 0 -> random.nextInt(1000);
                        case 1 -> -random.nextInt(50);
                        case 2 -> 0;
                        default -> 1 + random.nextInt(3);
                    };
                    Holder<?> clock = (Holder<?>)clocks.get(random.nextInt(clocks.size()));
                    switch (random.nextInt(3)) {
                        case 0 -> {
                            op.add("tick");
                            op.add(gameTime);
                            manager.tick(gameTime);
                            outputs.add(com.google.gson.JsonNull.INSTANCE);
                        }
                        case 1 -> {
                            HashMap<Holder<WorldClock>, ClockNetworkState> updates = new HashMap<>();
                            JsonArray entries = new JsonArray();
                            if (random.nextInt(4) != 0) {
                                ClockNetworkState state = new ClockNetworkState(
                                    random.nextInt(3) == 0 ? random.nextLong() : random.nextInt(48000),
                                    random.nextInt(3) == 0 ? random.nextFloat() * 2.0F - 0.5F : random.nextFloat(),
                                    rates[random.nextInt(rates.length)]
                                );
                                updates.put(holder(clock), state);
                                JsonArray entry = new JsonArray();
                                entry.add(encode(clock));
                                entry.add(Values.encode(state, ClockNetworkState.class));
                                entries.add(entry);
                            }
                            op.add("handleUpdates");
                            op.add(gameTime);
                            op.add(entries);
                            manager.handleUpdates(gameTime, updates);
                            outputs.add(com.google.gson.JsonNull.INSTANCE);
                        }
                        default -> {
                            op.add("getInstance");
                            op.add(encode(clock));
                            ClientClockManager.ClientClockInstance instance = manager.getInstance(holder(clock));
                            JsonArray state = new JsonArray();
                            state.add(instance.totalTicks());
                            state.add(Values.encode(instance.partialTick(), float.class));
                            state.add(Values.encode(instance.rate(), float.class));
                            state.add(instance.isPaused());
                            outputs.add(state);
                        }
                    }
                    ops.add(op);
                }
                JsonArray args = new JsonArray();
                args.add(ops);
                writer.result("programs", args, outputs, null);
            }
        }

        @SuppressWarnings("unchecked")
        private static Holder<WorldClock> holder(final Holder<?> clock) {
            return (Holder<WorldClock>)clock;
        }
    }
}
