package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.renderpearl.api.device.DeviceInfo;
import com.mojang.renderpearl.api.device.GpuDevice;
import java.io.IOException;
import java.lang.reflect.Field;
import java.lang.reflect.Proxy;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import java.util.Set;
import net.minecraft.client.renderer.Projection;
import org.joml.Matrix4f;

/**
 * Runs random programs against a Projection. A case is a fresh Projection running the ops in {@code args[1]} on a
 * device whose isZZeroToOne is {@code args[0]}; the result has one step per op,
 * {@code [output, zNear, zFar, fov, width, height, invertY, matrixVersion]}: getMatrix's matrix (null for the other
 * ops), then the getters afterwards. RenderSystem's device is a stand-in that only answers getDeviceInfo, which is
 * all getMatrix asks of it.
 */
final class ProjectionSuite implements Suite {
    private static final int PROGRAMS = 256;
    private static final int OPS_PER_PROGRAM = 24;

    @Override
    public String name() {
        return Projection.class.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Random random = new Random(0x960c);
        for (int i = 0; i < PROGRAMS; i++) {
            boolean zZeroToOne = i % 2 == 0;
            // One program in four takes any float, NaN and infinities included.
            List<Op> program = randomProgram(random, i % 4 == 3);
            JsonArray args = new JsonArray();
            args.add(zZeroToOne);
            args.add(Op.encode(program));
            writer.result("Projection", args, run(zZeroToOne, program), null);
        }
    }

    private static JsonArray run(final boolean zZeroToOne, final List<Op> program) {
        installDevice(zZeroToOne);
        Projection projection = new Projection();
        JsonArray steps = new JsonArray();
        for (Op op : program) {
            JsonArray step = new JsonArray();
            step.add(apply(projection, op));
            step.add(Values.floatBits(projection.zNear()));
            step.add(Values.floatBits(projection.zFar()));
            step.add(Values.floatBits(projection.fov()));
            step.add(Values.floatBits(projection.width()));
            step.add(Values.floatBits(projection.height()));
            step.add(projection.invertY());
            step.add(projection.getMatrixVersion());
            steps.add(step);
        }
        return steps;
    }

    private static JsonElement apply(final Projection projection, final Op op) {
        Object[] args = op.args();
        switch (op.name()) {
            case "setupPerspective" -> projection.setupPerspective((Float)args[0], (Float)args[1], (Float)args[2], (Float)args[3], (Float)args[4]);
            case "setupOrtho" -> projection.setupOrtho((Float)args[0], (Float)args[1], (Float)args[2], (Float)args[3], (Boolean)args[4]);
            case "setSize" -> projection.setSize((Float)args[0], (Float)args[1]);
            case "getMatrix" -> {
                return JomlValues.MATRIX4F.encode(projection.getMatrix((Matrix4f)JomlValues.MATRIX4F.instantiate(args[0])));
            }
            default -> throw new IllegalArgumentException(op.name());
        }
        return JsonNull.INSTANCE;
    }

    private static void installDevice(final boolean zZeroToOne) {
        DeviceInfo info = new DeviceInfo("rustcraft golden", "", "", zZeroToOne, "none", 1.0F, null, null, Set.of(), null, null);
        GpuDevice device = (GpuDevice)Proxy.newProxyInstance(
            GpuDevice.class.getClassLoader(),
            new Class<?>[]{GpuDevice.class},
            (proxy, method, methodArgs) -> {
                if (method.getName().equals("getDeviceInfo")) {
                    return info;
                }
                throw new UnsupportedOperationException(method.getName());
            }
        );
        try {
            Field field = RenderSystem.class.getDeclaredField("DEVICE");
            field.setAccessible(true);
            field.set(null, device);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }

    // Setups often repeat the previous one, which leaves the matrix clean, or change one of its arguments, which doesn't.
    private static List<Op> randomProgram(final Random random, final boolean wild) {
        List<Op> program = new ArrayList<>();
        Op setup = null;
        while (program.size() < OPS_PER_PROGRAM) {
            Op op = switch (random.nextInt(10)) {
                case 0, 1 -> perspective(random, wild);
                case 2, 3 -> ortho(random, wild);
                case 4, 5 -> setup == null ? perspective(random, wild) : random.nextBoolean() ? setup : changeOne(random, setup, wild);
                case 6 -> new Op("setSize", size(random, wild), size(random, wild));
                default -> new Op("getMatrix", JomlValues.MATRIX4F.random(random));
            };
            if (!op.name().equals("setSize") && !op.name().equals("getMatrix")) {
                setup = op;
            }
            program.add(op);
        }
        return program;
    }

    private static Op changeOne(final Random random, final Op setup, final boolean wild) {
        Object[] args = setup.args().clone();
        int index = random.nextInt(args.length);
        args[index] = args[index] instanceof Boolean invertY ? !invertY : (Object)pick(random, wild, (Float)args[index] * 2.0F, number(random, wild));
        return new Op(setup.name(), args);
    }

    // The camera's: near 0.05, far from the render and cloud distances, the FOV with its modifiers.
    private static Op perspective(final Random random, final boolean wild) {
        return new Op(
            "setupPerspective",
            pick(random, wild, 0.05F, random.nextFloat()),
            pick(random, wild, (2 + random.nextInt(31)) * 64.0F, random.nextFloat() * 4096.0F),
            pick(random, wild, 70.0F, 30.0F + random.nextFloat() * 80.0F),
            size(random, wild),
            size(random, wild)
        );
    }

    // The GUI's (near 1000, far 11000, inverted Y), the item atlas's and picture-in-picture's (-1000 to 1000), and the post chain's.
    private static Op ortho(final Random random, final boolean wild) {
        return switch (random.nextInt(3)) {
            case 0 -> new Op("setupOrtho", pick(random, wild, 1000.0F, -1000.0F), pick(random, wild, 11000.0F, 1000.0F), size(random, wild), size(random, wild), true);
            case 1 -> new Op("setupOrtho", pick(random, wild, 0.1F, 0.0F), pick(random, wild, 1000.0F, 100.0F), pick(random, wild, 1.0F, 2.0F), pick(random, wild, 1.0F, 2.0F), false);
            default -> new Op("setupOrtho", number(random, wild), number(random, wild), size(random, wild), size(random, wild), random.nextBoolean());
        };
    }

    private static float size(final Random random, final boolean wild) {
        return switch (random.nextInt(3)) {
            case 0 -> pick(random, wild, 854.0F, 480.0F);
            case 1 -> pick(random, wild, 427.0F, 240.0F);
            default -> 1.0F + random.nextInt(4096);
        };
    }

    private static float pick(final Random random, final boolean wild, final float usual, final float other) {
        if (wild && random.nextInt(4) == 0) {
            return (Float)Values.FLOATS.random(random);
        }
        return random.nextBoolean() ? usual : other;
    }

    private static float number(final Random random, final boolean wild) {
        return wild && random.nextInt(4) == 0 ? (Float)Values.FLOATS.random(random) : (random.nextFloat() * 2.0F - 1.0F) * 20000.0F;
    }
}
