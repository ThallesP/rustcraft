package rustcraft.golden;

import com.google.common.collect.Lists;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import com.mojang.blaze3d.font.GlyphInfo;
import com.mojang.blaze3d.font.GlyphProvider;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.ByteBufferBuilder;
import com.mojang.blaze3d.vertex.MeshData;
import com.mojang.renderpearl.api.buffers.GpuBuffer;
import com.mojang.renderpearl.api.buffers.GpuBufferSlice;
import com.mojang.renderpearl.api.device.DeviceInfo;
import com.mojang.renderpearl.api.device.GpuDevice;
import com.mojang.renderpearl.api.pipeline.RenderPipeline;
import java.io.IOException;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationHandler;
import java.lang.reflect.Method;
import java.lang.reflect.Proxy;
import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeSet;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Executor;
import net.minecraft.client.StringSplitter;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.font.FontManager;
import net.minecraft.client.gui.font.FontSet;
import net.minecraft.client.gui.font.TextRenderable;
import net.minecraft.client.gui.font.glyphs.BakedGlyph;
import net.minecraft.client.gui.font.glyphs.BakedSheetGlyph;
import net.minecraft.client.gui.navigation.ScreenRectangle;
import net.minecraft.client.renderer.texture.TextureManager;
import net.minecraft.network.chat.FormattedText;
import net.minecraft.network.chat.Style;
import net.minecraft.network.chat.TextColor;
import net.minecraft.resources.Identifier;
import net.minecraft.server.packs.resources.ResourceManager;
import org.joml.Matrix4f;

/**
 * The font (TESTING.md, "The font"): FontManager's fonts loaded from the vanilla pack, every glyph their providers
 * define baked in a fixed order (metrics, UVs and the pages so far, so the pages' layout), Font.width, the glyphs and
 * effects prepareText lays out (their bounds and vertices), and StringSplitter's cuts. The pages are on a stand-in
 * device that hands out stand-in textures, buffers and encoders, which take uploads and do nothing with them.
 */
final class FontSuites {
    private static final List<String> FONTS = List.of("minecraft:default", "minecraft:alt", "minecraft:illageralt", "minecraft:uniform");
    // Strings of the GUI's: plain, with every § code but obfuscation, accented, Cyrillic and Greek, a CJK codepoint the
    // jar's fonts haven't got, a supplementary one, spaces and newlines, and the empty string.
    static final List<String> STRINGS = List.of(
        "", " ", "Game Menu", "Back to Game", "Advancements", "Statistics", "Options...", "Save and Quit to Title",
        "Hello, world!", "§cRed §lbold §r§oitalic §nunder§mline§r plain", "§6Gold§7Gray§8DarkGray§9Blue§aGreen§bAqua",
        "§x§zunknown codes§", "Über Ärger öffnet Ölfässer", "Привет, мир", "Γειά σου Κόσμε", "中文", "😀 emoji",
        "tab\there", "line one\nline two", "a  b   c", "Wörter mit Umlauten und ß", "1234567890 !\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~",
        "5", "30", "The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog."
    );

    private FontSuites() {
    }

    static List<Suite> suites() {
        return List.of(new FontSuite());
    }

    // ---- The stand-in device ----

    private static Object standIn(final Class<?> type, final long size) {
        return Proxy.newProxyInstance(FontSuites.class.getClassLoader(), new Class<?>[]{type}, new StandIn(type, size));
    }

    private record StandIn(Class<?> type, long size) implements InvocationHandler {
        @Override
        public Object invoke(final Object proxy, final Method method, final Object[] args) throws Throwable {
            switch (method.getName()) {
                case "toString":
                    return "stand-in " + this.type.getSimpleName();
                case "hashCode":
                    return System.identityHashCode(proxy);
                case "equals":
                    return proxy == args[0];
                default:
                    break;
            }
            if (method.isDefault()) {
                return InvocationHandler.invokeDefault(proxy, method, args);
            }
            Class<?> result = method.getReturnType();
            if (result == void.class) {
                return null;
            } else if (result == boolean.class) {
                return false;
            } else if (result == int.class) {
                return 0;
            } else if (result == long.class) {
                return method.getName().equals("size") ? this.size : 0L;
            } else if (result == float.class) {
                return 0.0F;
            } else if (result == double.class) {
                return 0.0;
            } else if (result == DeviceInfo.class) {
                return RenderSuites.deviceInfo(List.of(true, true, false));
            } else if (result == GpuBufferSlice.MappedView.class) {
                long length = (long)args[1];
                return new GpuBufferSlice.MappedView(
                    new GpuBufferSlice((GpuBuffer)proxy, (long)args[0], length), ByteBuffer.allocateDirect((int)length), () -> {}
                );
            } else if (result == GpuBuffer.class && args != null && args.length == 3 && args[2] instanceof Long size) {
                return standIn(GpuBuffer.class, size);
            } else if (result == GpuBuffer.class && args != null && args.length == 3 && args[2] instanceof ByteBuffer data) {
                return standIn(GpuBuffer.class, data.remaining());
            } else if (result.isInterface()) {
                return standIn(result, 0L);
            } else if (result == String.class) {
                return "stand-in";
            } else if (result == List.class) {
                return List.of();
            }
            return null;
        }
    }

    private static void installDevice() {
        GpuDevice device = (GpuDevice)standIn(GpuDevice.class, 0L);
        try {
            Field field = RenderSystem.class.getDeclaredField("DEVICE");
            field.setAccessible(true);
            field.set(null, device);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
        RenderSystem.getSamplerCache().initialize();
    }

    // ---- Reflection ----

    @SuppressWarnings("unchecked")
    private static <T> T field(final Object owner, final Class<?> type, final String name) {
        try {
            Field field = type.getDeclaredField(name);
            field.setAccessible(true);
            return (T)field.get(owner);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }

    private static JsonPrimitive bits(final float value) {
        return new JsonPrimitive(String.format("%08x", Float.floatToRawIntBits(value)));
    }

    static JsonElement style(final Style style) {
        TextColor color = style.getColor();
        return RenderSuites.array(
            new JsonPrimitive(style.isBold()), new JsonPrimitive(style.isItalic()), new JsonPrimitive(style.isUnderlined()),
            new JsonPrimitive(style.isStrikethrough()), new JsonPrimitive(style.isObfuscated()),
            color == null ? JsonNull.INSTANCE : new JsonPrimitive(color.getValue())
        );
    }

    private static final class FontSuite implements Suite {
        @Override
        public String name() {
            return FontManager.class.getName();
        }

        @SuppressWarnings("unchecked")
        @Override
        public void write(final GoldenWriter writer) throws IOException {
            installDevice();
            ResourceManager resources = TextureSuites.vanillaResources();
            TextureManager textures = new TextureManager(resources);
            FontManager manager = new FontManager(textures, null, null);
            Map<Identifier, List<GlyphProvider.Conditional>> fontSets;
            Map<Identifier, FontSet> loaded = field(manager, FontManager.class, "fontSets");
            try {
                Method prepare = FontManager.class.getDeclaredMethod("prepare", ResourceManager.class, Executor.class);
                prepare.setAccessible(true);
                Object preparation = ((CompletableFuture<?>)prepare.invoke(manager, resources, (Executor)Runnable::run)).join();
                Method fontSetsOf = preparation.getClass().getDeclaredMethod("fontSets");
                fontSetsOf.setAccessible(true);
                fontSets = (Map<Identifier, List<GlyphProvider.Conditional>>)fontSetsOf.invoke(preparation);
                Method createFontSet = FontManager.class.getDeclaredMethod("createFontSet", Identifier.class, List.class, Set.class);
                createFontSet.setAccessible(true);
                // apply: each font set made of its providers in the files' order, with no font options.
                for (Map.Entry<Identifier, List<GlyphProvider.Conditional>> entry : fontSets.entrySet()) {
                    loaded.put(entry.getKey(), (FontSet)createFontSet.invoke(manager, entry.getKey(), Lists.reverse(entry.getValue()), Set.of()));
                }
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }

            // The fonts the vanilla pack defines.
            JsonArray fontIds = new JsonArray();
            fontSets.keySet().stream().map(Identifier::toString).sorted().forEach(fontIds::add);
            writer.field("fonts", fontIds);

            // Every glyph each font's providers define, baked in codepoint order, then a few they don't define.
            for (String fontName : FONTS) {
                Identifier font = Identifier.parse(fontName);
                FontSet fontSet = loaded.get(font);
                TreeSet<Integer> codepoints = new TreeSet<>();
                for (GlyphProvider.Conditional provider : fontSets.get(font)) {
                    codepoints.addAll(provider.provider().getSupportedGlyphs());
                }
                codepoints.addAll(List.of(0, 32, 0x200C, 0x4E2D, 0x1F600, 0x10FFFF));
                for (int codepoint : codepoints) {
                    writer.result("glyph", RenderSuites.array(new JsonPrimitive(fontName), new JsonPrimitive(codepoint), new JsonPrimitive(false)),
                        glyph(fontSet, fontSet.source(false).getGlyph(codepoint)), null);
                }
                for (int codepoint : List.of(32, 65, 0x4E2D)) {
                    writer.result("glyph", RenderSuites.array(new JsonPrimitive(fontName), new JsonPrimitive(codepoint), new JsonPrimitive(true)),
                        glyph(fontSet, fontSet.source(true).getGlyph(codepoint)), null);
                }
            }

            Font fontRenderer = manager.createFont();
            for (String string : STRINGS) {
                writer.result("width", RenderSuites.array(new JsonPrimitive(string)), new JsonPrimitive(fontRenderer.width(string)), null);
                writer.result("widthText", RenderSuites.array(new JsonPrimitive(string)),
                    new JsonPrimitive(fontRenderer.width(FormattedText.of(string, Style.EMPTY.withBold(true)))), null);
            }

            // prepareText at places and in colors the GUI draws text with, shadowed or not, with a background or not.
            int[][] places = {{0, 0}, {10, 20}, {-3, 7}};
            int[] colors = {-1, -16777216, -8323296, 0x80FFFFFF, 0xFFFF5555};
            int index = 0;
            for (String string : STRINGS) {
                int[] place = places[index % places.length];
                int color = colors[index % colors.length];
                boolean shadow = index % 2 == 0;
                int background = index % 3 == 0 ? 0x80000000 : 0;
                index++;
                Font.PreparedText prepared = fontRenderer.prepareText(string, place[0], place[1], color, shadow, background);
                JsonArray renderables = new JsonArray();
                prepared.visit(new Font.GlyphVisitor() {
                    @Override
                    public void acceptRenderable(final TextRenderable renderable) {
                        renderables.add(RenderSuites.array(
                            new JsonPrimitive(renderable instanceof TextRenderable.Styled ? "glyph" : "effect"),
                            bits(renderable.left()), bits(renderable.top()), bits(renderable.right()), bits(renderable.bottom()),
                            new JsonPrimitive(renderable.guiPipeline().getLocation().toString()), vertices(renderable)
                        ));
                    }
                });
                ScreenRectangle bounds = prepared.bounds();
                writer.result("prepareText", RenderSuites.array(
                    new JsonPrimitive(string), new JsonPrimitive(place[0]), new JsonPrimitive(place[1]), new JsonPrimitive(color),
                    new JsonPrimitive(shadow), new JsonPrimitive(background)
                ), RenderSuites.array(renderables, GuiSuites.nullableRectangle(bounds)), null);
            }

            StringSplitter splitter = fontRenderer.getSplitter();
            for (String string : STRINGS) {
                for (int width : new int[]{0, 1, 5, 20, 50, 100, 200}) {
                    JsonArray args = RenderSuites.array(new JsonPrimitive(string), new JsonPrimitive(width));
                    JsonArray lines = new JsonArray();
                    for (FormattedText line : splitter.splitLines(string, width, Style.EMPTY)) {
                        line.visit((style, contents) -> {
                            lines.add(RenderSuites.array(new JsonPrimitive(contents), style(style)));
                            return java.util.Optional.empty();
                        }, Style.EMPTY);
                    }
                    writer.result("splitLines", args, lines, null);
                    writer.result("plainIndexAtWidth", args, new JsonPrimitive(splitter.plainIndexAtWidth(string, width, Style.EMPTY)), null);
                    writer.result("plainTailByWidth", args, new JsonPrimitive(splitter.plainTailByWidth(string, width, Style.EMPTY)), null);
                    writer.result("findLineBreak", args, new JsonPrimitive(splitter.findLineBreak(string, width, Style.EMPTY)), null);
                }
                for (int dir : new int[]{-2, -1, 1, 2}) {
                    for (int from : new int[]{0, string.length() / 2, string.length()}) {
                        for (boolean strip : new boolean[]{false, true}) {
                            writer.result("getWordPosition", RenderSuites.array(
                                new JsonPrimitive(string), new JsonPrimitive(dir), new JsonPrimitive(from), new JsonPrimitive(strip)
                            ), new JsonPrimitive(StringSplitter.getWordPosition(string, dir, from, strip)), null);
                        }
                    }
                }
            }
        }

        /** A baked glyph: its metrics, then a sheet glyph's UVs and place and the font's pages so far. */
        private static JsonElement glyph(final FontSet fontSet, final BakedGlyph glyph) {
            GlyphInfo info = glyph.info();
            JsonArray result = RenderSuites.array(
                bits(info.getAdvance(false)), bits(info.getAdvance(true)), bits(info.getBoldOffset()), bits(info.getShadowOffset())
            );
            if (glyph instanceof BakedSheetGlyph sheet) {
                for (String name : List.of("u0", "u1", "v0", "v1", "left", "right", "up", "down")) {
                    result.add(bits((float)FontSuites.<Float>field(sheet, BakedSheetGlyph.class, name)));
                }
                Object stitcher = field(fontSet, FontSet.class, "stitcher");
                List<?> pages = field(stitcher, stitcher.getClass(), "textures");
                result.add(pages.size());
            }
            return result;
        }

        /** The renderable's vertices, as GlyphRenderState builds them for its pipeline: flat, fully lit, unposed. */
        private static JsonElement vertices(final TextRenderable renderable) {
            RenderPipeline pipeline = renderable.guiPipeline();
            try (ByteBufferBuilder buffer = new ByteBufferBuilder(1024)) {
                BufferBuilder builder = new BufferBuilder(buffer, pipeline.getPrimitiveTopology(), pipeline.getVertexFormatBinding(0));
                renderable.render(new Matrix4f(), builder, 15728880, true);
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
}
