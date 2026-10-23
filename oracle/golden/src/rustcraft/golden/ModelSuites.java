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
            new BakerySuite()
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
            ModelBakery.BakingResult result = bakery.bakeModels(new MaterialBaker(blocks, items), Runnable::run).join();

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
}
