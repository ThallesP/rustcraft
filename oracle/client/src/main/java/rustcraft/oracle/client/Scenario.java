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
import java.util.Set;
import org.jspecify.annotations.Nullable;

/** The input both clients replay. The format is documented in TESTING.md. */
public record Scenario(String name, World world, Player player, Options options, List<String> setup, List<Step> steps) {
    public record World(long seed, String preset, long time, String gameMode) {}

    public record Player(double x, double y, double z, float yRot, float xRot) {}

    /** {@code hideGui} is vanilla's F1 ({@code Hud.toggle}): no HUD, hand or crosshair. */
    public record Options(int width, int height, int fov, int guiScale, int renderDistance, boolean hideGui) {}

    /** Either a screenshot, or {@code ticks} ticks with {@code keys} held and an optional fixed {@code look}. */
    public record Step(int ticks, Set<String> keys, float @Nullable [] look, @Nullable String screenshot) {}

    public static final Set<String> KEYS = Set.of("forward", "back", "left", "right", "jump", "sneak", "sprint", "attack", "use");

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

        List<Step> steps = new ArrayList<>();
        for (JsonElement element : root.getAsJsonArray("steps")) {
            steps.add(parseStep(element.getAsJsonObject()));
        }

        return new Scenario(
            name,
            new World(world.get("seed").getAsLong(), world.get("preset").getAsString(), world.get("time").getAsLong(), world.get("game_mode").getAsString()),
            new Player(pos.get(0).getAsDouble(), pos.get(1).getAsDouble(), pos.get(2).getAsDouble(), player.get("y_rot").getAsFloat(), player.get("x_rot").getAsFloat()),
            new Options(
                window.get(0).getAsInt(),
                window.get(1).getAsInt(),
                options.get("fov").getAsInt(),
                options.get("gui_scale").getAsInt(),
                options.get("render_distance").getAsInt(),
                options.has("hide_gui") && options.get("hide_gui").getAsBoolean()
            ),
            List.copyOf(setup),
            List.copyOf(steps)
        );
    }

    private static Step parseStep(JsonObject step) {
        if (step.has("screenshot")) {
            return new Step(0, Set.of(), null, step.get("screenshot").getAsString());
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

        return new Step(step.get("ticks").getAsInt(), Set.copyOf(keys), look, null);
    }
}
