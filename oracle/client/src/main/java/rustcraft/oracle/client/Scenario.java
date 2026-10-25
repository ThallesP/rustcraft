package rustcraft.oracle.client;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import org.jspecify.annotations.Nullable;

/** The input both clients replay. The format is documented in TESTING.md. */
public record Scenario(String name, World world, Player player, Options options, List<String> setup, List<Step> title, List<Step> steps) {
    /** {@code tickRate} is the server's ticks a second, 0 for its own 20 and an unpaced client (ServerTicks). */
    public record World(long seed, String preset, long time, String gameMode, float tickRate) {}

    public record Player(double x, double y, double z, float yRot, float xRot) {}

    /**
     * {@code hideGui} is vanilla's F1 ({@code Hud.toggle}): no HUD, hand or crosshair. {@code hideSplash} is the
     * accessibility option {@code hideSplashTexts}: the title screen's splash is random.
     */
    public record Options(int width, int height, int fov, int guiScale, int renderDistance, boolean hideGui, boolean hideSplash) {}

    /**
     * Either a screenshot ({@code blinkOn}: taken just after the focused edit box's cursor came on), a key pressed and
     * released ({@code key}), characters typed ({@code type}), the cursor moved to a place in the window ({@code
     * cursor}), a mouse button pressed and released where the cursor is ({@code click}), the wheel scrolled ({@code
     * scroll}) or a command sent ({@code command}, without its slash) between two ticks, or {@code ticks} ticks with
     * {@code keys} held and an optional fixed {@code look}, and the clientbound packet types to wait for after its last
     * tick, or after the input ({@code waitFor}, TESTING.md), and a screenshot after every {@code screenshotsEvery}th of
     * its ticks (0 for none), named by the tick ({@link #tickScreenshot}).
     */
    public record Step(
        int ticks,
        Set<String> keys,
        float @Nullable [] look,
        @Nullable String screenshot,
        List<String> waitFor,
        @Nullable String key,
        @Nullable String type,
        double @Nullable [] cursor,
        @Nullable String command,
        @Nullable String click,
        @Nullable Double scroll,
        boolean blinkOn,
        int screenshotsEvery
    ) {}

    /** The name of the screenshot taken after the scenario's tick (its trace line's {@code tick}). */
    public static String tickScreenshot(int tick) {
        return String.format(Locale.ROOT, "tick-%04d", tick);
    }

    public static final Set<String> KEYS = Set.of("forward", "back", "left", "right", "jump", "sneak", "sprint", "attack", "use");

    /** The keys a {@code key} step presses, by their SDL scancodes (InputConstants' KEY_ constants). */
    public static final Map<String, Integer> PRESSABLE_KEYS = Map.of(
        "escape", 41,
        "enter", 40,
        "t", 23,
        "slash", 56,
        "tab", 43,
        "backspace", 42
    );

    /** The buttons a {@code click} step presses, by their SDL numbers (InputConstants' MOUSE_BUTTON_ constants). */
    public static final Map<String, Integer> MOUSE_BUTTONS = Map.of("left", 1);

    public static Scenario load(Path path) {
        JsonObject root;
        try {
            root = JsonParser.parseString(Files.readString(path)).getAsJsonObject();
        } catch (IOException exception) {
            throw new UncheckedIOException(exception);
        }

        String fileName = path.getFileName().toString();
        String name = fileName.endsWith(".json") ? fileName.substring(0, fileName.length() - ".json".length()) : fileName;

        JsonObject world = root.getAsJsonObject("world");
        JsonObject player = root.getAsJsonObject("player");
        JsonArray pos = player.getAsJsonArray("pos");
        JsonObject options = root.getAsJsonObject("options");
        JsonArray window = options.getAsJsonArray("window");

        // Commands that build the scenario's course, without their slash, sent before the player is put in place.
        List<String> setup = new ArrayList<>();
        if (root.has("setup")) {
            for (JsonElement command : root.getAsJsonArray("setup")) {
                setup.add(command.getAsString());
            }
        }

        // Steps on the title screen before the client joins: ticks, screenshots and input (TESTING.md).
        List<Step> title = new ArrayList<>();
        if (root.has("title")) {
            for (JsonElement element : root.getAsJsonArray("title")) {
                title.add(parseStep(element.getAsJsonObject()));
            }
        }

        List<Step> steps = new ArrayList<>();
        for (JsonElement element : root.getAsJsonArray("steps")) {
            steps.add(parseStep(element.getAsJsonObject()));
        }

        return new Scenario(
            name,
            new World(
                world.get("seed").getAsLong(),
                world.get("preset").getAsString(),
                world.get("time").getAsLong(),
                world.get("game_mode").getAsString(),
                world.has("tick_rate") ? world.get("tick_rate").getAsFloat() : 0.0F
            ),
            new Player(pos.get(0).getAsDouble(), pos.get(1).getAsDouble(), pos.get(2).getAsDouble(), player.get("y_rot").getAsFloat(), player.get("x_rot").getAsFloat()),
            new Options(
                window.get(0).getAsInt(),
                window.get(1).getAsInt(),
                options.get("fov").getAsInt(),
                options.get("gui_scale").getAsInt(),
                options.get("render_distance").getAsInt(),
                options.has("hide_gui") && options.get("hide_gui").getAsBoolean(),
                options.has("hide_splash") && options.get("hide_splash").getAsBoolean()
            ),
            List.copyOf(setup),
            List.copyOf(title),
            List.copyOf(steps)
        );
    }

    private static Step parseStep(JsonObject step) {
        if (step.has("screenshot")) {
            boolean blinkOn = step.has("blink_on") && step.get("blink_on").getAsBoolean();
            return new Step(0, Set.of(), null, step.get("screenshot").getAsString(), List.of(), null, null, null, null, null, null, blinkOn, 0);
        }
        // Packets that must each have come since the step began: after its last tick, or after its input.
        List<String> waitFor = new ArrayList<>();
        if (step.has("wait_for")) {
            for (JsonElement packet : step.getAsJsonArray("wait_for")) {
                waitFor.add(packet.getAsString());
            }
        }

        if (step.has("key")) {
            String key = step.get("key").getAsString();
            if (!PRESSABLE_KEYS.containsKey(key)) {
                throw new IllegalArgumentException("unknown key '" + key + "', expected one of " + PRESSABLE_KEYS.keySet());
            }
            return new Step(0, Set.of(), null, null, List.copyOf(waitFor), key, null, null, null, null, null, false, 0);
        }
        if (step.has("type")) {
            return new Step(0, Set.of(), null, null, List.copyOf(waitFor), null, step.get("type").getAsString(), null, null, null, null, false, 0);
        }
        if (step.has("cursor")) {
            JsonArray cursor = step.getAsJsonArray("cursor");
            return new Step(0, Set.of(), null, null, List.copyOf(waitFor), null, null, new double[]{cursor.get(0).getAsDouble(), cursor.get(1).getAsDouble()}, null, null, null, false, 0);
        }
        if (step.has("scroll")) {
            return new Step(0, Set.of(), null, null, List.copyOf(waitFor), null, null, null, null, null, step.get("scroll").getAsDouble(), false, 0);
        }
        if (step.has("click")) {
            String button = step.get("click").getAsString();
            if (!MOUSE_BUTTONS.containsKey(button)) {
                throw new IllegalArgumentException("unknown mouse button '" + button + "', expected one of " + MOUSE_BUTTONS.keySet());
            }
            return new Step(0, Set.of(), null, null, List.copyOf(waitFor), null, null, null, null, button, null, false, 0);
        }
        if (step.has("command")) {
            return new Step(0, Set.of(), null, null, List.copyOf(waitFor), null, null, null, step.get("command").getAsString(), null, null, false, 0);
        }

        Set<String> keys = new LinkedHashSet<>();
        if (step.has("keys")) {
            for (JsonElement key : step.getAsJsonArray("keys")) {
                String name = key.getAsString();
                if (!KEYS.contains(name)) {
                    throw new IllegalArgumentException("unknown key '" + name + "', expected one of " + KEYS);
                }
                keys.add(name);
            }
        }

        float[] look = null;
        if (step.has("look")) {
            JsonArray rotation = step.getAsJsonArray("look");
            look = new float[]{rotation.get(0).getAsFloat(), rotation.get(1).getAsFloat()};
        }

        int ticks = step.get("ticks").getAsInt();
        if (!waitFor.isEmpty() && ticks < 1) {
            throw new IllegalArgumentException("a step that waits for packets needs a tick");
        }
        // A screenshot after every so many of the step's ticks, a video of it (TESTING.md).
        int screenshotsEvery = step.has("screenshots_every") ? step.get("screenshots_every").getAsInt() : 0;
        if (screenshotsEvery < 0) {
            throw new IllegalArgumentException("screenshots_every can't be negative");
        }
        return new Step(ticks, Set.copyOf(keys), look, null, List.copyOf(waitFor), null, null, null, null, null, null, false, screenshotsEvery);
    }
}
