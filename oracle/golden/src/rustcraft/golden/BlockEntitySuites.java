package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Set;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.TagParser;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket;
import net.minecraft.util.ProblemReporter;
import net.minecraft.world.level.BaseSpawner;
import net.minecraft.world.level.SpawnData;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.EntityBlock;
import net.minecraft.world.level.block.entity.BannerBlockEntity;
import net.minecraft.world.level.block.entity.BannerPatternLayers;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.entity.BlockEntityType;
import net.minecraft.world.level.block.entity.BrushableBlockEntity;
import net.minecraft.world.level.block.entity.CampfireBlockEntity;
import net.minecraft.world.level.block.entity.ShulkerBoxBlockEntity;
import net.minecraft.world.level.block.entity.SignBlockEntity;
import net.minecraft.world.level.block.entity.SignText;
import net.minecraft.world.level.block.entity.SkullBlockEntity;
import net.minecraft.world.level.block.entity.SpawnerBlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.storage.TagValueInput;

/**
 * Block entities as the client makes them: which type each block's {@code newBlockEntity} makes and which blocks each type
 * is valid for ({@code BlockEntityType}), and block entities loaded from the tags a server sends with a chunk or a
 * {@code block_entity_data} packet ({@code BlockEntity.loadWithComponents} through a {@code TagValueInput}). See TESTING.md
 * ("Block entities").
 */
final class BlockEntitySuites {
    private BlockEntitySuites() {
    }

    private static boolean componentsBound;

    /**
     * Binds the items' components, as the client does once it has the server's registries
     * ({@code RegistryDataCollector.updateComponents}): some block entities' classes make item stacks when they load
     * (the vault's config).
     */
    static synchronized void bindComponents() {
        if (!componentsBound) {
            BuiltInRegistries.DATA_COMPONENT_INITIALIZERS.build(RegistryValues.registries()).forEach(pending -> pending.apply());
            componentsBound = true;
        }
    }

    static List<Suite> suites() {
        return List.of(new TypeSuite(), new LoadSuite(), new DataPacketSuite(), new HeightmapSuite());
    }

    private static Object field(final Object object, final Class<?> owner, final String name) {
        try {
            Field field = owner.getDeclaredField(name);
            field.setAccessible(true);
            return field.get(object);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }

    /**
     * {@code newBlockEntity}: each block → the type its default state's {@code newBlockEntity} makes, or null for one that
     * makes none or isn't an {@code EntityBlock}. {@code validBlocks}: each type → its valid blocks by name, in the block
     * registry's order (Java's is a {@code Set.of}).
     */
    static final class TypeSuite implements Suite {
        @Override
        public String name() {
            return BlockEntityType.class.getName();
        }

        @Override
        @SuppressWarnings("unchecked")
        public void write(final GoldenWriter writer) throws IOException {
            bindComponents();
            for (Block block : BuiltInRegistries.BLOCK) {
                JsonElement type = JsonNull.INSTANCE;
                if (block instanceof EntityBlock entityBlock) {
                    BlockEntity entity = entityBlock.newBlockEntity(BlockPos.ZERO, block.defaultBlockState());
                    if (entity != null) {
                        type = new JsonPrimitive(BuiltInRegistries.BLOCK_ENTITY_TYPE.getKey(entity.getType()).toString());
                    }
                }
                writer.result("newBlockEntity", ChunkSuites.array(BuiltInRegistries.BLOCK.getKey(block).toString()), type, null);
            }
            for (BlockEntityType<?> type : BuiltInRegistries.BLOCK_ENTITY_TYPE) {
                Set<Block> valid = (Set<Block>)field(type, BlockEntityType.class, "validBlocks");
                JsonArray blocks = new JsonArray();
                valid.stream().sorted(Comparator.comparingInt(BuiltInRegistries.BLOCK::getId)).forEach(block -> blocks.add(BuiltInRegistries.BLOCK.getKey(block).toString()));
                writer.result("validBlocks", ChunkSuites.array(BuiltInRegistries.BLOCK_ENTITY_TYPE.getKey(type).toString()), blocks, null);
            }
        }
    }

    private static JsonElement component(final Component component) {
        return component == null ? JsonNull.INSTANCE : new JsonPrimitive(component.toString());
    }

    private static JsonElement signText(final SignText text) {
        JsonArray messages = new JsonArray();
        text.getMessages(false).forEach(message -> messages.add(message.toString()));
        JsonArray filtered = new JsonArray();
        text.getMessages(true).forEach(message -> filtered.add(message.toString()));
        return ChunkSuites.array(messages, filtered, text.getColor().getSerializedName(), text.hasGlowingText());
    }

    private static JsonElement patterns(final BannerPatternLayers layers) {
        JsonArray result = new JsonArray();
        for (BannerPatternLayers.Layer layer : layers.layers()) {
            result.add(ChunkSuites.array(layer.pattern().unwrapKey().orElseThrow().identifier().toString(), layer.color().getSerializedName()));
        }
        return result;
    }

    private static JsonElement ints(final int[] values) {
        JsonArray result = new JsonArray();
        for (int value : values) {
            result.add(value);
        }
        return result;
    }

    /** What a block entity holds that its class loads, by class. */
    static JsonElement data(final BlockEntity entity) {
        return switch (entity) {
            case SignBlockEntity sign -> ChunkSuites.array(
                signText(sign.getText(net.minecraft.world.level.block.entity.SignTextSlot.FRONT)), signText(sign.getText(net.minecraft.world.level.block.entity.SignTextSlot.BACK)), sign.isWaxed()
            );
            case BannerBlockEntity banner -> ChunkSuites.array(banner.getBaseColor().getSerializedName(), component(banner.getCustomName()), patterns(banner.getPatterns()));
            case SkullBlockEntity skull -> ChunkSuites.array(
                skull.getNoteBlockSound() == null ? JsonNull.INSTANCE : new JsonPrimitive(skull.getNoteBlockSound().toString()),
                component((Component)field(skull, SkullBlockEntity.class, "customName"))
            );
            case SpawnerBlockEntity spawner -> {
                BaseSpawner base = spawner.getSpawner();
                SpawnData next = (SpawnData)field(base, BaseSpawner.class, "nextSpawnData");
                JsonArray fields = new JsonArray();
                for (String name : List.of("spawnDelay", "minSpawnDelay", "maxSpawnDelay", "spawnCount", "maxNearbyEntities", "requiredPlayerRange", "spawnRange")) {
                    fields.add((Integer)field(base, BaseSpawner.class, name));
                }
                fields.add(next == null ? JsonNull.INSTANCE : new JsonPrimitive(next.entityToSpawn().toString()));
                yield fields;
            }
            case CampfireBlockEntity campfire -> ChunkSuites.array(
                ints((int[])field(campfire, CampfireBlockEntity.class, "cookingProgress")), ints((int[])field(campfire, CampfireBlockEntity.class, "cookingTime"))
            );
            case BrushableBlockEntity brushable -> {
                Object direction = field(brushable, BrushableBlockEntity.class, "hitDirection");
                yield direction == null ? JsonNull.INSTANCE : new JsonPrimitive(direction.toString());
            }
            case ShulkerBoxBlockEntity box -> ChunkSuites.array(box.getAnimationStatus().name(), String.format("%08x", Float.floatToRawIntBits(box.getProgress(1.0F))));
            default -> JsonNull.INSTANCE;
        };
    }

    /** A loaded block entity: {@code [type, state, isRemoved, data]}. */
    static JsonElement fields(final BlockEntity entity) {
        return ChunkSuites.array(
            BuiltInRegistries.BLOCK_ENTITY_TYPE.getKey(entity.getType()).toString(), Block.getId(entity.getBlockState()), entity.isRemoved(), data(entity)
        );
    }

    /**
     * ClientboundBlockEntityDataPacket: decodes positions (blocks around the walk's spawn, at the packing's limits, any
     * longs), type ids (every type, the registry's size, -1, any) and tags (empty, the load suite's, and others), cut
     * short or with more after: {@code [[x, y, z, type, tag], unread, written]}.
     */
    static final class DataPacketSuite implements Suite {
        @Override
        public String name() {
            return ClientboundBlockEntityDataPacket.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            java.util.Random random = new java.util.Random(0xbe0d);
            int size = BuiltInRegistries.BLOCK_ENTITY_TYPE.size();
            List<Integer> ids = new ArrayList<>();
            for (int id = 0; id < size; id++) {
                ids.add(id);
            }
            ids.addAll(List.of(size, -1, Integer.MAX_VALUE));
            List<String> tags = List.of(
                "{}", "{front_text:{messages:[\"a\",\"b\",\"c\",\"d\"]}}", "{patterns:[{pattern:\"minecraft:globe\",color:\"teal\"}]}",
                "{Delay:5s,SpawnData:{entity:{id:\"minecraft:pig\"}}}", "{x:1,y:2,z:3,id:\"minecraft:chest\",Items:[]}", "{a:[I;1,2,3],b:{c:1b}}"
            );
            for (int i = 0; i < ids.size(); i++) {
                long pos = switch (i % 4) {
                    case 0 -> new BlockPos(random.nextInt(33) - 16, -64 + random.nextInt(8), random.nextInt(33) - 16).asLong();
                    case 1 -> new BlockPos(33554431 * (random.nextBoolean() ? 1 : -1), 2047, -33554432).asLong();
                    default -> random.nextLong();
                };
                CompoundTag tag;
                try {
                    tag = TagParser.parseCompoundFully(tags.get(i % tags.size()));
                } catch (Exception e) {
                    throw new IllegalStateException(e);
                }
                net.minecraft.network.RegistryFriendlyByteBuf out = LevelChunkSuites.writer();
                out.writeLong(pos);
                out.writeVarInt(ids.get(i));
                out.writeNbt(tag);
                byte[] bytes = new byte[out.readableBytes()];
                out.getBytes(out.readerIndex(), bytes);
                for (byte[] variant : i % 5 == 0 ? LevelChunkSuites.variants(random, bytes) : List.of(bytes)) {
                    LevelChunkSuites.decodeCase(
                        writer, "STREAM_CODEC.decode", variant, LevelChunkSuites::reader, LevelChunkSuites::writer, ClientboundBlockEntityDataPacket.STREAM_CODEC,
                        packet -> ChunkSuites.array(
                            packet.getPos().getX(), packet.getPos().getY(), packet.getPos().getZ(),
                            BuiltInRegistries.BLOCK_ENTITY_TYPE.getKey(packet.getType()).toString(), packet.getTag().toString()
                        )
                    );
                }
            }
        }
    }

    /** The cases: a block, and the tags its block entity is loaded from. */
    private record Tags(Block block, List<String> tags) {
    }

    /**
     * {@code load}: {@code [state, tag]} → the block entity its block makes ({@code newBlockEntity}, or the type that's valid
     * for it where that makes none: the moving piston's) at (1, 64, -2), loaded from the tag (in hex) with the server's
     * registries, then {@code fields}. Every type with an empty tag, then tags a server sends of signs, banners, skulls,
     * spawners, campfires, brushable blocks and shulker boxes, with values out of range, of the wrong type and missing.
     */
    static final class LoadSuite implements Suite {
        @Override
        public String name() {
            return BlockEntity.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            bindComponents();
            RegistryAccess registries = RegistryValues.registries();
            BlockPos pos = new BlockPos(1, 64, -2);
            List<Tags> cases = new ArrayList<>();
            for (BlockEntityType<?> type : BuiltInRegistries.BLOCK_ENTITY_TYPE) {
                @SuppressWarnings("unchecked")
                Set<Block> valid = (Set<Block>)field(type, BlockEntityType.class, "validBlocks");
                valid.stream().min(Comparator.comparingInt(BuiltInRegistries.BLOCK::getId)).ifPresent(block -> cases.add(new Tags(block, List.of("{}"))));
            }
            List<String> signs = List.of(
                "{front_text:{messages:[\"a\",{text:\"b\",color:\"red\"},\"\",{translate:\"sign.edit\"}]}}",
                "{front_text:{messages:[\"1\",\"2\",\"3\",\"4\"],color:\"lime\",has_glowing_text:1b},back_text:{messages:[\"w\",\"x\",\"y\",\"z\"],filtered_messages:[\"W\",\"X\",\"Y\",\"Z\"]},is_waxed:1b}",
                "{front_text:{messages:[\"a\",\"b\",\"c\"]},back_text:{messages:[\"a\",\"b\",\"c\",\"d\"],color:\"chartreuse\"}}",
                "{front_text:{messages:[\"a\",\"b\",\"c\",\"d\"],filtered_messages:[\"a\",\"b\"]},is_waxed:\"yes\",allow_op_features:1b}",
                "{front_text:{messages:[\"a\",\"b\",\"c\",\"d\"],color:\"blue\",has_glowing_text:2b},back_text:{messages:[\"\",\"\",\"\",\"\"],has_glowing_text:\"x\"}}",
                "{front_text:\"text\",back_text:{}}"
            );
            cases.add(new Tags(Blocks.OAK_SIGN, signs));
            cases.add(new Tags(Blocks.SPRUCE_WALL_SIGN, signs));
            cases.add(new Tags(Blocks.CHERRY_HANGING_SIGN, signs));
            cases.add(new Tags(Blocks.BAMBOO_WALL_HANGING_SIGN, signs));
            List<String> banners = List.of(
                "{patterns:[{pattern:\"minecraft:stripe_top\",color:\"red\"},{pattern:\"minecraft:creeper\",color:\"black\"}]}",
                "{CustomName:{text:\"Flag\",italic:1b},patterns:[]}",
                "{patterns:[{pattern:\"minecraft:no_such_pattern\",color:\"red\"}]}",
                "{patterns:[{pattern:\"minecraft:globe\",color:\"teal\"}],CustomName:5}",
                "{patterns:\"minecraft:globe\"}"
            );
            cases.add(new Tags(Blocks.BANNER.pick(net.minecraft.world.item.DyeColor.WHITE), banners));
            cases.add(new Tags(Blocks.WALL_BANNER.pick(net.minecraft.world.item.DyeColor.MAGENTA), banners));
            List<String> skulls = List.of(
                "{note_block_sound:\"minecraft:block.note_block.harp\",custom_name:\"Bob\"}",
                "{note_block_sound:\"Not A Sound\",custom_name:{text:\"x\",color:\"gold\"}}",
                "{custom_name:[\"a\",\"b\"],profile:\"Steve\"}"
            );
            cases.add(new Tags(Blocks.SKELETON_SKULL, skulls));
            cases.add(new Tags(Blocks.PLAYER_WALL_HEAD, skulls));
            cases.add(new Tags(Blocks.SPAWNER, List.of(
                "{Delay:5s,SpawnData:{entity:{id:\"minecraft:pig\"}},MinSpawnDelay:100s,MaxSpawnDelay:300s,SpawnCount:2s,MaxNearbyEntities:3s,RequiredPlayerRange:8s,SpawnRange:2s}",
                "{Delay:70000,MinSpawnDelay:1.5f,SpawnData:{entity:{id:\"minecraft:zombie\",Health:5.0f}}}",
                "{SpawnData:{entity:\"pig\"},SpawnRange:\"far\"}",
                "{SpawnData:{}}"
            )));
            cases.add(new Tags(Blocks.CAMPFIRE, List.of(
                "{CookingTimes:[I;1,2,3,4],CookingTotalTimes:[I;600,600,0,0]}",
                "{CookingTimes:[I;7],CookingTotalTimes:[I;1,2,3,4,5,6]}",
                "{CookingTimes:[B;1b,2b],CookingTotalTimes:[L;5L]}",
                "{CookingTimes:[1,2]}"
            )));
            cases.add(new Tags(Blocks.SUSPICIOUS_GRAVEL, List.of("{hit_direction:2}", "{hit_direction:5b}", "{hit_direction:9}", "{hit_direction:-1}", "{hit_direction:\"up\"}")));
            cases.add(new Tags(Blocks.DYED_SHULKER_BOX.pick(net.minecraft.world.item.DyeColor.RED), List.of("{}", "{Items:[]}")));
            for (Tags tags : cases) {
                BlockState state = tags.block().defaultBlockState();
                for (String snbt : tags.tags()) {
                    CompoundTag tag;
                    try {
                        tag = TagParser.parseCompoundFully(snbt);
                    } catch (Exception e) {
                        throw new IllegalStateException(snbt, e);
                    }
                    BlockEntity entity = create(pos, state);
                    entity.loadWithComponents(TagValueInput.create(ProblemReporter.DISCARDING, registries, tag));
                    writer.result("load", ChunkSuites.array(Block.getId(state), BiomeSuites.nbt(tag)), fields(entity), null);
                }
            }
        }

        static BlockEntity create(final BlockPos pos, final BlockState state) {
            if (state.getBlock() instanceof EntityBlock entityBlock) {
                BlockEntity entity = entityBlock.newBlockEntity(pos, state);
                if (entity != null) {
                    return entity;
                }
            }
            for (BlockEntityType<?> type : BuiltInRegistries.BLOCK_ENTITY_TYPE) {
                if (type.isValid(state)) {
                    return type.create(pos, state);
                }
            }
            throw new IllegalArgumentException(state.toString());
        }
    }
}
