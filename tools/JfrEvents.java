import java.nio.file.Path;
import java.time.Instant;
import java.util.Map;
import jdk.jfr.consumer.RecordedEvent;
import jdk.jfr.consumer.RecordedMethod;
import jdk.jfr.consumer.RecordingFile;

/**
 * Turns the method traces of vanilla's JFR recording (tools/bench.py) into the events the port's benchmark writes
 * (src/rustcraft/bench.rs): one JSON line per traced call, {"event":…,"start":…,"duration":…} in nanoseconds since the
 * Unix epoch, with the call's thread. A chunk's handler runs twice, first on Netty's thread, which only queues it for
 * the game's (PacketUtils.ensureRunningOnSameThread): only the render thread's call is the chunk's. Run as a source
 * file with the JDK's java:
 *
 *     java tools/JfrEvents.java recording.jfr > events.jsonl
 */
public final class JfrEvents {
    private static final Map<String, String> EVENTS = Map.of(
        "net.minecraft.client.Minecraft::runTick", "frame",
        "com.mojang.renderpearl.backend.opengl.GlCommandEncoder::submit", "submit",
        "net.minecraft.client.Minecraft::tick", "tick",
        "net.minecraft.client.renderer.chunk.SectionCompiler::compile", "compile",
        "net.minecraft.client.multiplayer.ClientLevel::update", "light",
        "net.minecraft.client.multiplayer.ClientPacketListener::handleLevelChunkWithLight", "chunk"
    );
    private static final String RENDER_THREAD = "Render thread";

    public static void main(String[] args) throws Exception {
        StringBuilder out = new StringBuilder();
        try (RecordingFile recording = new RecordingFile(Path.of(args[0]))) {
            while (recording.hasMoreEvents()) {
                RecordedEvent event = recording.readEvent();
                if (!event.getEventType().getName().equals("jdk.MethodTrace")) {
                    continue;
                }
                RecordedMethod method = event.getValue("method");
                String name = EVENTS.get(method.getType().getName() + "::" + method.getName());
                if (name == null) {
                    continue;
                }
                Instant start = event.getStartTime();
                long startNanos = start.getEpochSecond() * 1_000_000_000L + start.getNano();
                String thread = event.getThread() == null ? "" : event.getThread().getJavaName();
                if (name.equals("chunk") && !thread.equals(RENDER_THREAD)) {
                    continue;
                }
                out.append("{\"event\":\"").append(name)
                    .append("\",\"start\":").append(startNanos)
                    .append(",\"duration\":").append(event.getDuration().toNanos())
                    .append(",\"thread\":\"").append(thread.replace("\"", "'")).append("\"}\n");
                if (out.length() > 1 << 20) {
                    System.out.print(out);
                    out.setLength(0);
                }
            }
        }
        System.out.print(out);
    }
}
