package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonPrimitive;
import com.mojang.blaze3d.platform.NativeImage;
import java.io.IOException;
import java.io.InputStream;
import java.lang.reflect.Field;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import net.minecraft.client.renderer.texture.SpriteContents;
import net.minecraft.client.renderer.texture.Stitcher;
import net.minecraft.client.renderer.texture.atlas.SpriteResourceLoader;
import net.minecraft.client.renderer.texture.atlas.SpriteSource;
import net.minecraft.client.renderer.texture.atlas.SpriteSourceList;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.Identifier;
import net.minecraft.server.packs.PackLocationInfo;
import net.minecraft.server.packs.PackResources;
import net.minecraft.server.packs.PackType;
import net.minecraft.server.packs.VanillaPackResources;
import net.minecraft.server.packs.VanillaPackResourcesBuilder;
import net.minecraft.server.packs.repository.PackSource;
import net.minecraft.server.packs.resources.MultiPackResourceManager;
import net.minecraft.server.packs.resources.Resource;
import net.minecraft.server.packs.resources.ResourceManager;

/**
 * Textures and the block atlas, from the client's own resource manager over the vanilla pack (TESTING.md, "Textures and
 * the block atlas"). An image is {@code [width, height, md5]}, the MD5 of its RGBA bytes as they lie in memory.
 */
final class TextureSuites {
    // llvmpipe's GL_MAX_TEXTURE_SIZE, the atlas's maxSupportedTextureSize there. The block atlas is far smaller.
    private static final int MAX_TEXTURE_SIZE = 16384;
    // The oracle client's options: mipmapLevels 4, and texture filtering that isn't anisotropic, so no anisotropy bit.
    private static final int MAX_MIPMAP_LEVELS = 4;
    private static final int ANISOTROPY_BIT = 0;
    private static final Identifier BLOCKS = Identifier.withDefaultNamespace("blocks");
    private static final Identifier GUI = Identifier.withDefaultNamespace("gui");

    private TextureSuites() {
    }

    static List<Suite> suites() {
        return List.of(new ResourcesSuite(), new NativeImageSuite(), new AtlasSuite());
    }

    // ---- The vanilla pack ----

    private static ResourceManager manager;

    /**
     * ClientPackSource.createVanillaPackSource's pack, its layers as the pack repository opens them, in a
     * MultiPackResourceManager like the client's. The launcher's asset index, its second layer's path, isn't on this
     * machine, so that layer is empty: the index holds sounds and translations, none of the textures read here.
     */
    static synchronized ResourceManager vanillaResources() {
        if (manager == null) {
            // The client's bootstrap, which registers the atlases' sprite sources among others, as Main does.
            net.minecraft.client.ClientBootstrap.bootstrap();
            PackLocationInfo location = new PackLocationInfo("vanilla", Component.literal("vanilla"), PackSource.BUILT_IN, Optional.empty());
            VanillaPackResources vanilla = new VanillaPackResourcesBuilder()
                .exposeNamespace("minecraft", "realms")
                .pushJarResources()
                .pushLayer()
                .pushAssetPath(PackType.CLIENT_RESOURCES, java.nio.file.Path.of("no-asset-index"))
                .build(location);
            List<PackResources> layers = vanilla.asResourcesSupplier().openResources(location, null).toList();
            manager = new MultiPackResourceManager(PackType.CLIENT_RESOURCES, layers);
        }
        return manager;
    }

    static String md5(final NativeImage image) {
        int[] pixels = image.getPixelsABGR();
        ByteBuffer bytes = ByteBuffer.allocate(pixels.length * 4).order(ByteOrder.LITTLE_ENDIAN);
        bytes.asIntBuffer().put(pixels);
        try {
            return HexFormat.of().formatHex(MessageDigest.getInstance("MD5").digest(bytes.array()));
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalStateException(e);
        }
    }

    static JsonArray image(final NativeImage image) {
        return RenderSuites.array(new JsonPrimitive(image.getWidth()), new JsonPrimitive(image.getHeight()), new JsonPrimitive(md5(image)));
    }

    // ---- Suites ----

    /** What the resource manager lists under textures/ and atlases/, and the atlases' resource stacks. */
    private static final class ResourcesSuite implements Suite {
        @Override
        public String name() {
            return MultiPackResourceManager.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            ResourceManager resources = vanillaResources();
            JsonArray namespaces = new JsonArray();
            resources.getNamespaces().stream().sorted().forEach(namespaces::add);
            writer.result("getNamespaces()Ljava/util/Set;", new JsonArray(), namespaces, null);
            for (String directory : List.of("textures", "textures/block", "atlases", "models/block", "blockstates")) {
                JsonArray listed = new JsonArray();
                resources.listResources(directory, id -> true).keySet().forEach(id -> listed.add(id.toString()));
                writer.result("listResources(Ljava/lang/String;Lnet/minecraft/server/packs/resources/ResourceManager$Selector;)Ljava/util/Map;",
                    RenderSuites.array(new JsonPrimitive(directory)), listed, null);
            }
            for (Identifier id : resources.listResources("atlases", id -> id.getPath().endsWith(".json")).keySet()) {
                writer.result("getResourceStack(Lnet/minecraft/resources/Identifier;)Ljava/util/List;",
                    RenderSuites.array(new JsonPrimitive(id.toString())), new JsonPrimitive(resources.getResourceStack(id).size()), null);
            }
        }
    }

    /** NativeImage.read of every PNG under textures/: the RGBA stb decodes. */
    private static final class NativeImageSuite implements Suite {
        @Override
        public String name() {
            return NativeImage.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Map<Identifier, Resource> textures = vanillaResources().listResources("textures", id -> id.getPath().endsWith(".png"));
            for (Map.Entry<Identifier, Resource> texture : textures.entrySet()) {
                JsonArray args = RenderSuites.array(new JsonPrimitive(texture.getKey().toString()));
                try (InputStream stream = texture.getValue().open(); NativeImage image = NativeImage.read(stream)) {
                    writer.result("read(Ljava/io/InputStream;)Lcom/mojang/blaze3d/platform/NativeImage;", args, image(image), null);
                } catch (IOException e) {
                    writer.threw("read(Ljava/io/InputStream;)Lcom/mojang/blaze3d/platform/NativeImage;", args, e, null);
                }
            }
        }
    }

    /**
     * The block atlas's CPU half, as SpriteLoader.loadAndStitch runs it with the oracle client's options: the sprites
     * its sources list and SpriteResourceLoader loads, the Stitcher's layout, and each sprite's mip levels after
     * increaseMipLevel. SpriteLoader.stitch reads the options from Minecraft.getInstance(), which the generator has
     * none of, so this runs its steps with the options' values. The GPU half, drawing the sprites into the atlas
     * texture, is checked against the oracle client's dump instead.
     */
    private static final class AtlasSuite implements Suite {
        @Override
        public String name() {
            return "net.minecraft.client.renderer.texture.SpriteLoader";
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            // The GUI atlas, which AtlasManager stitches without mipmaps, only for its layout.
            for (SpriteContents sprite : stitch(writer, GUI, 0)) {
                sprite.close();
            }

            List<SpriteContents> sprites = stitch(writer, BLOCKS, MAX_MIPMAP_LEVELS);
            int mipLevel = stitchedMipLevel(sprites, MAX_MIPMAP_LEVELS);

            // Preparations.readyForUpload: every sprite's mip levels.
            for (SpriteContents sprite : sprites) {
                sprite.increaseMipLevel(mipLevel);
                JsonArray levels = new JsonArray();
                for (NativeImage level : byMipLevel(sprite)) {
                    levels.add(image(level));
                }
                writer.result("increaseMipLevel", RenderSuites.array(new JsonPrimitive(sprite.name().toString()), new JsonPrimitive(mipLevel)),
                    RenderSuites.array(new JsonPrimitive(sprite.isAnimated()), levels), null);
                sprite.close();
            }
        }

        /** The atlas's sprites, loaded and stitched, its layout written as a "stitch" case. */
        private static List<SpriteContents> stitch(final GoldenWriter writer, final Identifier atlas, final int maxMipmapLevels) throws IOException {
            ResourceManager resources = vanillaResources();
            List<SpriteSource.Loader> loaders = SpriteSourceList.load(resources, atlas).list(resources);
            SpriteResourceLoader resourceLoader = SpriteResourceLoader.create(Set.of());
            List<SpriteContents> sprites = new ArrayList<>();
            for (SpriteSource.Loader loader : loaders) {
                SpriteContents contents = loader.get(resourceLoader);
                if (contents != null) {
                    sprites.add(contents);
                }
            }

            int mipLevel = stitchedMipLevel(sprites, maxMipmapLevels);
            Stitcher<SpriteContents> stitcher = new Stitcher<>(MAX_TEXTURE_SIZE, MAX_TEXTURE_SIZE, mipLevel, ANISOTROPY_BIT);
            sprites.forEach(stitcher::registerSprite);
            stitcher.stitch();
            JsonArray placed = new JsonArray();
            stitcher.gatherSprites((contents, x, y, padding) -> placed.add(RenderSuites.array(
                new JsonPrimitive(contents.name().toString()), new JsonPrimitive(x), new JsonPrimitive(y),
                new JsonPrimitive(contents.width()), new JsonPrimitive(contents.height()), new JsonPrimitive(padding)
            )));
            writer.result("stitch", RenderSuites.array(new JsonPrimitive(atlas.toString()), new JsonPrimitive(maxMipmapLevels)), RenderSuites.array(
                new JsonPrimitive(stitcher.getWidth()), new JsonPrimitive(stitcher.getHeight()), new JsonPrimitive(mipLevel), placed
            ), null);
            return sprites;
        }

        /** SpriteLoader.stitch's mip level: the smallest sprite's, at most the atlas's maximum. */
        private static int stitchedMipLevel(final List<SpriteContents> sprites, final int maxMipmapLevels) {
            int minTexelSize = Integer.MAX_VALUE;
            int lowestOneBit = 1 << maxMipmapLevels;
            for (SpriteContents sprite : sprites) {
                minTexelSize = Math.min(minTexelSize, Math.min(sprite.width(), sprite.height()));
                int lowestTextureBit = Math.min(Integer.lowestOneBit(sprite.width()), Integer.lowestOneBit(sprite.height()));
                if (lowestTextureBit < lowestOneBit) {
                    lowestOneBit = lowestTextureBit;
                }
            }
            int minSize = Math.min(minTexelSize, lowestOneBit);
            int minPowerOfTwo = net.minecraft.util.Mth.log2(minSize);
            return minPowerOfTwo < maxMipmapLevels ? minPowerOfTwo : maxMipmapLevels;
        }

        private static NativeImage[] byMipLevel(final SpriteContents sprite) {
            try {
                Field field = SpriteContents.class.getDeclaredField("byMipLevel");
                field.setAccessible(true);
                return Objects.requireNonNull((NativeImage[])field.get(sprite));
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }
    }
}
