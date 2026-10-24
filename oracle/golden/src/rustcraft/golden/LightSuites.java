package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.BitSet;
import java.util.Deque;
import java.util.HashMap;
import java.util.Iterator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import it.unimi.dsi.fastutil.longs.LongIterator;
import it.unimi.dsi.fastutil.longs.LongOpenHashSet;
import net.minecraft.core.BlockPos;
import net.minecraft.core.SectionPos;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.protocol.game.ClientboundLevelChunkPacketData;
import net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket;
import net.minecraft.network.protocol.game.ClientboundLightUpdatePacketData;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.LevelHeightAccessor;
import net.minecraft.world.level.LightLayer;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.BlockStateProperties;
import net.minecraft.world.level.chunk.DataLayer;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.chunk.LightChunk;
import net.minecraft.world.level.chunk.LightChunkGetter;
import net.minecraft.world.level.chunk.ProtoChunk;
import net.minecraft.world.level.chunk.UpgradeData;
import net.minecraft.world.level.lighting.LevelLightEngine;
import net.minecraft.world.level.lighting.LightEngine;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.material.Fluids;

/**
 * The client's light: vanilla's {@code LevelLightEngine} fed the walk scenario's chunks the way the client's packet
 * handlers feed it, read back. See TESTING.md ("The client's light"). A {@code ClientLevel} needs a {@code Minecraft},
 * so the harness stands in for the level and its chunk source: it holds the chunks (a {@code ProtoChunk} each, its
 * sections read from the packet and its sky light sources filled like {@code LevelChunk.replaceWithPacketData}), queues
 * the light updates like {@code ClientLevel.queueLightUpdate}, and runs {@code ClientPacketListener}'s light code, which
 * it copies, and {@code LevelChunk.setBlockState}'s light part. A program is a list of events: a chunk packet, a light
 * packet (the chunk's captured light, sent again), forgetting a chunk, a frame ({@code ClientLevel.update}: the queued
 * updates, then {@code runLightUpdates}), setting blocks as the server's block updates do, and reading the light back.
 */
final class LightSuites {
    // The overworld's height (data/minecraft/dimension_type/overworld.json), which the walk's login names.
    private static final LevelHeightAccessor HEIGHT = LevelHeightAccessor.create(-64, 384);

    private LightSuites() {
    }

    static List<Suite> suites() {
        return List.of(new LongOpenHashSetSuite(), new LevelLightEngineSuite());
    }

    /**
     * fastutil's {@code LongOpenHashSet}, whose order the light engine checks its blocks in: {@code programs},
     * {@code [expected, f, ops]} (the load factor as its bits) on {@code new LongOpenHashSet(expected, f)}, each op's output: {@code ["add", key]} →
     * whether it was new, {@code ["contains", key]}, {@code ["clear"]}, {@code ["trim", n]} → its answer,
     * {@code ["size"]} and {@code ["iterate"]} → the keys in order. Keys are blocks around the walk's spawn packed like
     * the light engine's, 0 now and then (kept apart from the table), and any longs; the tables grow past their first
     * size and shrink back, like the light engine's each run.
     */
    static final class LongOpenHashSetSuite implements Suite {
        @Override
        public String name() {
            return LongOpenHashSet.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0x10a9);
            for (int program = 0; program < 240; program++) {
                int expected = program % 4 == 0 ? random.nextInt(40) : 512;
                float f = program % 4 == 0 ? (random.nextBoolean() ? 0.75F : 0.25F + random.nextFloat() * 0.7F) : 0.5F;
                int adds = program % 24 == 0 ? 1500 + random.nextInt(2000) : random.nextInt(700);
                JsonArray ops = new JsonArray();
                List<Long> added = new ArrayList<>();
                for (int i = 0; i < adds; i++) {
                    long key = switch (random.nextInt(20)) {
                        case 0 -> 0L;
                        case 1, 2 -> random.nextLong();
                        case 3, 4 -> added.isEmpty() ? 1L : added.get(random.nextInt(added.size()));
                        default -> BlockPos.asLong(random.nextInt(48) - 24, random.nextInt(80) - 64, random.nextInt(48) - 24);
                    };
                    added.add(key);
                    ops.add(ChunkSuites.array("add", key));
                    int roll = random.nextInt(200);
                    if (roll == 0) {
                        ops.add(ChunkSuites.array("iterate"));
                        ops.add(ChunkSuites.array("clear"));
                        ops.add(ChunkSuites.array("trim", 512));
                    } else if (roll < 4) {
                        ops.add(ChunkSuites.array("contains", random.nextBoolean() ? key : random.nextLong()));
                    } else if (roll == 4) {
                        ops.add(ChunkSuites.array("size"));
                    }
                }
                ops.add(ChunkSuites.array("size"));
                ops.add(ChunkSuites.array("iterate"));
                ops.add(ChunkSuites.array("clear"));
                ops.add(ChunkSuites.array("trim", random.nextBoolean() ? 512 : random.nextInt(2000)));
                for (int i = 0; i < 20; i++) {
                    ops.add(ChunkSuites.array("add", BlockPos.asLong(random.nextInt(48) - 24, random.nextInt(80) - 64, random.nextInt(48) - 24)));
                }
                ops.add(ChunkSuites.array("iterate"));
                String bits = String.format("%08x", Float.floatToRawIntBits(f));
                writer.result("programs", ChunkSuites.array(expected, bits, ops), run(expected, f, ops), null);
            }
        }

        private static JsonArray run(final int expected, final float f, final JsonArray ops) {
            LongOpenHashSet set = new LongOpenHashSet(expected, f);
            JsonArray outputs = new JsonArray();
            for (JsonElement element : ops) {
                JsonArray op = element.getAsJsonArray();
                outputs.add(switch (op.get(0).getAsString()) {
                    case "add" -> new JsonPrimitive(set.add(op.get(1).getAsLong()));
                    case "contains" -> new JsonPrimitive(set.contains(op.get(1).getAsLong()));
                    case "clear" -> {
                        set.clear();
                        yield JsonNull.INSTANCE;
                    }
                    case "trim" -> new JsonPrimitive(set.trim(op.get(1).getAsInt()));
                    case "size" -> new JsonPrimitive(set.size());
                    case "iterate" -> {
                        JsonArray keys = new JsonArray();
                        for (LongIterator iterator = set.iterator(); iterator.hasNext(); ) {
                            keys.add(iterator.nextLong());
                        }
                        yield keys;
                    }
                    default -> throw new IllegalArgumentException(op.toString());
                });
            }
            return outputs;
        }
    }

    /** The captured chunk packets by position, in the order the server sent them. */
    private static Map<Long, ClientboundLevelChunkWithLightPacket> chunks;

    static Map<Long, ClientboundLevelChunkWithLightPacket> chunks() {
        if (chunks == null) {
            Map<Long, ClientboundLevelChunkWithLightPacket> packets = new LinkedHashMap<>();
            for (byte[] payload : LevelChunkSuites.payloads("minecraft:level_chunk_with_light")) {
                ClientboundLevelChunkWithLightPacket packet = ClientboundLevelChunkWithLightPacket.STREAM_CODEC.decode(LevelChunkSuites.reader(payload));
                packets.put(ChunkPos.pack(packet.x(), packet.z()), packet);
            }
            chunks = packets;
        }
        return chunks;
    }

    /** The level, its chunk source and its light engine, and the light updates queued for it. */
    static final class Harness implements LightChunkGetter {
        private final BlockGetter level = new BlockGetter() {
            @Override
            public BlockEntity getBlockEntity(final BlockPos pos) {
                return null;
            }

            @Override
            public BlockState getBlockState(final BlockPos pos) {
                return Blocks.AIR.defaultBlockState();
            }

            @Override
            public FluidState getFluidState(final BlockPos pos) {
                return Fluids.EMPTY.defaultFluidState();
            }

            @Override
            public int getHeight() {
                return HEIGHT.getHeight();
            }

            @Override
            public int getMinY() {
                return HEIGHT.getMinY();
            }
        };
        private final Map<Long, ProtoChunk> loaded = new HashMap<>();
        private final Deque<Runnable> lightUpdateQueue = new ArrayDeque<>();
        final LevelLightEngine lightEngine = new LevelLightEngine(this, true, true);

        @Override
        public LightChunk getChunkForLighting(final int x, final int z) {
            return this.loaded.get(ChunkPos.pack(x, z));
        }

        @Override
        public BlockGetter getLevel() {
            return this.level;
        }

        /** ClientPacketListener.handleLevelChunkWithLight. */
        void chunk(final ClientboundLevelChunkWithLightPacket packet) {
            int x = packet.x();
            int z = packet.z();
            this.replaceWithPacketData(x, z, packet.chunkData());
            ClientboundLightUpdatePacketData lightData = packet.lightData();
            this.lightUpdateQueue.add(() -> {
                this.applyLightData(x, z, lightData, false);
                ProtoChunk chunk = this.loaded.get(ChunkPos.pack(x, z));
                if (chunk != null) {
                    this.enableChunkLight(chunk, x, z);
                }
            });
        }

        /** ClientChunkCache.replaceWithPacketData, and LevelChunk.replaceWithPacketData's sections and light sources. */
        private void replaceWithPacketData(final int x, final int z, final ClientboundLevelChunkPacketData chunkData) {
            ProtoChunk chunk = this.loaded.computeIfAbsent(
                ChunkPos.pack(x, z), pos -> new ProtoChunk(new ChunkPos(x, z), UpgradeData.EMPTY, HEIGHT, LevelChunkSuites.factory(), null)
            );
            FriendlyByteBuf buffer = chunkData.getReadBuffer();
            for (LevelChunkSection section : chunk.getSections()) {
                section.read(buffer);
            }
            chunk.initializeLightSources();
        }

        /** ClientPacketListener.handleLightUpdatePacket. */
        void light(final int x, final int z, final ClientboundLightUpdatePacketData lightData) {
            this.lightUpdateQueue.add(() -> this.applyLightData(x, z, lightData, true));
        }

        /** ClientPacketListener.handleForgetLevelChunk: ClientChunkCache.drop (ClientLevel.unload), and queueLightRemoval. */
        void forget(final ChunkPos chunkPos) {
            if (this.loaded.remove(chunkPos.pack()) != null) {
                this.lightEngine.setLightEnabled(chunkPos, false);
            }
            this.lightUpdateQueue.add(() -> {
                LevelLightEngine lightEngine = this.lightEngine;
                lightEngine.setLightEnabled(chunkPos, false);

                for (int sectionY = lightEngine.getMinLightSection(); sectionY < lightEngine.getMaxLightSection(); sectionY++) {
                    SectionPos sectionPos = SectionPos.of(chunkPos, sectionY);
                    lightEngine.queueSectionData(LightLayer.BLOCK, sectionPos, null);
                    lightEngine.queueSectionData(LightLayer.SKY, sectionPos, null);
                }

                for (int sectionY = HEIGHT.getMinSectionY(); sectionY <= HEIGHT.getMaxSectionY(); sectionY++) {
                    lightEngine.updateSectionStatus(SectionPos.of(chunkPos, sectionY), true);
                }
            });
        }

        private void enableChunkLight(final ProtoChunk chunk, final int x, final int z) {
            LevelLightEngine lightEngine = this.lightEngine;
            LevelChunkSection[] sections = chunk.getSections();
            ChunkPos chunkPos = chunk.getPos();

            for (int sectionIndex = 0; sectionIndex < sections.length; sectionIndex++) {
                LevelChunkSection section = sections[sectionIndex];
                int sectionY = HEIGHT.getSectionYFromSectionIndex(sectionIndex);
                lightEngine.updateSectionStatus(SectionPos.of(chunkPos, sectionY), section.hasOnlyAir());
            }
        }

        private void applyLightData(final int x, final int z, final ClientboundLightUpdatePacketData lightData, final boolean scheduleRebuild) {
            LevelLightEngine lightEngine = this.lightEngine;
            BitSet skyYMask = lightData.skyYMask();
            BitSet emptySkyYMask = lightData.emptySkyYMask();
            Iterator<byte[]> skyUpdates = lightData.skyUpdates().iterator();
            this.readSectionList(x, z, lightEngine, LightLayer.SKY, skyYMask, emptySkyYMask, skyUpdates, scheduleRebuild);
            BitSet blockYMask = lightData.blockYMask();
            BitSet emptyBlockYMask = lightData.emptyBlockYMask();
            Iterator<byte[]> blockUpdates = lightData.blockUpdates().iterator();
            this.readSectionList(x, z, lightEngine, LightLayer.BLOCK, blockYMask, emptyBlockYMask, blockUpdates, scheduleRebuild);
            lightEngine.setLightEnabled(new ChunkPos(x, z), true);
        }

        private void readSectionList(
            final int chunkX, final int chunkZ, final LevelLightEngine lightEngine, final LightLayer layer, final BitSet yMask, final BitSet emptyYMask,
            final Iterator<byte[]> updates, final boolean scheduleRebuild
        ) {
            for (int sectionIndex = 0; sectionIndex < lightEngine.getLightSectionCount(); sectionIndex++) {
                int sectionY = lightEngine.getMinLightSection() + sectionIndex;
                boolean haveData = yMask.get(sectionIndex);
                boolean haveEmpty = emptyYMask.get(sectionIndex);
                if (haveData || haveEmpty) {
                    lightEngine.queueSectionData(
                        layer, SectionPos.of(chunkX, sectionY, chunkZ), haveData ? new DataLayer((byte[])updates.next().clone()) : new DataLayer()
                    );
                }
            }
        }

        /** ClientLevel.update: ClientLevel.pollLightUpdates, then the engine's runLightUpdates, whose count it returns. */
        int frame() {
            int size = this.lightUpdateQueue.size();
            int lightUpdatesPerFrame = size < 1000 ? Math.max(10, size / 10) : size;

            for (int i = 0; i < lightUpdatesPerFrame; i++) {
                Runnable update = this.lightUpdateQueue.poll();
                if (update == null) {
                    break;
                }

                update.run();
            }
            return this.lightEngine.runLightUpdates();
        }

        int queued() {
            return this.lightUpdateQueue.size();
        }

        /** The chunk at the position, if the harness has it. */
        ProtoChunk loadedChunk(final int x, final int z) {
            return this.loaded.get(ChunkPos.pack(x, z));
        }

        /**
         * ClientLevel.setServerVerifiedBlockState: Level.setBlock (in the level's bounds, in a chunk it has), and
         * LevelChunk.setBlockState's light part.
         */
        void set(final BlockPos pos, final BlockState state) {
            int chunkX = SectionPos.blockToSectionCoord(pos.getX());
            int chunkZ = SectionPos.blockToSectionCoord(pos.getZ());
            if (!HEIGHT.isInsideBuildHeight(pos.getY()) || !ChunkPos.isValid(chunkX, chunkZ)) {
                return;
            }
            ProtoChunk chunk = this.loaded.get(ChunkPos.pack(chunkX, chunkZ));
            if (chunk == null) {
                return;
            }

            int y = pos.getY();
            LevelChunkSection section = chunk.getSection(chunk.getSectionIndex(y));
            boolean wasEmpty = section.hasOnlyAir();
            if (wasEmpty && state.isAir()) {
                return;
            }

            int localX = pos.getX() & 15;
            int localY = y & 15;
            int localZ = pos.getZ() & 15;
            BlockState oldState = section.setBlockState(localX, localY, localZ, state);
            if (oldState == state) {
                return;
            }

            boolean isEmpty = section.hasOnlyAir();
            if (wasEmpty != isEmpty) {
                this.lightEngine.updateSectionStatus(pos, isEmpty);
            }

            if (LightEngine.hasDifferentLightProperties(oldState, state)) {
                chunk.getSkyLightSources().update(chunk, localX, y, localZ);
                this.lightEngine.checkBlock(pos);
            }
        }

        boolean has(final int x, final int z) {
            return this.loaded.containsKey(ChunkPos.pack(x, z));
        }

        ProtoChunk get(final int x, final int z) {
            return this.loaded.get(ChunkPos.pack(x, z));
        }
    }

    /**
     * {@code programs}: {@code [events]} → each event's output, in order. Events: {@code ["chunk", x, z]} (the captured
     * chunk packet), {@code ["light", x, z]} (its light, as a light packet), {@code ["forget", x, z]}, {@code ["frame"]}
     * → {@code [runLightUpdates' count, hasLightWork, updates still queued]}, {@code ["set", x, y, z, state]} and
     * {@code ["fill", x1, y1, z1, x2, y2, z2, state]} (each block in turn, x outermost), {@code ["box", x, y, z, r]} →
     * the sky's and the blocks' light of the cube {@code r} around the block as runs (y outermost, x innermost), and
     * {@code ["read", minX, maxX, minZ, maxZ]} → each chunk column's {@code [x, z, lightOnInColumn, sources, sections]}:
     * a loaded chunk's sky light sources ({@code [lowest of each column as runs, highest]}, or null), and each light
     * section's {@code [skyType, blockType, sky, block]}, its debug types and its light as runs in index order. The other
     * events output null. Every program also ends with {@code getRawBrightness} at blocks around the walk's spawn.
     */
    static final class LevelLightEngineSuite implements Suite {
        @Override
        public String name() {
            return LevelLightEngine.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            List<JsonArray> programs = new ArrayList<>();
            programs.add(batches(true));
            programs.add(allThenFrames());
            programs.add(framePerChunk());
            Random random = new Random(0x1197);
            for (int i = 0; i < 12; i++) {
                programs.add(randomLoad(random));
            }
            for (int i = 0; i < 40; i++) {
                programs.add(propagation(random, i));
            }
            for (JsonArray program : programs) {
                writer.result("programs", ChunkSuites.array(program), run(program), null);
            }
        }

        private static JsonArray event(final Object... parts) {
            return ChunkSuites.array(parts);
        }

        private static JsonArray chunkEvent(final String name, final long pos) {
            return event(name, ChunkPos.getX(pos), ChunkPos.getZ(pos));
        }

        /** The capture's chunk packets in its order, a frame after each chunk batch finished, then frames until the queue's empty. */
        private static JsonArray batches(final boolean read) {
            JsonArray program = new JsonArray();
            for (JsonObject packet : LevelChunkSuites.capture()) {
                if (!packet.get("direction").getAsString().equals("clientbound")) {
                    continue;
                }
                String name = packet.get("name").getAsString();
                if (name.equals("minecraft:level_chunk_with_light")) {
                    ClientboundLevelChunkWithLightPacket chunk = ClientboundLevelChunkWithLightPacket.STREAM_CODEC.decode(
                        LevelChunkSuites.reader(ChunkSuites.unhex(packet.get("payload").getAsString()))
                    );
                    program.add(event("chunk", chunk.x(), chunk.z()));
                } else if (name.equals("minecraft:chunk_batch_finished")) {
                    program.add(event("frame"));
                }
            }
            for (int i = 0; i < 16; i++) {
                program.add(event("frame"));
            }
            if (read) {
                program.add(read());
            }
            return program;
        }

        private static JsonArray read() {
            return event("read", -6, 6, -6, 6);
        }

        private static JsonArray allThenFrames() {
            JsonArray program = new JsonArray();
            chunks().keySet().forEach(pos -> program.add(chunkEvent("chunk", pos)));
            for (int i = 0; i < 16; i++) {
                program.add(event("frame"));
            }
            program.add(read());
            return program;
        }

        private static JsonArray framePerChunk() {
            JsonArray program = new JsonArray();
            chunks().keySet().forEach(pos -> {
                program.add(chunkEvent("chunk", pos));
                program.add(event("frame"));
            });
            program.add(read());
            return program;
        }

        /**
         * The chunks in a shuffled order, with frames, light packets, chunks forgotten and sent again, anywhere among them,
         * and 24 columns of the light read back.
         */
        private static JsonArray randomLoad(final Random random) {
            List<Long> positions = new ArrayList<>(chunks().keySet());
            java.util.Collections.shuffle(positions, random);
            JsonArray program = new JsonArray();
            List<Long> sent = new ArrayList<>();
            for (long pos : positions) {
                program.add(chunkEvent("chunk", pos));
                sent.add(pos);
                int roll = random.nextInt(16);
                if (roll < 4) {
                    program.add(event("frame"));
                } else if (roll < 6) {
                    program.add(chunkEvent("light", sent.get(random.nextInt(sent.size()))));
                } else if (roll < 8) {
                    long forgotten = sent.get(random.nextInt(sent.size()));
                    program.add(chunkEvent("forget", forgotten));
                    if (random.nextBoolean()) {
                        program.add(event("frame"));
                    }
                    if (random.nextInt(3) != 0) {
                        program.add(chunkEvent("chunk", forgotten));
                    }
                }
            }
            int frames = random.nextInt(4) == 0 ? 2 : 16;
            for (int i = 0; i < frames; i++) {
                program.add(event("frame"));
            }
            // A column at a time, 24 of them at random from the loaded chunks and the ones around them.
            for (int i = 0; i < 24; i++) {
                int x = random.nextInt(13) - 6;
                int z = random.nextInt(13) - 6;
                program.add(event("read", x, x, z, z));
            }
            return program;
        }

        /** States whose light each class decides without its shapes, or with the whole block's: the port's so far. */
        private static List<BlockState> pool() {
            return List.of(
                Blocks.AIR.defaultBlockState(), Blocks.STONE.defaultBlockState(), Blocks.DIRT.defaultBlockState(), Blocks.GLASS.defaultBlockState(),
                Blocks.STAINED_GLASS.white().defaultBlockState(), Blocks.TINTED_GLASS.defaultBlockState(),
                Blocks.OAK_LEAVES.defaultBlockState().setValue(BlockStateProperties.PERSISTENT, true), Blocks.ICE.defaultBlockState(),
                Blocks.SLIME_BLOCK.defaultBlockState(), Blocks.BARRIER.defaultBlockState(), Blocks.WATER.defaultBlockState(), Blocks.LAVA.defaultBlockState(),
                Blocks.GLOWSTONE.defaultBlockState(), Blocks.SEA_LANTERN.defaultBlockState(), Blocks.JACK_O_LANTERN.defaultBlockState(),
                Blocks.SHROOMLIGHT.defaultBlockState(), Blocks.MAGMA_BLOCK.defaultBlockState(), Blocks.CRYING_OBSIDIAN.defaultBlockState(),
                Blocks.REDSTONE_LAMP.defaultBlockState().setValue(BlockStateProperties.LIT, true), Blocks.OCHRE_FROGLIGHT.defaultBlockState()
            );
        }

        private static final List<BlockState> LIGHTS = List.of(
            Blocks.GLOWSTONE.defaultBlockState(), Blocks.SEA_LANTERN.defaultBlockState(), Blocks.JACK_O_LANTERN.defaultBlockState(),
            Blocks.SHROOMLIGHT.defaultBlockState(), Blocks.MAGMA_BLOCK.defaultBlockState(), Blocks.CRYING_OBSIDIAN.defaultBlockState(),
            Blocks.REDSTONE_LAMP.defaultBlockState().setValue(BlockStateProperties.LIT, true), Blocks.LAVA.defaultBlockState()
        );

        private static final List<BlockState> ROOFS = List.of(
            Blocks.STONE.defaultBlockState(), Blocks.DIRT.defaultBlockState(), Blocks.GLASS.defaultBlockState(), Blocks.TINTED_GLASS.defaultBlockState(),
            Blocks.OAK_LEAVES.defaultBlockState().setValue(BlockStateProperties.PERSISTENT, true), Blocks.ICE.defaultBlockState(), Blocks.WATER.defaultBlockState()
        );

        /**
         * The walk's world loaded like {@code batches}, then blocks placed and removed around a block near the spawn (at
         * the ground, just above it, a few blocks up, or high in the sky, and now and then a section or two below it, which
         * high up stores sections below ones already stored): roofs, walls, light sources and single blocks,
         * a frame and the light around it after each; now and then a chunk's light sent again, or a chunk forgotten and
         * sent again. Then the light of the chunks around it.
         */
        private static JsonArray propagation(final Random random, final int index) {
            JsonArray program = batches(false);
            int chunkX = random.nextInt(5) - 2;
            int chunkZ = random.nextInt(5) - 2;
            int[] heights = {-61, -59, -55, -48, -33, 40, 300};
            int x = chunkX * 16 + random.nextInt(16);
            int z = chunkZ * 16 + random.nextInt(16);
            int y = heights[index % heights.length];
            List<BlockState> pool = pool();
            int ops = 4 + random.nextInt(9);
            for (int op = 0; op < ops; op++) {
                int roll = random.nextInt(10);
                int ox = x + random.nextInt(9) - 4;
                // Now and then a section or two below: high up, that stores sections under ones stored already.
                int oy = random.nextInt(6) == 0 ? y - 12 - random.nextInt(14) : y + random.nextInt(7) - 2;
                int oz = z + random.nextInt(9) - 4;
                if (roll < 3) {
                    // A roof or a floor, up to 9 by 9.
                    int rx = random.nextInt(5);
                    int rz = random.nextInt(5);
                    BlockState roof = random.nextInt(4) == 0 ? Blocks.AIR.defaultBlockState() : ROOFS.get(random.nextInt(ROOFS.size()));
                    program.add(event("fill", ox - rx, oy, oz - rz, ox + rx, oy, oz + rz, Block.getId(roof)));
                } else if (roll < 4) {
                    // A wall.
                    boolean alongX = random.nextBoolean();
                    int length = random.nextInt(5);
                    int height = random.nextInt(4);
                    BlockState wall = ROOFS.get(random.nextInt(ROOFS.size()));
                    program.add(event(
                        "fill", alongX ? ox - length : ox, oy, alongX ? oz : oz - length, alongX ? ox + length : ox, oy + height, alongX ? oz : oz + length,
                        Block.getId(wall)
                    ));
                } else if (roll < 7) {
                    program.add(event("set", ox, oy, oz, Block.getId(LIGHTS.get(random.nextInt(LIGHTS.size())))));
                } else if (roll < 9) {
                    program.add(event("set", ox, oy, oz, Block.getId(pool.get(random.nextInt(pool.size())))));
                } else if (random.nextBoolean()) {
                    program.add(event("light", SectionPos.blockToSectionCoord(ox), SectionPos.blockToSectionCoord(oz)));
                } else {
                    program.add(event("forget", SectionPos.blockToSectionCoord(ox), SectionPos.blockToSectionCoord(oz)));
                    program.add(event("chunk", SectionPos.blockToSectionCoord(ox), SectionPos.blockToSectionCoord(oz)));
                    program.add(event("frame"));
                }
                program.add(event("frame"));
                program.add(event("box", ox, oy, oz, 7));
            }
            program.add(event("read", chunkX - 1, chunkX + 1, chunkZ - 1, chunkZ + 1));
            return program;
        }

        private static JsonArray run(final JsonArray program) {
            Harness harness = new Harness();
            JsonArray outputs = new JsonArray();
            for (JsonElement element : program) {
                JsonArray event = element.getAsJsonArray();
                String name = event.get(0).getAsString();
                outputs.add(switch (name) {
                    case "chunk" -> {
                        harness.chunk(chunks().get(ChunkPos.pack(event.get(1).getAsInt(), event.get(2).getAsInt())));
                        yield JsonNull.INSTANCE;
                    }
                    case "light" -> {
                        int x = event.get(1).getAsInt();
                        int z = event.get(2).getAsInt();
                        harness.light(x, z, chunks().get(ChunkPos.pack(x, z)).lightData());
                        yield JsonNull.INSTANCE;
                    }
                    case "forget" -> {
                        harness.forget(new ChunkPos(event.get(1).getAsInt(), event.get(2).getAsInt()));
                        yield JsonNull.INSTANCE;
                    }
                    case "frame" -> {
                        int count = harness.frame();
                        yield ChunkSuites.array(count, harness.lightEngine.hasLightWork(), harness.queued());
                    }
                    case "set" -> {
                        harness.set(new BlockPos(event.get(1).getAsInt(), event.get(2).getAsInt(), event.get(3).getAsInt()), Block.stateById(event.get(4).getAsInt()));
                        yield JsonNull.INSTANCE;
                    }
                    case "fill" -> {
                        BlockState state = Block.stateById(event.get(7).getAsInt());
                        for (int x = event.get(1).getAsInt(); x <= event.get(4).getAsInt(); x++) {
                            for (int y = event.get(2).getAsInt(); y <= event.get(5).getAsInt(); y++) {
                                for (int z = event.get(3).getAsInt(); z <= event.get(6).getAsInt(); z++) {
                                    harness.set(new BlockPos(x, y, z), state);
                                }
                            }
                        }
                        yield JsonNull.INSTANCE;
                    }
                    case "box" -> box(harness, event.get(1).getAsInt(), event.get(2).getAsInt(), event.get(3).getAsInt(), event.get(4).getAsInt());
                    case "read" -> read(harness, event.get(1).getAsInt(), event.get(2).getAsInt(), event.get(3).getAsInt(), event.get(4).getAsInt());
                    default -> throw new IllegalArgumentException(name);
                });
            }
            Random random = new Random(program.size());
            JsonArray raw = new JsonArray();
            for (int i = 0; i < 64; i++) {
                BlockPos pos = new BlockPos(random.nextInt(48) - 24, random.nextInt(24) - 66, random.nextInt(48) - 24);
                int darkening = random.nextInt(17);
                raw.add(ChunkSuites.array(pos.getX(), pos.getY(), pos.getZ(), darkening, harness.lightEngine.getRawBrightness(pos, darkening)));
            }
            outputs.add(raw);
            return outputs;
        }

        private static JsonArray box(final Harness harness, final int x, final int y, final int z, final int r) {
            JsonArray layers = new JsonArray();
            for (LightLayer layer : List.of(LightLayer.SKY, LightLayer.BLOCK)) {
                List<Integer> values = new ArrayList<>();
                for (int dy = -r; dy <= r; dy++) {
                    for (int dz = -r; dz <= r; dz++) {
                        for (int dx = -r; dx <= r; dx++) {
                            values.add(harness.lightEngine.getLayerListener(layer).getLightValue(new BlockPos(x + dx, y + dy, z + dz)));
                        }
                    }
                }
                layers.add(ChunkSuites.runs(values.iterator(), JsonPrimitive::new));
            }
            return layers;
        }

        private static JsonArray read(final Harness harness, final int minX, final int maxX, final int minZ, final int maxZ) {
            LevelLightEngine engine = harness.lightEngine;
            JsonArray columns = new JsonArray();
            for (int chunkX = minX; chunkX <= maxX; chunkX++) {
                for (int chunkZ = minZ; chunkZ <= maxZ; chunkZ++) {
                    JsonElement sources = JsonNull.INSTANCE;
                    ProtoChunk chunk = harness.get(chunkX, chunkZ);
                    if (chunk != null) {
                        List<Integer> lowest = new ArrayList<>();
                        for (int z = 0; z < 16; z++) {
                            for (int x = 0; x < 16; x++) {
                                lowest.add(chunk.getSkyLightSources().getLowestSourceY(x, z));
                            }
                        }
                        sources = ChunkSuites.array(ChunkSuites.runs(lowest.iterator(), JsonPrimitive::new), chunk.getSkyLightSources().getHighestLowestSourceY());
                    }
                    JsonArray sections = new JsonArray();
                    for (int sectionY = engine.getMinLightSection(); sectionY < engine.getMaxLightSection(); sectionY++) {
                        SectionPos section = SectionPos.of(chunkX, sectionY, chunkZ);
                        JsonArray light = new JsonArray();
                        light.add(engine.getDebugSectionType(LightLayer.SKY, section).name());
                        light.add(engine.getDebugSectionType(LightLayer.BLOCK, section).name());
                        for (LightLayer layer : List.of(LightLayer.SKY, LightLayer.BLOCK)) {
                            light.add(LevelChunkSuites.runsOf(16, (x, y, z) -> new JsonPrimitive(
                                engine.getLayerListener(layer).getLightValue(new BlockPos(section.minBlockX() + x, section.minBlockY() + y, section.minBlockZ() + z))
                            )));
                        }
                        sections.add(light);
                    }
                    columns.add(ChunkSuites.array(chunkX, chunkZ, engine.lightOnInColumn(SectionPos.getZeroNode(chunkX, chunkZ)), sources, sections));
                }
            }
            return columns;
        }
    }
}
