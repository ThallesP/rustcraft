package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import com.mojang.serialization.Codec;
import com.mojang.serialization.DataResult;
import io.netty.buffer.Unpooled;
import java.io.IOException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Function;
import net.minecraft.client.multiplayer.ClientRegistryLayer;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderGetter;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.HolderSet;
import net.minecraft.core.LayeredRegistryAccess;
import net.minecraft.core.MappedRegistry;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.RegistrySynchronization;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.IntTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.NbtOps;
import net.minecraft.nbt.StringTag;
import net.minecraft.nbt.Tag;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.chat.ChatType;
import net.minecraft.network.chat.ChatTypeDecoration;
import net.minecraft.network.protocol.common.ClientboundUpdateTagsPacket;
import net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket;
import net.minecraft.resources.RegistryDataLoader;
import net.minecraft.resources.RegistryOps;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.packs.PackType;
import net.minecraft.server.packs.repository.ServerPacksSource;
import net.minecraft.server.packs.resources.CloseableResourceManager;
import net.minecraft.server.packs.resources.MultiPackResourceManager;
import net.minecraft.tags.TagLoader;
import net.minecraft.tags.TagNetworkSerialization;
import net.minecraft.world.attribute.EnvironmentAttribute;
import net.minecraft.world.attribute.EnvironmentAttributeMap;
import net.minecraft.world.damagesource.DamageType;
import net.minecraft.world.entity.animal.wolf.WolfVariant;
import net.minecraft.world.entity.decoration.painting.PaintingVariant;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.dimension.DimensionType;
import com.mojang.serialization.Lifecycle;

/**
 * The synchronized registries as the client loads them ({@code RegistryDataLoader}'s network path, with the tags the walk
 * capture's {@code update_tags} sent): the walk capture's registry data, each element read from the vanilla pack like a
 * known pack's ({@code json}), and from the NBT a server
 * writes of its own element with the registry's network codec ({@code nbt}), for the registries whose classes the port
 * has. {@code errors}: that NBT changed so the codec fails, and the codec's message. See TESTING.md ("The synchronized
 * registries").
 */
final class RegistrySuites {
    private RegistrySuites() {
    }

    static List<Suite> suites() {
        return List.of(new LoaderSuite());
    }

    /** The registries whose elements' fields the suite writes, and how. */
    private static final Map<ResourceKey<? extends Registry<?>>, Function<Object, JsonElement>> FIELDS = new LinkedHashMap<>();

    static {
        FIELDS.put(Registries.BIOME, value -> {
            JsonArray fields = BiomeSuites.fields((Biome)value);
            fields.add(attributes(((Biome)value).getAttributes()));
            return fields;
        });
        FIELDS.put(Registries.CHAT_TYPE, value -> {
            ChatType type = (ChatType)value;
            return ChunkSuites.array(decoration(type.chat()), decoration(type.narration()));
        });
        FIELDS.put(Registries.WOLF_VARIANT, value -> {
            WolfVariant variant = (WolfVariant)value;
            return ChunkSuites.array(assets(variant.adultInfo()), assets(variant.babyInfo()));
        });
        FIELDS.put(Registries.PAINTING_VARIANT, value -> {
            PaintingVariant variant = (PaintingVariant)value;
            return ChunkSuites.array(
                variant.width(), variant.height(), variant.assetId().toString(), component(variant.title()), component(variant.author())
            );
        });
        FIELDS.put(Registries.DIMENSION_TYPE, value -> {
            DimensionType type = (DimensionType)value;
            return ChunkSuites.array(
                type.hasFixedTime(), type.hasSkyLight(), type.hasCeiling(), type.hasEnderDragonFight(), BiomeSuites.hex(type.coordinateScale()), type.minY(),
                type.height(), type.logicalHeight(), holderSet(type.infiniburn()), String.format("%08x", Float.floatToRawIntBits(type.ambientLight())),
                type.monsterSpawnLightTest().toString(), type.monsterSpawnBlockLightLimit(), type.skybox().getSerializedName(),
                type.cardinalLightType().getSerializedName(), attributes(type.attributes()), holderSet(type.timelines()),
                type.defaultClock().<JsonElement>map(clock -> new JsonPrimitive(clock.unwrapKey().orElseThrow().identifier().toString())).orElse(JsonNull.INSTANCE)
            );
        });
        FIELDS.put(Registries.DAMAGE_TYPE, value -> {
            DamageType type = (DamageType)value;
            return ChunkSuites.array(
                type.msgId(), type.scaling().getSerializedName(), String.format("%08x", Float.floatToRawIntBits(type.exhaustion())),
                type.effects().getSerializedName(), type.deathMessageType().getSerializedName()
            );
        });
    }

    /** The map's attributes, by registered name, sorted. */
    private static JsonArray attributes(final EnvironmentAttributeMap map) {
        JsonArray names = new JsonArray();
        map.keySet().stream().map(attribute -> attribute.toString()).sorted().forEach(names::add);
        return names;
    }

    /** A tag as {@code #tag}, a direct set as its elements' identifiers. */
    private static JsonElement holderSet(final HolderSet<?> set) {
        Optional<? extends net.minecraft.tags.TagKey<?>> key = set.unwrapKey();
        if (key.isPresent()) {
            return new JsonPrimitive("#" + key.get().location());
        }
        JsonArray elements = new JsonArray();
        set.stream().forEach(holder -> elements.add(holder.unwrapKey().orElseThrow().identifier().toString()));
        return elements;
    }

    private static JsonElement decoration(final ChatTypeDecoration decoration) {
        JsonArray parameters = new JsonArray();
        decoration.parameters().forEach(parameter -> parameters.add(parameter.getSerializedName()));
        return ChunkSuites.array(decoration.translationKey(), parameters, decoration.style().toString());
    }

    private static JsonElement assets(final WolfVariant.AssetInfo info) {
        return ChunkSuites.array(
            info.wild().id().toString(), info.wild().texturePath().toString(), info.tame().id().toString(), info.tame().texturePath().toString(),
            info.angry().id().toString(), info.angry().texturePath().toString()
        );
    }

    private static JsonElement component(final Optional<net.minecraft.network.chat.Component> component) {
        return component.<JsonElement>map(c -> new JsonPrimitive(c.toString())).orElse(JsonNull.INSTANCE);
    }

    /** The walk capture's {@code update_tags}, by registry. */
    private static Map<ResourceKey<? extends Registry<?>>, TagNetworkSerialization.NetworkPayload> tags() {
        List<byte[]> payloads = LevelChunkSuites.payloads("minecraft:update_tags");
        return ClientboundUpdateTagsPacket.STREAM_CODEC.decode(Unpooled.wrappedBuffer(payloads.getFirst())).tags();
    }

    /**
     * The client's loading context: its static registries with the tags {@code update_tags} sent for them, as
     * {@code RegistryDataCollector.loadNewElementsAndTags} builds it.
     */
    private static List<HolderLookup.RegistryLookup<?>> context() {
        LayeredRegistryAccess<ClientRegistryLayer> base = ClientRegistryLayer.createRegistryAccess();
        RegistryAccess.Frozen loadingContext = base.getAccessForLoading(ClientRegistryLayer.REMOTE);
        List<Registry.PendingTags<?>> staticTags = new ArrayList<>();
        tags().forEach((key, tags) -> {
            if (!tags.isEmpty() && !RegistrySynchronization.isNetworkable(key)) {
                staticTags.add(pendingTags(loadingContext, key, tags));
            }
        });
        return TagLoader.buildUpdatedLookups(loadingContext, staticTags);
    }

    /** {@code resolveRegistryTags}. */
    @SuppressWarnings({"unchecked", "rawtypes"})
    private static Registry.PendingTags<?> pendingTags(
        final RegistryAccess.Frozen context, final ResourceKey<? extends Registry<?>> key, final TagNetworkSerialization.NetworkPayload tags
    ) {
        Registry staticRegistry = context.lookupOrThrow((ResourceKey)key);
        return staticRegistry.prepareTagReload(tags.resolve(staticRegistry));
    }

    /** The walk capture's registry data, by registry, in the order it came. */
    private static Map<ResourceKey<? extends Registry<?>>, List<RegistrySynchronization.PackedRegistryEntry>> captured() {
        Map<ResourceKey<? extends Registry<?>>, List<RegistrySynchronization.PackedRegistryEntry>> registries = new LinkedHashMap<>();
        for (byte[] payload : LevelChunkSuites.payloads("minecraft:registry_data")) {
            ClientboundRegistryDataPacket packet = ClientboundRegistryDataPacket.STREAM_CODEC.decode(new FriendlyByteBuf(Unpooled.wrappedBuffer(payload)));
            registries.computeIfAbsent(packet.registry(), key -> new ArrayList<>()).addAll(packet.entries());
        }
        return registries;
    }

    private static RegistryAccess.Frozen load(
        final Map<ResourceKey<? extends Registry<?>>, List<RegistrySynchronization.PackedRegistryEntry>> entries, final CloseableResourceManager resources
    ) {
        // loadNewElementsAndTags: each registry's elements, and the synchronized registries' tags.
        Map<ResourceKey<? extends Registry<?>>, RegistryDataLoader.NetworkedRegistryData> toLoad = new HashMap<>();
        entries.forEach((key, elements) -> toLoad.put(key, new RegistryDataLoader.NetworkedRegistryData(elements, TagNetworkSerialization.NetworkPayload.EMPTY)));
        tags().forEach((key, tags) -> {
            if (!tags.isEmpty() && RegistrySynchronization.isNetworkable(key)) {
                toLoad.compute(key, (k, previous) -> new RegistryDataLoader.NetworkedRegistryData(previous != null ? previous.elements() : List.of(), tags));
            }
        });
        return RegistryDataLoader.load(toLoad, resources, context(), RegistryDataLoader.SYNCHRONIZED_REGISTRIES, Runnable::run).join();
    }

    /** The registry's network codec: {@code SYNCHRONIZED_REGISTRIES}' element codec. */
    @SuppressWarnings("unchecked")
    private static Codec<Object> codec(final ResourceKey<? extends Registry<?>> key) {
        for (RegistryDataLoader.RegistryData<?> data : RegistryDataLoader.SYNCHRONIZED_REGISTRIES) {
            if (data.key().equals(key)) {
                return (Codec<Object>)data.elementCodec();
            }
        }
        throw new IllegalArgumentException(key.toString());
    }

    /**
     * {@code createContext}'s lookup for one element's decoding: the static registries with their tags, and for every
     * synchronized registry a registration lookup of a registry of its own, which hands out a holder for any key.
     */
    private static RegistryOps<Tag> loadingOps(final CloseableResourceManager resources) {
        Map<ResourceKey<? extends Registry<?>>, HolderGetter<?>> getters = new HashMap<>();
        context().forEach(lookup -> getters.put(lookup.key(), lookup));
        for (RegistryDataLoader.RegistryData<?> data : RegistryDataLoader.SYNCHRONIZED_REGISTRIES) {
            getters.put(data.key(), new MappedRegistry<>(data.key(), Lifecycle.stable()).createRegistrationLookup());
        }
        return RegistryOps.create(NbtOps.INSTANCE, new RegistryOps.RegistryInfoLookup() {
            @Override
            @SuppressWarnings("unchecked")
            public <T> Optional<HolderGetter<T>> lookup(final ResourceKey<? extends Registry<? extends T>> key) {
                return Optional.ofNullable((HolderGetter<T>)getters.get(key));
            }
        });
    }

    /**
     * {@code REGISTRIES}: the capture's registry data, {@code [[registry, [identifier, …]], …]}. {@code json}:
     * {@code [registry, identifier]} → the element's fields, loaded from the vanilla pack. {@code nbt}:
     * {@code [registry, identifier, nbt]} → the fields of the element loaded from the NBT a server writes of its own with the
     * network codec. {@code errors}: {@code [registry, nbt]} → the codec's error decoding it in a loading context.
     */
    static final class LoaderSuite implements Suite {
        @Override
        public String name() {
            return RegistryDataLoader.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Map<ResourceKey<? extends Registry<?>>, List<RegistrySynchronization.PackedRegistryEntry>> captured = captured();
            JsonArray registries = new JsonArray();
            captured.forEach((key, elements) -> {
                JsonArray ids = new JsonArray();
                elements.forEach(element -> ids.add(element.id().toString()));
                registries.add(ChunkSuites.array(key.identifier().toString(), ids));
            });
            writer.field("REGISTRIES", registries);

            CloseableResourceManager resources = new MultiPackResourceManager(
                PackType.SERVER_DATA, ServerPacksSource.createVanillaPackSource().asResourcesSupplier().openResources(null, null).toList()
            );
            RegistryAccess.Frozen fromPack = load(captured, resources);
            // What a server sends of its own elements: its world registries' values, by the network codecs.
            RegistryAccess.Frozen server = RegistryValues.registries();
            RegistryOps<Tag> serverOps = server.createSerializationContext(NbtOps.INSTANCE);
            Map<ResourceKey<? extends Registry<?>>, List<RegistrySynchronization.PackedRegistryEntry>> sent = new LinkedHashMap<>();
            Map<ResourceKey<? extends Registry<?>>, Map<String, Tag>> sentTags = new HashMap<>();
            captured.forEach((key, elements) -> {
                List<RegistrySynchronization.PackedRegistryEntry> withData = new ArrayList<>();
                Map<String, Tag> tags = new LinkedHashMap<>();
                for (RegistrySynchronization.PackedRegistryEntry element : elements) {
                    Tag data = null;
                    if (FIELDS.containsKey(key)) {
                        Object value = server.lookupOrThrow(key).getValueOrThrow(ResourceKey.create((ResourceKey)key, element.id()));
                        data = codec(key).encodeStart(serverOps, value).getOrThrow();
                        tags.put(element.id().toString(), data);
                    }
                    withData.add(new RegistrySynchronization.PackedRegistryEntry(element.id(), Optional.ofNullable(data)));
                }
                sent.put(key, withData);
                sentTags.put(key, tags);
            });
            RegistryAccess.Frozen fromNetwork = load(sent, resources);

            for (Map.Entry<ResourceKey<? extends Registry<?>>, Function<Object, JsonElement>> fields : FIELDS.entrySet()) {
                ResourceKey<? extends Registry<?>> key = fields.getKey();
                for (RegistrySynchronization.PackedRegistryEntry element : captured.get(key)) {
                    ResourceKey<Object> elementKey = ResourceKey.create((ResourceKey)key, element.id());
                    String registry = key.identifier().toString();
                    String id = element.id().toString();
                    writer.result("json", ChunkSuites.array(registry, id), fields.getValue().apply(fromPack.lookupOrThrow((ResourceKey)key).getValueOrThrow(elementKey)), null);
                    writer.result(
                        "nbt", ChunkSuites.array(registry, id, BiomeSuites.nbt(sentTags.get(key).get(id))),
                        fields.getValue().apply(fromNetwork.lookupOrThrow((ResourceKey)key).getValueOrThrow(elementKey)), null
                    );
                }
            }

            RegistryOps<Tag> ops = loadingOps(resources);
            List<Object[]> errors = new ArrayList<>();
            CompoundTag overworld = (CompoundTag)sentTags.get(Registries.DIMENSION_TYPE).get("minecraft:overworld");
            CompoundTag nether = (CompoundTag)sentTags.get(Registries.DIMENSION_TYPE).get("minecraft:the_nether");
            errors.add(new Object[]{Registries.DIMENSION_TYPE, with(overworld, tag -> tag.putInt("height", 100))});
            errors.add(new Object[]{Registries.DIMENSION_TYPE, with(overworld, tag -> tag.putInt("min_y", 8))});
            errors.add(new Object[]{Registries.DIMENSION_TYPE, with(overworld, tag -> tag.putInt("logical_height", 400))});
            errors.add(new Object[]{Registries.DIMENSION_TYPE, with(overworld, tag -> tag.putInt("min_y", 2016))});
            errors.add(new Object[]{Registries.DIMENSION_TYPE, with(overworld, tag -> tag.putInt("height", 5000))});
            errors.add(new Object[]{Registries.DIMENSION_TYPE, with(overworld, tag -> tag.putDouble("coordinate_scale", 0.0))});
            errors.add(new Object[]{Registries.DIMENSION_TYPE, with(nether, tag -> tag.putInt("monster_spawn_light_level", 20))});
            errors.add(new Object[]{Registries.DIMENSION_TYPE, with(overworld, tag -> {
                CompoundTag light = new CompoundTag();
                light.putString("type", "minecraft:uniform");
                light.putInt("min_inclusive", 5);
                light.putInt("max_inclusive", 2);
                tag.put("monster_spawn_light_level", light);
            })});
            errors.add(new Object[]{Registries.DIMENSION_TYPE, with(overworld, tag -> {
                CompoundTag light = new CompoundTag();
                light.putString("type", "minecraft:nope");
                tag.put("monster_spawn_light_level", light);
            })});
            errors.add(new Object[]{Registries.DIMENSION_TYPE, with(overworld, tag -> tag.putString("skybox", "moon"))});
            errors.add(new Object[]{Registries.DIMENSION_TYPE, with(overworld, tag -> tag.putInt("infiniburn", 5))});
            errors.add(new Object[]{Registries.DIMENSION_TYPE, with(overworld, tag -> tag.remove("has_skylight"))});
            CompoundTag plains = (CompoundTag)sentTags.get(Registries.BIOME).get("minecraft:plains");
            errors.add(new Object[]{Registries.BIOME, with(plains, tag -> tag.getCompoundOrEmpty("effects").putString("water_color", "#12345"))});
            errors.add(new Object[]{Registries.BIOME, with(plains, tag -> tag.getCompoundOrEmpty("effects").putString("water_color", "#12345g"))});
            errors.add(new Object[]{Registries.BIOME, with(plains, tag -> tag.putString("temperature_modifier", "warm"))});
            errors.add(new Object[]{Registries.BIOME, with(plains, tag -> tag.remove("effects"))});
            errors.add(new Object[]{Registries.BIOME, with(plains, tag -> tag.getCompoundOrEmpty("effects").put("grass_color", list(IntTag.valueOf(1))))});
            CompoundTag painting = (CompoundTag)sentTags.get(Registries.PAINTING_VARIANT).get("minecraft:alban");
            errors.add(new Object[]{Registries.PAINTING_VARIANT, with(painting, tag -> tag.putInt("width", 17))});
            errors.add(new Object[]{Registries.PAINTING_VARIANT, with(painting, tag -> tag.putString("asset_id", "Not An Id"))});
            CompoundTag damage = (CompoundTag)sentTags.get(Registries.DAMAGE_TYPE).get("minecraft:arrow");
            errors.add(new Object[]{Registries.DAMAGE_TYPE, with(damage, tag -> tag.remove("scaling"))});
            errors.add(new Object[]{Registries.DAMAGE_TYPE, with(damage, tag -> tag.putString("effects", "tickling"))});
            CompoundTag chat = (CompoundTag)sentTags.get(Registries.CHAT_TYPE).get("minecraft:chat");
            errors.add(new Object[]{Registries.CHAT_TYPE, with(chat, tag -> tag.getCompoundOrEmpty("chat").put("parameters", list(StringTag.valueOf("sender"), StringTag.valueOf("bogus"))))});
            CompoundTag wolf = (CompoundTag)sentTags.get(Registries.WOLF_VARIANT).get("minecraft:ashen");
            errors.add(new Object[]{Registries.WOLF_VARIANT, with(wolf, tag -> tag.getCompoundOrEmpty("assets").remove("wild"))});
            for (Object[] error : errors) {
                ResourceKey<? extends Registry<?>> key = (ResourceKey<? extends Registry<?>>)error[0];
                Tag tag = (Tag)error[1];
                DataResult<Object> result = codec(key).parse(ops, tag);
                JsonElement output = result.error().<JsonElement>map(e -> new JsonPrimitive(e.message())).orElse(JsonNull.INSTANCE);
                writer.result("errors", ChunkSuites.array(key.identifier().toString(), BiomeSuites.nbt(tag)), output, null);
            }
        }

        interface Change {
            void apply(CompoundTag tag);
        }

        private static ListTag list(final Tag... tags) {
            ListTag list = new ListTag();
            for (Tag tag : tags) {
                list.add(tag);
            }
            return list;
        }

        /** A copy of the tag, changed. */
        private static CompoundTag with(final CompoundTag tag, final Change change) {
            CompoundTag copy = tag.copy();
            change.apply(copy);
            return copy;
        }
    }
}
