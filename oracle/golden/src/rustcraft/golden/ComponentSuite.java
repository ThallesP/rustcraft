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
import net.minecraft.network.chat.contents.NbtContents;
import net.minecraft.network.chat.contents.ObjectContents;
import net.minecraft.network.chat.contents.data.BlockDataSource;
import net.minecraft.network.chat.contents.data.DataSource;
import net.minecraft.network.chat.contents.data.EntityDataSource;
import net.minecraft.network.chat.contents.data.StorageDataSource;
import net.minecraft.network.chat.contents.ScoreContents;
import net.minecraft.network.chat.contents.objects.AtlasSprite;
import net.minecraft.network.chat.contents.objects.PlayerSprite;
import net.minecraft.world.item.component.ResolvableProfile;
import net.minecraft.world.item.ItemStackTemplate;
import net.minecraft.core.component.DataComponentPatch;
import net.minecraft.network.chat.contents.SelectorContents;
import net.minecraft.network.chat.contents.TranslatableContents;
import net.minecraft.resources.Identifier;
import com.mojang.datafixers.util.Either;
import net.minecraft.commands.arguments.selector.EntitySelector;
import net.minecraft.util.CompilableString;

/**
 * Text components: {@code ComponentSerialization.CODEC} on NBT, both ways, and each component's {@code toString} and
 * {@code getString}. A component is given and recorded as a spec both sides build it from: {@code [contents, style,
 * [sibling, …]]}, its contents {@code ["text", text]}, {@code ["translatable", key, fallback or null, [argument, …]]},
 * {@code ["keybind", name]}, {@code ["score", name, objective]} with the name {@code ["left", selector]} or
 * {@code ["right", name]}, {@code ["selector", selector, separator spec or null]}, {@code ["nbt", path, interpret,
 * plain, separator spec or null, source]} with the source {@code ["entity", selector]}, {@code ["block",
 * coordinates]} or {@code ["storage", id]}, or {@code ["object", info, fallback spec or null]} with the info
 * {@code ["atlas", atlas, sprite]} or {@code ["player", "static" or "dynamic", the profile's tag, hat]}; an argument
 * {@code ["string", s]}, {@code ["number", box, value]}, {@code ["boolean", b]} or {@code ["component", spec]}; and its
 * style null for {@code Style.EMPTY} itself, else {@code [color, shadowColor, bold, italic, underlined, strikethrough,
 * obfuscated, clickEvent, hoverEvent, insertion, font]}, each null when unset: a color {@code ["named", name]} or
 * {@code ["rgb", value]}, a click event {@code [action, field, …]}, a hover event {@code ["show_text", spec]} or
 * {@code ["show_item", id, count, the component patch's tag]}, a font {@code ["resource", id]} or
 * {@code ["atlas_sprite", atlas, sprite]}.
 *
 * <p>Components are random trees up to 4 deep of literals (empty ones, any strings), translations (keys en_us has,
 * with and without its formats' arguments, and unknown keys, with fallbacks now and then, and arguments of every kind
 * a server sends) and key bindings, styled with every field, the click events a server may send and one it may not
 * ({@code open_file}), shown text, and the atlas fonts no codec writes. Then 500 more trees whose contents may be
 * scores (their names selectors that parse and that don't), selectors (with separators), NBT (paths, each data source,
 * the flags) and objects (atlas sprites, players' heads), some showing an item on hover, apart from the first batch,
 * which the packets' generator shares; a tree holding a player's head has no {@code toString} case, since Java prints
 * the profile by identity, nor one showing an item with components, a patch printing its values by their classes, which
 * the port keeps as data. Decoded are their encodings, the encodings with one tag inside replaced or removed, and
 * hand-made tags: plain strings, lists of components, compounds with a {@code type}, an empty {@code extra}, links of
 * other schemes and none, colors out of range, named colors that aren't, entity tooltips, scores, selectors, NBT,
 * objects and items shown on hover, valid and not, and any tags. Dialogs aren't made.
 */
final class ComponentSuite implements Suite {
    private static final int COMPONENTS = 1500;
    private static final int RICH_COMPONENTS = 500;
    private static final List<String> KEYS = List.of(
        "chat.type.text", "chat.type.announcement", "multiplayer.disconnect.kicked", "multiplayer.player.joined", "death.attack.fall",
        "book.pageIndicator", "commands.advancement.criterionNotFound", "gui.done", "key.jump", "translation.test.args",
        "translation.test.complex", "translation.test.escape", "translation.test.invalid", "translation.test.invalid2", "translation.test.world",
        "unknown.key", "%s", "a%", "%1$s %2$s", "%%", "100%s", "%d", "%5$s", "%0$s", "%2147483648$s", "%1$s%"
    );
    private static final List<String> TEXTS = List.of("", "a", "Hello, world!", "§cRed", "multi\nline", "é", "😀", "%s", "\"quoted\"", "x".repeat(40));
    private static final List<String> KEYBINDS = List.of("key.jump", "key.attack", "key.inventory", "unknown");
    private static final List<String> URLS = List.of(
        "https://minecraft.net", "http://example.com/a?b=c#d", "HTTPS://Example.COM:8080/%20x", "https://[::1]/", "ftp://not.allowed/", "mailto:a@b"
    );
    private static final List<String> COLORS = List.of("black", "dark_blue", "gold", "white", "red", "light_purple");
    // Selectors and names a score or selector component holds, which parse; and ones that don't, for hand-made tags.
    private static final List<String> SELECTORS = List.of(
        "@p", "@a", "@s", "@e[type=pig,limit=2]", "@a[scores={kills=1..}]", "Steve", "f81d4fae-7dec-11d0-a765-00a0c91e6bf6", "@n[nbt={a:1b}]",
        "@e[x=1,dx=2,sort=nearest]"
    );
    // NBT paths and block coordinates, which parse, and ones that don't.
    private static final List<String> PATHS = List.of("a", "a.b", "a[0]", "a[]", "a[-1].b", "a[{b:1}]", "{a:1b}", "a{b:\"c\"}", "\"quoted key\".x", "'s'[0]", "Inventory[{Slot:0b}].id");
    private static final List<String> BAD_PATHS = List.of("", "a.", ".a", "a b", "a[x]", "a]{", "a..b", "a{", "a[{]", "x.{a:1}", "a[0", "\"unclosed");
    private static final List<String> COORDINATES = List.of("~ ~ ~", "1 2 3", "^ ^ ^1", "~1 ~-2 ~0.5", "-5 64 7", "^1.5 ^ ^");
    private static final List<String> BAD_COORDINATES = List.of("1 2", "1 ^ 2", "x y z", "1.5 2 3", "~ ~", "^ ~ ^", "", "1 2 3 4");
    // Items a hover event shows, and patches that decode in Java (the port keeps their values as data, and so reads
    // values Java's codecs reject too, which no case makes).
    private static final List<String> ITEMS = List.of("minecraft:stone", "diamond_sword", "minecraft:written_book", "player_head", "minecraft:potion");
    private static final List<String> PATCHES = List.of(
        "{}", "{\"minecraft:damage\":5}", "{\"!minecraft:lore\":{}}", "{\"minecraft:custom_name\":\"Name\"}",
        "{\"minecraft:max_stack_size\":16,\"!minecraft:enchantments\":{}}", "{\"minecraft:enchantment_glint_override\":true}",
        "{\"minecraft:custom_data\":{\"a\":1}}"
    );
    private static final List<String> BAD_SELECTORS = List.of(
        "", "@x", "@e[", "@e[type=nope]", "@s[limit=1]", "seventeen_letters", "@e]", "@p trailing", "\"unclosed", "@e[nbt={]", "*"
    );

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
        // Components holding every kind of contents a server can send, apart so the batch above (which the packets'
        // generator shares) stays as it was.
        for (int i = 0; i < RICH_COMPONENTS; i++) {
            Component component = randomRichComponent(random, 0);
            JsonArray args = new JsonArray();
            args.add(spec(component));
            // A player's profile prints by identity (ResolvableProfile has no toString of its own).
            // A patch prints its values by their classes, which the port keeps as data.
            if (!component.toString().contains("ResolvableProfile$") && !component.toString().matches("(?s).*components=\\{[^}].*")) {
                writer.result("toString", args, new JsonPrimitive(component.toString()), null);
            }
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
        writeJson(writer, random, codec);
        for (Tag input : inputs) {
            JsonArray args = new JsonArray();
            args.add(NbtValues.encodeTag(input));
            // A translation's argument that's a compound holding an end throws: JavaOps can't make a map with a null. So
            // does an NBT path ending in [ (StringIndexOutOfBoundsException, the reader peeking past the end).
            try {
                DataResult<Pair<Component, Tag>> decoded = codec.decode(NbtOps.INSTANCE, input);
                writer.result("decode", args, DfuValues.result(decoded, pair -> DfuValues.pair(pair, ComponentSuite::spec, NbtValues::encodeTag)), null);
            } catch (NullPointerException | StringIndexOutOfBoundsException e) {
                writer.threw("decode", args, e, null);
            }
        }
    }

    // The same on JSON: each component's encoding and the text lenientJson's Gson writes of it, then decodes of those
    // encodings, some with a member replaced, and of hand-made JSON (parsed leniently, as the login's disconnect is).
    private static void writeJson(final GoldenWriter writer, final Random random, final Codec<Component> codec) throws IOException {
        com.google.gson.Gson gson = new com.google.gson.GsonBuilder().disableHtmlEscaping().create();
        List<JsonElement> inputs = new ArrayList<>();
        for (int i = 0; i < 600; i++) {
            Component component = randomComponent(random, 0);
            JsonArray args = new JsonArray();
            args.add(spec(component));
            DataResult<JsonElement> encoded = codec.encodeStart(com.mojang.serialization.JsonOps.INSTANCE, component);
            writer.result("encodeJson", args, DfuValues.result(encoded, GsonSuites::encode), null);
            encoded.result().ifPresent(json -> {
                try {
                    writer.result("toJson", args, new JsonPrimitive(gson.toJson(json)), null);
                } catch (IOException e) {
                    throw new IllegalStateException(e);
                }
                inputs.add(json);
                if (random.nextInt(3) == 0) {
                    inputs.add(mutateJson(json.deepCopy(), random));
                }
            });
        }
        for (String text : List.of(
            "\"plain\"", "\"\"", "[\"a\",{\"text\":\"b\",\"bold\":true}]", "[]", "{\"text\":\"x\",\"color\":null}", "{\"text\":\"x\",\"extra\":null}",
            "{\"translate\":\"%s %s %s\",\"with\":[1,2.5,true]}", "{\"translate\":\"%s\",\"with\":[{\"text\":\"c\"}]}", "{\"translate\":\"%s\",\"with\":[[1]]}",
            "{\"translate\":\"%s\",\"with\":[1e400]}", "{\"translate\":\"%s\",\"with\":[12345678901, 3.0, 300, -7]}",
            "{\"type\":\"text\",\"text\":\"t\"}", "{\"type\":\"keybind\",\"keybind\":\"key.jump\"}", "{\"type\":\"bogus\"}", "{\"text\":1}", "{\"text\":true}",
            "{\"text\":\"a\",\"color\":\"#00FF00\",\"bold\":1,\"italic\":\"true\"}", "{\"text\":\"a\",\"shadow_color\":[1.0,0.5,0.25,1.0]}",
            "{\"text\":\"a\",\"shadow_color\":-16777216}", "{\"text\":\"a\",\"click_event\":{\"action\":\"open_url\",\"url\":\"https://x.y/z\"}}",
            "{\"text\":\"a\",\"hover_event\":{\"action\":\"show_text\",\"value\":\"hi\"}}", "1", "true", "{text:unquoted}", "{\"text\":\"a\",\"extra\":[]}",
            "{\"text\":\"a\",\"insertion\":\"i\",\"font\":\"minecraft:uniform\"}", "{\"keybind\":\"key.inventory\",\"underlined\":false}"
        )) {
            inputs.add(com.google.gson.JsonParser.parseString(text));
        }
        for (JsonElement input : inputs) {
            JsonArray args = new JsonArray();
            args.add(GsonSuites.encode(input));
            try {
                DataResult<Pair<Component, JsonElement>> decoded = codec.decode(com.mojang.serialization.JsonOps.INSTANCE, input);
                writer.result("decodeJson", args, DfuValues.result(decoded, pair -> DfuValues.pair(pair, ComponentSuite::spec, GsonSuites::encode)), null);
            } catch (NullPointerException | NumberFormatException e) {
                // A null in a translation's arguments, and a NaN or infinite one, which BigDecimal can't hold.
                writer.threw("decodeJson", args, e, null);
            }
        }
    }

    private static JsonElement mutateJson(final JsonElement json, final Random random) {
        if (json instanceof com.google.gson.JsonObject object && !object.isEmpty()) {
            List<String> keys = new ArrayList<>(object.keySet());
            String key = keys.get(random.nextInt(keys.size()));
            switch (random.nextInt(3)) {
                case 0 -> object.remove(key);
                case 1 -> object.add(key, mutateJson(object.get(key), random));
                default -> object.add(key, new JsonPrimitive(random.nextInt(5)));
            }
            return object;
        }
        if (json instanceof JsonArray array && !array.isEmpty()) {
            int index = random.nextInt(array.size());
            array.set(index, mutateJson(array.get(index), random));
            return array;
        }
        return random.nextBoolean() ? new JsonPrimitive(TEXTS.get(random.nextInt(TEXTS.size()))) : new JsonPrimitive(random.nextBoolean());
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
        // The types the port has, and one there isn't.
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
        for (String url : List.of("https://ok.example", "not a url", "relative/path", "ftp://x", "https://a b", "")) {
            CompoundTag linked = new CompoundTag();
            linked.putString("text", "link");
            CompoundTag click = new CompoundTag();
            click.putString("action", "open_url");
            click.putString("url", url);
            linked.put("click_event", click);
            tags.add(linked);
        }
        // Entity tooltips, as a server's player names have them: the UUID as four ints or a string, a name or none, and
        // types and UUIDs that don't read.
        for (int i = 0; i < 12; i++) {
            CompoundTag named = new CompoundTag();
            named.putString("text", "player");
            CompoundTag hover = new CompoundTag();
            hover.putString("action", "show_entity");
            hover.putString("id", List.of("minecraft:player", "minecraft:zombie", "pig", "minecraft:nope", "Bad Id").get(i % 5));
            switch (i % 4) {
                case 0 -> hover.putIntArray("uuid", new int[]{1, 2, 3, 4});
                case 1 -> hover.putString("uuid", "2e8fe3e3-15fa-36a7-9329-26262812f095");
                case 2 -> hover.putString("uuid", List.of("not-a-uuid", "1-2-3-4-5", "zz-1-1-1-1", "", "12345678-1234-1234-1234-1234567890123").get(i % 5));
                default -> hover.putIntArray("uuid", new int[]{1, 2, 3});
            }
            if (i % 3 != 0) {
                CompoundTag name = new CompoundTag();
                name.putString("text", "Name" + i);
                hover.put("name", name);
            }
            named.put("hover_event", hover);
            tags.add(named);
        }
        CompoundTag both = new CompoundTag();
        both.putString("text", "t");
        both.putString("translate", "gui.done");
        tags.add(both);
        // Scores and selectors, as a server sends them unresolved: names that are selectors and ones that aren't,
        // selectors that don't parse, separators, missing and mistyped fields, and the type named or not.
        for (int i = 0; i < SELECTORS.size() + BAD_SELECTORS.size(); i++) {
            String selector = i < SELECTORS.size() ? SELECTORS.get(i) : BAD_SELECTORS.get(i - SELECTORS.size());
            CompoundTag score = new CompoundTag();
            CompoundTag inner = new CompoundTag();
            inner.putString("name", selector);
            if (i % 4 != 3) {
                inner.putString("objective", "obj" + i);
            }
            score.put("score", inner);
            if (i % 3 == 0) {
                score.putString("type", "score");
            }
            tags.add(score);
            CompoundTag selectorTag = new CompoundTag();
            selectorTag.putString("selector", selector);
            if (i % 2 == 0) {
                CompoundTag separator = new CompoundTag();
                separator.putString("text", ", ");
                selectorTag.put("separator", separator);
            } else if (i % 5 == 0) {
                selectorTag.putInt("separator", 3);
            }
            if (i % 3 == 1) {
                selectorTag.putString("type", "selector");
            }
            tags.add(selectorTag);
        }
        CompoundTag scoreNotCompound = new CompoundTag();
        scoreNotCompound.putString("score", "@p");
        tags.add(scoreNotCompound);
        CompoundTag selectorNotString = new CompoundTag();
        selectorNotString.putInt("selector", 1);
        tags.add(selectorNotString);
        // Items shown on hover: each item, counts in range and out, patches that decode and that don't (unknown and
        // transient types, a removal that isn't a map, a type both set and removed), air and unknown items.
        List<String> items = new ArrayList<>(ITEMS);
        items.addAll(List.of("minecraft:air", "nope", "Bad Item", ""));
        List<String> patches = new ArrayList<>(PATCHES);
        patches.addAll(List.of(
            "{\"minecraft:nope\":1}", "{\"minecraft:creative_slot_lock\":{}}", "{\"!minecraft:damage\":1}", "{\"Bad Key\":1}",
            "{\"minecraft:damage\":5,\"!minecraft:damage\":{}}", "{\"!minecraft:damage\":{},\"minecraft:damage\":7}", "{\"!\":{}}",
            "{\"custom:thing\":1}"
        ));
        for (int i = 0; i < items.size() * patches.size(); i++) {
            CompoundTag shown = new CompoundTag();
            shown.putString("text", "item");
            CompoundTag hover = new CompoundTag();
            hover.putString("action", "show_item");
            hover.putString("id", items.get(i % items.size()));
            switch (i % 5) {
                case 0 -> hover.putInt("count", 1 + i % 99);
                case 1 -> hover.putInt("count", List.of(0, 100, -1, 99).get(i / 5 % 4));
                case 2 -> hover.putString("count", "x");
                default -> {
                }
            }
            if (i % 3 != 2) {
                try {
                    hover.put("components", net.minecraft.nbt.TagParser.parseCompoundFully(patches.get(i / items.size() % patches.size())));
                } catch (com.mojang.brigadier.exceptions.CommandSyntaxException e) {
                    throw new IllegalStateException(e);
                }
            } else if (i % 7 == 0) {
                hover.putString("components", "{}");
            }
            shown.put("hover_event", hover);
            tags.add(shown);
        }
        // NBT: paths and coordinates that parse and that don't, each source, the flags (both on, mistyped), separators,
        // the source named by type or not, and no source.
        List<String> allPaths = new ArrayList<>(PATHS);
        allPaths.addAll(BAD_PATHS);
        allPaths.add("a[");
        for (int i = 0; i < allPaths.size() * 3; i++) {
            CompoundTag nbt = new CompoundTag();
            nbt.putString("nbt", allPaths.get(i % allPaths.size()));
            switch (i % 4) {
                case 0 -> nbt.putString("entity", i % 8 == 0 ? BAD_SELECTORS.get(i / 8 % BAD_SELECTORS.size()) : SELECTORS.get(i % SELECTORS.size()));
                case 1 -> {
                    List<String> coordinates = i % 3 == 0 ? BAD_COORDINATES : COORDINATES;
                    nbt.putString("block", coordinates.get(i % coordinates.size()));
                }
                case 2 -> nbt.putString("storage", List.of("minecraft:store", "Bad Id", "x").get(i % 3));
                default -> {
                }
            }
            if (i % 5 == 0) {
                nbt.putBoolean("interpret", true);
            }
            if (i % 7 == 0) {
                nbt.putBoolean("plain", true);
            }
            if (i % 11 == 0) {
                nbt.putString("interpret", "yes");
            }
            if (i % 6 == 0) {
                CompoundTag separator = new CompoundTag();
                separator.putString("text", "; ");
                nbt.put("separator", separator);
            } else if (i % 13 == 0) {
                nbt.putInt("separator", 1);
            }
            if (i % 9 == 0) {
                nbt.putString("source", List.of("entity", "block", "storage", "bogus").get(i / 9 % 4));
            }
            if (i % 10 == 0) {
                nbt.putString("type", "nbt");
            }
            tags.add(nbt);
        }
        // Objects: sprites of atlases, players' heads by name, id, properties and skin, and ones that don't read.
        for (int i = 0; i < 40; i++) {
            CompoundTag object = new CompoundTag();
            if (i % 2 == 0) {
                object.putString("sprite", List.of("stone", "item/apple", "Bad Sprite", "custom:thing").get(i / 2 % 4));
                if (i % 3 == 0) {
                    object.putString("atlas", List.of("minecraft:items", "blocks", "not an id").get(i % 9 / 3));
                }
            } else {
                object.put("player", profileTag(new Random(i)));
                if (i % 3 == 0) {
                    object.putBoolean("hat", false);
                } else if (i % 7 == 0) {
                    object.putString("hat", "no");
                }
            }
            if (i % 4 == 1) {
                object.putString("object", List.of("atlas", "player", "bogus").get(i % 3));
            }
            if (i % 5 == 0) {
                object.putString("type", "object");
            }
            if (i % 6 == 0) {
                CompoundTag fallback = new CompoundTag();
                fallback.putString("text", "fb");
                object.put("fallback", fallback);
            }
            tags.add(object);
        }
        CompoundTag scoreTyped = new CompoundTag();
        scoreTyped.putString("type", "score");
        scoreTyped.putString("selector", "@p");
        tags.add(scoreTyped);
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

    /** A random component, as the suite makes them, for the packets that hold one. */
    static Component randomComponent(final Random random) {
        return randomComponent(random, 1);
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

    // A component whose contents may also be a score or a selector, with siblings of any kind.
    private static MutableComponent randomRichComponent(final Random random, final int depth) {
        MutableComponent component = switch (random.nextInt(8)) {
            case 7 -> {
                String path = PATHS.get(random.nextInt(PATHS.size()));
                CompilableString<net.minecraft.commands.arguments.NbtPathArgument.NbtPath> nbtPath =
                    NbtContents.NBT_PATH_CODEC.parse(NbtOps.INSTANCE, StringTag.valueOf(path)).getOrThrow();
                DataSource source = switch (random.nextInt(3)) {
                    case 0 -> new EntityDataSource(EntitySelector.COMPILABLE_CODEC.parse(NbtOps.INSTANCE, StringTag.valueOf(SELECTORS.get(random.nextInt(SELECTORS.size())))).getOrThrow());
                    case 1 -> new BlockDataSource(BlockDataSource.BLOCK_POS_CODEC.parse(NbtOps.INSTANCE, StringTag.valueOf(COORDINATES.get(random.nextInt(COORDINATES.size())))).getOrThrow());
                    default -> new StorageDataSource(Identifier.parse(random.nextBoolean() ? "minecraft:store" : "custom:data/x"));
                };
                boolean interpreting = random.nextInt(3) == 0;
                boolean plain = !interpreting && random.nextInt(3) == 0;
                Optional<Component> separator = depth < 3 && random.nextBoolean() ? Optional.of(randomComponent(random, depth + 1)) : Optional.empty();
                yield MutableComponent.create(new NbtContents(nbtPath, interpreting, plain, separator, source));
            }
            case 5 -> {
                List<String> sprites = List.of("stone", "item/apple", "custom:thing", "block/oak_log");
                Identifier sprite = Identifier.parse(sprites.get(random.nextInt(sprites.size())));
                AtlasSprite info = random.nextBoolean()
                    ? new AtlasSprite(AtlasSprite.DEFAULT_ATLAS, sprite)
                    : new AtlasSprite(Identifier.parse(random.nextBoolean() ? "minecraft:items" : "custom:atlas"), sprite);
                Optional<Component> fallback = depth < 3 && random.nextBoolean() ? Optional.of(randomComponent(random, depth + 1)) : Optional.empty();
                yield MutableComponent.create(new ObjectContents(info, fallback));
            }
            case 6 -> {
                ResolvableProfile profile = switch (random.nextInt(4)) {
                    case 0 -> ResolvableProfile.createUnresolved("Steve");
                    case 1 -> ResolvableProfile.createUnresolved(new java.util.UUID(random.nextLong(), random.nextLong()));
                    case 2 -> ResolvableProfile.createResolved(new com.mojang.authlib.GameProfile(new java.util.UUID(random.nextLong(), random.nextLong()), "Alex"));
                    default -> ResolvableProfile.CODEC.parse(NbtOps.INSTANCE, profileTag(random)).result().orElse(ResolvableProfile.createUnresolved("x"));
                };
                Optional<Component> fallback = random.nextBoolean() ? Optional.of(Component.literal("[head]")) : Optional.empty();
                yield MutableComponent.create(new ObjectContents(new PlayerSprite(profile, random.nextBoolean()), fallback));
            }
            case 0 -> {
                String name = random.nextBoolean() ? SELECTORS.get(random.nextInt(SELECTORS.size())) : BAD_SELECTORS.get(random.nextInt(BAD_SELECTORS.size()));
                Either<CompilableString<EntitySelector>, String> either = EntitySelector.COMPILABLE_CODEC.parse(NbtOps.INSTANCE, StringTag.valueOf(name))
                    .result()
                    .<Either<CompilableString<EntitySelector>, String>>map(Either::left)
                    .orElseGet(() -> Either.right(name));
                yield MutableComponent.create(new ScoreContents(either, TEXTS.get(random.nextInt(TEXTS.size()))));
            }
            case 1 -> {
                String source = SELECTORS.get(random.nextInt(SELECTORS.size()));
                CompilableString<EntitySelector> selector = EntitySelector.COMPILABLE_CODEC.parse(NbtOps.INSTANCE, StringTag.valueOf(source)).getOrThrow();
                Optional<Component> separator = depth < 3 && random.nextBoolean() ? Optional.of(randomComponent(random, depth + 1)) : Optional.empty();
                yield MutableComponent.create(new SelectorContents(selector, separator));
            }
            default -> randomComponent(random, Math.max(depth, 2));
        };
        if (random.nextInt(3) == 0) {
            component.setStyle(randomStyle(random, depth));
        }
        if (random.nextInt(5) == 0) {
            CompoundTag item = new CompoundTag();
            item.putString("id", ITEMS.get(random.nextInt(ITEMS.size())));
            if (random.nextBoolean()) {
                item.putInt("count", 1 + random.nextInt(99));
            }
            if (random.nextBoolean()) {
                try {
                    item.put("components", net.minecraft.nbt.TagParser.parseCompoundFully(PATCHES.get(random.nextInt(PATCHES.size()))));
                } catch (com.mojang.brigadier.exceptions.CommandSyntaxException e) {
                    throw new IllegalStateException(e);
                }
            }
            ItemStackTemplate template = ItemStackTemplate.MAP_CODEC.codec().parse(NbtOps.INSTANCE, item).getOrThrow();
            component.setStyle(component.getStyle().withHoverEvent(new HoverEvent.ShowItem(template)));
        }
        if (depth < 2) {
            for (int i = random.nextInt(3); i > 0; i--) {
                component.append(randomRichComponent(random, depth + 1));
            }
        }
        return component;
    }

    // A profile as a server writes one: a name, an id, properties, a skin patch.
    private static Tag profileTag(final Random random) {
        CompoundTag profile = new CompoundTag();
        if (random.nextBoolean()) {
            profile.putString("name", List.of("Steve", "Alex", "bad name", "", "seventeen_letters", "é").get(random.nextInt(6)));
        }
        if (random.nextBoolean()) {
            profile.putIntArray("id", new int[]{random.nextInt(), random.nextInt(), random.nextInt(), random.nextInt()});
        }
        switch (random.nextInt(4)) {
            case 0 -> {
                ListTag properties = new ListTag();
                CompoundTag property = new CompoundTag();
                property.putString("name", "textures");
                property.putString("value", "e30=");
                if (random.nextBoolean()) {
                    property.putString("signature", "c2ln");
                }
                properties.add(property);
                profile.put("properties", properties);
            }
            case 1 -> {
                CompoundTag properties = new CompoundTag();
                ListTag values = new ListTag();
                values.add(StringTag.valueOf("a"));
                values.add(StringTag.valueOf("b"));
                properties.put("textures", values);
                profile.put("properties", properties);
            }
            default -> {
            }
        }
        if (random.nextInt(3) == 0) {
            profile.putString("texture", "minecraft:entity/player/wide/steve");
        }
        if (random.nextInt(3) == 0) {
            profile.putString("model", List.of("slim", "wide", "fat").get(random.nextInt(3)));
        }
        if (random.nextInt(5) == 0) {
            profile.putString("cape", "custom:cape");
        }
        return random.nextInt(6) == 0 ? StringTag.valueOf("Steve") : profile;
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
            style = style.withClickEvent(switch (random.nextInt(8)) {
                case 7 -> new ClickEvent.OpenUrl(java.net.URI.create(URLS.get(random.nextInt(URLS.size()))));
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
            case ScoreContents score -> {
                spec.add("score");
                JsonArray name = new JsonArray();
                score.name().ifLeft(selector -> {
                    name.add("left");
                    name.add(selector.source());
                }).ifRight(text -> {
                    name.add("right");
                    name.add(text);
                });
                spec.add(name);
                spec.add(score.objective());
            }
            case SelectorContents selector -> {
                spec.add("selector");
                spec.add(selector.selector().source());
                spec.add(selector.separator().map(ComponentSuite::spec).orElse(JsonNull.INSTANCE));
            }
            case NbtContents nbt -> {
                spec.add("nbt");
                spec.add(nbt.nbtPath().source());
                spec.add(nbt.interpreting());
                spec.add(nbt.plain());
                spec.add(nbt.separator().map(ComponentSuite::spec).orElse(JsonNull.INSTANCE));
                JsonArray source = new JsonArray();
                switch (nbt.dataSource()) {
                    case EntityDataSource entity -> {
                        source.add("entity");
                        source.add(entity.selector().source());
                    }
                    case BlockDataSource block -> {
                        source.add("block");
                        source.add(block.coordinates().source());
                    }
                    case StorageDataSource storage -> {
                        source.add("storage");
                        source.add(storage.id().toString());
                    }
                    default -> throw new IllegalArgumentException("source " + nbt.dataSource());
                }
                spec.add(source);
            }
            case ObjectContents object -> {
                spec.add("object");
                JsonArray info = new JsonArray();
                switch (object.contents()) {
                    case AtlasSprite atlas -> {
                        info.add("atlas");
                        info.add(atlas.atlas().toString());
                        info.add(atlas.sprite().toString());
                    }
                    case PlayerSprite player -> {
                        info.add("player");
                        info.add(player.player() instanceof ResolvableProfile.Static ? "static" : "dynamic");
                        info.add(NbtValues.encodeTag(ResolvableProfile.CODEC.encodeStart(NbtOps.INSTANCE, player.player()).getOrThrow()));
                        info.add(player.hat());
                    }
                    default -> throw new IllegalArgumentException("object " + object);
                }
                spec.add(info);
                spec.add(object.fallback().map(ComponentSuite::spec).orElse(JsonNull.INSTANCE));
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
                case ClickEvent.OpenUrl(java.net.URI uri) -> clickSpec.add(uri.toString());
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
        } else if (hover instanceof HoverEvent.ShowItem(ItemStackTemplate item)) {
            JsonArray hoverSpec = new JsonArray();
            hoverSpec.add("show_item");
            hoverSpec.add(item.item().getRegisteredName());
            hoverSpec.add(item.count());
            hoverSpec.add(NbtValues.encodeTag(DataComponentPatch.CODEC.encodeStart(NbtOps.INSTANCE, item.components()).getOrThrow()));
            spec.add(hoverSpec);
        } else if (hover instanceof HoverEvent.ShowEntity(HoverEvent.EntityTooltipInfo entity)) {
            JsonArray hoverSpec = new JsonArray();
            hoverSpec.add("show_entity");
            hoverSpec.add(net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getKey(entity.type).toString());
            hoverSpec.add(Values.encode(entity.uuid, java.util.UUID.class));
            hoverSpec.add(entity.name.map(ComponentSuite::spec).orElse(JsonNull.INSTANCE));
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
