package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonPrimitive;
import com.mojang.blaze3d.vertex.PoseStack;
import com.mojang.blaze3d.vertex.VertexConsumer;
import java.io.IOException;
import java.util.List;
import java.util.Random;
import net.minecraft.client.model.geom.ModelPart;
import net.minecraft.client.model.geom.builders.CubeDeformation;
import net.minecraft.client.model.geom.builders.LayerDefinition;
import net.minecraft.client.model.player.PlayerModel;
import org.joml.Quaternionf;

/**
 * Entity models' geometry (TESTING.md, "Entity models"): the player's model, wide and slim, baked from its mesh like
 * {@code LayerDefinitions} bakes {@code ModelLayers.PLAYER} and {@code PLAYER_SLIM}, and a part of it rendered into a
 * recording vertex consumer at a pose: every vertex's position, color, texture coordinates, overlay, light and normal,
 * as {@code ModelPart.Cube.compile} hands them to {@code addVertex}. The poses translate, rotate and scale (unevenly
 * too, which the normals are renormalized for), and the part is turned about Z first, like the first-person arm.
 */
final class EntityModelSuites {
    private static final String[] PARTS = {"root", "head", "body", "right_arm", "left_arm", "right_leg", "left_leg"};

    private EntityModelSuites() {
    }

    static List<Suite> suites() {
        return List.of(new ModelPartSuite());
    }

    private static ModelPart part(final ModelPart root, final String name) {
        return name.equals("root") ? root : root.getChild(name);
    }

    /** Records each vertex the 11-argument {@code addVertex} gets, as float and int bits. */
    private static final class Recorder implements VertexConsumer {
        final JsonArray vertices = new JsonArray();

        @Override
        public void addVertex(
            final float x,
            final float y,
            final float z,
            final int color,
            final float u,
            final float v,
            final int overlayCoords,
            final int lightCoords,
            final float nx,
            final float ny,
            final float nz
        ) {
            JsonArray vertex = new JsonArray();
            for (float value : new float[]{x, y, z}) {
                vertex.add(Values.floatBits(value));
            }
            vertex.add(color);
            vertex.add(Values.floatBits(u));
            vertex.add(Values.floatBits(v));
            vertex.add(overlayCoords);
            vertex.add(lightCoords);
            for (float value : new float[]{nx, ny, nz}) {
                vertex.add(Values.floatBits(value));
            }
            this.vertices.add(vertex);
        }

        @Override
        public VertexConsumer addVertex(final float x, final float y, final float z) {
            throw new UnsupportedOperationException();
        }

        @Override
        public VertexConsumer setColor(final int r, final int g, final int b, final int a) {
            throw new UnsupportedOperationException();
        }

        @Override
        public VertexConsumer setColor(final int color) {
            throw new UnsupportedOperationException();
        }

        @Override
        public VertexConsumer setUv(final float u, final float v) {
            throw new UnsupportedOperationException();
        }

        @Override
        public VertexConsumer setUv1(final int u, final int v) {
            throw new UnsupportedOperationException();
        }

        @Override
        public VertexConsumer setUv2(final int u, final int v) {
            throw new UnsupportedOperationException();
        }

        @Override
        public VertexConsumer setUv3(final float u, final float v) {
            throw new UnsupportedOperationException();
        }

        @Override
        public VertexConsumer setNormal(final float x, final float y, final float z) {
            throw new UnsupportedOperationException();
        }

        @Override
        public VertexConsumer setLineWidth(final float width) {
            throw new UnsupportedOperationException();
        }
    }

    /**
     * ModelPart.render: {@code [slim, part, pose, sleeves, color]} → the vertices, where pose is {@code [x, y, z,
     * xRot, yRot, zRot, xScale, yScale, zScale, partZRot]}: the pose stack translated, rotated by
     * {@code rotationXYZ} and scaled, the part's zRot set, and the arms' sleeves shown or hidden.
     */
    private static final class ModelPartSuite implements Suite {
        @Override
        public String name() {
            return ModelPart.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0x3a7d);
            for (int i = 0; i < 300; i++) {
                boolean slim = i % 2 == 1;
                String partName = PARTS[(i / 2) % PARTS.length];
                boolean sleeves = i % 5 != 0;
                float[] pose = new float[10];
                if (i >= 14) {
                    for (int j = 0; j < 3; j++) {
                        pose[j] = random.nextFloat() * 4.0F - 2.0F;
                        pose[3 + j] = (random.nextFloat() * 2.0F - 1.0F) * (float) Math.PI;
                    }
                    boolean uniform = random.nextBoolean();
                    float scale = 0.25F + random.nextFloat() * 2.0F;
                    for (int j = 0; j < 3; j++) {
                        pose[6 + j] = uniform ? scale : (random.nextBoolean() ? -1.0F : 1.0F) * (0.25F + random.nextFloat() * 2.0F);
                    }
                    pose[9] = random.nextFloat() - 0.5F;
                } else {
                    pose[6] = pose[7] = pose[8] = 1.0F;
                    pose[9] = 0.1F;
                }
                int color = i % 3 == 0 ? -1 : random.nextInt();
                ModelPart root = LayerDefinition.create(PlayerModel.createMesh(CubeDeformation.NONE, slim), 64, 64).bakeRoot();
                root.getChild("right_arm").getChild("right_sleeve").visible = sleeves;
                root.getChild("left_arm").getChild("left_sleeve").visible = sleeves;
                ModelPart part = part(root, partName);
                part.zRot = pose[9];
                PoseStack poseStack = new PoseStack();
                poseStack.translate(pose[0], pose[1], pose[2]);
                poseStack.rotate(new Quaternionf().rotationXYZ(pose[3], pose[4], pose[5]));
                poseStack.scale(pose[6], pose[7], pose[8]);
                Recorder recorder = new Recorder();
                part.render(poseStack, recorder, 0xF000A0, 0xA0003, color);
                JsonArray poseArgs = new JsonArray();
                for (float value : pose) {
                    poseArgs.add(Values.floatBits(value));
                }
                JsonArray args = RenderSuites.array(
                    new JsonPrimitive(slim), new JsonPrimitive(partName), poseArgs, new JsonPrimitive(sleeves), new JsonPrimitive(color)
                );
                writer.result("render", args, recorder.vertices, null);
            }
        }
    }
}
