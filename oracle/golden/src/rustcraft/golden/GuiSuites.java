package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.ByteBufferBuilder;
import com.mojang.blaze3d.vertex.MeshData;
import com.mojang.renderpearl.api.pipeline.RenderPipeline;
import java.io.IOException;
import java.lang.reflect.Field;
import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HexFormat;
import java.util.List;
import java.util.Optional;
import java.util.Random;
import java.util.Set;
import net.minecraft.client.gui.navigation.ScreenDirection;
import net.minecraft.client.gui.navigation.ScreenRectangle;
import net.minecraft.client.gui.render.GuiRenderer;
import net.minecraft.client.gui.render.TextureSetup;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.client.renderer.state.gui.BlitRenderState;
import net.minecraft.client.renderer.state.gui.ColoredRectangleRenderState;
import net.minecraft.client.renderer.state.gui.GuiElementRenderState;
import net.minecraft.client.renderer.state.gui.GuiRenderState;
import net.minecraft.client.renderer.state.gui.TiledBlitRenderState;
import net.minecraft.client.renderer.texture.SpriteContents;
import net.minecraft.client.renderer.texture.atlas.SpriteResourceLoader;
import net.minecraft.client.renderer.texture.atlas.SpriteSource;
import net.minecraft.client.renderer.texture.atlas.SpriteSourceList;
import net.minecraft.client.resources.metadata.gui.GuiMetadataSection;
import net.minecraft.client.resources.metadata.gui.GuiSpriteScaling;
import net.minecraft.resources.Identifier;
import net.minecraft.server.packs.resources.ResourceManager;
import org.joml.Matrix3x2f;

/**
 * The GUI's pure pieces (TESTING.md, "The GUI"): ScreenRectangle's geometry, GuiRenderState's strata and layers with
 * GuiRenderer's sort, the element states' vertices, and every GUI sprite's scaling.
 */
final class GuiSuites {
    private static final Identifier GUI = Identifier.withDefaultNamespace("gui");

    private GuiSuites() {
    }

    static List<Suite> suites() {
        return List.of(new ScreenRectangleSuite(), new GuiRenderStateSuite(), new SpriteScalingSuite());
    }

    // ---- Values ----

    static JsonArray rectangle(final ScreenRectangle rectangle) {
        return RenderSuites.array(
            new JsonPrimitive(rectangle.left()), new JsonPrimitive(rectangle.top()), new JsonPrimitive(rectangle.width()), new JsonPrimitive(rectangle.height())
        );
    }

    static JsonElement nullableRectangle(final ScreenRectangle rectangle) {
        return rectangle == null ? JsonNull.INSTANCE : rectangle(rectangle);
    }

    static JsonPrimitive bits(final float value) {
        return new JsonPrimitive(String.format("%08x", Float.floatToRawIntBits(value)));
    }

    static JsonArray pose(final Matrix3x2f pose) {
        return RenderSuites.array(bits(pose.m00), bits(pose.m01), bits(pose.m10), bits(pose.m11), bits(pose.m20), bits(pose.m21));
    }

    /** Rectangles in a GUI of 427 by 240 (the walk's), mostly small and overlapping, some empty, some negative. */
    static ScreenRectangle randomRectangle(final Random random) {
        return switch (random.nextInt(8)) {
            case 0 -> new ScreenRectangle(random.nextInt(41) - 20, random.nextInt(41) - 20, random.nextInt(3), random.nextInt(3));
            case 1 -> new ScreenRectangle(random.nextInt(427), random.nextInt(240), random.nextInt(427), random.nextInt(240));
            case 2 -> new ScreenRectangle(random.nextInt(2000) - 1000, random.nextInt(2000) - 1000, random.nextInt(2000) - 1000, random.nextInt(2000) - 1000);
            default -> new ScreenRectangle(random.nextInt(60), random.nextInt(60), random.nextInt(30), random.nextInt(30));
        };
    }

    /** The GUI's poses: the identity, pixel translations, GUI scales, a quarter turn, and those combined. */
    static Matrix3x2f randomPose(final Random random) {
        Matrix3x2f pose = new Matrix3x2f();
        int steps = random.nextInt(3);
        for (int i = 0; i < steps; i++) {
            switch (random.nextInt(5)) {
                case 0 -> pose.translate(random.nextInt(41) - 20, random.nextInt(41) - 20);
                case 1 -> pose.translate(random.nextFloat() * 40.0F - 20.0F, random.nextFloat() * 40.0F - 20.0F);
                case 2 -> {
                    float scale = new float[]{0.5F, 2.0F, 4.0F, 1.0F / 3.0F}[random.nextInt(4)];
                    pose.scale(scale, scale);
                }
                case 3 -> pose.scale(1.0F + random.nextFloat(), 1.0F + random.nextFloat());
                default -> pose.rotate((float)(random.nextInt(8) * Math.PI / 4.0));
            }
        }
        return pose;
    }

    // ---- ScreenRectangle ----

    private static final class ScreenRectangleSuite implements Suite {
        @Override
        public String name() {
            return ScreenRectangle.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(41);
            List<ScreenRectangle> rectangles = new ArrayList<>(List.of(
                ScreenRectangle.empty(), new ScreenRectangle(0, 0, 427, 240), new ScreenRectangle(122, 218, 182, 22),
                new ScreenRectangle(121, 217, 24, 23), new ScreenRectangle(194, 201, 9, 9), new ScreenRectangle(186, 201, 9, 9),
                new ScreenRectangle(Integer.MAX_VALUE, Integer.MIN_VALUE, Integer.MAX_VALUE, -1)
            ));
            for (int i = 0; i < 300; i++) {
                rectangles.add(randomRectangle(random));
            }
            for (ScreenRectangle a : rectangles) {
                JsonArray self = rectangle(a);
                ScreenRectangle b = rectangles.get(random.nextInt(rectangles.size()));
                // A neighbour, touching or overlapping by a pixel.
                ScreenRectangle near = new ScreenRectangle(a.left() + a.width() + random.nextInt(3) - 1, a.top() + random.nextInt(3) - 1, 1 + random.nextInt(5), 1 + random.nextInt(5));
                for (ScreenRectangle other : List.of(b, near, a)) {
                    JsonArray args = RenderSuites.array(self, rectangle(other));
                    writer.result("intersection", args, nullableRectangle(a.intersection(other)), null);
                    writer.result("intersects", args, new JsonPrimitive(a.intersects(other)), null);
                    writer.result("encompasses", args, new JsonPrimitive(a.encompasses(other)), null);
                    writer.result("overlaps", args, new JsonPrimitive(a.overlaps(other)), null);
                }
                int x = a.left() + random.nextInt(Math.max(1, Math.abs(a.width()) + 2)) - 1;
                int y = a.top() + random.nextInt(Math.max(1, Math.abs(a.height()) + 2)) - 1;
                writer.result("containsPoint", RenderSuites.array(self, new JsonPrimitive(x), new JsonPrimitive(y)), new JsonPrimitive(a.containsPoint(x, y)), null);
                for (ScreenDirection direction : ScreenDirection.values()) {
                    writer.result("getBorder", RenderSuites.array(self, new JsonPrimitive(direction.name())), rectangle(a.getBorder(direction)), null);
                }
                Matrix3x2f pose = randomPose(random);
                JsonArray poseArgs = RenderSuites.array(self, pose(pose));
                writer.result("transformMaxBounds", poseArgs, rectangle(a.transformMaxBounds(pose)), null);
                writer.result("transformAxisAligned", poseArgs, rectangle(a.transformAxisAligned(pose)), null);
            }
        }
    }

    // ---- GuiRenderState ----

    /**
     * Programs of two frames over one GuiRenderState: each adds elements (fills, blits and tiled blits of random
     * rectangles under random poses and scissors, with the GUI's pipelines and no texture) between new strata and
     * layers, then sorts the layers with GuiRenderer's comparator and lists the elements in the order the renderer
     * draws them. A program's second frame starts after reset(), which keeps the last element's bounds, so its first
     * element can go above the first layer. An element is its index in the frame's ops; the ops' encodings are what
     * the port builds the same elements from, and "vertices" is each element's vertices, as a BufferBuilder of its
     * pipeline's format writes them.
     */
    private static final class GuiRenderStateSuite implements Suite {
        // The pipelines the GUI fills with, and the ones it blits with, whose vertex formats the elements build.
        private static final List<RenderPipeline> FILL_PIPELINES = List.of(
            RenderPipelines.GUI, RenderPipelines.GUI_INVERT, RenderPipelines.GUI_TEXT_HIGHLIGHT
        );
        private static final List<RenderPipeline> BLIT_PIPELINES = List.of(
            RenderPipelines.GUI_TEXTURED, RenderPipelines.CROSSHAIR, RenderPipelines.GUI_TEXTURED_PREMULTIPLIED_ALPHA
        );

        @Override
        public String name() {
            return GuiRenderState.class.getName();
        }

        @SuppressWarnings("unchecked")
        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Comparator<GuiElementRenderState> comparator;
            try {
                Field field = GuiRenderer.class.getDeclaredField("ELEMENT_SORT_COMPARATOR");
                field.setAccessible(true);
                comparator = (Comparator<GuiElementRenderState>)field.get(null);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }

            Random random = new Random(43);
            for (int program = 0; program < 400; program++) {
                GuiRenderState state = new GuiRenderState();
                JsonArray frames = new JsonArray();
                for (int frame = 0; frame < 2; frame++) {
                    state.reset();
                    JsonArray ops = new JsonArray();
                    List<GuiElementRenderState> elements = new ArrayList<>();
                    int opCount = 1 + random.nextInt(program < 200 ? 12 : 40);
                    for (int i = 0; i < opCount; i++) {
                        int kind = random.nextInt(20);
                        if (kind == 0) {
                            state.nextStratum();
                            ops.add(RenderSuites.array(new JsonPrimitive("nextStratum")));
                        } else if (kind == 1) {
                            state.up();
                            ops.add(RenderSuites.array(new JsonPrimitive("up")));
                        } else {
                            List<RenderPipeline> pipelines = kind % 3 == 0 ? FILL_PIPELINES : BLIT_PIPELINES;
                            RenderPipeline pipeline = pipelines.get(random.nextInt(pipelines.size()));
                            Matrix3x2f pose = random.nextInt(3) == 0 ? randomPose(random) : new Matrix3x2f();
                            // A tiled blit's vertices grow with its area over its tiles', so it keeps to small ones.
                            ScreenRectangle rectangle = random.nextInt(4) == 0 && kind % 3 != 2 ? randomRectangle(random) : new ScreenRectangle(
                                random.nextInt(80), random.nextInt(60), random.nextInt(24), random.nextInt(24)
                            );
                            int x0 = rectangle.left();
                            int y0 = rectangle.top();
                            int x1 = rectangle.right();
                            int y1 = rectangle.bottom();
                            ScreenRectangle scissor = random.nextInt(5) == 0 ? randomRectangle(random) : null;
                            int color = random.nextInt();
                            float u0 = random.nextFloat();
                            float u1 = random.nextFloat();
                            float v0 = random.nextFloat();
                            float v1 = random.nextFloat();
                            GuiElementRenderState element;
                            JsonArray op;
                            switch (kind % 3) {
                                case 0 -> {
                                    int color2 = random.nextBoolean() ? color : random.nextInt();
                                    element = new ColoredRectangleRenderState(pipeline, TextureSetup.noTexture(), pose, x0, y0, x1, y1, color, color2, scissor);
                                    op = RenderSuites.array(
                                        new JsonPrimitive("fill"), new JsonPrimitive(pipeline.getLocation().toString()), pose(pose),
                                        new JsonPrimitive(x0), new JsonPrimitive(y0), new JsonPrimitive(x1), new JsonPrimitive(y1),
                                        new JsonPrimitive(color), new JsonPrimitive(color2), nullableRectangle(scissor)
                                    );
                                }
                                case 1 -> {
                                    element = new BlitRenderState(pipeline, TextureSetup.noTexture(), pose, x0, y0, x1, y1, u0, u1, v0, v1, color, scissor);
                                    op = RenderSuites.array(
                                        new JsonPrimitive("blit"), new JsonPrimitive(pipeline.getLocation().toString()), pose(pose),
                                        new JsonPrimitive(x0), new JsonPrimitive(y0), new JsonPrimitive(x1), new JsonPrimitive(y1),
                                        bits(u0), bits(u1), bits(v0), bits(v1), new JsonPrimitive(color), nullableRectangle(scissor)
                                    );
                                }
                                default -> {
                                    int tileWidth = 1 + random.nextInt(9);
                                    int tileHeight = 1 + random.nextInt(9);
                                    element = new TiledBlitRenderState(
                                        pipeline, TextureSetup.noTexture(), pose, tileWidth, tileHeight, x0, y0, x1, y1, u0, u1, v0, v1, color, scissor
                                    );
                                    op = RenderSuites.array(
                                        new JsonPrimitive("tiled"), new JsonPrimitive(pipeline.getLocation().toString()), pose(pose),
                                        new JsonPrimitive(tileWidth), new JsonPrimitive(tileHeight),
                                        new JsonPrimitive(x0), new JsonPrimitive(y0), new JsonPrimitive(x1), new JsonPrimitive(y1),
                                        bits(u0), bits(u1), bits(v0), bits(v1), new JsonPrimitive(color), nullableRectangle(scissor)
                                    );
                                }
                            }
                            state.addGuiElement(element);
                            elements.add(element);
                            ops.add(op);
                        }
                    }

                    state.sortElements(comparator);
                    JsonArray order = new JsonArray();
                    state.forEachElement(element -> order.add(elements.indexOf(element)), GuiRenderState.TraverseRange.ALL);
                    JsonArray bounds = new JsonArray();
                    JsonArray vertices = new JsonArray();
                    for (GuiElementRenderState element : elements) {
                        bounds.add(nullableRectangle(element.bounds()));
                        vertices.add(vertices(element));
                    }
                    frames.add(RenderSuites.array(ops, order, bounds, vertices));
                }
                writer.result("program", RenderSuites.array(new JsonPrimitive(program)), frames, null);
            }
        }

        private static JsonElement vertices(final GuiElementRenderState element) {
            RenderPipeline pipeline = element.pipeline();
            try (ByteBufferBuilder buffer = new ByteBufferBuilder(1024)) {
                BufferBuilder builder = new BufferBuilder(buffer, pipeline.getPrimitiveTopology(), pipeline.getVertexFormatBinding(0));
                element.buildVertices(builder);
                try (MeshData mesh = builder.build()) {
                    if (mesh == null) {
                        return JsonNull.INSTANCE;
                    }
                    ByteBuffer bytes = mesh.vertexBuffer();
                    byte[] array = new byte[bytes.remaining()];
                    bytes.duplicate().get(array);
                    return new JsonPrimitive(HexFormat.of().formatHex(array));
                }
            }
        }
    }

    // ---- The GUI atlas's sprites ----

    /** Every sprite the GUI atlas loads, with its gui section's scaling as GuiGraphicsExtractor reads it. */
    private static final class SpriteScalingSuite implements Suite {
        @Override
        public String name() {
            return GuiSpriteScaling.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            ResourceManager resources = TextureSuites.vanillaResources();
            SpriteResourceLoader loader = SpriteResourceLoader.create(Set.of(GuiMetadataSection.TYPE));
            for (SpriteSource.Loader source : SpriteSourceList.load(resources, GUI).list(resources)) {
                SpriteContents contents = source.get(loader);
                if (contents == null) {
                    continue;
                }
                Optional<GuiMetadataSection> section = contents.getAdditionalMetadata(GuiMetadataSection.TYPE);
                writer.result(
                    "scaling", RenderSuites.array(new JsonPrimitive(contents.name().toString())),
                    section.map(gui -> encode(gui.scaling())).orElse(JsonNull.INSTANCE), null
                );
                contents.close();
            }
        }

        private static JsonElement encode(final GuiSpriteScaling scaling) {
            return switch (scaling) {
                case GuiSpriteScaling.Stretch stretch -> RenderSuites.array(new JsonPrimitive("stretch"));
                case GuiSpriteScaling.Tile tile -> RenderSuites.array(new JsonPrimitive("tile"), new JsonPrimitive(tile.width()), new JsonPrimitive(tile.height()));
                case GuiSpriteScaling.NineSlice nineSlice -> RenderSuites.array(
                    new JsonPrimitive("nine_slice"), new JsonPrimitive(nineSlice.width()), new JsonPrimitive(nineSlice.height()),
                    RenderSuites.array(
                        new JsonPrimitive(nineSlice.border().left()), new JsonPrimitive(nineSlice.border().top()),
                        new JsonPrimitive(nineSlice.border().right()), new JsonPrimitive(nineSlice.border().bottom())
                    ),
                    new JsonPrimitive(nineSlice.stretchInner())
                );
                default -> throw new IllegalArgumentException("unknown scaling " + scaling);
            };
        }
    }
}
