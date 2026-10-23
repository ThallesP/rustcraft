package rustcraft.oracle.client;

import java.io.UncheckedIOException;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.net.UnknownHostException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.Function;
import net.fabricmc.fabric.api.client.gametest.v1.FabricClientGameTest;
import net.fabricmc.fabric.api.client.gametest.v1.TestInput;
import net.fabricmc.fabric.api.client.gametest.v1.context.ClientGameTestContext;
import net.fabricmc.fabric.api.client.gametest.v1.context.TestServerContext;
import net.fabricmc.fabric.api.client.gametest.v1.context.TestSingleplayerContext;
import net.minecraft.client.CloudStatus;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.client.Options;
import net.minecraft.client.gui.screens.ConnectScreen;
import net.minecraft.client.gui.screens.LevelLoadingScreen;
import net.minecraft.client.gui.screens.TitleScreen;
import net.minecraft.client.gui.screens.worldselection.WorldCreationUiState;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.multiplayer.ServerData;
import net.minecraft.client.multiplayer.ServerStatusPinger;
import net.minecraft.client.multiplayer.resolver.ServerAddress;
import net.minecraft.client.renderer.texture.SpriteContents;
import net.minecraft.client.renderer.texture.TextureAtlas;
import net.minecraft.client.renderer.texture.TextureManager;
import net.minecraft.core.registries.Registries;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.common.ServerboundCustomPayloadPacket;
import net.minecraft.network.protocol.common.custom.DiscardedPayload;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.level.ParticleStatus;
import net.minecraft.server.network.EventLoopGroupHolder;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.minecraft.world.level.gamerules.GameRules;
import net.minecraft.world.level.levelgen.presets.WorldPreset;

/**
 * Replays one scenario (TESTING.md) in the vanilla client and records trace.jsonl and screenshots.
 *
 * <p>Inputs come from system properties set by tools/oracle_client.py: {@code rustcraft.oracle.scenario},
 * {@code rustcraft.oracle.out} and, to join a server instead of a singleplayer world, {@code rustcraft.oracle.server}.
 * With {@code rustcraft.oracle.ping} instead, it pings that server the way the server list does, and replays nothing.
 */
public final class OracleGameTest implements FabricClientGameTest {
    private static final String PLAYER_NAME = "rustcraft";
    private static final int SETTLE_TICKS = 20;
    private static final int LOAD_TIMEOUT_TICKS = 60 * 20;
    private static final int SERVER_TIMEOUT_TICKS = 5 * 60 * 20;
    private static final long WAIT_FOR_TIMEOUT_MILLIS = 30_000;
    // Sent as a custom payload with no data; vanilla servers discard unknown payloads.
    private static final Identifier SCENARIO_START = Identifier.fromNamespaceAndPath("rustcraft", "scenario_start");
    // tools/oracle_client.py starts the server when the client prints this.
    private static final String WAITING_FOR_SERVER = "rustcraft oracle: waiting for the server";

    private static final Map<String, Function<Options, KeyMapping>> KEY_MAPPINGS = Map.of(
        "forward", options -> options.keyUp,
        "back", options -> options.keyDown,
        "left", options -> options.keyLeft,
        "right", options -> options.keyRight,
        "jump", options -> options.keyJump,
        "sneak", options -> options.keyShift,
        "sprint", options -> options.keySprint,
        "attack", options -> options.keyAttack,
        "use", options -> options.keyUse
    );

    @Override
    public void runTest(ClientGameTestContext context) {
        String ping = System.getProperty("rustcraft.oracle.ping", "");
        if (!ping.isEmpty()) {
            ping(context, ping);
            return;
        }

        Scenario scenario = Scenario.load(Path.of(requiredProperty("rustcraft.oracle.scenario")));
        Path outDir = Path.of(requiredProperty("rustcraft.oracle.out")).resolve(scenario.name());
        String server = System.getProperty("rustcraft.oracle.server", "");

        applyOptions(context, scenario.options());
        if (server.isEmpty()) {
            runSingleplayer(context, scenario, outDir);
        } else {
            runMultiplayer(context, scenario, outDir, server);
        }
    }

    private static void runSingleplayer(ClientGameTestContext context, Scenario scenario, Path outDir) {
        try (TestSingleplayerContext singleplayer = context.worldBuilder().adjustSettings(ui -> configureWorld(ui, scenario.world())).create()) {
            TestServerContext server = singleplayer.getServer();
            setupCommands(scenario, PLAYER_NAME).forEach(server::runCommand);
            context.waitTick();
            singleplayer.getConnection().waitForClientboundPackets();
            singleplayer.getConnection().waitForChunksRender();
            ServerTicks ticks = ServerTicks.unpaced(context);
            settle(context, ticks);
            context.runOnClient(OracleGameTest::restartTextureAnimations);
            replay(context, ticks, scenario, outDir);
        }
    }

    /**
     * What the server list does for one server (ServerSelectionList's entry, ticked like JoinMultiplayerScreen ticks its
     * pinger): asks for its status, then pings it, and closes the connection once the pong comes.
     */
    private static void ping(ClientGameTestContext context, String address) {
        waitForServer(context);
        ServerStatusPinger pinger = new ServerStatusPinger();
        ServerData data = new ServerData("Minecraft Server", address, ServerData.Type.OTHER);
        AtomicBoolean ponged = new AtomicBoolean();
        context.runOnClient(client -> {
            try {
                pinger.pingServer(data, () -> {}, () -> ponged.set(true), EventLoopGroupHolder.remote(client.options.useNativeTransport()));
            } catch (UnknownHostException e) {
                throw new UncheckedIOException(e);
            }
        });
        context.waitFor(client -> {
            pinger.tick();
            return ponged.get();
        }, LOAD_TIMEOUT_TICKS);
        // The pong closed the connection; the next tick hands the pinger the disconnection.
        context.waitTicks(2);
        context.runOnClient(client -> pinger.tick());
        System.out.println("rustcraft oracle: pinged " + address + ": " + data.motd.getString() + ", " + data.ping + " ms");
    }

    // tools/oracle_client.py capture starts the server once the client is up and creates this file when it's ready, so the
    // server has ticked as long when the client joins as when the port joins it: the server's timers (the players'
    // latency, every 600 ticks) then fall at the same points of both runs.
    private static void waitForServer(ClientGameTestContext context) {
        String serverReady = System.getProperty("rustcraft.oracle.serverReady", "");
        if (!serverReady.isEmpty()) {
            System.out.println(WAITING_FOR_SERVER);
            context.waitFor(client -> Files.exists(Path.of(serverReady)), SERVER_TIMEOUT_TICKS);
        }
    }

    private static void runMultiplayer(ClientGameTestContext context, Scenario scenario, Path outDir, String address) {
        waitForServer(context);
        context.runOnClient(client -> ConnectScreen.startConnecting(
            client.gui.screen(),
            client,
            ServerAddress.parseString(address),
            new ServerData("rustcraft oracle", address, ServerData.Type.OTHER),
            false,
            null
        ));
        context.waitFor(client -> client.level != null && client.player != null && !(client.gui.screen() instanceof LevelLoadingScreen), LOAD_TIMEOUT_TICKS);
        // Every chunk the client shows is in before the setup changes blocks: the server sends a chunk's light only while
        // the chunk is at the edge of what the client has (ChunkHolder.broadcastChanges), and a chunk still to come would
        // come with the setup's blocks and light (TESTING.md).
        context.waitFor(OracleGameTest::areChunksLoaded, LOAD_TIMEOUT_TICKS);
        ServerTicks ticks = scenario.world().tickRate() > 0.0F
            ? ServerTicks.pace(context, scenario.world().tickRate())
            : ServerTicks.unpaced(context);

        // The world itself (seed, flat preset) comes from the server's properties; commands set the rest. The server sends
        // a command's blocks at the end of the tick it ran in, so a command that places blocks waits for them before the
        // next goes: each command's blocks then come in a tick of their own, grouped the same way every run (TESTING.md).
        for (String command : setupCommands(scenario, "@s")) {
            ticks.awaitSlot();
            int blockUpdates = blockUpdates();
            context.runOnClient(client -> client.player.connection.sendCommand(command));
            if (placesBlocks(command)) {
                ticks.waitFor(client -> blockUpdates() > blockUpdates, LOAD_TIMEOUT_TICKS);
            }
        }
        ticks.waitFor(client -> isAtScenarioPosition(client, scenario.player()), LOAD_TIMEOUT_TICKS);
        ticks.waitFor(OracleGameTest::areChunksRendered, LOAD_TIMEOUT_TICKS);
        settle(context, ticks);
        context.runOnClient(OracleGameTest::restartTextureAnimations);
        // Marks scenario tick 0 in packet captures: the join before it depends on the server's real-time ticking,
        // everything after must match. Sent from the client thread before the first replayed tick, so it goes out first.
        ticks.awaitSlot();
        context.runOnClient(client -> client.getConnection().send(new ServerboundCustomPayloadPacket(new DiscardedPayload(SCENARIO_START))));
        ticks.startScenario();
        replay(context, ticks, scenario, outDir);

        // Paced, the client leaves at the slot after its last tick, once the server has answered that tick.
        ticks.awaitSlot();
        context.runOnClient(client -> {
            client.level.disconnect(Component.literal("Scenario finished"));
            client.disconnectWithSavingScreen();
        });
        context.waitFor(client -> client.level == null);
        context.waitTicks(2);
        context.setScreen(TitleScreen::new);
        ticks.finish();
    }

    /**
     * Puts the world in the scenario's state, the same way in both modes. Singleplayer worlds are created with most of
     * this already, but a dedicated server may have ticked (and spawned slimes) before the client joined. The scenario's
     * own commands build its course before the player is put in place.
     */
    private static List<String> setupCommands(Scenario scenario, String player) {
        List<String> commands = new ArrayList<>(List.of(
            "gamerule advance_time false",
            "gamerule advance_weather false",
            "gamerule random_tick_speed 0",
            "gamerule spawn_mobs false",
            "difficulty normal",
            "time set " + scenario.world().time(),
            "weather clear",
            "gamemode " + scenario.world().gameMode() + " " + player,
            // Below the world they're discarded: no death animation, no drops, and slimes don't split like /kill makes them.
            "tp @e[type=!minecraft:player] 0 -1000 0"
        ));
        commands.addAll(scenario.setup());
        commands.add("tp " + player + " " + position(scenario.player()));
        return commands;
    }

    private static boolean placesBlocks(String command) {
        return command.startsWith("fill ") || command.startsWith("setblock ");
    }

    /** The block changes the server has sent the client so far, one block's or a section's. */
    private static int blockUpdates() {
        return PacketArrivals.count("minecraft:block_update") + PacketArrivals.count("minecraft:section_blocks_update");
    }

    private static void replay(ClientGameTestContext context, ServerTicks ticks, Scenario scenario, Path outDir) {
        TestInput input = context.getInput();
        int tick = 0;
        try (TraceWriter trace = new TraceWriter(outDir.resolve("trace.jsonl"))) {
            for (Scenario.Step step : scenario.steps()) {
                if (step.screenshot() != null) {
                    OracleScreenshot.take(context, outDir.resolve("screenshots").resolve(step.screenshot() + ".png"));
                    continue;
                }

                Map<String, Integer> waitingSince = new HashMap<>();
                for (String packet : step.waitFor()) {
                    waitingSince.put(packet, PacketArrivals.count(packet));
                }

                holdOnly(input, step.keys());
                for (int i = 0; i < step.ticks(); i++) {
                    if (step.look() != null) {
                        input.lookAt(step.look()[0], step.look()[1]);
                    }
                    // The line is written as the tick ends (TickTrace), before the packets that come after it.
                    TickTrace.arm(tick++);
                    ticks.tick();
                    trace.write(TickTrace.take());
                    if (i == step.ticks() - 1 && !waitingSince.isEmpty()) {
                        // The server's answer to this tick: once it has come, the client handles what it has queued,
                        // which it would at its next frame anyway, before the next tick.
                        waitForPackets(waitingSince);
                        context.runOnClient(client -> client.packetProcessor().processQueuedPackets());
                    }
                }
            }
        }
        holdOnly(input, Set.of());
    }

    /**
     * Waits, the client parked between ticks and the network running, until each packet type has been queued for the
     * client's game thread more often than it had been when the step began (PacketArrivals).
     */
    private static void waitForPackets(Map<String, Integer> waitingSince) {
        long deadline = System.currentTimeMillis() + WAIT_FOR_TIMEOUT_MILLIS;
        while (!waitingSince.entrySet().stream().allMatch(entry -> PacketArrivals.count(entry.getKey()) > entry.getValue())) {
            if (System.currentTimeMillis() > deadline) {
                throw new IllegalStateException("timed out waiting for " + waitingSince.keySet());
            }
            try {
                Thread.sleep(5);
            } catch (InterruptedException exception) {
                Thread.currentThread().interrupt();
                throw new IllegalStateException(exception);
            }
        }
    }

    private static void holdOnly(TestInput input, Set<String> keys) {
        KEY_MAPPINGS.forEach((name, mapping) -> {
            if (keys.contains(name)) {
                input.holdKey(mapping);
            } else {
                input.releaseKey(mapping);
            }
        });
    }

    /**
     * Lets the player land and the world finish reacting to the teleport before the first traced tick, then clears
     * the setup's chat feedback and toasts so they don't show up in screenshots.
     */
    private static void settle(ClientGameTestContext context, ServerTicks ticks) {
        ticks.waitTicks(SETTLE_TICKS);
        ticks.waitFor(client -> client.player.onGround(), LOAD_TIMEOUT_TICKS);
        context.runOnClient(client -> {
            client.gui.hud.getChat().clearMessages(false);
            client.gui.toastManager().clear();
        });
    }

    /**
     * Starts every atlas's animations over, each at its first frame and drawn, as the scenario starts. They advance a tick
     * at a time from the client's start (TextureManager.tick), and how many ticks the join took varies from run to run,
     * so a screenshot of an animated texture (water, lava, fire) would otherwise show another frame each run. The port
     * does the same at the same point (TESTING.md).
     */
    private static void restartTextureAnimations(Minecraft client) {
        try {
            Field byPath = TextureManager.class.getDeclaredField("byPath");
            byPath.setAccessible(true);
            Field states = TextureAtlas.class.getDeclaredField("animatedTexturesStates");
            states.setAccessible(true);
            Method upload = TextureAtlas.class.getDeclaredMethod("uploadAnimationFrames");
            upload.setAccessible(true);
            Class<?> stateClass = SpriteContents.AnimationState.class;
            Field frame = stateClass.getDeclaredField("frame");
            Field subFrame = stateClass.getDeclaredField("subFrame");
            Field isDirty = stateClass.getDeclaredField("isDirty");
            for (Field field : List.of(frame, subFrame, isDirty)) {
                field.setAccessible(true);
            }
            for (Object texture : ((Map<?, ?>)byPath.get(client.getTextureManager())).values()) {
                if (texture instanceof TextureAtlas atlas) {
                    for (Object state : (List<?>)states.get(atlas)) {
                        frame.setInt(state, 0);
                        subFrame.setInt(state, 0);
                        isDirty.setBoolean(state, true);
                    }
                    upload.invoke(atlas);
                }
            }
        } catch (ReflectiveOperationException exception) {
            throw new IllegalStateException(exception);
        }
    }

    private static void applyOptions(ClientGameTestContext context, Scenario.Options scenarioOptions) {
        context.getInput().resizeWindow(scenarioOptions.width(), scenarioOptions.height());
        context.runOnClient(client -> {
            Options options = client.options;
            options.fov().set(scenarioOptions.fov());
            options.guiScale().set(scenarioOptions.guiScale());
            options.renderDistance().set(scenarioOptions.renderDistance());
            options.bobView().set(false);
            options.cloudStatus().set(CloudStatus.OFF);
            options.particles().set(ParticleStatus.MINIMAL);
            options.enableVsync().set(false);
            // The vignette eases toward the light level 1% per frame, so it depends on how long loading took.
            options.vignette().set(false);
            // Otherwise joining a server shows the Social Interactions hint toast.
            options.joinedFirstServer = true;
            // F1 (keyToggleGui, Gui.handleKeybinds), for scenarios that compare the level alone.
            if (scenarioOptions.hideGui() != client.gui.hud.isHidden()) {
                client.gui.hud.toggle();
            }
            client.resizeGui();
        });
    }

    private static void configureWorld(WorldCreationUiState ui, Scenario.World world) {
        ui.setSeed(Long.toString(world.seed()));
        ui.setGameMode(switch (world.gameMode()) {
            case "survival" -> WorldCreationUiState.SelectedGameMode.SURVIVAL;
            case "creative" -> WorldCreationUiState.SelectedGameMode.CREATIVE;
            case "spectator" -> WorldCreationUiState.SelectedGameMode.DEBUG;
            default -> throw new IllegalArgumentException("unknown game_mode " + world.gameMode());
        });
        ResourceKey<WorldPreset> preset = ResourceKey.create(Registries.WORLD_PRESET, Identifier.withDefaultNamespace(world.preset()));
        ui.setWorldType(new WorldCreationUiState.WorldTypeEntry(
            ui.getSettings().worldgenLoadContext().lookupOrThrow(Registries.WORLD_PRESET).getOrThrow(preset)
        ));
        // Fabric's consistent settings already stop time, weather and mob spawning.
        ui.getGameRules().set(GameRules.RANDOM_TICK_SPEED, 0, null);
    }

    private static boolean isAtScenarioPosition(Minecraft client, Scenario.Player player) {
        return client.player != null
            && Math.abs(client.player.getX() - player.x()) < 1.0E-6
            && Math.abs(client.player.getZ() - player.z()) < 1.0E-6;
    }

    private static boolean areChunksRendered(Minecraft client) {
        return areChunksLoaded(client) && client.levelRenderer.hasRenderedAllSections();
    }

    /** Every chunk within the render distance around the player is in the level. */
    private static boolean areChunksLoaded(Minecraft client) {
        ClientLevel level = client.level;
        if (level == null || client.player == null) {
            return false;
        }

        ChunkPos center = client.player.chunkPosition();
        int distance = client.options.getEffectiveRenderDistance();
        for (int dz = -distance; dz <= distance; dz++) {
            for (int dx = -distance; dx <= distance; dx++) {
                if (level.getChunk(center.x() + dx, center.z() + dz, ChunkStatus.FULL, false) == null) {
                    return false;
                }
            }
        }
        return true;
    }

    private static String position(Scenario.Player player) {
        // Double.toString keeps the decimal point, so /tp doesn't center integer coordinates on the block.
        return String.format(Locale.ROOT, "%s %s %s %s %s", player.x(), player.y(), player.z(), player.yRot(), player.xRot());
    }

    private static String requiredProperty(String name) {
        String value = System.getProperty(name, "");
        if (value.isEmpty()) {
            throw new IllegalStateException("missing system property " + name + " (run through tools/oracle_client.py)");
        }
        return value;
    }
}
