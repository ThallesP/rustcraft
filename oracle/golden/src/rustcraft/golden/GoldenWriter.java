package rustcraft.golden;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.io.BufferedWriter;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

final class GoldenWriter implements AutoCloseable {
    private static final Gson GSON = new GsonBuilder().disableHtmlEscaping().serializeNulls().create();

    private final BufferedWriter out;
    private int lines;

    GoldenWriter(final Path file) throws IOException {
        this.out = Files.newBufferedWriter(file, StandardCharsets.UTF_8);
    }

    void field(final String name, final JsonElement value) throws IOException {
        JsonObject line = new JsonObject();
        line.addProperty("field", name);
        line.add("value", value);
        this.write(line);
    }

    void skipped(final String method, final String reason) throws IOException {
        JsonObject line = new JsonObject();
        line.addProperty("method", method);
        line.addProperty("skipped", reason);
        this.write(line);
    }

    void result(final String method, final JsonElement args, final JsonElement result, final JsonElement after) throws IOException {
        JsonObject line = this.caseLine(method, args, after);
        line.add("result", result);
        this.write(line);
    }

    void threw(final String method, final JsonElement args, final Throwable thrown, final JsonElement after) throws IOException {
        JsonObject line = this.caseLine(method, args, after);
        line.addProperty("throws", thrown.getClass().getName());
        this.write(line);
    }

    int lines() {
        return this.lines;
    }

    @Override
    public void close() throws IOException {
        this.out.close();
    }

    private JsonObject caseLine(final String method, final JsonElement args, final JsonElement after) {
        JsonObject line = new JsonObject();
        line.addProperty("method", method);
        line.add("args", args);
        if (after != null) {
            line.add("after", after);
        }
        return line;
    }

    private void write(final JsonObject line) throws IOException {
        this.out.write(GSON.toJson(line));
        this.out.newLine();
        this.lines++;
    }
}
