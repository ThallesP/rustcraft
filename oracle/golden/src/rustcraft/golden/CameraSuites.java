package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import net.minecraft.client.Camera;
import net.minecraft.client.renderer.culling.Frustum;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.phys.AABB;
import org.joml.Matrix4f;
import org.joml.Vector3fc;

/**
 * The camera's view and culling, the pure pieces of a frame (TESTING.md, "The frame"): Camera's rotation and its
 * view rotation matrix, and the culling frustum Camera.update makes from it, tested against the boxes the level
 * renderer culls. A Camera is made with its constructor (Minecraft.getInstance() is null in the generator, which
 * the constructor only keeps), and setRotation, which is protected, is called by reflection.
 */
final class CameraSuites {
    private CameraSuites() {
    }

    static List<Suite> suites() {
        return List.of(new RotationSuite(), new FrustumSuite());
    }

    /** The walk's rotations, the right angles and their neighbours, then random ones. */
    private static List<float[]> rotations() {
        List<float[]> rotations = new ArrayList<>();
        float[] special = {0.0F, -0.0F, 90.0F, -90.0F, 180.0F, -180.0F, 270.0F, 360.0F, 30.0F, 45.0F, 89.9F, -89.9F, 1.0E-5F, 720.5F};
        for (float yRot : special) {
            for (float xRot : new float[]{0.0F, 30.0F, -30.0F, 90.0F, -90.0F, 45.0F, 12.345F}) {
                rotations.add(new float[]{yRot, xRot});
            }
        }
        Random random = new Random(0xca3e);
        for (int i = 0; i < 400; i++) {
            rotations.add(new float[]{random.nextFloat() * 720.0F - 360.0F, random.nextFloat() * 180.0F - 90.0F});
        }
        return rotations;
    }

    private static Camera camera(final float yRot, final float xRot) {
        try {
            Camera camera = new Camera();
            Method setRotation = Camera.class.getDeclaredMethod("setRotation", float.class, float.class);
            setRotation.setAccessible(true);
            setRotation.invoke(camera, yRot, xRot);
            return camera;
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }

    private static JsonArray matrix(final Matrix4f matrix) {
        JsonArray values = new JsonArray();
        float[] elements = new float[16];
        matrix.get(elements);
        for (float element : elements) {
            values.add(Values.floatBits(element));
        }
        values.add(matrix.properties());
        return values;
    }

    private static JsonArray vector(final Vector3fc vector) {
        JsonArray values = new JsonArray();
        values.add(Values.floatBits(vector.x()));
        values.add(Values.floatBits(vector.y()));
        values.add(Values.floatBits(vector.z()));
        return values;
    }

    /**
     * Camera.setRotation: {@code [yRot, xRot]} → {@code [viewRotationMatrix, forwards, up, left]}, the matrix's 16
     * elements in column order and its properties.
     */
    private static final class RotationSuite implements Suite {
        @Override
        public String name() {
            return Camera.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            for (float[] rotation : rotations()) {
                Camera camera = camera(rotation[0], rotation[1]);
                JsonArray args = RenderSuites.array(new JsonPrimitive(Values.floatBits(rotation[0])), new JsonPrimitive(Values.floatBits(rotation[1])));
                writer.result(
                    "setRotation",
                    args,
                    RenderSuites.array(
                        matrix(camera.getViewRotationMatrix(new Matrix4f())), vector(camera.forwardVector()), vector(camera.upVector()), vector(camera.leftVector())
                    ),
                    null
                );
            }
        }
    }

    /**
     * Camera.update's culling frustum, {@code new Frustum(viewRotation, projectionForCulling)} prepared at the
     * camera's position: {@code [x, y, z, yRot, xRot, fov, aspect, depthFar]} → {@code [cubes, boxes, offset]}, the
     * frustum's cubeInFrustum of the 9×5×9 sections around the camera's (as BoundingBoxes), isVisible of the same
     * sections as AABBs shifted by a third of a block, and the camera after offsetToFullyIncludeCameraCube(8).
     */
    private static final class FrustumSuite implements Suite {
        @Override
        public String name() {
            return Frustum.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0xf305);
            List<float[]> rotations = rotations();
            for (int i = 0; i < 160; i++) {
                float[] rotation = rotations.get(i * 3 % rotations.size());
                double x = i == 0 ? 0.5 : (random.nextDouble() - 0.5) * 200.0;
                double y = i == 0 ? -60.0 + 1.62F : random.nextDouble() * 300.0 - 64.0;
                double z = i == 0 ? 0.5 : (random.nextDouble() - 0.5) * 200.0;
                float fov = i % 4 == 0 ? 70.0F : 30.0F + random.nextInt(81);
                float aspect = i % 2 == 0 ? 854.0F / 480.0F : 0.5F + random.nextFloat() * 2.0F;
                float depthFar = i % 3 == 0 ? 256.0F : 64.0F + random.nextInt(32) * 64.0F;
                Camera camera = camera(rotation[0], rotation[1]);
                Matrix4f projection = new Matrix4f().perspective(fov * (float) (Math.PI / 180.0), aspect, 0.05F, depthFar, true);
                Frustum frustum = new Frustum(camera.getViewRotationMatrix(new Matrix4f()), projection);
                frustum.prepare(x, y, z);
                JsonArray cubes = new JsonArray();
                JsonArray boxes = new JsonArray();
                int sectionX = Math.floorDiv((int)Math.floor(x), 16);
                int sectionY = Math.floorDiv((int)Math.floor(y), 16);
                int sectionZ = Math.floorDiv((int)Math.floor(z), 16);
                for (int dx = -4; dx <= 4; dx++) {
                    for (int dy = -2; dy <= 2; dy++) {
                        for (int dz = -4; dz <= 4; dz++) {
                            int minX = (sectionX + dx) * 16;
                            int minY = (sectionY + dy) * 16;
                            int minZ = (sectionZ + dz) * 16;
                            cubes.add(frustum.cubeInFrustum(new BoundingBox(minX, minY, minZ, minX + 15, minY + 15, minZ + 15)));
                            boxes.add(frustum.isVisible(new AABB(minX + 0.33, minY + 0.33, minZ + 0.33, minX + 16.33, minY + 16.33, minZ + 16.33)));
                        }
                    }
                }
                Frustum offset = new Frustum(frustum).offsetToFullyIncludeCameraCube(8);
                JsonArray args = RenderSuites.array(
                    new JsonPrimitive(Values.doubleBits(x)),
                    new JsonPrimitive(Values.doubleBits(y)),
                    new JsonPrimitive(Values.doubleBits(z)),
                    new JsonPrimitive(Values.floatBits(rotation[0])),
                    new JsonPrimitive(Values.floatBits(rotation[1])),
                    new JsonPrimitive(Values.floatBits(fov)),
                    new JsonPrimitive(Values.floatBits(aspect)),
                    new JsonPrimitive(Values.floatBits(depthFar))
                );
                writer.result(
                    "cull",
                    args,
                    RenderSuites.array(
                        cubes,
                        boxes,
                        RenderSuites.array(
                            new JsonPrimitive(Values.doubleBits(offset.getCamX())),
                            new JsonPrimitive(Values.doubleBits(offset.getCamY())),
                            new JsonPrimitive(Values.doubleBits(offset.getCamZ()))
                        )
                    ),
                    null
                );
            }
        }
    }
}
