package rustcraft.golden;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.google.gson.JsonPrimitive;
import com.google.gson.internal.LazilyParsedNumber;
import com.mojang.math.OctahedralGroup;
import com.mojang.math.Quadrant;
import com.mojang.math.SymmetricGroup3;
import com.mojang.math.Transformation;
import java.io.IOException;
import java.io.Reader;
import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeSet;
import java.util.concurrent.CompletableFuture;
import net.minecraft.client.model.geom.EntityModelSet;
import net.minecraft.client.renderer.block.dispatch.BlockModelRotation;
import net.minecraft.client.renderer.block.dispatch.BlockStateModel;
import net.minecraft.client.renderer.block.dispatch.BlockStateModelPart;
import net.minecraft.client.renderer.block.dispatch.ModelState;
import net.minecraft.client.renderer.block.dispatch.SingleVariant;
import net.minecraft.client.renderer.block.dispatch.WeightedVariants;
import net.minecraft.client.renderer.block.dispatch.multipart.MultiPartModel;
import net.minecraft.client.renderer.texture.MissingTextureAtlasSprite;
import net.minecraft.client.renderer.texture.SpriteContents;
import net.minecraft.client.renderer.texture.SpriteLoader;
import net.minecraft.client.renderer.texture.Stitcher;
import net.minecraft.client.renderer.texture.TextureAtlas;
import net.minecraft.client.renderer.texture.TextureAtlasSprite;
import net.minecraft.client.renderer.texture.atlas.SpriteResourceLoader;
import net.minecraft.client.renderer.texture.atlas.SpriteSource;
import net.minecraft.client.renderer.texture.atlas.SpriteSourceList;
import net.minecraft.client.resources.model.BlockStateModelLoader;
import net.minecraft.client.resources.model.ModelBakery;
import net.minecraft.client.resources.model.ModelDiscovery;
import net.minecraft.client.resources.model.ResolvedModel;
import net.minecraft.client.resources.model.UnbakedModel;
import net.minecraft.client.resources.model.cuboid.CuboidModel;
import net.minecraft.client.resources.model.cuboid.ItemModelGenerator;
import net.minecraft.client.resources.model.cuboid.MissingCuboidModel;
import net.minecraft.client.resources.model.geometry.BakedQuad;
import net.minecraft.client.resources.model.sprite.Material;
import net.minecraft.client.resources.model.sprite.MaterialBaker;
import net.minecraft.core.BlockMath;
import net.minecraft.core.Direction;
import net.minecraft.core.FrontAndTop;
import net.minecraft.resources.FileToIdConverter;
import net.minecraft.resources.Identifier;
import net.minecraft.server.packs.resources.Resource;
import net.minecraft.server.packs.resources.ResourceManager;
import net.minecraft.util.random.Weighted;
import net.minecraft.util.random.WeightedList;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;
import org.joml.Matrix4f;
import org.joml.Matrix4fc;
import org.joml.Vector3fc;

/**
 * Block models, from the math that turns them to every block state's baked quads (TESTING.md, "Block models"): the
 * rotation groups, BlockMath, BlockModelRotation, the numbers Gson reads the models' files with, and ModelBakery's bake
 * of every block state, run the way ModelManager runs it over the vanilla pack and its block and item atlases.
 */
final class ModelSuites {
    private static final Gson GSON = new GsonBuilder().disableHtmlEscaping().serializeNulls().create();
    // TextureSuites' options: the oracle client's mipmap levels, no anisotropy, llvmpipe's largest texture.
    private static final int MAX_TEXTURE_SIZE = 16384;
    private static final int MAX_MIPMAP_LEVELS = 4;

    private ModelSuites() {
    }

    static List<Suite> suites() {
        return List.of(
            InstanceSuite.every(SymmetricGroup3.class),
            InstanceSuite.every(OctahedralGroup.class),
            InstanceSuite.every(Quadrant.class).extraInputs("parseJson(I)Lcom/mojang/math/Quadrant;", quadrantDegrees()),
            InstanceSuite.every(FrontAndTop.class),
            new StaticSuite(BlockMath.class)
                .extraInputs(
                    "getFaceTransformation(Lcom/mojang/math/Transformation;Lnet/minecraft/core/Direction;)Lcom/mojang/math/Transformation;",
                    faceTransformationInputs()
                ),
            // Before the bake, which asks isIdentity about the rotations' inverse face matrices.
            new RotationSuite(),
            new NumberSuite(),
            new BakerySuite(),
            new StaticSuite(net.minecraft.util.LightCoordsUtil.class),
            new MeshSuite(),
            new SectionCompilerSuite()
        );
    }

    private static List<Object[]> quadrantDegrees() {
        List<Object[]> inputs = new ArrayList<>();
        for (int degrees = -720; degrees <= 720; degrees += 45) {
            inputs.add(new Object[]{degrees});
        }
        inputs.add(new Object[]{Integer.MIN_VALUE});
        inputs.add(new Object[]{Integer.MAX_VALUE});
        return inputs;
    }

    // Every rotation a block model's variant can have, each as BlockModelRotation makes its transformation.
    private static List<Object[]> faceTransformationInputs() {
        List<Object[]> inputs = new ArrayList<>();
        for (OctahedralGroup group : OctahedralGroup.values()) {
            for (Direction direction : Direction.values()) {
                Transformation transformation = group == OctahedralGroup.IDENTITY
                    ? Transformation.IDENTITY
                    : new Transformation(new Matrix4f(group.transformation()));
                inputs.add(new Object[]{transformation, direction});
            }
        }
        return inputs;
    }

    static JsonArray matrix(final Matrix4fc matrix) {
        return (JsonArray)JomlValues.MATRIX4F.encode(matrix);
    }

    private static JsonArray faces(final java.util.function.Function<Direction, Matrix4fc> matrix) {
        JsonArray faces = new JsonArray();
        for (Direction direction : Direction.values()) {
            faces.add(matrix(matrix.apply(direction)));
        }
        return faces;
    }

    /**
     * Each rotation: its toString, transformation and face matrices, then its uv-locked state's. FaceBakery asks
     * isIdentity about an inverse face matrix before anything else reads it, which adds the properties it works out, so
     * those are recorded as FaceBakery sees them, after the same check.
     */
    private static final class RotationSuite implements Suite {
        @Override
        public String name() {
            return BlockModelRotation.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            for (OctahedralGroup group : OctahedralGroup.values()) {
                BlockModelRotation rotation = BlockModelRotation.get(group);
                ModelState uvLocked = rotation.withUvLock();
                JsonArray result = RenderSuites.array(
                    new JsonPrimitive(rotation.toString()),
                    MathValues.encode(rotation.transformation()),
                    faces(rotation::faceTransformation),
                    faces(rotation::inverseFaceTransformation),
                    new JsonPrimitive(uvLocked.toString()),
                    MathValues.encode(uvLocked.transformation()),
                    faces(uvLocked::faceTransformation),
                    faces(face -> {
                        Matrix4fc inverse = uvLocked.inverseFaceTransformation(face);
                        com.mojang.math.MatrixUtil.isIdentity(inverse);
                        return inverse;
                    })
                );
                writer.result(
                    "get(Lcom/mojang/math/OctahedralGroup;)Lnet/minecraft/client/renderer/block/dispatch/BlockModelRotation;",
                    RenderSuites.array(new JsonPrimitive(group.name())), result, null
                );
            }
        }
    }

    /** Every number in the vanilla pack's models and block states, as Gson reads it: its text, parsed as each type. */
    private static final class NumberSuite implements Suite {
        @Override
        public String name() {
            return LazilyParsedNumber.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            ResourceManager resources = TextureSuites.vanillaResources();
            Set<String> numbers = new TreeSet<>();
            for (String directory : List.of("models", "blockstates")) {
                for (Resource resource : resources.listResources(directory, id -> id.getPath().endsWith(".json")).values()) {
                    try (Reader reader = resource.openAsReader()) {
                        collectNumbers(JsonParser.parseReader(reader), numbers);
                    }
                }
            }
            for (String text : numbers) {
                LazilyParsedNumber number = new LazilyParsedNumber(text);
                JsonArray args = RenderSuites.array(new JsonPrimitive(text));
                writer.result("floatValue()F", args, new JsonPrimitive(Values.floatBits(number.floatValue())), null);
                writer.result("intValue()I", args, new JsonPrimitive(number.intValue()), null);
            }
        }

        private static void collectNumbers(final JsonElement element, final Set<String> numbers) {
            if (element.isJsonArray()) {
                element.getAsJsonArray().forEach(child -> collectNumbers(child, numbers));
            } else if (element.isJsonObject()) {
                element.getAsJsonObject().entrySet().forEach(entry -> collectNumbers(entry.getValue(), numbers));
            } else if (element.isJsonPrimitive() && element.getAsJsonPrimitive().isNumber()) {
                numbers.add(element.getAsString());
            }
        }
    }

    // ---- The bake ----

    /**
     * ModelManager's reload, as far as block states go: the block and item atlases stitched (their sprites' places, since
     * a quad's UVs are the atlas's), every model file, the block states' definitions, the models they need found and
     * resolved, then ModelBakery's bake with a MaterialBaker over the two atlases. Item models aren't discovered:
     * nothing a block state's model needs comes from them.
     *
     * <p>Each block state's model is {@code [kind, particle, materialFlags, contents]}: a single variant's part, a weighted
     * list's {@code [weight, model]} entries, or a multipart's models the state selects. A part is {@code
     * [useAmbientOcclusion, particle, materialFlags, quads]}, its quads unculled then culled toward each direction in
     * Direction's order, and a quad {@code [positions, packedUVs, direction, [atlas, sprite, layer, tintIndex,
     * shadeDirectionOverride, lightEmission]]}. A material is {@code [atlas, sprite, forceTranslucent]}. {@code bake} has
     * the MD5 of each state's encoding, {@code bake[full]} the whole of it for each block's default state.
     */
    private static final class BakerySuite implements Suite {
        @Override
        public String name() {
            return ModelBakery.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            ModelBakery.BakingResult result = bake();
            writer.result("missingModels.block", new JsonArray(), model(result.missingModels().block(), null), null);
            for (BlockState state : Block.BLOCK_STATE_REGISTRY) {
                JsonArray args = RenderSuites.array(new JsonPrimitive(Block.getId(state)));
                JsonElement encoded = model(result.getBlockStateModel(state), state);
                writer.result("bake", args, new JsonPrimitive(md5(GSON.toJson(encoded))), null);
                if (state == state.getBlock().defaultBlockState()) {
                    writer.result("bake[full]", args, encoded, null);
                }
            }
        }

        private static ModelBakery.BakingResult baked;
        private static MaterialBaker materials;

        /** The bake, made once for this suite and the meshes'. */
        static synchronized ModelBakery.BakingResult bake() throws IOException {
            if (baked != null) {
                return baked;
            }
            ResourceManager resources = TextureSuites.vanillaResources();
            SpriteLoader.Preparations blocks = stitch(resources, TextureAtlas.LOCATION_BLOCKS, "blocks", MAX_MIPMAP_LEVELS);
            SpriteLoader.Preparations items = stitch(resources, TextureAtlas.LOCATION_ITEMS, "items", 0);

            FileToIdConverter lister = FileToIdConverter.json("models");
            Map<Identifier, UnbakedModel> models = new HashMap<>();
            for (Map.Entry<Identifier, Resource> file : lister.listMatchingResources(resources).entrySet()) {
                try (Reader reader = file.getValue().openAsReader()) {
                    models.put(lister.fileToId(file.getKey()), CuboidModel.fromStream(reader));
                }
            }
            BlockStateModelLoader.LoadedModels blockStates = BlockStateModelLoader.loadBlockStates(resources, Runnable::run).join();
            ModelDiscovery discovery = new ModelDiscovery(models, MissingCuboidModel.missingModel());
            discovery.addSpecialModel(ItemModelGenerator.GENERATED_ITEM_MODEL_ID, new ItemModelGenerator());
            blockStates.models().values().forEach(discovery::addRoot);
            Map<Identifier, ResolvedModel> resolved = discovery.resolve();

            ModelBakery bakery = new ModelBakery(
                EntityModelSet.EMPTY, id -> null, null, blockStates.models(), Map.of(), resolved, discovery.missingModel()
            );
            materials = new MaterialBaker(blocks, items);
            baked = bakery.bakeModels(materials, Runnable::run).join();
            return baked;
        }

        /** The bake's materials, which the fluids' models are baked from too (ModelManager.loadModels). */
        static synchronized MaterialBaker materials() throws IOException {
            bake();
            return materials;
        }

        // SpriteLoader.stitch's layout (TextureSuites' AtlasSuite), into the sprites it makes. Mip levels don't change
        // what models read: the transparency a quad's region has is worked out from the sprite's first image.
        private static SpriteLoader.Preparations stitch(
            final ResourceManager resources, final Identifier location, final String definition, final int maxMipmapLevels
        ) {
            List<SpriteSource.Loader> loaders = SpriteSourceList.load(resources, Identifier.withDefaultNamespace(definition)).list(resources);
            SpriteResourceLoader resourceLoader = SpriteResourceLoader.create(Set.of());
            List<SpriteContents> sprites = new ArrayList<>();
            for (SpriteSource.Loader loader : loaders) {
                SpriteContents contents = loader.get(resourceLoader);
                if (contents != null) {
                    sprites.add(contents);
                }
            }
            int minTexelSize = Integer.MAX_VALUE;
            int lowestOneBit = 1 << maxMipmapLevels;
            for (SpriteContents sprite : sprites) {
                minTexelSize = Math.min(minTexelSize, Math.min(sprite.width(), sprite.height()));
                int lowestTextureBit = Math.min(Integer.lowestOneBit(sprite.width()), Integer.lowestOneBit(sprite.height()));
                if (lowestTextureBit < lowestOneBit) {
                    lowestOneBit = lowestTextureBit;
                }
            }
            int minPowerOfTwo = net.minecraft.util.Mth.log2(Math.min(minTexelSize, lowestOneBit));
            int mipLevel = minPowerOfTwo < maxMipmapLevels ? minPowerOfTwo : maxMipmapLevels;
            Stitcher<SpriteContents> stitcher = new Stitcher<>(MAX_TEXTURE_SIZE, MAX_TEXTURE_SIZE, mipLevel, 0);
            sprites.forEach(stitcher::registerSprite);
            stitcher.stitch();
            int width = stitcher.getWidth();
            int height = stitcher.getHeight();
            Map<Identifier, TextureAtlasSprite> regions = new HashMap<>();
            try {
                Constructor<TextureAtlasSprite> constructor = TextureAtlasSprite.class.getDeclaredConstructor(
                    Identifier.class, SpriteContents.class, int.class, int.class, int.class, int.class, int.class
                );
                constructor.setAccessible(true);
                stitcher.gatherSprites((contents, x, y, padding) -> {
                    try {
                        regions.put(contents.name(), constructor.newInstance(location, contents, width, height, x, y, padding));
                    } catch (ReflectiveOperationException e) {
                        throw new IllegalStateException(e);
                    }
                });
            } catch (NoSuchMethodException e) {
                throw new IllegalStateException(e);
            }
            return new SpriteLoader.Preparations(
                width, height, mipLevel, regions.get(MissingTextureAtlasSprite.getLocation()), regions, CompletableFuture.completedFuture(null)
            );
        }

        private static JsonElement model(final BlockStateModel model, final BlockState state) {
            JsonArray array = new JsonArray();
            JsonElement contents;
            String kind;
            switch (model) {
                case SingleVariant single -> {
                    kind = "single";
                    contents = part((BlockStateModelPart)field(SingleVariant.class, "model", single));
                }
                case WeightedVariants weighted -> {
                    kind = "weighted";
                    JsonArray entries = new JsonArray();
                    @SuppressWarnings("unchecked")
                    WeightedList<BlockStateModel> list = (WeightedList<BlockStateModel>)field(WeightedVariants.class, "list", weighted);
                    for (Weighted<BlockStateModel> entry : list.unwrap()) {
                        entries.add(RenderSuites.array(new JsonPrimitive(entry.weight()), model(entry.value(), state)));
                    }
                    contents = entries;
                }
                case MultiPartModel multiPart -> {
                    kind = "multipart";
                    JsonArray selected = new JsonArray();
                    for (BlockStateModel selectedModel : selectModels(multiPart, state)) {
                        selected.add(model(selectedModel, state));
                    }
                    contents = selected;
                }
                default -> throw new IllegalArgumentException("no encoding for " + model.getClass().getName());
            }
            array.add(kind);
            array.add(material(model.particleMaterial()));
            array.add(model.materialFlags());
            array.add(contents);
            return array;
        }

        @SuppressWarnings("unchecked")
        private static List<BlockStateModel> selectModels(final MultiPartModel model, final BlockState state) {
            try {
                Object shared = field(MultiPartModel.class, "shared", model);
                Method select = shared.getClass().getDeclaredMethod("selectModels", BlockState.class);
                select.setAccessible(true);
                return (List<BlockStateModel>)select.invoke(shared, state);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }

        private static JsonElement part(final BlockStateModelPart part) {
            JsonArray quads = new JsonArray();
            quads.add(quads(part.getQuads(null)));
            for (Direction direction : Direction.values()) {
                quads.add(quads(part.getQuads(direction)));
            }
            return RenderSuites.array(
                new JsonPrimitive(part.useAmbientOcclusion()), material(part.particleMaterial()), new JsonPrimitive(part.materialFlags()), quads
            );
        }

        private static JsonArray quads(final List<BakedQuad> quads) {
            JsonArray array = new JsonArray();
            for (BakedQuad quad : quads) {
                JsonArray positions = new JsonArray();
                JsonArray uvs = new JsonArray();
                for (int vertex = 0; vertex < BakedQuad.VERTEX_COUNT; vertex++) {
                    positions.add(JomlValues.VECTOR3F.encode((Vector3fc)quad.position(vertex)));
                    uvs.add(quad.packedUV(vertex));
                }
                BakedQuad.MaterialInfo info = quad.materialInfo();
                JsonArray material = RenderSuites.array(
                    new JsonPrimitive(info.sprite().atlasLocation().toString()),
                    new JsonPrimitive(info.sprite().contents().name().toString()),
                    new JsonPrimitive(info.layer().name()),
                    new JsonPrimitive(info.tintIndex()),
                    info.shadeDirectionOverride() == null ? JsonNull.INSTANCE : new JsonPrimitive(info.shadeDirectionOverride().name()),
                    new JsonPrimitive(info.lightEmission())
                );
                array.add(RenderSuites.array(positions, uvs, new JsonPrimitive(quad.direction().name()), material));
            }
            return array;
        }

        private static JsonArray material(final Material.Baked material) {
            return RenderSuites.array(
                new JsonPrimitive(material.sprite().atlasLocation().toString()),
                new JsonPrimitive(material.sprite().contents().name().toString()),
                new JsonPrimitive(material.forceTranslucent())
            );
        }

        private static Object field(final Class<?> type, final String name, final Object object) {
            try {
                Field field = type.getDeclaredField(name);
                field.setAccessible(true);
                return field.get(object);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }

        private static String md5(final String text) {
            try {
                return HexFormat.of().formatHex(MessageDigest.getInstance("MD5").digest(text.getBytes(java.nio.charset.StandardCharsets.UTF_8)));
            } catch (NoSuchAlgorithmException e) {
                throw new IllegalStateException(e);
            }
        }
    }

    /**
     * Block models meshed by ModelBlockRenderer, flat and with ambient occlusion, a section at a time the way
     * SectionCompiler.compile meshes one: each non-air block of the section in BlockPos.betweenClosed's order whose
     * render shape is a model, at its place in the section, with its state's seed, its quads into a BufferBuilder for
     * their layer. The level is a pattern of blocks whose shapes and rendering the port has (MeshLevel), with light that
     * varies from block to block and the default cardinal lighting, and BlockColors are empty: the biomes' tints come
     * later. Each case is a pattern's seed and whether ambient occlusion is on, its result each started layer's {@code
     * [layer, vertexCount, md5]} of the vertex bytes, in ChunkSectionLayer's order.
     */
    private static final class MeshSuite implements Suite {
        @Override
        public String name() {
            return net.minecraft.client.renderer.block.ModelBlockRenderer.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            ModelBakery.BakingResult result = BakerySuite.bake();
            for (int seed = 0; seed < 12; seed++) {
                for (boolean ambientOcclusion : new boolean[]{false, true}) {
                    MeshLevel level = new MeshLevel(seed);
                    writer.result(
                        "compileSection",
                        RenderSuites.array(new JsonPrimitive(seed), new JsonPrimitive(ambientOcclusion)),
                        compile(level, result, ambientOcclusion),
                        null
                    );
                }
            }
        }

        private static JsonArray compile(final MeshLevel level, final ModelBakery.BakingResult models, final boolean ambientOcclusion) {
            net.minecraft.client.renderer.block.ModelBlockRenderer renderer = new net.minecraft.client.renderer.block.ModelBlockRenderer(
                ambientOcclusion, true, new net.minecraft.client.color.block.BlockColors()
            );
            Map<net.minecraft.client.renderer.chunk.ChunkSectionLayer, com.mojang.blaze3d.vertex.BufferBuilder> started =
                new java.util.EnumMap<>(net.minecraft.client.renderer.chunk.ChunkSectionLayer.class);
            Map<net.minecraft.client.renderer.chunk.ChunkSectionLayer, com.mojang.blaze3d.vertex.ByteBufferBuilder> buffers =
                new java.util.EnumMap<>(net.minecraft.client.renderer.chunk.ChunkSectionLayer.class);
            net.minecraft.client.renderer.block.BlockQuadOutput output = (x, y, z, quad, instance) -> started.computeIfAbsent(
                quad.materialInfo().layer(),
                layer -> new com.mojang.blaze3d.vertex.BufferBuilder(
                    buffers.computeIfAbsent(layer, l -> new com.mojang.blaze3d.vertex.ByteBufferBuilder(l.bufferSize())),
                    com.mojang.renderpearl.api.pipeline.PrimitiveTopology.QUADS,
                    layer.vertexFormat()
                )
            ).putBlockBakedQuad(x, y, z, quad, instance);
            for (net.minecraft.core.BlockPos pos : net.minecraft.core.BlockPos.betweenClosed(
                new net.minecraft.core.BlockPos(0, 0, 0), new net.minecraft.core.BlockPos(15, 15, 15)
            )) {
                BlockState state = level.getBlockState(pos);
                if (!state.isAir() && state.getRenderShape() == net.minecraft.world.level.block.RenderShape.MODEL) {
                    renderer.tesselateBlock(
                        output, pos.getX() & 15, pos.getY() & 15, pos.getZ() & 15, level, pos, state, models.getBlockStateModel(state), state.getSeed(pos)
                    );
                }
            }
            JsonArray layers = new JsonArray();
            for (Map.Entry<net.minecraft.client.renderer.chunk.ChunkSectionLayer, com.mojang.blaze3d.vertex.BufferBuilder> entry : started.entrySet()) {
                try (com.mojang.blaze3d.vertex.MeshData mesh = entry.getValue().build()) {
                    if (mesh != null) {
                        java.nio.ByteBuffer vertices = mesh.vertexBuffer();
                        byte[] bytes = new byte[vertices.remaining()];
                        vertices.get(bytes);
                        layers.add(RenderSuites.array(
                            new JsonPrimitive(entry.getKey().name()), new JsonPrimitive(mesh.drawState().vertexCount()), new JsonPrimitive(md5(bytes))
                        ));
                    }
                }
            }
            buffers.values().forEach(com.mojang.blaze3d.vertex.ByteBufferBuilder::close);
            return layers;
        }

        private static String md5(final byte[] bytes) {
            try {
                return HexFormat.of().formatHex(MessageDigest.getInstance("MD5").digest(bytes));
            } catch (NoSuchAlgorithmException e) {
                throw new IllegalStateException(e);
            }
        }
    }

    /**
     * SectionCompiler.compile over RenderSectionRegions made the way RenderRegionCache makes them: the section and its 26
     * neighbours, each a SectionCopy of the level's section (none for one of only air, one past the level's height, or one
     * of a chunk the level doesn't have, Java's EmptyLevelChunk), read through the region. "walk" compiles every section
     * with blocks of the walk capture's chunks (the chunk packet's goldens decode the same chunks), as a level of those
     * chunks holds them, the camera at the walk's eye; "pattern" compiles the middle of 27 sections of MeshLevel's blocks
     * (air for its waterlogged states: fluids come with FluidRenderer), translucent ones among them, with the camera in
     * different places, which decides the order of the translucent
     * layer's sorted indices. Light is MeshLevel's (seed 0 for the walk), BlockColors are empty, and no block entities or
     * fluids are in the sections. "fluids" compiles the middle of 27 sections of FluidLevel's blocks, water and lava at
     * every level among blocks and waterlogged ones, with the fluids' models baked like ModelManager bakes them,
     * BlockColors.createDefault() and tints that vary from block to block (GoldenRegion.tinted), the cameras the
     * pattern's, and the vanilla data pack's block tags bound like a client has them from a vanilla server. A region is made without its constructor, which asks a ClientLevel for its lighting,
     * and answers the level's questions itself. A result is {@code [layers, visibility]}: each rendered layer's {@code
     * [layer, vertexCount, indexCount, md5 of the vertices, md5 of the sorted indices or null]} in ChunkSectionLayer's
     * order, and VisGraph's VisibilitySet as a long, bit {@code from + 6 * to} for each pair of faces it connects.
     */
    private static final class SectionCompilerSuite implements Suite {
        private static final double[] WALK_CAMERA = {0.5, -60.0 + 1.62F, 0.5};
        private static final double[][] PATTERN_CAMERAS = {{8.5, 8.5, 8.5}, {-20.25, 7.0, 3.5}, {30.0, 40.0, -12.0}, {8.0, -3.75, 25.5}};
        private static final int MIN_SECTION_Y = -4;

        @Override
        public String name() {
            return net.minecraft.client.renderer.chunk.SectionCompiler.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            ModelBakery.BakingResult result = BakerySuite.bake();
            net.minecraft.client.renderer.block.BlockStateModelSet models = new net.minecraft.client.renderer.block.BlockStateModelSet(
                result.blockStateModels(), result.missingModels().block()
            );
            // The walk's chunks by position, their sections as the chunk packet's data fills them.
            Map<Long, net.minecraft.world.level.chunk.LevelChunkSection[]> chunks = new java.util.LinkedHashMap<>();
            for (byte[] payload : LevelChunkSuites.payloads("minecraft:level_chunk_with_light")) {
                net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket packet =
                    net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket.STREAM_CODEC.decode(LevelChunkSuites.reader(payload));
                net.minecraft.network.FriendlyByteBuf buffer = packet.chunkData().getReadBuffer();
                net.minecraft.world.level.chunk.LevelChunkSection[] sections = new net.minecraft.world.level.chunk.LevelChunkSection[LevelChunkSuites.SECTION_COUNT];
                for (int i = 0; i < sections.length; i++) {
                    sections[i] = new net.minecraft.world.level.chunk.LevelChunkSection(LevelChunkSuites.factory());
                    sections[i].read(buffer);
                }
                chunks.put(net.minecraft.world.level.ChunkPos.pack(packet.x(), packet.z()), sections);
            }
            for (Map.Entry<Long, net.minecraft.world.level.chunk.LevelChunkSection[]> chunk : chunks.entrySet()) {
                net.minecraft.world.level.ChunkPos pos = net.minecraft.world.level.ChunkPos.unpack(chunk.getKey());
                for (int index = 0; index < LevelChunkSuites.SECTION_COUNT; index++) {
                    if (chunk.getValue()[index].hasOnlyAir()) {
                        continue;
                    }
                    int sectionY = MIN_SECTION_Y + index;
                    net.minecraft.client.renderer.chunk.RenderSectionRegion region = region(0, pos.x(), sectionY, pos.z(), (x, y, z) -> {
                        net.minecraft.world.level.chunk.LevelChunkSection[] sections = chunks.get(net.minecraft.world.level.ChunkPos.pack(x, z));
                        int sectionIndex = y - MIN_SECTION_Y;
                        if (sections == null || sectionIndex < 0 || sectionIndex >= sections.length || sections[sectionIndex].hasOnlyAir()) {
                            return null;
                        }
                        return sections[sectionIndex].getStates().copy();
                    });
                    for (boolean ambientOcclusion : new boolean[]{false, true}) {
                        writer.result(
                            "walk",
                            RenderSuites.array(new JsonPrimitive(pos.x()), new JsonPrimitive(sectionY), new JsonPrimitive(pos.z()), new JsonPrimitive(ambientOcclusion)),
                            compile(models, region, net.minecraft.core.SectionPos.of(pos.x(), sectionY, pos.z()), ambientOcclusion, WALK_CAMERA),
                            null
                        );
                    }
                }
            }
            for (int seed = 0; seed < 8; seed++) {
                MeshLevel level = new MeshLevel(seed);
                net.minecraft.client.renderer.chunk.RenderSectionRegion region = region(seed, 0, 0, 0, (x, y, z) -> {
                    net.minecraft.world.level.chunk.PalettedContainer<BlockState> states = LevelChunkSuites.factory().createForBlockStates();
                    for (int i = 0; i < 4096; i++) {
                        int bx = i & 15;
                        int bz = i >> 4 & 15;
                        int by = i >> 8;
                        BlockState state = level.getBlockState(new net.minecraft.core.BlockPos(16 * x + bx, 16 * y + by, 16 * z + bz));
                        // Fluids come with FluidRenderer: a waterlogged state is left out.
                        if (state.getFluidState().isEmpty()) {
                            states.getAndSetUnchecked(bx, by, bz, state);
                        }
                    }
                    return states;
                });
                for (double[] camera : PATTERN_CAMERAS) {
                    for (boolean ambientOcclusion : new boolean[]{false, true}) {
                        writer.result(
                            "pattern",
                            RenderSuites.array(
                                new JsonPrimitive(seed),
                                new JsonPrimitive(ambientOcclusion),
                                new JsonPrimitive(hexDouble(camera[0])),
                                new JsonPrimitive(hexDouble(camera[1])),
                                new JsonPrimitive(hexDouble(camera[2]))
                            ),
                            compile(models, region, net.minecraft.core.SectionPos.of(0, 0, 0), ambientOcclusion, camera),
                            null
                        );
                    }
                }
            }
            net.minecraft.client.renderer.block.FluidStateModelSet fluidModels = new net.minecraft.client.renderer.block.FluidStateModelSet(
                net.minecraft.client.renderer.block.FluidStateModelSet.bake(BakerySuite.materials()), result.missingModels().fluid()
            );
            net.minecraft.client.color.block.BlockColors blockColors = net.minecraft.client.color.block.BlockColors.createDefault();
            // A client has the server's block tags, a vanilla server's data pack's, which getFlow reads
            // (blocks_fluid_flow); the generator's registries have none bound, so they're bound for these cases only.
            bindBlockTags(vanillaBlockTags());
            try {
                writeFluids(writer, models, fluidModels, blockColors);
            } finally {
                bindBlockTags(Map.of());
            }
            writeFlat(writer, models);
        }

        /**
         * "flat": the sections around a scenario's course on the flat world (bedrock, two dirt and grass at -61), with
         * the blocks its setup's commands place as they're written ({@code strict}), and the light a client has there:
         * the sky's 15 everywhere but inside opaque blocks, and no block light. The faces of a dug hole and those beside
         * a wall and a log get their ambient occlusion from neighbours flat ground hasn't got. A case is the scenario, the
         * section and whether ambient occlusion is on, the camera at the scenario's eye.
         */
        private static void writeFlat(final GoldenWriter writer, final net.minecraft.client.renderer.block.BlockStateModelSet models) throws IOException {
            for (String scenario : List.of("block-update")) {
                Map<net.minecraft.core.BlockPos, BlockState> blocks = setupBlocks(scenario);
                for (int[] section : new int[][]{{0, -4, 0}, {-1, -4, 0}}) {
                    net.minecraft.client.renderer.chunk.RenderSectionRegion region = region(0, section[0], section[1], section[2], (x, y, z) -> {
                        net.minecraft.world.level.chunk.PalettedContainer<BlockState> states = LevelChunkSuites.factory().createForBlockStates();
                        boolean empty = true;
                        for (int i = 0; i < 4096; i++) {
                            int bx = i & 15;
                            int bz = i >> 4 & 15;
                            int by = i >> 8;
                            BlockState state = flatBlock(blocks, new net.minecraft.core.BlockPos(16 * x + bx, 16 * y + by, 16 * z + bz));
                            states.getAndSetUnchecked(bx, by, bz, state);
                            empty &= state.isAir();
                        }
                        return empty ? null : states;
                    });
                    ((GoldenRegion)region).flatLight = true;
                    for (boolean ambientOcclusion : new boolean[]{false, true}) {
                        writer.result(
                            "flat",
                            RenderSuites.array(
                                new JsonPrimitive(scenario),
                                new JsonPrimitive(section[0]),
                                new JsonPrimitive(section[1]),
                                new JsonPrimitive(section[2]),
                                new JsonPrimitive(ambientOcclusion)
                            ),
                            compile(models, region, net.minecraft.core.SectionPos.of(section[0], section[1], section[2]), ambientOcclusion, WALK_CAMERA),
                            null
                        );
                    }
                }
            }
        }

        /** The flat preset's block, or the one the setup placed there last. */
        private static BlockState flatBlock(final Map<net.minecraft.core.BlockPos, BlockState> blocks, final net.minecraft.core.BlockPos pos) {
            BlockState placed = blocks.get(pos);
            if (placed != null) {
                return placed;
            }
            return switch (pos.getY()) {
                case -64 -> net.minecraft.world.level.block.Blocks.BEDROCK.defaultBlockState();
                case -63, -62 -> net.minecraft.world.level.block.Blocks.DIRT.defaultBlockState();
                case -61 -> net.minecraft.world.level.block.Blocks.GRASS_BLOCK.defaultBlockState();
                default -> net.minecraft.world.level.block.Blocks.AIR.defaultBlockState();
            };
        }

        /** The blocks a scenario's {@code fill} and {@code setblock} commands place, the last one's at each place. */
        private static Map<net.minecraft.core.BlockPos, BlockState> setupBlocks(final String scenario) throws IOException {
            JsonObject root = JsonParser.parseString(java.nio.file.Files.readString(
                java.nio.file.Path.of("../../oracle/scenarios/" + scenario + ".json")
            )).getAsJsonObject();
            Map<net.minecraft.core.BlockPos, BlockState> blocks = new HashMap<>();
            for (JsonElement command : root.getAsJsonArray("setup")) {
                String[] parts = command.getAsString().split(" ");
                boolean fill = parts[0].equals("fill");
                int[] numbers = new int[fill ? 6 : 3];
                for (int i = 0; i < numbers.length; i++) {
                    numbers[i] = Integer.parseInt(parts[1 + i]);
                }
                BlockState state;
                try {
                    state = net.minecraft.commands.arguments.blocks.BlockStateParser
                        .parseForBlock(net.minecraft.core.registries.BuiltInRegistries.BLOCK, parts[1 + numbers.length], false)
                        .blockState();
                } catch (com.mojang.brigadier.exceptions.CommandSyntaxException e) {
                    throw new IllegalStateException(e);
                }
                int[] box = fill ? numbers : new int[]{numbers[0], numbers[1], numbers[2], numbers[0], numbers[1], numbers[2]};
                for (net.minecraft.core.BlockPos pos : net.minecraft.core.BlockPos.betweenClosed(
                    Math.min(box[0], box[3]), Math.min(box[1], box[4]), Math.min(box[2], box[5]),
                    Math.max(box[0], box[3]), Math.max(box[1], box[4]), Math.max(box[2], box[5])
                )) {
                    blocks.put(pos.immutable(), state);
                }
            }
            return blocks;
        }

        private static Map<net.minecraft.tags.TagKey<net.minecraft.world.level.block.Block>, List<net.minecraft.core.Holder<net.minecraft.world.level.block.Block>>> vanillaBlockTags() {
            net.minecraft.server.packs.resources.ResourceManager resources = new net.minecraft.server.packs.resources.MultiPackResourceManager(
                net.minecraft.server.packs.PackType.SERVER_DATA,
                net.minecraft.server.packs.repository.ServerPacksSource.createVanillaPackSource().asResourcesSupplier().openResources(null, null).toList()
            );
            @SuppressWarnings("unchecked")
            net.minecraft.tags.TagLoader.ElementLookup<net.minecraft.core.Holder<net.minecraft.world.level.block.Block>> lookup =
                (net.minecraft.tags.TagLoader.ElementLookup<net.minecraft.core.Holder<net.minecraft.world.level.block.Block>>)
                    net.minecraft.tags.TagLoader.ElementLookup.fromFrozenRegistry(net.minecraft.core.registries.BuiltInRegistries.BLOCK);
            return net.minecraft.tags.TagLoader.loadTagsForRegistry(resources, net.minecraft.core.registries.Registries.BLOCK, lookup);
        }

        private static void bindBlockTags(
            final Map<net.minecraft.tags.TagKey<net.minecraft.world.level.block.Block>, List<net.minecraft.core.Holder<net.minecraft.world.level.block.Block>>> tags
        ) {
            net.minecraft.core.registries.BuiltInRegistries.BLOCK
                .prepareTagReload(new net.minecraft.tags.TagLoader.LoadResult<>(net.minecraft.core.registries.Registries.BLOCK, tags))
                .apply();
        }

        private static void writeFluids(
            final GoldenWriter writer,
            final net.minecraft.client.renderer.block.BlockStateModelSet models,
            final net.minecraft.client.renderer.block.FluidStateModelSet fluidModels,
            final net.minecraft.client.color.block.BlockColors blockColors
        ) throws IOException {
            for (int seed = 0; seed < 8; seed++) {
                final int levelSeed = seed;
                net.minecraft.client.renderer.chunk.RenderSectionRegion region = region(seed, 0, 0, 0, (x, y, z) -> {
                    net.minecraft.world.level.chunk.PalettedContainer<BlockState> states = LevelChunkSuites.factory().createForBlockStates();
                    for (int i = 0; i < 4096; i++) {
                        int bx = i & 15;
                        int bz = i >> 4 & 15;
                        int by = i >> 8;
                        states.getAndSetUnchecked(bx, by, bz, FluidLevel.getBlockState(16 * x + bx, 16 * y + by, 16 * z + bz, levelSeed));
                    }
                    return states;
                });
                ((GoldenRegion)region).tinted = true;
                for (double[] camera : PATTERN_CAMERAS) {
                    for (boolean ambientOcclusion : new boolean[]{false, true}) {
                        writer.result(
                            "fluids",
                            RenderSuites.array(
                                new JsonPrimitive(seed),
                                new JsonPrimitive(ambientOcclusion),
                                new JsonPrimitive(hexDouble(camera[0])),
                                new JsonPrimitive(hexDouble(camera[1])),
                                new JsonPrimitive(hexDouble(camera[2]))
                            ),
                            compile(models, fluidModels, blockColors, region, net.minecraft.core.SectionPos.of(0, 0, 0), ambientOcclusion, camera),
                            null
                        );
                    }
                }
            }
        }

        private static String hexDouble(final double value) {
            return String.format("%016x", Double.doubleToRawLongBits(value));
        }

        /** A section's block states by its position, or null for none. */
        private interface SectionStates {
            net.minecraft.world.level.chunk.@org.jspecify.annotations.Nullable PalettedContainer<BlockState> at(int x, int y, int z);
        }

        // RenderRegionCache.createRegion, around the section at (x, y, z).
        private static net.minecraft.client.renderer.chunk.RenderSectionRegion region(
            final int seed, final int x, final int y, final int z, final SectionStates states
        ) {
            net.minecraft.client.renderer.chunk.SectionCopy[] copies = new net.minecraft.client.renderer.chunk.SectionCopy[27];
            for (int sz = z - 1; sz <= z + 1; sz++) {
                for (int sy = y - 1; sy <= y + 1; sy++) {
                    for (int sx = x - 1; sx <= x + 1; sx++) {
                        int index = net.minecraft.client.renderer.chunk.RenderSectionRegion.index(x - 1, y - 1, z - 1, sx, sy, sz);
                        copies[index] = copy(states.at(sx, sy, sz));
                    }
                }
            }
            try {
                GoldenRegion region = (GoldenRegion)unsafe().allocateInstance(GoldenRegion.class);
                region.seed = seed;
                setField(net.minecraft.client.renderer.chunk.RenderSectionRegion.class, region, "minSectionX", x - 1);
                setField(net.minecraft.client.renderer.chunk.RenderSectionRegion.class, region, "minSectionY", y - 1);
                setField(net.minecraft.client.renderer.chunk.RenderSectionRegion.class, region, "minSectionZ", z - 1);
                setField(net.minecraft.client.renderer.chunk.RenderSectionRegion.class, region, "sections", copies);
                return region;
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }

        // A SectionCopy of a level's section: its states (a copy already), no block entities, not the debug level's.
        private static net.minecraft.client.renderer.chunk.SectionCopy copy(
            final net.minecraft.world.level.chunk.@org.jspecify.annotations.Nullable PalettedContainer<BlockState> states
        ) {
            try {
                net.minecraft.client.renderer.chunk.SectionCopy copy =
                    (net.minecraft.client.renderer.chunk.SectionCopy)unsafe().allocateInstance(net.minecraft.client.renderer.chunk.SectionCopy.class);
                setField(net.minecraft.client.renderer.chunk.SectionCopy.class, copy, "blockEntities", Map.of());
                setField(net.minecraft.client.renderer.chunk.SectionCopy.class, copy, "section", states);
                setField(net.minecraft.client.renderer.chunk.SectionCopy.class, copy, "debug", false);
                setField(
                    net.minecraft.client.renderer.chunk.SectionCopy.class,
                    copy,
                    "levelHeightAccessor",
                    net.minecraft.world.level.LevelHeightAccessor.create(-64, 384)
                );
                return copy;
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }

        private static void setField(final Class<?> owner, final Object object, final String name, final Object value) throws ReflectiveOperationException {
            Field field = owner.getDeclaredField(name);
            field.setAccessible(true);
            field.set(object, value);
        }

        private static sun.misc.Unsafe unsafe() throws ReflectiveOperationException {
            Field field = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
            field.setAccessible(true);
            return (sun.misc.Unsafe)field.get(null);
        }

        private static JsonArray compile(
            final net.minecraft.client.renderer.block.BlockStateModelSet models,
            final net.minecraft.client.renderer.chunk.RenderSectionRegion region,
            final net.minecraft.core.SectionPos sectionPos,
            final boolean ambientOcclusion,
            final double[] camera
        ) {
            return compile(
                models,
                new net.minecraft.client.renderer.block.FluidStateModelSet(Map.of(), null),
                new net.minecraft.client.color.block.BlockColors(),
                region,
                sectionPos,
                ambientOcclusion,
                camera
            );
        }

        private static JsonArray compile(
            final net.minecraft.client.renderer.block.BlockStateModelSet models,
            final net.minecraft.client.renderer.block.FluidStateModelSet fluidModels,
            final net.minecraft.client.color.block.BlockColors blockColors,
            final net.minecraft.client.renderer.chunk.RenderSectionRegion region,
            final net.minecraft.core.SectionPos sectionPos,
            final boolean ambientOcclusion,
            final double[] camera
        ) {
            net.minecraft.client.renderer.chunk.SectionCompiler compiler = new net.minecraft.client.renderer.chunk.SectionCompiler(
                ambientOcclusion, true, models, fluidModels, blockColors
            );
            try (net.minecraft.client.renderer.SectionBufferBuilderPack buffers = new net.minecraft.client.renderer.SectionBufferBuilderPack()) {
                // SectionRenderDispatcher.RenderSection.createVertexSorting.
                com.mojang.blaze3d.vertex.VertexSorting sorting = com.mojang.blaze3d.vertex.VertexSorting.byDistance(
                    (float)(camera[0] - sectionPos.minBlockX()), (float)(camera[1] - sectionPos.minBlockY()), (float)(camera[2] - sectionPos.minBlockZ())
                );
                net.minecraft.client.renderer.chunk.SectionCompiler.Results results = compiler.compile(sectionPos, region, sorting, buffers);
                JsonArray layers = new JsonArray();
                for (Map.Entry<net.minecraft.client.renderer.chunk.ChunkSectionLayer, com.mojang.blaze3d.vertex.MeshData> entry : results.renderedLayers.entrySet()) {
                    com.mojang.blaze3d.vertex.MeshData mesh = entry.getValue();
                    java.nio.ByteBuffer indices = mesh.indexBuffer();
                    layers.add(RenderSuites.array(
                        new JsonPrimitive(entry.getKey().name()),
                        new JsonPrimitive(mesh.drawState().vertexCount()),
                        new JsonPrimitive(mesh.drawState().indexCount()),
                        new JsonPrimitive(MeshSuite.md5(bytes(mesh.vertexBuffer()))),
                        indices == null ? JsonNull.INSTANCE : new JsonPrimitive(MeshSuite.md5(bytes(indices)))
                    ));
                }
                long visibility = 0;
                for (Direction from : Direction.values()) {
                    for (Direction to : Direction.values()) {
                        if (results.visibilitySet.visibilityBetween(from, to)) {
                            visibility |= 1L << from.ordinal() + 6 * to.ordinal();
                        }
                    }
                }
                results.release();
                return RenderSuites.array(layers, new JsonPrimitive(visibility));
            }
        }

        private static byte[] bytes(final java.nio.ByteBuffer buffer) {
            java.nio.ByteBuffer view = buffer.duplicate();
            byte[] bytes = new byte[view.remaining()];
            view.get(bytes);
            return bytes;
        }
    }

    /**
     * A region that answers the level's questions itself, MeshLevel's light among them, and tints: white, or with {@code
     * tinted} a colour from MeshLevel's hash of the block and the seed, whatever the resolver.
     */
    static final class GoldenRegion extends net.minecraft.client.renderer.chunk.RenderSectionRegion {
        int seed;
        boolean tinted;
        // "flat"'s light: the sky's 15 but inside opaque blocks, no block light.
        boolean flatLight;

        // Never called: regions are made without a constructor (SectionCompilerSuite.region).
        private GoldenRegion() {
            super(null, 0, 0, 0, null);
        }

        @Override
        public int getBrightness(final net.minecraft.world.level.LightLayer layer, final net.minecraft.core.BlockPos pos) {
            if (this.flatLight) {
                return layer == net.minecraft.world.level.LightLayer.SKY && !this.getBlockState(pos).isSolidRender() ? 15 : 0;
            }
            return layer == net.minecraft.world.level.LightLayer.SKY
                ? Math.floorMod(3 * pos.getX() + 5 * pos.getY() + 7 * pos.getZ() + this.seed, 16)
                : Math.floorMod(7 * pos.getX() + 11 * pos.getY() + 13 * pos.getZ(), 16);
        }

        @Override
        public net.minecraft.world.level.lighting.LevelLightEngine getLightEngine() {
            return net.minecraft.world.level.lighting.LevelLightEngine.EMPTY;
        }

        @Override
        public net.minecraft.world.level.CardinalLighting cardinalLighting() {
            return net.minecraft.world.level.CardinalLighting.DEFAULT;
        }

        @Override
        public int getBlockTint(final net.minecraft.core.BlockPos pos, final net.minecraft.world.level.ColorResolver color) {
            return this.tinted ? 0xFF000000 | MeshLevel.hash(pos.getX(), pos.getY(), pos.getZ(), this.seed + 1) & 0xFFFFFF : -1;
        }

        @Override
        public int getHeight() {
            return 384;
        }

        @Override
        public int getMinY() {
            return -64;
        }
    }

    /**
     * Fluids among blocks, for FluidRenderer: by MeshLevel's hash of the block and the seed, two blocks in seven air,
     * three water or lava (one in three lava) at any of LiquidBlock's levels (a source, flowing, or falling), the rest
     * one of NEIGHBOURS' states, waterlogged ones among them, which hide, overlay or don't hide a fluid's faces.
     */
    static final class FluidLevel {
        static final List<String> NEIGHBOUR_NAMES = List.of(
            "stone", "grass_block", "glass", "ice", "oak_leaves", "oak_slab", "oak_stairs", "oak_fence", "mangrove_roots",
            "soul_sand", "glowstone", "white_stained_glass"
        );
        static final List<net.minecraft.world.level.block.Block> NEIGHBOURS = NEIGHBOUR_NAMES.stream()
            .map(name -> net.minecraft.core.registries.BuiltInRegistries.BLOCK.getValue(Identifier.withDefaultNamespace(name)))
            .toList();

        static BlockState getBlockState(final int x, final int y, final int z, final int seed) {
            int h = MeshLevel.hash(x, y, z, seed);
            int kind = h % 7;
            if (kind < 2) {
                return net.minecraft.world.level.block.Blocks.AIR.defaultBlockState();
            }
            if (kind < 5) {
                net.minecraft.world.level.block.Block fluid = h / 7 % 3 == 0
                    ? net.minecraft.world.level.block.Blocks.LAVA
                    : net.minecraft.world.level.block.Blocks.WATER;
                return fluid.defaultBlockState().setValue(net.minecraft.world.level.block.LiquidBlock.LEVEL, h / 21 % 16);
            }
            net.minecraft.world.level.block.Block block = NEIGHBOURS.get(h / 7 % NEIGHBOURS.size());
            List<BlockState> states = block.getStateDefinition().getPossibleStates();
            return states.get(h / 7 / NEIGHBOURS.size() % states.size());
        }
    }

    /**
     * A level of blocks in a pattern: each block, at any height, one of BLOCKS' states (air two times in five) picked by
     * a hash of its position and the seed, a pattern that crosses the section's sides. Sky light is {@code floorMod(3x +
     * 5y + 7z + seed, 16)} and block light {@code floorMod(7x + 11y + 13z, 16)}; tints are white.
     */
    static final class MeshLevel implements net.minecraft.client.renderer.block.BlockAndTintGetter {
        // Whole blocks of the kinds whose shapes and rendering the port has: opaque, cut out, translucent, glowing,
        // emissive, rotated by axis or facing, invisible, and with faces inside that aren't culled.
        static final List<String> BLOCK_NAMES = List.of(
            "stone", "dirt", "grass_block", "bedrock", "cobblestone", "oak_planks", "oak_log", "glass", "white_stained_glass",
            "tinted_glass", "ice", "oak_leaves", "glowstone", "magma_block", "crafting_table", "furnace", "bookshelf", "barrier",
            "quartz_pillar", "snow_block", "slime_block", "beacon", "vault", "mangrove_roots"
        );
        static final List<net.minecraft.world.level.block.Block> BLOCKS = BLOCK_NAMES.stream()
            .map(name -> net.minecraft.core.registries.BuiltInRegistries.BLOCK.getValue(Identifier.withDefaultNamespace(name)))
            .toList();
        private final int seed;

        MeshLevel(final int seed) {
            this.seed = seed;
        }

        static int hash(final int x, final int y, final int z, final int seed) {
            int h = x * 73856093 ^ y * 19349663 ^ z * 83492791 ^ seed * 2654435;
            h ^= h >>> 13;
            h *= 0x5bd1e995;
            h ^= h >>> 15;
            return h & Integer.MAX_VALUE;
        }

        @Override
        public BlockState getBlockState(final net.minecraft.core.BlockPos pos) {
            int h = hash(pos.getX(), pos.getY(), pos.getZ(), this.seed);
            if (h % 5 < 2) {
                return net.minecraft.world.level.block.Blocks.AIR.defaultBlockState();
            }
            net.minecraft.world.level.block.Block block = BLOCKS.get(h / 5 % BLOCKS.size());
            List<BlockState> states = block.getStateDefinition().getPossibleStates();
            return states.get(h / 5 / BLOCKS.size() % states.size());
        }

        @Override
        public net.minecraft.world.level.material.FluidState getFluidState(final net.minecraft.core.BlockPos pos) {
            return this.getBlockState(pos).getFluidState();
        }

        @Override
        public net.minecraft.world.level.block.entity.@org.jspecify.annotations.Nullable BlockEntity getBlockEntity(final net.minecraft.core.BlockPos pos) {
            return null;
        }

        @Override
        public int getHeight() {
            return 384;
        }

        @Override
        public int getMinY() {
            return -64;
        }

        @Override
        public net.minecraft.world.level.lighting.LevelLightEngine getLightEngine() {
            return net.minecraft.world.level.lighting.LevelLightEngine.EMPTY;
        }

        @Override
        public int getBrightness(final net.minecraft.world.level.LightLayer layer, final net.minecraft.core.BlockPos pos) {
            return layer == net.minecraft.world.level.LightLayer.SKY
                ? Math.floorMod(3 * pos.getX() + 5 * pos.getY() + 7 * pos.getZ() + this.seed, 16)
                : Math.floorMod(7 * pos.getX() + 11 * pos.getY() + 13 * pos.getZ(), 16);
        }

        @Override
        public net.minecraft.world.level.CardinalLighting cardinalLighting() {
            return net.minecraft.world.level.CardinalLighting.DEFAULT;
        }

        @Override
        public int getBlockTint(final net.minecraft.core.BlockPos pos, final net.minecraft.world.level.ColorResolver color) {
            return -1;
        }
    }
}
