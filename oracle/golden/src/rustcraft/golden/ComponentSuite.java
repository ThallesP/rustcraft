package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import com.mojang.datafixers.util.Pair;
import com.mojang.serialization.Codec;
import com.mojang.serialization.DataResult;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.Random;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.IntTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.NbtOps;
import net.minecraft.nbt.StringTag;
import net.minecraft.nbt.Tag;
import net.minecraft.network.chat.ClickEvent;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.ComponentContents;
import net.minecraft.network.chat.ComponentSerialization;
import net.minecraft.network.chat.FontDescription;
import net.minecraft.network.chat.HoverEvent;
import net.minecraft.network.chat.MutableComponent;
import net.minecraft.network.chat.Style;
import net.minecraft.network.chat.TextColor;
import net.minecraft.network.chat.contents.KeybindContents;
import net.minecraft.network.chat.contents.PlainTextContents;
import net.minecraft.network.chat.contents.TranslatableContents;
import net.minecraft.resources.Identifier;

/**
 * Text components: {@code ComponentSerialization.CODEC} on NBT, both ways, and each component's {@code toString} and
 * {@code getString}. A component is given and recorded as a spec both sides build it from: {@code [contents, style,
 * [sibling, …]]}, its contents {@code ["text", text]}, {@code ["translatable", key, fallback or null, [argument, …]]}
 * or {@code ["keybind", name]}, an argument {@code ["string", s]}, {@code ["number", box, value]}, {@code ["boolean",
 * b]} or {@code ["component", spec]}, and its style null for {@code Style.EMPTY} itself, else {@code [color,
 * shadowColor, bold, italic, underlined, strikethrough, obfuscated, clickEvent, hoverEvent, insertion, font]}, each
 * null when unset: a color {@code ["named", name]} or {@code ["rgb", value]}, a click event {@code [action, field,
 * …]}, a hover event {@code ["show_text", spec]}, a font {@code ["resource", id]} or {@code ["atlas_sprite", atlas,
 * sprite]}.
 *
 * <p>Components are random trees up to 4 deep of literals (empty ones, any strings), translations (keys en_us has,
 * with and without its formats' arguments, and unknown keys, with fallbacks now and then, and arguments of every kind
 * a server sends) and key bindings, styled with every field, the click events a server may send and one it may not
 * ({@code open_file}), shown text, and the atlas fonts no codec writes. Decoded are their encodings, the encodings with
 * one tag inside replaced or removed, and hand-made tags: plain strings, lists of components, compounds with a
 * {@code type}, an empty {@code extra}, colors out of range, named colors that aren't, and any tags. Contents, click
 * and hover events and fonts the port doesn't have yet (scores, selectors, NBT, objects, URLs, dialogs, items,
 * entities, player sprites) aren't made.
 */
final class ComponentSuite implements Suite {
    private static final int COMPONENTS = 1500;
    private static final List<String> KEYS = List.of(
        "chat.type.text", "chat.type.announcement", "multiplayer.disconnect.kicked", "multiplayer.player.joined", "death.attack.fall",
        "book.pageIndicator", "commands.advancement.criterionNotFound", "gui.done", "key.jump", "translation.test.args",
        "translation.test.complex", "translation.test.escape", "translation.test.invalid", "translation.test.invalid2", "translation.test.world",
        "unknown.key", "%s", "a%", "%1$s %2$s", "%%", "100%s", "%d", "%5$s", "%0$s", "%2147483648$s", "%1$s%"
    );
    private static final List<String> TEXTS = List.of("", "a", "Hello, world!", "§cRed", "multi\nline", "é", "😀", "%s", "\"quoted\"", "x".repeat(40));
    private static final List<String> KEYBINDS = List.of("key.jump", "key.attack", "key.inventory", "unknown");
    private static final List<String> COLORS = List.of("black", "dark_blue", "gold", "white", "red", "light_purple");

    @Override
    public String name() {
        return "net.minecraft.network.chat.ComponentSerialization";
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Random random = new Random(46);
        Codec<Component> codec = ComponentSerialization.CODEC;
        List<Tag> inputs = new ArrayList<>();
        for (int i = 0; i < COMPONENTS; i++) {
            Component component = randomComponent(random, 0);
            JsonArray args = new JsonArray();
            args.add(spec(component));
            writer.result("toString", args, new JsonPrimitive(component.toString()), null);
            writer.result("getString", args, new JsonPrimitive(component.getString()), null);
            DataResult<Tag> encoded = codec.encodeStart(NbtOps.INSTANCE, component);
            writer.result("encode", args, DfuValues.result(encoded, NbtValues::encodeTag), null);
            encoded.resultOrPartial().ifPresent(tag -> {
                inputs.add(tag);
                if (random.nextInt(3) == 0) {
                    inputs.add(mutate(tag, random));
                }
            });
        }
        inputs.addAll(handMade());
        for (int i = 0; i < 100; i++) {
            inputs.add((Tag)NbtValues.TAGS.random(random));
        }
        for (Tag input : inputs) {
            JsonArray args = new JsonArray();
            args.add(NbtValues.encodeTag(input));
            DataResult<Pair<Component, Tag>> decoded = codec.decode(NbtOps.INSTANCE, input);
            writer.result("decode", args, DfuValues.result(decoded, pair -> DfuValues.pair(pair, ComponentSuite::spec, NbtValues::encodeTag)), null);
        }
    }

    // Tags a server could send that encodings don't make.
    private static List<Tag> handMade() {
        List<Tag> tags = new ArrayList<>();
        tags.add(StringTag.valueOf("plain"));
        tags.add(StringTag.valueOf(""));
        ListTag list = new ListTag();
        list.add(StringTag.valueOf("a"));
        CompoundTag bold = new CompoundTag();
        bold.putString("text", "b");
        bold.putBoolean("bold", true);
        list.add(bold);
        tags.add(list);
        tags.add(new ListTag());
        // The types the port has, and one there isn't; the contents not ported yet (score, selector, nbt, object) aren't
        // made at all.
        for (String type : List.of("text", "translatable", "keybind", "bogus")) {
            CompoundTag typed = new CompoundTag();
            typed.putString("type", type);
            typed.putString("text", "t");
            typed.putString("translate", "gui.done");
            typed.putString("keybind", "key.jump");
            tags.add(typed);
        }
        CompoundTag emptyExtra = new CompoundTag();
        emptyExtra.putString("text", "x");
        emptyExtra.put("extra", new ListTag());
        tags.add(emptyExtra);
        for (String color : List.of("#FFFFFF", "#1000000", "#-1", "#+ff", "#ggg", "notacolor", "RED", "#", "#ＦＦ")) {
            CompoundTag colored = new CompoundTag();
            colored.putString("text", "c");
            colored.putString("color", color);
            tags.add(colored);
        }
        CompoundTag both = new CompoundTag();
        both.putString("text", "t");
        both.putString("translate", "gui.done");
        tags.add(both);
        CompoundTag fallbackNotString = new CompoundTag();
        fallbackNotString.putString("translate", "unknown.key");
        fallbackNotString.put("fallback", IntTag.valueOf(3));
        tags.add(fallbackNotString);
        CompoundTag withPrimitives = new CompoundTag();
        withPrimitives.putString("translate", "%s %s %s %s");
        ListTag with = new ListTag();
        with.add(IntTag.valueOf(1));
        with.add(StringTag.valueOf("s"));
        CompoundTag nested = new CompoundTag();
        nested.putString("text", "n");
        nested.putBoolean("italic", false);
        with.add(nested);
        with.add(new CompoundTag());
        withPrimitives.put("with", with);
        tags.add(withPrimitives);
        return tags;
    }

    private static Tag mutate(final Tag tag, final Random random) {
        Tag copy = tag.copy();
        if (copy instanceof CompoundTag compound && !compound.isEmpty()) {
            List<String> keys = new ArrayList<>(compound.keySet());
            String key = keys.get(random.nextInt(keys.size()));
            switch (random.nextInt(3)) {
                case 0 -> compound.remove(key);
                case 1 -> compound.put(key, mutate(compound.get(key), random));
                default -> compound.put(key, (Tag)NbtValues.TAGS.random(random));
            }
            return compound;
        }
        if (copy instanceof ListTag list && !list.isEmpty()) {
            int index = random.nextInt(list.size());
            list.set(index, mutate(list.get(index), random));
            return list;
        }
        return random.nextBoolean() ? StringTag.valueOf(TEXTS.get(random.nextInt(TEXTS.size()))) : (Tag)NbtValues.TAGS.random(random);
    }

    private static MutableComponent randomComponent(final Random random, final int depth) {
        MutableComponent component = switch (random.nextInt(depth == 0 ? 6 : 8)) {
            case 0, 1, 2 -> Component.literal(TEXTS.get(random.nextInt(TEXTS.size())));
            case 3, 4 -> {
                String key = KEYS.get(random.nextInt(KEYS.size()));
                List<Object> args = new ArrayList<>();
                for (int i = random.nextInt(4); i > 0; i--) {
                    args.add(randomArgument(random, depth));
                }
                String fallback = random.nextInt(4) == 0 ? TEXTS.get(random.nextInt(TEXTS.size())) : null;
                yield Component.translatableWithFallback(key, fallback, args.toArray());
            }
            case 5 -> Component.keybind(KEYBINDS.get(random.nextInt(KEYBINDS.size())));
            default -> Component.literal(TEXTS.get(random.nextInt(TEXTS.size())));
        };
        if (random.nextInt(3) != 0) {
            component.setStyle(randomStyle(random, depth));
        }
        if (depth < 3) {
            for (int i = random.nextInt(depth == 0 ? 4 : 2); i > 0; i--) {
                component.append(randomComponent(random, depth + 1));
            }
        }
        return component;
    }

    private static Object randomArgument(final Random random, final int depth) {
        return switch (random.nextInt(depth < 3 ? 9 : 7)) {
            case 0, 1 -> TEXTS.get(random.nextInt(TEXTS.size()));
            case 2 -> random.nextInt(100);
            case 3 -> (byte)random.nextInt(10);
            case 4 -> random.nextBoolean();
            case 5 -> random.nextInt(4) == 0 ? Float.NaN : random.nextFloat() * 10;
            case 6 -> (long)random.nextInt() * 3;
            default -> randomComponent(random, depth + 1);
        };
    }

    private static Style randomStyle(final Random random, final int depth) {
        Style style = Style.EMPTY;
        if (random.nextBoolean()) {
            style = style.withColor(random.nextBoolean()
                ? TextColor.parseColor(COLORS.get(random.nextInt(COLORS.size()))).getOrThrow()
                : TextColor.fromRgb(random.nextInt(0x1000000)));
        }
        if (random.nextInt(5) == 0) {
            style = style.withShadowColor(random.nextInt());
        }
        if (random.nextInt(3) == 0) {
            style = style.withBold(random.nextBoolean());
        }
        if (random.nextInt(3) == 0) {
            style = style.withItalic(random.nextBoolean());
        }
        if (random.nextInt(4) == 0) {
            style = style.withUnderlined(random.nextBoolean());
        }
        if (random.nextInt(4) == 0) {
            style = style.withStrikethrough(random.nextBoolean());
        }
        if (random.nextInt(4) == 0) {
            style = style.withObfuscated(random.nextBoolean());
        }
        if (random.nextInt(4) == 0) {
            style = style.withClickEvent(switch (random.nextInt(7)) {
                case 0 -> new ClickEvent.RunCommand("/say " + TEXTS.get(random.nextInt(3)));
                case 1 -> new ClickEvent.SuggestCommand("/tp ");
                case 2 -> new ClickEvent.ChangePage(random.nextInt(5) - 1);
                case 3 -> new ClickEvent.CopyToClipboard(TEXTS.get(random.nextInt(TEXTS.size())));
                case 4 -> new ClickEvent.Custom(Identifier.withDefaultNamespace("custom"), random.nextBoolean() ? Optional.empty() : Optional.of((Tag)NbtValues.TAGS.random(random)));
                case 5 -> new ClickEvent.OpenFile("screenshots");
                default -> new ClickEvent.RunCommand("bad§char");
            });
        }
        if (depth < 3 && random.nextInt(5) == 0) {
            style = style.withHoverEvent(new HoverEvent.ShowText(randomComponent(random, depth + 1)));
        }
        if (random.nextInt(5) == 0) {
            style = style.withInsertion(TEXTS.get(random.nextInt(TEXTS.size())));
        }
        if (random.nextInt(6) == 0) {
            style = style.withFont(random.nextInt(4) == 0
                ? new FontDescription.AtlasSprite(Identifier.withDefaultNamespace("blocks"), Identifier.withDefaultNamespace("stone"))
                : new FontDescription.Resource(Identifier.withDefaultNamespace(random.nextBoolean() ? "default" : "uniform")));
        }
        return style;
    }

    static JsonElement spec(final Component component) {
        JsonArray spec = new JsonArray();
        spec.add(contentsSpec(component.getContents()));
        spec.add(styleSpec(component.getStyle()));
        JsonArray siblings = new JsonArray();
        component.getSiblings().forEach(sibling -> siblings.add(spec(sibling)));
        spec.add(siblings);
        return spec;
    }

    private static JsonElement contentsSpec(final ComponentContents contents) {
        JsonArray spec = new JsonArray();
        switch (contents) {
            case PlainTextContents text -> {
                spec.add("text");
                spec.add(text.text());
            }
            case TranslatableContents translatable -> {
                spec.add("translatable");
                spec.add(translatable.getKey());
                spec.add(translatable.getFallback() == null ? JsonNull.INSTANCE : new JsonPrimitive(translatable.getFallback()));
                JsonArray args = new JsonArray();
                for (Object arg : translatable.getArgs()) {
                    JsonArray argSpec = new JsonArray();
                    switch (arg) {
                        case String s -> {
                            argSpec.add("string");
                            argSpec.add(s);
                        }
                        case Number n -> {
                            argSpec.add("number");
                            argSpec.add(n.getClass().getSimpleName());
                            argSpec.add(Values.encode(n, n.getClass()));
                        }
                        case Boolean b -> {
                            argSpec.add("boolean");
                            argSpec.add(b);
                        }
                        case Component c -> {
                            argSpec.add("component");
                            argSpec.add(spec(c));
                        }
                        default -> throw new IllegalArgumentException("argument " + arg);
                    }
                    args.add(argSpec);
                }
                spec.add(args);
            }
            case KeybindContents keybind -> {
                spec.add("keybind");
                spec.add(keybind.getName());
            }
            default -> throw new IllegalArgumentException("contents " + contents);
        }
        return spec;
    }

    private static JsonElement styleSpec(final Style style) {
        if (style.isEmpty()) {
            return JsonNull.INSTANCE;
        }
        JsonArray spec = new JsonArray();
        TextColor color = style.getColor();
        if (color == null) {
            spec.add(JsonNull.INSTANCE);
        } else {
            JsonArray colorSpec = new JsonArray();
            String serialized = color.serialize();
            if (serialized.startsWith("#")) {
                colorSpec.add("rgb");
                colorSpec.add(color.getValue());
            } else {
                colorSpec.add("named");
                colorSpec.add(serialized);
            }
            spec.add(colorSpec);
        }
        spec.add(style.getShadowColor() == null ? JsonNull.INSTANCE : new JsonPrimitive(style.getShadowColor()));
        for (String flag : List.of("bold", "italic", "underlined", "strikethrough", "obfuscated")) {
            Boolean value = flag(style, flag);
            spec.add(value == null ? JsonNull.INSTANCE : new JsonPrimitive(value));
        }
        ClickEvent click = style.getClickEvent();
        if (click == null) {
            spec.add(JsonNull.INSTANCE);
        } else {
            JsonArray clickSpec = new JsonArray();
            clickSpec.add(click.action().getSerializedName());
            switch (click) {
                case ClickEvent.RunCommand(String command) -> clickSpec.add(command);
                case ClickEvent.SuggestCommand(String command) -> clickSpec.add(command);
                case ClickEvent.ChangePage(int page) -> clickSpec.add(page);
                case ClickEvent.CopyToClipboard(String value) -> clickSpec.add(value);
                case ClickEvent.OpenFile(String path) -> clickSpec.add(path);
                case ClickEvent.Custom(Identifier id, Optional<Tag> payload) -> {
                    clickSpec.add(id.toString());
                    clickSpec.add(payload.<JsonElement>map(NbtValues::encodeTag).orElse(JsonNull.INSTANCE));
                }
                default -> throw new IllegalArgumentException("click " + click);
            }
            spec.add(clickSpec);
        }
        HoverEvent hover = style.getHoverEvent();
        if (hover instanceof HoverEvent.ShowText(Component value)) {
            JsonArray hoverSpec = new JsonArray();
            hoverSpec.add("show_text");
            hoverSpec.add(spec(value));
            spec.add(hoverSpec);
        } else if (hover == null) {
            spec.add(JsonNull.INSTANCE);
        } else {
            throw new IllegalArgumentException("hover " + hover);
        }
        spec.add(style.getInsertion() == null ? JsonNull.INSTANCE : new JsonPrimitive(style.getInsertion()));
        FontDescription font = fontField(style);
        if (font == null) {
            spec.add(JsonNull.INSTANCE);
        } else {
            JsonArray fontSpec = new JsonArray();
            switch (font) {
                case FontDescription.Resource(Identifier id) -> {
                    fontSpec.add("resource");
                    fontSpec.add(id.toString());
                }
                case FontDescription.AtlasSprite(Identifier atlas, Identifier sprite) -> {
                    fontSpec.add("atlas_sprite");
                    fontSpec.add(atlas.toString());
                    fontSpec.add(sprite.toString());
                }
                default -> throw new IllegalArgumentException("font " + font);
            }
            spec.add(fontSpec);
        }
        return spec;
    }

    // A flag as it's set, read by reflection: the getters say only whether it's true.
    private static Boolean flag(final Style style, final String name) {
        try {
            java.lang.reflect.Field field = Style.class.getDeclaredField(name);
            field.setAccessible(true);
            return (Boolean)field.get(style);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }

    private static FontDescription fontField(final Style style) {
        try {
            java.lang.reflect.Field field = Style.class.getDeclaredField("font");
            field.setAccessible(true);
            return (FontDescription)field.get(style);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }
}
