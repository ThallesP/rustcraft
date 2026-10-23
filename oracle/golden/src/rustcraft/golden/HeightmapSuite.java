package rustcraft.golden;

import com.google.gson.JsonArray;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import net.minecraft.core.BlockPos;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.protocol.game.ClientboundLevelChunkPacketData;
import net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.LevelHeightAccessor;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.BlockStateProperties;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.chunk.ProtoChunk;
import net.minecraft.world.level.chunk.UpgradeData;
import net.minecraft.world.level.levelgen.Heightmap;

/**
 * {@code Heightmap.update} as the client's {@code LevelChunk.setBlockState} runs it: {@code programs}, {@code [chunk, ops]} on a
 * chunk of the walk's capture (its sections and the heightmaps the packet sent, the others new) → after each set of a
 * block {@code [x, y, z, state]} the column's first available height in each of the full chunk's four heightmaps
 * (world surface, ocean floor, motion blocking, motion blocking without leaves), or null where the chunk doesn't change
 * (the same state, or air into an empty section), and at the end each heightmap's raw data. See TESTING.md ("Block entities").
 */
final class HeightmapSuite implements Suite {
    private static final LevelHeightAccessor HEIGHT = LevelHeightAccessor.create(-64, 384);
    private static final Heightmap.Types[] TYPES = {
        Heightmap.Types.WORLD_SURFACE, Heightmap.Types.OCEAN_FLOOR, Heightmap.Types.MOTION_BLOCKING, Heightmap.Types.MOTION_BLOCKING_NO_LEAVES
    };

    @Override
    public String name() {
        return Heightmap.class.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        // The client's block tags, a vanilla server's, which the heightmaps' predicates read.
        ShapeSuites.BlockStateShapeSuite.bindBlockTags();
        List<BlockState> pool = List.of(
            Blocks.AIR.defaultBlockState(), Blocks.STONE.defaultBlockState(), Blocks.GLASS.defaultBlockState(), Blocks.OAK_LEAVES.defaultBlockState(),
            Blocks.WATER.defaultBlockState(), Blocks.WATER.defaultBlockState().setValue(BlockStateProperties.LEVEL, 4), Blocks.LAVA.defaultBlockState(),
            Blocks.COBWEB.defaultBlockState(), Blocks.BAMBOO_SAPLING.defaultBlockState(), Blocks.TORCH.defaultBlockState(), Blocks.SNOW.defaultBlockState(),
            Blocks.OAK_SLAB.defaultBlockState(), Blocks.OAK_STAIRS.defaultBlockState().setValue(BlockStateProperties.WATERLOGGED, true), Blocks.KELP.defaultBlockState(),
            Blocks.LADDER.defaultBlockState(), Blocks.OAK_FENCE.defaultBlockState(), Blocks.SHORT_GRASS.defaultBlockState(), Blocks.CHEST.defaultBlockState(),
            Blocks.AZALEA_LEAVES.defaultBlockState(), Blocks.BARRIER.defaultBlockState(), Blocks.SCAFFOLDING.defaultBlockState(), Blocks.CAVE_AIR.defaultBlockState()
        );
        List<byte[]> payloads = LevelChunkSuites.payloads("minecraft:level_chunk_with_light");
        Random random = new Random(0x4ea9);
        int programs = 0;
        for (byte[] payload : payloads) {
            ClientboundLevelChunkWithLightPacket packet = ClientboundLevelChunkWithLightPacket.STREAM_CODEC.decode(LevelChunkSuites.reader(payload));
            if (Math.abs(packet.x()) > 1 || Math.abs(packet.z()) > 1) {
                continue;
            }
            ProtoChunk chunk = new ProtoChunk(new ChunkPos(packet.x(), packet.z()), UpgradeData.EMPTY, HEIGHT, LevelChunkSuites.factory(), null);
            ClientboundLevelChunkPacketData data = packet.chunkData();
            FriendlyByteBuf buffer = data.getReadBuffer();
            for (LevelChunkSection section : chunk.getSections()) {
                section.read(buffer);
            }
            data.getHeightmaps().forEach(chunk::setHeightmap);
            for (Heightmap.Types type : TYPES) {
                chunk.getOrCreateHeightmapUnprimed(type);
            }
            JsonArray ops = new JsonArray();
            JsonArray outputs = new JsonArray();
            int[][] columns = {{random.nextInt(16), random.nextInt(16)}, {random.nextInt(16), random.nextInt(16)}, {7, 7}};
            for (int i = 0; i < 120; i++) {
                int[] column = columns[random.nextInt(columns.length)];
                int x = packet.x() * 16 + column[0];
                int z = packet.z() * 16 + column[1];
                int y = switch (random.nextInt(4)) {
                    case 0 -> -64 + random.nextInt(8);
                    case 1 -> -62 + random.nextInt(10);
                    case 2 -> -50 + random.nextInt(40);
                    default -> 300 + random.nextInt(19);
                };
                BlockState state = pool.get(random.nextInt(pool.size()));
                ops.add(ChunkSuites.array(x, y, z, Block.getId(state)));
                outputs.add(this.set(chunk, new BlockPos(x, y, z), state));
            }
            JsonArray raw = new JsonArray();
            for (Heightmap.Types type : TYPES) {
                raw.add(ChunkSuites.longs(chunk.getOrCreateHeightmapUnprimed(type).getRawData()));
            }
            outputs.add(raw);
            writer.result("programs", ChunkSuites.array(ChunkSuites.hex(payload), ops), outputs, null);
            programs++;
        }
        if (programs == 0) {
            throw new IllegalStateException("no chunks around the spawn");
        }
    }

    /** LevelChunk.setBlockState's section and heightmaps. */
    private com.google.gson.JsonElement set(final ProtoChunk chunk, final BlockPos pos, final BlockState state) {
        int y = pos.getY();
        LevelChunkSection section = chunk.getSection(chunk.getSectionIndex(y));
        if (section.hasOnlyAir() && state.isAir()) {
            return com.google.gson.JsonNull.INSTANCE;
        }
        int localX = pos.getX() & 15;
        int localZ = pos.getZ() & 15;
        BlockState old = section.setBlockState(localX, y & 15, localZ, state);
        if (old == state) {
            return com.google.gson.JsonNull.INSTANCE;
        }
        chunk.getOrCreateHeightmapUnprimed(Heightmap.Types.MOTION_BLOCKING).update(localX, y, localZ, state);
        chunk.getOrCreateHeightmapUnprimed(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES).update(localX, y, localZ, state);
        chunk.getOrCreateHeightmapUnprimed(Heightmap.Types.OCEAN_FLOOR).update(localX, y, localZ, state);
        chunk.getOrCreateHeightmapUnprimed(Heightmap.Types.WORLD_SURFACE).update(localX, y, localZ, state);
        JsonArray heights = new JsonArray();
        for (Heightmap.Types type : TYPES) {
            heights.add(chunk.getOrCreateHeightmapUnprimed(type).getFirstAvailable(localX, localZ));
        }
        return heights;
    }
}
