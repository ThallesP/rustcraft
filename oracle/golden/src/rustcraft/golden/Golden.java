package rustcraft.golden;

import com.mojang.math.Axis;
import com.mojang.math.GivensParameters;
import com.mojang.math.MatrixUtil;
import com.mojang.math.OctahedralGroup;
import com.mojang.math.Transformation;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Locale;
import java.util.Random;
import net.minecraft.core.AxisCycle;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.Vec3i;
import net.minecraft.util.ARGB;
import net.minecraft.util.Mth;
import net.minecraft.util.StringRepresentable;
import net.minecraft.world.level.block.Rotation;
import net.minecraft.world.level.levelgen.RandomSupport;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.BlockHitResult;
import net.minecraft.world.phys.HitResult;
import net.minecraft.world.phys.Vec2;
import net.minecraft.world.phys.Vec3;
import org.joml.FrustumIntersection;
import org.joml.Matrix3f;
import org.joml.Matrix3x2f;
import org.joml.Matrix4f;
import org.joml.Matrix4fc;
import org.joml.Quaternionf;
import org.joml.Vector2f;
import org.joml.Vector3f;
import org.joml.Vector3i;
import org.joml.Vector4f;

/** Writes vanilla's answers for the Rust golden tests, one JSON Lines file per class. See TESTING.md. */
public final class Golden {
    private static final double SIN_SCALE = 10430.378350470453;
    // Last-bit disagreements between libms are rare, so the methods HotSpot computes with its own
    // assembly get this many extra inputs each, spread over every path through the assembly.
    private static final int INTRINSIC_INPUTS = 20000;

    private static final List<Suite> SUITES = List.of(
        new StaticSuite(Mth.class)
            .extraInputs("sin(D)F", sinTableInputs(0))
            .extraInputs("cos(D)F", sinTableInputs(-16384))
            .extraInputs("atan2(DD)D", atan2Inputs())
            .extraInputs("rayIntersectsAABB(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/AABB;)Z", clipRays(3))
            // Steps by 2π until the difference is in range, which never ends once 2π is below the float's precision.
            .where("rotLerpRad(FFF)F", args -> !(Math.abs((Float)args[2] - (Float)args[1]) >= 0x1p26F)),
        new StaticSuite(ARGB.class),
        new StaticSuite(RandomSupport.class).exclude("generateUniqueSeed()J", "seeded from System.nanoTime"),
        new RandomSourceSuite(),
        new StaticSuite(Math.class)
            .only(
                "abs(I)I", "abs(J)J", "abs(F)F", "abs(D)D",
                "min(FF)F", "max(FF)F", "min(DD)D", "max(DD)D",
                "floorDiv(II)I", "floorDiv(JJ)J", "floorMod(II)I", "floorMod(JJ)J",
                "sin(D)D", "cos(D)D", "tan(D)D", "asin(D)D", "log(D)D", "pow(DD)D", "sqrt(D)D",
                "acos(D)D", "atan2(DD)D", "rint(D)D", "round(F)I", "round(D)J", "signum(F)F", "signum(D)D",
                "toRadians(D)D", "toDegrees(D)D", "hypot(DD)D", "nextUp(D)D", "nextDown(D)D"
            )
            .extraInputs("sin(D)D", trigInputs(1, -253, 16))
            .extraInputs("cos(D)D", trigInputs(2, -253, 16))
            .extraInputs("tan(D)D", trigInputs(5, -40, 13))
            .extraInputs("log(D)D", logInputs())
            .extraInputs("pow(DD)D", powInputs())
            .extraInputs("hypot(DD)D", hypotInputs()),
        new StringSuite(),
        // JOML's own fast paths are off (no -Djoml.fastmath), so its trig goes through java.lang.Math.
        new StaticSuite(org.joml.Math.class).exclude("random()D", "seeded from the clock"),
        new StaticSuite(Matrix4fc.class),
        new InstanceSuite(
            Vector3f.class,
            "<init>()V", "<init>(FFF)V", "<init>(Lorg/joml/Vector3fc;)V",
            "set(FFF)Lorg/joml/Vector3f;", "set(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;", "zero()Lorg/joml/Vector3f;",
            "add(FFF)Lorg/joml/Vector3f;", "add(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;",
            "sub(FFF)Lorg/joml/Vector3f;", "sub(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;",
            "mul(F)Lorg/joml/Vector3f;", "mul(FFF)Lorg/joml/Vector3f;", "mul(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;",
            "mul(Lorg/joml/Matrix3fc;)Lorg/joml/Vector3f;", "mul(Lorg/joml/Matrix3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "negate()Lorg/joml/Vector3f;", "cross(FFF)Lorg/joml/Vector3f;", "cross(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;",
            "lerp(Lorg/joml/Vector3fc;F)Lorg/joml/Vector3f;", "lerp(Lorg/joml/Vector3fc;FLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "dot(FFF)F", "dot(Lorg/joml/Vector3fc;)F", "length()F", "lengthSquared()F",
            "distance(FFF)F", "distance(Lorg/joml/Vector3fc;)F", "distanceSquared(FFF)F", "distanceSquared(Lorg/joml/Vector3fc;)F",
            "normalize()Lorg/joml/Vector3f;", "normalize(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "rotate(Lorg/joml/Quaternionfc;)Lorg/joml/Vector3f;", "rotate(Lorg/joml/Quaternionfc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "mulProject(Lorg/joml/Matrix4fc;)Lorg/joml/Vector3f;", "mulProject(Lorg/joml/Matrix4fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "mulPosition(Lorg/joml/Matrix4fc;)Lorg/joml/Vector3f;", "mulPosition(Lorg/joml/Matrix4fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "mulDirection(Lorg/joml/Matrix4fc;)Lorg/joml/Vector3f;", "mulDirection(Lorg/joml/Matrix4fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;"
        ),
        new InstanceSuite(
            Quaternionf.class,
            "<init>()V", "<init>(FFFF)V", "<init>(Lorg/joml/Quaternionfc;)V",
            "set(FFFF)Lorg/joml/Quaternionf;", "set(Lorg/joml/Quaternionfc;)Lorg/joml/Quaternionf;", "identity()Lorg/joml/Quaternionf;",
            "rotationX(F)Lorg/joml/Quaternionf;", "rotationY(F)Lorg/joml/Quaternionf;", "rotationZ(F)Lorg/joml/Quaternionf;",
            "rotationXYZ(FFF)Lorg/joml/Quaternionf;", "rotationYXZ(FFF)Lorg/joml/Quaternionf;", "rotationZYX(FFF)Lorg/joml/Quaternionf;",
            "rotationAxis(FFFF)Lorg/joml/Quaternionf;", "rotationAxis(FLorg/joml/Vector3fc;)Lorg/joml/Quaternionf;",
            "conjugate()Lorg/joml/Quaternionf;", "conjugate(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            "normalize()Lorg/joml/Quaternionf;", "normalize(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            "mul(Lorg/joml/Quaternionfc;)Lorg/joml/Quaternionf;", "mul(Lorg/joml/Quaternionfc;Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            "mul(FFFF)Lorg/joml/Quaternionf;", "mul(FFFFLorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            "transform(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;", "transform(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transform(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "slerp(Lorg/joml/Quaternionfc;FLorg/joml/Quaternionf;)Lorg/joml/Quaternionf;"
        )
            .extraInputs("slerp(Lorg/joml/Quaternionfc;FLorg/joml/Quaternionf;)Lorg/joml/Quaternionf;", slerpInputs()),
        new InstanceSuite(
            Matrix3f.class,
            "<init>()V", "<init>(FFFFFFFFF)V", "<init>(Lorg/joml/Matrix3fc;)V", "<init>(Lorg/joml/Matrix4fc;)V",
            "set(Lorg/joml/Matrix3fc;)Lorg/joml/Matrix3f;", "set(Lorg/joml/Matrix4fc;)Lorg/joml/Matrix3f;",
            "set(FFFFFFFFF)Lorg/joml/Matrix3f;", "identity()Lorg/joml/Matrix3f;",
            "mul(Lorg/joml/Matrix3fc;)Lorg/joml/Matrix3f;", "mul(Lorg/joml/Matrix3fc;Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "invert()Lorg/joml/Matrix3f;", "invert(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "transpose()Lorg/joml/Matrix3f;", "transpose(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "scale(FFF)Lorg/joml/Matrix3f;", "scale(FFFLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;", "scale(F)Lorg/joml/Matrix3f;",
            "rotateX(F)Lorg/joml/Matrix3f;", "rotateX(FLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "rotateY(F)Lorg/joml/Matrix3f;", "rotateY(FLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "rotateZ(F)Lorg/joml/Matrix3f;", "rotateZ(FLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "rotate(FFFF)Lorg/joml/Matrix3f;", "rotate(FFFFLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "rotate(FLorg/joml/Vector3fc;)Lorg/joml/Matrix3f;", "rotate(FLorg/joml/Vector3fc;Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "rotate(Lorg/joml/Quaternionfc;)Lorg/joml/Matrix3f;", "rotate(Lorg/joml/Quaternionfc;Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "transform(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;", "transform(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transform(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;"
        ),
        new InstanceSuite(
            Matrix4f.class,
            "<init>()V", "<init>(FFFFFFFFFFFFFFFF)V", "<init>(Lorg/joml/Matrix4fc;)V",
            "properties()I", "determineProperties()Lorg/joml/Matrix4f;", "identity()Lorg/joml/Matrix4f;",
            "set(Lorg/joml/Matrix4fc;)Lorg/joml/Matrix4f;", "set(FFFFFFFFFFFFFFFF)Lorg/joml/Matrix4f;",
            "mul(Lorg/joml/Matrix4fc;)Lorg/joml/Matrix4f;", "mul(Lorg/joml/Matrix4fc;Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "translation(FFF)Lorg/joml/Matrix4f;", "setTranslation(FFF)Lorg/joml/Matrix4f;",
            "translate(FFF)Lorg/joml/Matrix4f;", "translate(FFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "scaling(FFF)Lorg/joml/Matrix4f;", "scale(FFF)Lorg/joml/Matrix4f;", "scale(FFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotationX(F)Lorg/joml/Matrix4f;", "rotationY(F)Lorg/joml/Matrix4f;", "rotationZ(F)Lorg/joml/Matrix4f;",
            "rotateX(F)Lorg/joml/Matrix4f;", "rotateX(FLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotateY(F)Lorg/joml/Matrix4f;", "rotateY(FLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotateZ(F)Lorg/joml/Matrix4f;", "rotateZ(FLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotationTowardsXY(FF)Lorg/joml/Matrix4f;", "rotateTowardsXY(FFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotation(FFFF)Lorg/joml/Matrix4f;", "rotation(FLorg/joml/Vector3fc;)Lorg/joml/Matrix4f;",
            "rotate(FFFF)Lorg/joml/Matrix4f;", "rotate(FFFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotate(FLorg/joml/Vector3fc;)Lorg/joml/Matrix4f;", "rotate(FLorg/joml/Vector3fc;Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotation(Lorg/joml/Quaternionfc;)Lorg/joml/Matrix4f;",
            "rotate(Lorg/joml/Quaternionfc;)Lorg/joml/Matrix4f;", "rotate(Lorg/joml/Quaternionfc;Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotationAround(Lorg/joml/Quaternionfc;FFF)Lorg/joml/Matrix4f;",
            "rotateAround(Lorg/joml/Quaternionfc;FFF)Lorg/joml/Matrix4f;",
            "rotateAround(Lorg/joml/Quaternionfc;FFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "invert()Lorg/joml/Matrix4f;", "invert(Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "setPerspective(FFFF)Lorg/joml/Matrix4f;", "setPerspective(FFFFZ)Lorg/joml/Matrix4f;",
            "perspective(FFFF)Lorg/joml/Matrix4f;", "perspective(FFFFZ)Lorg/joml/Matrix4f;",
            "perspective(FFFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;", "perspective(FFFFZLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "setOrtho(FFFFFF)Lorg/joml/Matrix4f;", "setOrtho(FFFFFFZ)Lorg/joml/Matrix4f;",
            "assume(I)Lorg/joml/Matrix4f;",
            "transformProject(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformProject(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformProject(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformPosition(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformPosition(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformPosition(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformDirection(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformDirection(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformDirection(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transform(Lorg/joml/Vector4f;)Lorg/joml/Vector4f;", "transform(Lorg/joml/Vector4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "transformTranspose(Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "transformTranspose(Lorg/joml/Vector4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "mul(Lorg/joml/Matrix3x2fc;)Lorg/joml/Matrix4f;", "mul(Lorg/joml/Matrix3x2fc;Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "invertAffine()Lorg/joml/Matrix4f;", "invertAffine(Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;", "isFinite()Z",
            "translation(Lorg/joml/Vector3fc;)Lorg/joml/Matrix4f;", "scale(Lorg/joml/Vector3fc;)Lorg/joml/Matrix4f;",
            "getTranslation(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;", "equals(Ljava/lang/Object;)Z", "hashCode()I"
        )
            .extraInputs("equals(Ljava/lang/Object;)Z", matrixEqualsInputs()),
        new InstanceSuite(
            Vector4f.class,
            "<init>()V", "<init>(F)V", "<init>(FFFF)V", "<init>(Lorg/joml/Vector3fc;F)V", "<init>(Lorg/joml/Vector4fc;)V",
            "set(FFFF)Lorg/joml/Vector4f;", "set(Lorg/joml/Vector4fc;)Lorg/joml/Vector4f;", "set(Lorg/joml/Vector3fc;F)Lorg/joml/Vector4f;",
            "mul(F)Lorg/joml/Vector4f;", "mul(FLorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "mul(Lorg/joml/Vector4fc;)Lorg/joml/Vector4f;", "mul(Lorg/joml/Vector4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "mul(Lorg/joml/Matrix4fc;)Lorg/joml/Vector4f;", "mul(Lorg/joml/Matrix4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "mulTranspose(Lorg/joml/Matrix4fc;)Lorg/joml/Vector4f;", "mulTranspose(Lorg/joml/Matrix4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "div(F)Lorg/joml/Vector4f;", "div(FLorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "lerp(Lorg/joml/Vector4fc;F)Lorg/joml/Vector4f;", "lerp(Lorg/joml/Vector4fc;FLorg/joml/Vector4f;)Lorg/joml/Vector4f;"
        ),
        new Matrix4fStackSuite(),
        new InstanceSuite(
            Vector2f.class,
            "<init>()V", "<init>(FF)V", "set(FF)Lorg/joml/Vector2f;", "zero()Lorg/joml/Vector2f;",
            "add(FF)Lorg/joml/Vector2f;", "add(FFLorg/joml/Vector2f;)Lorg/joml/Vector2f;",
            "mul(FF)Lorg/joml/Vector2f;", "mul(FFLorg/joml/Vector2f;)Lorg/joml/Vector2f;"
        ),
        new InstanceSuite(
            Matrix3x2f.class,
            "<init>()V", "<init>(Lorg/joml/Matrix3x2fc;)V", "set(Lorg/joml/Matrix3x2fc;)Lorg/joml/Matrix3x2f;",
            "translate(FF)Lorg/joml/Matrix3x2f;", "translate(FFLorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            "scale(FF)Lorg/joml/Matrix3x2f;", "scale(FFLorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            "scale(F)Lorg/joml/Matrix3x2f;", "scale(FLorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            "rotate(F)Lorg/joml/Matrix3x2f;", "rotate(FLorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            "invert(Lorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            "transformPosition(Lorg/joml/Vector2f;)Lorg/joml/Vector2f;",
            "transformPosition(FFLorg/joml/Vector2f;)Lorg/joml/Vector2f;"
        ),
        new Matrix3x2fStackSuite(),
        new InstanceSuite(
            FrustumIntersection.class,
            "<init>()V", "set(Lorg/joml/Matrix4fc;)Lorg/joml/FrustumIntersection;",
            "set(Lorg/joml/Matrix4fc;Z)Lorg/joml/FrustumIntersection;", "intersectAab(FFFFFF)I", "testPoint(FFF)Z"
        )
            .withFields()
            .extraInputs("intersectAab(FFFFFF)I", frustumBoxes())
            .extraInputs("testPoint(FFF)Z", frustumPoints()),
        new InstanceSuite(
            Axis.class,
            "rotation(F)Lorg/joml/Quaternionf;", "rotationDegrees(F)Lorg/joml/Quaternionf;",
            "rotate(Lorg/joml/Matrix3f;F)Lorg/joml/Matrix3f;", "rotateDegrees(Lorg/joml/Matrix3f;F)Lorg/joml/Matrix3f;",
            "rotate(Lorg/joml/Matrix4f;F)Lorg/joml/Matrix4f;", "rotateDegrees(Lorg/joml/Matrix4f;F)Lorg/joml/Matrix4f;"
        ),
        new InstanceSuite(
            GivensParameters.class,
            "<init>(FF)V", "fromUnnormalized(FF)Lcom/mojang/math/GivensParameters;", "fromPositiveAngle(F)Lcom/mojang/math/GivensParameters;",
            "sinHalf()F", "cosHalf()F", "inverse()Lcom/mojang/math/GivensParameters;", "cos()F", "sin()F",
            "aroundX(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;", "aroundY(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            "aroundZ(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;", "aroundX(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "aroundY(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;", "aroundZ(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;"
        ),
        new StaticSuite(MatrixUtil.class)
            .extraInputs("eigenvalueJacobi(Lorg/joml/Matrix3f;ILorg/joml/Quaternionf;)V", jacobiInputs())
            .extraInputs(
                "svdDecompose(Lorg/joml/Matrix4fc;Lorg/joml/Vector3f;Lorg/joml/Quaternionf;Lorg/joml/Vector3f;Lorg/joml/Quaternionf;)V",
                svdInputs()
            )
            // Each step costs about two matrix products, so billions of them would take minutes. The game takes 5.
            .where("eigenvalueJacobi(Lorg/joml/Matrix3f;ILorg/joml/Quaternionf;)V", args -> (Integer)args[1] <= 1024),
        new InstanceSuite(
            Transformation.class,
            "<init>(Lorg/joml/Matrix4fc;)V", "<init>(Lorg/joml/Vector3fc;Lorg/joml/Quaternionfc;Lorg/joml/Vector3fc;Lorg/joml/Quaternionfc;)V",
            "compose(Lcom/mojang/math/Transformation;)Lcom/mojang/math/Transformation;", "inverse()Lcom/mojang/math/Transformation;",
            "getMatrix()Lorg/joml/Matrix4fc;", "getMatrixCopy()Lorg/joml/Matrix4f;", "translation()Lorg/joml/Vector3fc;",
            "leftRotation()Lorg/joml/Quaternionfc;", "scale()Lorg/joml/Vector3fc;", "rightRotation()Lorg/joml/Quaternionfc;",
            "equals(Ljava/lang/Object;)Z", "hashCode()I", "slerp(Lcom/mojang/math/Transformation;F)Lcom/mojang/math/Transformation;",
            "compose(Lorg/joml/Matrix4fc;Ljava/util/Optional;)Lorg/joml/Matrix4fc;"
        )
            .withFields()
            .extraInputs(
                "<init>(Lorg/joml/Vector3fc;Lorg/joml/Quaternionfc;Lorg/joml/Vector3fc;Lorg/joml/Quaternionfc;)V", transformationPartsInputs()
            )
            .extraInputs("equals(Ljava/lang/Object;)Z", transformationEqualsInputs())
            .extraInputs("slerp(Lcom/mojang/math/Transformation;F)Lcom/mojang/math/Transformation;", transformationSlerpInputs()),
        new PoseStackSuite(),
        new ProjectionSuite(),
        new StaticSuite(Float.class)
            .only("floatToIntBits(F)I", "hashCode(F)I")
            .extraInputs("floatToIntBits(F)I", floatNaNs())
            .extraInputs("hashCode(F)I", floatNaNs()),
        new StaticSuite(Double.class)
            .only("doubleToLongBits(D)J", "hashCode(D)I")
            .extraInputs("doubleToLongBits(D)J", doubleNaNs())
            .extraInputs("hashCode(D)I", doubleNaNs()),
        new InstanceSuite(Vector3i.class, "<init>(III)V", "set(III)Lorg/joml/Vector3i;", "get(I)I", "setComponent(II)Lorg/joml/Vector3i;"),
        InstanceSuite.every(Vec3i.class)
            .extraInputs("closerThan(Lnet/minecraft/core/Vec3i;D)Z", distanceTies(0))
            .extraInputs("closerToCenterThan(Lnet/minecraft/core/Position;D)Z", distanceTies(1)),
        InstanceSuite.every(Direction.class)
            .extraInputs("byName(Ljava/lang/String;)Lnet/minecraft/core/Direction;", names(Direction.values()))
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/core/Direction;", names(Direction.values()))
            .extraInputs("getNearest(IIILnet/minecraft/core/Direction;)Lnet/minecraft/core/Direction;", nearestInputs(false))
            .extraInputs("getNearest(Lnet/minecraft/core/Vec3i;Lnet/minecraft/core/Direction;)Lnet/minecraft/core/Direction;", nearestInputs(true))
            .extraInputs("getApproximateNearest(FFF)Lnet/minecraft/core/Direction;", approximateNearestInputs(0))
            .extraInputs("getApproximateNearest(DDD)Lnet/minecraft/core/Direction;", approximateNearestInputs(1))
            .extraInputs("getApproximateNearest(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/core/Direction;", approximateNearestInputs(2))
            .extraInputs("fromYRot(D)Lnet/minecraft/core/Direction;", yRotBoundaries())
            .extraInputs("isFacingAngle(F)Z", facingAngles())
            .extraInputs("rotate(Lorg/joml/Matrix4fc;Lnet/minecraft/core/Direction;)Lnet/minecraft/core/Direction;", modelRotations())
            .extraInputs("axisStepOrder(Lnet/minecraft/world/phys/Vec3;)Lcom/google/common/collect/ImmutableList;", axisStepOrderTies()),
        InstanceSuite.every(Direction.Axis.class)
            .extraInputs("byName(Ljava/lang/String;)Lnet/minecraft/core/Direction$Axis;", names(Direction.Axis.values()))
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/core/Direction$Axis;", names(Direction.Axis.values()))
            .extraInputs("test(Lnet/minecraft/core/Direction;)Z", nullDirections(Direction.Axis.values())),
        InstanceSuite.every(Direction.AxisDirection.class)
            .extraInputs(
                "valueOf(Ljava/lang/String;)Lnet/minecraft/core/Direction$AxisDirection;", names(Direction.AxisDirection.values())
            ),
        InstanceSuite.every(Direction.Plane.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/core/Direction$Plane;", names(Direction.Plane.values()))
            .extraInputs("test(Lnet/minecraft/core/Direction;)Z", nullDirections(Direction.Plane.values())),
        InstanceSuite.every(Vec3.class)
            .extraInputs("closerThan(Lnet/minecraft/core/Position;D)Z", distanceTies(2))
            .extraInputs("closerThan(Lnet/minecraft/world/phys/Vec3;DD)Z", distanceTies(3))
            .extraInputs("normalize()Lnet/minecraft/world/phys/Vec3;", normalizeThresholds())
            .extraInputs("directionFromRotation(FF)Lnet/minecraft/world/phys/Vec3;", lookRotations()),
        InstanceSuite.every(Vec2.class)
            .extraInputs("normalized()Lnet/minecraft/world/phys/Vec2;", normalizedThresholds()),
        InstanceSuite.every(AxisCycle.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/core/AxisCycle;", names(AxisCycle.values())),
        InstanceSuite.every(Rotation.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/world/level/block/Rotation;", names(Rotation.values()))
            .extraInputs("rotate(II)I", rotationSteps()),
        InstanceSuite.every(BlockPos.class)
            .inherited("toString()Ljava/lang/String;")
            .extraInputs("asLong(III)J", packingLimits())
            .extraInputs("of(J)Lnet/minecraft/core/BlockPos;", packedInputs(0))
            .extraInputs("getX(J)I", packedInputs(0))
            .extraInputs("getY(J)I", packedInputs(0))
            .extraInputs("getZ(J)I", packedInputs(0))
            .extraInputs("getFlatIndex(J)J", packedInputs(0))
            .extraInputs("offset(JLnet/minecraft/core/Direction;)J", packedInputs(1))
            .extraInputs("offset(JIII)J", packedInputs(2))
            .extraInputs("clampLocationWithin(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/Vec3;", clampBounds())
            .extraInputs("withinManhattan(Lnet/minecraft/core/BlockPos;I)Ljava/lang/Iterable;", manhattanReaches(1))
            .extraInputs("withinClippedManhattan(Lnet/minecraft/core/BlockPos;III)Ljava/lang/Iterable;", manhattanReaches(3))
            .extraInputs("withinBoxByManhattanDistance(Lnet/minecraft/core/BlockPos;III)Ljava/lang/Iterable;", manhattanReaches(3))
            // With a negative reach and a large depth, the search never finds a block, so it runs until the depth wraps.
            .where("withinClippedManhattan(Lnet/minecraft/core/BlockPos;III)Ljava/lang/Iterable;", args -> manhattanEnds(args, Math.max(Math.max((Integer)args[1], (Integer)args[2]), (Integer)args[3])))
            .where("withinBoxByManhattanDistance(Lnet/minecraft/core/BlockPos;III)Ljava/lang/Iterable;", args -> manhattanEnds(args, (Integer)args[1] + (Integer)args[2] + (Integer)args[3]))
            .extraInputs("betweenClosed(IIIIII)Ljava/lang/Iterable;", smallBoxes(false))
            .extraInputs("betweenClosedStream(IIIIII)Ljava/util/stream/Stream;", smallBoxes(false))
            .extraInputs("betweenClosed(Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;)Ljava/lang/Iterable;", smallBoxes(true))
            .extraInputs("betweenClosedStream(Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;)Ljava/util/stream/Stream;", smallBoxes(true))
            .extraInputs("neighborColumn(IIII)Ljava/lang/Iterable;", neighborColumns())
            .extraInputs("spiralAround(Lnet/minecraft/core/BlockPos;ILnet/minecraft/core/Direction;Lnet/minecraft/core/Direction;)Ljava/lang/Iterable;", spirals())
            .extraInputs("betweenCornersInDirection(Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;Lnet/minecraft/world/phys/Vec3;)Ljava/lang/Iterable;", cornerWalks(true))
            .extraInputs("betweenCornersInDirection(IIIIIILnet/minecraft/world/phys/Vec3;)Ljava/lang/Iterable;", cornerWalks(false))
            .extraInputs("randomBetweenClosed(Lnet/minecraft/util/RandomSource;IIIIIII)Ljava/lang/Iterable;", randomBoxes(false))
            .extraInputs("randomInCube(Lnet/minecraft/util/RandomSource;ILnet/minecraft/core/BlockPos;I)Ljava/lang/Iterable;", randomBoxes(true))
            .extraInputs("betweenClosed(Lnet/minecraft/world/phys/AABB;)Ljava/lang/Iterable;", blockAlignedBoxes(0))
            .extraInputs("betweenClosedStream(Lnet/minecraft/world/phys/AABB;)Ljava/util/stream/Stream;", blockAlignedBoxes(0))
            .extraInputs("betweenCornersInDirection(Lnet/minecraft/world/phys/AABB;Lnet/minecraft/world/phys/Vec3;)Ljava/lang/Iterable;", blockAlignedBoxes(1)),
        InstanceSuite.every(BlockPos.MutableBlockPos.class)
            .inherited("toString()Ljava/lang/String;")
            .extraInputs("set(J)Lnet/minecraft/core/BlockPos$MutableBlockPos;", packedInputs(3)),
        InstanceSuite.every(BlockPos.TraversalNodeStatus.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/core/BlockPos$TraversalNodeStatus;", names(BlockPos.TraversalNodeStatus.values())),
        InstanceSuite.every(AABB.class)
            .extraInputs("encapsulatingFullBlocks(Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/phys/AABB;", limitBlockPairs())
            .extraInputs("contract(DDD)Lnet/minecraft/world/phys/AABB;", signedZeroMoves(false))
            .extraInputs("expandTowards(DDD)Lnet/minecraft/world/phys/AABB;", signedZeroMoves(false))
            .extraInputs("expandTowards(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/AABB;", signedZeroMoves(true))
            .extraInputs("equals(Ljava/lang/Object;)Z", boxEqualsInputs())
            .extraInputs("intersects(Lnet/minecraft/world/phys/AABB;)Z", touchingBoxes(0))
            .extraInputs("intersect(Lnet/minecraft/world/phys/AABB;)Lnet/minecraft/world/phys/AABB;", touchingBoxes(0))
            .extraInputs("minmax(Lnet/minecraft/world/phys/AABB;)Lnet/minecraft/world/phys/AABB;", touchingBoxes(0))
            .extraInputs("distanceToSqr(Lnet/minecraft/world/phys/AABB;)D", touchingBoxes(0))
            .extraInputs("intersects(DDDDDD)Z", touchingBoxes(1))
            .extraInputs("intersects(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;)Z", touchingBoxes(2))
            .extraInputs("intersects(Lnet/minecraft/core/BlockPos;)Z", touchingBoxes(3))
            .extraInputs("contains(Lnet/minecraft/world/phys/Vec3;)Z", facePoints(0))
            .extraInputs("contains(DDD)Z", facePoints(1))
            .extraInputs("distanceToSqr(Lnet/minecraft/world/phys/Vec3;)D", facePoints(0))
            .extraInputs("clip(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;)Ljava/util/Optional;", clipRays(0))
            .extraInputs("clip(DDDDDDLnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;)Ljava/util/Optional;", clipRays(1))
            .extraInputs(
                "clip(Ljava/lang/Iterable;Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/phys/BlockHitResult;",
                clipRays(2)
            )
            .extraInputs("collidedAlongVector(Lnet/minecraft/world/phys/Vec3;Ljava/util/List;)Z", collisionPaths()),
        InstanceSuite.every(AABB.Builder.class),
        // A BlockHitResult is the only HitResult so far: an EntityHitResult needs entities.
        InstanceSuite.every(HitResult.class),
        InstanceSuite.every(HitResult.Type.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/world/phys/HitResult$Type;", names(HitResult.Type.values())),
        InstanceSuite.every(BlockHitResult.class)
    );

    public static void main(final String[] args) throws IOException {
        Path outputDir = Path.of(args[0]);
        Files.createDirectories(outputDir);
        for (Suite suite : SUITES) {
            Path file = outputDir.resolve(suite.name() + ".jsonl");
            try (GoldenWriter writer = new GoldenWriter(file)) {
                suite.write(writer);
                System.out.printf("%s: %d lines%n", file.getFileName(), writer.lines());
            }
        }
    }

    // One input per entry of Mth's 65536-entry sine table, offset by `shift` entries (cos reads the table a quarter turn ahead).
    private static List<Object[]> sinTableInputs(final int shift) {
        List<Object[]> inputs = new ArrayList<>();
        for (int index = 0; index < 65536; index++) {
            inputs.add(new Object[]{(index + shift + 0.5) / SIN_SCALE});
        }
        return inputs;
    }

    // Angles game code passes, near multiples of pi/64 (where the reduction cancels the most, and
    // tan's poles), every magnitude from 2^low to 2^high, which spans the main path and its
    // short-cuts for small inputs (2^-252 to 90112 for sin and cos, 2^-29 to 12800 for tan), the
    // large-argument path up to 2^1024, tiny and subnormal inputs, and arbitrary bit patterns.
    private static List<Object[]> trigInputs(final long seed, final int low, final int high) {
        Random random = new Random(seed);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < INTRINSIC_INPUTS; i++) {
            double x = switch (i % 6) {
                case 0 -> (random.nextDouble() * 2.0 - 1.0) * 8.0 * Math.PI;
                case 1 -> {
                    double multiple = random.nextInt(1 << 20) * (Math.PI / 64.0);
                    yield multiple + (random.nextInt(9) - 4) * Math.ulp(multiple);
                }
                case 2 -> magnitude(random, low, high);
                case 3 -> magnitude(random, high, 1023);
                case 4 -> magnitude(random, -1074, low);
                default -> randomBits(random);
            };
            inputs.add(new Object[]{random.nextBoolean() ? x : -x});
        }
        return inputs;
    }

    // Around 1, where the result is smallest, (0, 1) like the squared radii nextGaussian takes,
    // every exponent from the subnormals up, and arbitrary bit patterns, negatives included.
    private static List<Object[]> logInputs() {
        Random random = new Random(3);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < INTRINSIC_INPUTS; i++) {
            double x = switch (i % 5) {
                case 0 -> nearOne(random);
                case 1 -> random.nextDouble();
                case 2 -> random.nextDouble() * 1000.0;
                case 3 -> magnitude(random, -1074, 1023);
                default -> randomBits(random);
            };
            inputs.add(new Object[]{x});
        }
        return inputs;
    }

    // Bases and exponents like game code's, small integer exponents of any base, bases near 1
    // (where log2 takes a more accurate path) with large exponents, results across the whole range
    // (subnormal, overflowing and underflowing ones included), negative bases with integer and
    // other exponents, tiny exponents, the exponents with special cases, and arbitrary bit patterns.
    private static List<Object[]> powInputs() {
        Random random = new Random(6);
        double[] specialExponents = {
            0.5, -0.5, 2.0, 1.0, -1.0, 3.0, 0.0, -0.0, Double.POSITIVE_INFINITY, Double.NEGATIVE_INFINITY, Double.NaN
        };
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < INTRINSIC_INPUTS; i++) {
            Object[] input = switch (i % 10) {
                case 0 -> new Object[]{random.nextDouble() * 1000.0, (random.nextDouble() * 2.0 - 1.0) * 8.0};
                case 1 -> new Object[]{sign(random) * magnitude(random, -32, 31), (double)(random.nextInt(41) - 20)};
                case 2 -> new Object[]{sign(random) * nearOne(random), sign(random) * Math.rint(magnitude(random, 0, 63))};
                case 3 -> new Object[]{nearOne(random), sign(random) * magnitude(random, -20, 63)};
                case 4 -> {
                    double x = magnitude(random, -1074, 1023);
                    double log2 = Math.log(x) / Math.log(2.0);
                    yield new Object[]{x, (random.nextDouble() * 2200.0 - 1120.0) / log2};
                }
                case 5 -> new Object[]{-magnitude(random, -1074, 1023), (double)(random.nextInt(41) - 20)};
                case 6 -> {
                    double y = sign(random) * magnitude(random, -4, 59);
                    yield new Object[]{-magnitude(random, -64, 63), random.nextBoolean() ? Math.rint(y) : y};
                }
                case 7 -> new Object[]{magnitude(random, -1074, 1023), sign(random) * magnitude(random, -1074, 0)};
                case 8 -> new Object[]{
                    random.nextBoolean() ? randomBits(random) : sign(random) * nearOne(random),
                    specialExponents[random.nextInt(specialExponents.length)]
                };
                default -> new Object[]{randomBits(random), randomBits(random)};
            };
            inputs.add(input);
        }
        return inputs;
    }

    // Pairs like qrGivensQuat's (floats, often one of them zero), then every path through fdlibm's hypot: exponents
    // about 60 apart (where it returns the sum), both over 2^500 (scaled down first) or under 2^-500 (scaled up),
    // subnormals, the two ways it splits the sum of squares (a - b above b or not, and a = 2b where they meet), the
    // thresholds, and any bits.
    private static List<Object[]> hypotInputs() {
        Random random = new Random(10);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 6000; i++) {
            double[] pair = switch (i % 6) {
                case 0 -> new double[]{JomlValues.component(random), random.nextInt(4) == 0 ? 0.0 : JomlValues.component(random)};
                case 1 -> {
                    double a = magnitude(random, -1000, 1000);
                    yield new double[]{a, Math.scalb(1.0 + random.nextDouble(), Math.getExponent(a) - 56 - random.nextInt(9))};
                }
                case 2 -> new double[]{magnitude(random, 480, 1023), magnitude(random, 400, 1023)};
                case 3 -> new double[]{magnitude(random, -1074, -480), magnitude(random, -1074, -400)};
                case 4 -> {
                    double a = magnitude(random, -60, 60);
                    if (random.nextBoolean()) {
                        double b = a / 2.0;
                        for (int ulps = random.nextInt(5) - 2; ulps != 0; ulps -= Integer.signum(ulps)) {
                            b = ulps > 0 ? Math.nextUp(b) : Math.nextDown(b);
                        }
                        yield new double[]{a, b};
                    }
                    yield new double[]{a, a * (0.25 + random.nextDouble() * 0.75)};
                }
                default -> new double[]{randomBits(random), randomBits(random)};
            };
            int first = random.nextInt(2);
            inputs.add(new Object[]{sign(random) * pair[first], sign(random) * pair[1 - first]});
        }
        double[] thresholds = {
            0x1.00000_ffff_ffffp500, Math.nextUp(0x1.00000_ffff_ffffp500), 0x1.0p-500, Math.nextDown(0x1.0p-500),
            Double.MIN_NORMAL, Math.nextDown(Double.MIN_NORMAL)
        };
        for (double a : thresholds) {
            for (double b : thresholds) {
                inputs.add(new Object[]{a, b});
                inputs.add(new Object[]{a, b * 0x1.0p-30});
            }
        }
        return inputs;
    }

    // Rotations toward ones a little away (where the dot product nears 1, and slerp turns into a lerp at 1 - 1e-6) or
    // anywhere, half of them unit quaternions, a quarter in the other hemisphere (a negative dot product), at Display's
    // progress from 0 to 1 or anything.
    private static List<Object[]> slerpInputs() {
        Random random = new Random(11);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 3000; i++) {
            Quaternionf from = JomlValues.randomQuaternionf(random);
            if (random.nextBoolean()) {
                from.normalize();
            }
            float angle = switch (i % 3) {
                case 0 -> random.nextFloat() * 0.01F;
                case 1 -> Math.scalb(random.nextFloat(), -random.nextInt(24));
                default -> random.nextFloat() * 2.0F * (float)Math.PI;
            };
            Quaternionf to = from.mul(new Quaternionf().rotationAxis(angle, JomlValues.randomVector3f(random).normalize()), new Quaternionf());
            if (random.nextInt(4) == 0) {
                to.set(-to.x, -to.y, -to.z, -to.w);
            }
            float alpha = random.nextInt(8) == 0 ? (Float)Values.FLOATS.random(random) : random.nextFloat();
            inputs.add(new Object[]{from, to, alpha, JomlValues.QUATERNIONF.random(random)});
        }
        return inputs;
    }

    // Matrices next to copies of themselves, equal whatever their properties say, or with one component's zero flipped
    // or moved up an ulp, which makes them differ.
    private static List<Object[]> matrixEqualsInputs() {
        Random random = new Random(12);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 600; i++) {
            Matrix4f matrix = (Matrix4f)JomlValues.MATRIX4F.random(random);
            Matrix4f other = new Matrix4f(matrix);
            switch (i % 3) {
                case 0 -> other.assume(random.nextInt(32));
                case 1 -> {
                    int column = random.nextInt(4);
                    int row = random.nextInt(4);
                    float value = other.get(column, row);
                    other.set(column, row, value == 0.0F ? -value : Math.nextUp(value));
                }
                default -> {
                }
            }
            inputs.add(new Object[]{matrix, other});
        }
        return inputs;
    }

    // What svdDecompose passes: M^T M for the 3x3 part M of a transformation like the game's, for its 5 steps or a few
    // more or fewer. Then symmetric matrices whose first rotation sits where approxGivensQuat switches to a quarter turn,
    // G * a12^2 = (2 * (a11 - a22))^2 give or take a few ulps of a12, which pins down G.
    private static List<Object[]> jacobiInputs() {
        Random random = new Random(13);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 2000; i++) {
            Matrix3f matrix = new Matrix3f(((Transformation)MathValues.TRANSFORMATION.random(random)).getMatrix());
            Matrix3f symmetric = new Matrix3f(matrix).transpose().mul(matrix);
            int steps = i % 4 == 0 ? random.nextInt(11) : 5;
            inputs.add(new Object[]{symmetric, steps, JomlValues.QUATERNIONF.random(random)});
        }
        float g = 3.0F + 2.0F * (float)Math.sqrt(2.0F);
        for (int i = 0; i < 500; i++) {
            float a11 = random.nextFloat() * 8.0F;
            float a22 = random.nextFloat() * 8.0F;
            float ch = 2.0F * (a11 - a22);
            float a12 = (float)(ch / Math.sqrt(g));
            for (int ulps = random.nextInt(7) - 3; ulps != 0; ulps -= Integer.signum(ulps)) {
                a12 = ulps > 0 ? Math.nextUp(a12) : Math.nextDown(a12);
            }
            float a33 = random.nextFloat() * 8.0F;
            inputs.add(new Object[]{new Matrix3f(a11, a12, 0.0F, a12, a22, 0.0F, 0.0F, 0.0F, a33), 1 + random.nextInt(5), new Quaternionf()});
        }
        return inputs;
    }

    // The matrices of transformations like the game's, which Transformation decomposes.
    private static List<Object[]> svdInputs() {
        Random random = new Random(14);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 3000; i++) {
            Transformation transformation = (Transformation)MathValues.TRANSFORMATION.random(random);
            inputs.add(new Object[]{
                new Matrix4f(transformation.getMatrix()), JomlValues.VECTOR3F.random(random), JomlValues.QUATERNIONF.random(random),
                JomlValues.VECTOR3F.random(random), JomlValues.QUATERNIONF.random(random)
            });
        }
        return inputs;
    }

    // Parts like the game's, in every pattern of null (no translation, rotation or scale).
    private static List<Object[]> transformationPartsInputs() {
        Random random = new Random(15);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 1600; i++) {
            int nulls = i % 16;
            inputs.add(new Object[]{
                (nulls & 1) != 0 ? null : MathValues.randomTranslation(random), (nulls & 2) != 0 ? null : MathValues.randomRotation(random),
                (nulls & 4) != 0 ? null : MathValues.randomScale(random), (nulls & 8) != 0 ? null : MathValues.randomRotation(random)
            });
        }
        return inputs;
    }

    // Transformations next to copies of themselves, decomposed or not, or ones built from their matrix with any
    // properties, which are all equal since only the matrix's components count; IDENTITY next to an identity matrix.
    private static List<Object[]> transformationEqualsInputs() {
        Random random = new Random(16);
        List<Object[]> inputs = new ArrayList<>();
        inputs.add(new Object[]{Transformation.IDENTITY, new Transformation(new Matrix4f())});
        for (int i = 0; i < 300; i++) {
            Transformation transformation = (Transformation)MathValues.TRANSFORMATION.random(random);
            Transformation other = switch (i % 3) {
                case 0 -> transformation;
                case 1 -> new Transformation(transformation.getMatrixCopy());
                default -> new Transformation(transformation.getMatrixCopy().assume(random.nextInt(32)));
            };
            inputs.add(new Object[]{transformation, other});
        }
        return inputs;
    }

    // Display entities interpolating from one transformation to the next.
    private static List<Object[]> transformationSlerpInputs() {
        Random random = new Random(17);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 1000; i++) {
            inputs.add(new Object[]{MathValues.TRANSFORMATION.random(random), MathValues.TRANSFORMATION.random(random), random.nextFloat()});
        }
        return inputs;
    }

    // Every angle in every octant often enough to read each entry of Mth.atan2's tables, whose
    // COS_TAB comes from Math.cos, at radii from 2^-20 to 2^20.
    private static List<Object[]> atan2Inputs() {
        Random random = new Random(4);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 4096; i++) {
            double angle = (i + random.nextDouble()) * (2.0 * Math.PI / 4096.0);
            double radius = Math.scalb(1.0 + random.nextDouble(), random.nextInt(41) - 20);
            inputs.add(new Object[]{radius * Math.sin(angle), radius * Math.cos(angle)});
        }
        return inputs;
    }

    // Boxes like the game culls, relative to a camera anywhere in the world: chunk sections around it, entity-sized
    // boxes near it, the cubes offsetToFullyIncludeCameraCube(8) tests (the camera's 8-block cube, stepped back along
    // the view), single points, boxes of any size (inverted ones included), and boxes with a corner on a plane. Then
    // every frustum edge with a box in front of an unrotated camera that has each float edge in each coordinate.
    private static List<Object[]> frustumBoxes() {
        Random random = new Random(8);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 8000; i++) {
            FrustumIntersection frustum = (FrustumIntersection)JomlValues.FRUSTUM_INTERSECTION.random(random);
            if (i % 8 >= 5) {
                inputs.add(boxOnPlane(frustum, random));
                continue;
            }
            double camX = (random.nextDouble() * 2.0 - 1.0) * 1000.0;
            double camY = -64.0 + random.nextDouble() * 384.0;
            double camZ = (random.nextDouble() * 2.0 - 1.0) * 1000.0;
            double[] box = switch (i % 8) {
                case 0 -> {
                    double x = (Math.floor(camX / 16.0) + random.nextInt(25) - 12) * 16.0;
                    double y = (Math.floor(camY / 16.0) + random.nextInt(9) - 4) * 16.0;
                    double z = (Math.floor(camZ / 16.0) + random.nextInt(25) - 12) * 16.0;
                    yield new double[]{x, y, z, x + 16.0, y + 16.0, z + 16.0};
                }
                case 1 -> {
                    double x = camX + (random.nextDouble() * 2.0 - 1.0) * 48.0;
                    double y = camY + (random.nextDouble() * 2.0 - 1.0) * 24.0;
                    double z = camZ + (random.nextDouble() * 2.0 - 1.0) * 48.0;
                    double width = random.nextBoolean() ? 0.6 : random.nextDouble() * 4.0;
                    double height = random.nextBoolean() ? 1.8 : random.nextDouble() * 4.0;
                    yield new double[]{x - width / 2.0, y, z - width / 2.0, x + width / 2.0, y + height, z + width / 2.0};
                }
                case 2 -> {
                    double step = random.nextInt(8) * 4.0;
                    double x = camX - (random.nextDouble() * 2.0 - 1.0) * step;
                    double y = camY - (random.nextDouble() * 2.0 - 1.0) * step;
                    double z = camZ - (random.nextDouble() * 2.0 - 1.0) * step;
                    yield new double[]{
                        Math.floor(x / 8.0) * 8.0, Math.floor(y / 8.0) * 8.0, Math.floor(z / 8.0) * 8.0,
                        Math.ceil(x / 8.0) * 8.0, Math.ceil(y / 8.0) * 8.0, Math.ceil(z / 8.0) * 8.0
                    };
                }
                case 3 -> {
                    // Half of them around the near plane, which is 0.05 away.
                    double range = random.nextBoolean() ? 300.0 : 0.1;
                    double x = camX + (random.nextDouble() * 2.0 - 1.0) * range;
                    double y = camY + (random.nextDouble() * 2.0 - 1.0) * range;
                    double z = camZ + (random.nextDouble() * 2.0 - 1.0) * range;
                    yield new double[]{x, y, z, x, y, z};
                }
                default -> {
                    double[] corners = new double[6];
                    for (int axis = 0; axis < 3; axis++) {
                        double cam = axis == 0 ? camX : axis == 1 ? camY : camZ;
                        double min = cam + (random.nextDouble() * 2.0 - 1.0) * 512.0;
                        corners[axis] = min;
                        corners[axis + 3] = min + (random.nextInt(8) == 0 ? -1.0 : 1.0) * random.nextDouble() * 256.0;
                    }
                    yield corners;
                }
            };
            inputs.add(new Object[]{
                frustum, (float)(box[0] - camX), (float)(box[1] - camY), (float)(box[2] - camZ),
                (float)(box[3] - camX), (float)(box[4] - camY), (float)(box[5] - camZ)
            });
        }

        float[] inFront = {-1.0F, -1.0F, -11.0F, 1.0F, 1.0F, -9.0F};
        for (Object frustum : JomlValues.FRUSTUM_INTERSECTION.edges()) {
            for (int coordinate = 0; coordinate < 6; coordinate++) {
                for (Object edge : Values.FLOATS.edges()) {
                    Object[] input = {frustum, inFront[0], inFront[1], inFront[2], inFront[3], inFront[4], inFront[5]};
                    input[1 + coordinate] = edge;
                    inputs.add(input);
                }
            }
        }
        return inputs;
    }

    // A box of up to 16 blocks a side with its corner furthest along a plane's normal, or nearest, on the plane: which
    // side that corner rounds to depends on the order the plane's terms are added in.
    private static Object[] boxOnPlane(final FrustumIntersection frustum, final Random random) {
        int plane = random.nextInt(6);
        float[] normal = JomlValues.frustumPlane(frustum, plane);
        float[] corner = pointOnPlane(frustum, plane, random, 256.0);
        boolean furthest = random.nextBoolean();
        Object[] input = new Object[7];
        input[0] = frustum;
        for (int axis = 0; axis < 3; axis++) {
            float size = random.nextInt(4) == 0 ? 0.0F : random.nextFloat() * 16.0F;
            // The furthest corner takes the min where the normal is negative, the nearest one the max.
            boolean cornerIsMin = (normal[axis] < 0.0F) == furthest;
            input[1 + axis] = cornerIsMin ? corner[axis] : corner[axis] - size;
            input[4 + axis] = cornerIsMin ? corner[axis] + size : corner[axis];
        }
        return input;
    }

    // Points relative to the camera, near it and far, like Frustum.pointInFrustum's, and twice as many on a plane.
    private static List<Object[]> frustumPoints() {
        Random random = new Random(9);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 6000; i++) {
            FrustumIntersection frustum = (FrustumIntersection)JomlValues.FRUSTUM_INTERSECTION.random(random);
            if (i % 3 != 0) {
                float[] point = pointOnPlane(frustum, random.nextInt(6), random, random.nextBoolean() ? 4.0 : 256.0);
                inputs.add(new Object[]{frustum, point[0], point[1], point[2]});
                continue;
            }
            double range = i % 2 == 0 ? 4.0 : 512.0;
            inputs.add(new Object[]{
                frustum, (float)((random.nextDouble() * 2.0 - 1.0) * range), (float)((random.nextDouble() * 2.0 - 1.0) * range),
                (float)((random.nextDouble() * 2.0 - 1.0) * range)
            });
        }
        return inputs;
    }

    // A point on one of the frustum's planes, as near as floats get, then moved up to two ulps along the axis the plane
    // depends on most, so which side of the plane it's on depends on the order the plane's terms are added in. It's
    // also at a random distance inside one plane of each other pair, so it's often on the frustum's face, where the
    // other planes let it through. Planes that don't pin down a point get the other two coordinates within the range.
    private static float[] pointOnPlane(final FrustumIntersection frustum, final int index, final Random random, final double range) {
        float[] plane = JomlValues.frustumPlane(frustum, index);
        int second = 2 * ((index / 2 + 1) % 3) + random.nextInt(2);
        int third = 2 * ((index / 2 + 2) % 3) + random.nextInt(2);
        double scale = switch (random.nextInt(3)) {
            case 0 -> 0.01;
            case 1 -> 1.0;
            default -> range;
        };
        double[] point = solvePlanes(
            plane, JomlValues.frustumPlane(frustum, second), JomlValues.frustumPlane(frustum, third),
            random.nextDouble() * scale, random.nextDouble() * scale
        );
        int axis = 0;
        for (int i = 1; i < 3; i++) {
            if (Math.abs(plane[i]) > Math.abs(plane[axis])) {
                axis = i;
            }
        }
        if (point == null) {
            point = new double[3];
            double sum = plane[3];
            for (int i = 0; i < 3; i++) {
                if (i != axis) {
                    point[i] = (float)((random.nextDouble() * 2.0 - 1.0) * range);
                    sum += plane[i] * point[i];
                }
            }
            point[axis] = -sum / plane[axis];
        }
        float[] rounded = {(float)point[0], (float)point[1], (float)point[2]};
        for (int ulps = random.nextInt(5) - 2; ulps != 0; ulps -= Integer.signum(ulps)) {
            rounded[axis] = ulps > 0 ? Math.nextUp(rounded[axis]) : Math.nextDown(rounded[axis]);
        }
        return rounded;
    }

    // The point at distance 0 from plane a, b from plane b and c from plane c (planes are [x, y, z, w], and a point's
    // distance is x * px + y * py + z * pz + w), by Cramer's rule, or null when the planes don't meet in one point.
    private static double[] solvePlanes(final float[] a, final float[] b, final float[] c, final double distanceB, final double distanceC) {
        double[][] rows = {{a[0], a[1], a[2]}, {b[0], b[1], b[2]}, {c[0], c[1], c[2]}};
        double[] rhs = {-a[3], distanceB - b[3], distanceC - c[3]};
        double determinant = determinant(rows);
        if (!Double.isFinite(determinant) || Math.abs(determinant) < 1.0E-6) {
            return null;
        }
        double[] point = new double[3];
        for (int axis = 0; axis < 3; axis++) {
            double[][] replaced = {rows[0].clone(), rows[1].clone(), rows[2].clone()};
            for (int row = 0; row < 3; row++) {
                replaced[row][axis] = rhs[row];
            }
            point[axis] = determinant(replaced) / determinant;
        }
        return Double.isFinite(point[0]) && Double.isFinite(point[1]) && Double.isFinite(point[2]) ? point : null;
    }

    private static double determinant(final double[][] m) {
        return m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    }

    // NaNs other than the canonical one, quiet and signalling, of either sign, which floatToIntBits makes canonical.
    private static List<Object[]> floatNaNs() {
        List<Object[]> inputs = new ArrayList<>();
        for (int bits : new int[]{0x7f800001, 0x7fc00001, 0x7fffffff, 0xff800001, 0xffc00000, 0xffffffff}) {
            inputs.add(new Object[]{Float.intBitsToFloat(bits)});
        }
        return inputs;
    }

    private static List<Object[]> doubleNaNs() {
        List<Object[]> inputs = new ArrayList<>();
        for (long bits : new long[]{0x7ff0000000000001L, 0x7ff8000000000001L, 0x7fffffffffffffffL, 0xfff0000000000001L, 0xfff8000000000000L, -1L}) {
            inputs.add(new Object[]{Double.longBitsToDouble(bits)});
        }
        return inputs;
    }

    // An enum's names as byName takes them (serialized, lower case) and as valueOf does (the constant's), in other cases,
    // and ones neither knows.
    private static List<Object[]> names(final Enum<?>[] constants) {
        List<Object[]> inputs = new ArrayList<>();
        for (Enum<?> constant : constants) {
            String name = constant.name();
            String serialized = constant instanceof StringRepresentable representable ? representable.getSerializedName() : name.toLowerCase(Locale.ROOT);
            inputs.add(new Object[]{name});
            inputs.add(new Object[]{serialized});
            inputs.add(new Object[]{name.charAt(0) + name.substring(1).toLowerCase(Locale.ROOT)});
            inputs.add(new Object[]{" " + serialized});
        }
        inputs.add(new Object[]{"minecraft:north"});
        inputs.add(new Object[]{"null"});
        return inputs;
    }

    // Small vectors, so two or three components often tie for the largest, with every fallback, null included.
    private static List<Object[]> nearestInputs(final boolean vec3i) {
        Random random = new Random(18);
        List<Direction> fallbacks = new ArrayList<>(Arrays.asList(Direction.values()));
        fallbacks.add(null);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 700; i++) {
            int x = random.nextInt(7) - 3;
            int y = random.nextInt(7) - 3;
            int z = random.nextInt(7) - 3;
            Direction fallback = fallbacks.get(i % fallbacks.size());
            inputs.add(vec3i ? new Object[]{new Vec3i(x, y, z), fallback} : new Object[]{x, y, z, fallback});
        }
        return inputs;
    }

    // Every vector made of components that tie, are too small for the dot product to pass Float.MIN_VALUE (which is
    // positive, so nothing beats NORTH unless it's above it), or are infinite or NaN, as floats, doubles or a Vec3.
    private static List<Object[]> approximateNearestInputs(final int kind) {
        float[] floats = {0.0F, -0.0F, 1.0F, -1.0F, Float.MIN_VALUE, -Float.MIN_VALUE, 2.0F * Float.MIN_VALUE, Float.POSITIVE_INFINITY, Float.NaN};
        double[] doubles = {0.0, -0.0, 1.0, -1.0, Double.MIN_VALUE, 0x1.0p-149, 0x1.8p-149, 1.0E300, Double.NaN};
        List<Object[]> inputs = new ArrayList<>();
        for (int x = 0; x < 9; x++) {
            for (int y = 0; y < 9; y++) {
                for (int z = 0; z < 9; z++) {
                    inputs.add(switch (kind) {
                        case 0 -> new Object[]{floats[x], floats[y], floats[z]};
                        case 1 -> new Object[]{doubles[x], doubles[y], doubles[z]};
                        default -> new Object[]{new Vec3(doubles[x], doubles[y], doubles[z])};
                    });
                }
            }
        }
        return inputs;
    }

    // Every multiple of 45 degrees from -720 to 720, where fromYRot switches direction, and an ulp either side.
    private static List<Object[]> yRotBoundaries() {
        List<Object[]> inputs = new ArrayList<>();
        for (int k = -16; k <= 16; k++) {
            double angle = k * 45.0;
            inputs.add(new Object[]{Math.nextDown(angle)});
            inputs.add(new Object[]{angle});
            inputs.add(new Object[]{Math.nextUp(angle)});
        }
        return inputs;
    }

    // Each direction facing every degree of a turn and a bit, and the quarter turns where the dot product is about 0.
    private static List<Object[]> facingAngles() {
        List<Object[]> inputs = new ArrayList<>();
        for (Direction direction : Direction.values()) {
            for (int degrees = -450; degrees <= 450; degrees++) {
                inputs.add(new Object[]{direction, (float)degrees});
            }
            for (int k = -4; k <= 4; k++) {
                float angle = k * 90.0F;
                inputs.add(new Object[]{direction, Math.nextDown(angle)});
                inputs.add(new Object[]{direction, Math.nextUp(angle)});
            }
        }
        return inputs;
    }

    // BlockModelRotation's matrices, one per OctahedralGroup, which UnbakedCuboidGeometry turns faces' cull directions by.
    private static List<Object[]> modelRotations() {
        List<Object[]> inputs = new ArrayList<>();
        for (OctahedralGroup group : OctahedralGroup.values()) {
            for (Direction direction : Direction.values()) {
                inputs.add(new Object[]{new Matrix4f(group.transformation()), direction});
            }
        }
        return inputs;
    }

    // Movements whose x and z are as large as each other, of either sign.
    private static List<Object[]> axisStepOrderTies() {
        Random random = new Random(19);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 100; i++) {
            double a = i % 10 == 0 ? 0.0 : (random.nextDouble() * 2.0 - 1.0) * 4.0;
            inputs.add(new Object[]{new Vec3(a, random.nextDouble() - 0.5, random.nextBoolean() ? a : -a)});
        }
        return inputs;
    }

    private static List<Object[]> nullDirections(final Object[] receivers) {
        List<Object[]> inputs = new ArrayList<>();
        for (Object receiver : receivers) {
            inputs.add(new Object[]{receiver, null});
        }
        return inputs;
    }

    // Two points exactly as far apart as the distance they're checked against, or an ulp of the distance either side, which
    // the strict comparisons tell apart: blocks (kind 0), a block and a point that far from its center (1), points (2),
    // and points as far apart horizontally and vertically as the cylinder's radius and half height (3).
    private static List<Object[]> distanceTies(final int kind) {
        Random random = new Random(22 + kind);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 300; i++) {
            Vec3i block = CoreValues.randomVec3i(random);
            block = new Vec3i(block.getX() % 1000000, block.getY() % 1000, block.getZ() % 1000000);
            Vec3 point = new Vec3(block.getX() + random.nextInt(8) / 8.0, block.getY() + random.nextInt(8) / 8.0, block.getZ() + random.nextInt(8) / 8.0);
            int apart = random.nextInt(64) - 32;
            int axis = random.nextInt(3);
            double distance = Math.abs(apart);
            distance = switch (random.nextInt(4)) {
                case 0 -> Math.nextDown(distance);
                case 1 -> Math.nextUp(distance);
                default -> distance;
            };
            inputs.add(switch (kind) {
                case 0 -> new Object[]{block, block.relative(Direction.Axis.VALUES[axis], apart), distance};
                case 1 -> new Object[]{block, Vec3.atCenterOf(block).with(Direction.Axis.VALUES[axis], Vec3.atCenterOf(block).get(Direction.Axis.VALUES[axis]) + apart), distance};
                case 2 -> new Object[]{point, point.with(Direction.Axis.VALUES[axis], point.get(Direction.Axis.VALUES[axis]) + apart), distance};
                default -> {
                    int up = random.nextInt(8) - 4;
                    double height = random.nextBoolean() ? Math.abs(up) : Math.abs(up) + random.nextInt(2) * 0.5;
                    Direction.Axis horizontal = random.nextBoolean() ? Direction.Axis.X : Direction.Axis.Z;
                    Vec3 other = point.with(horizontal, point.get(horizontal) + apart).add(0.0, up, 0.0);
                    yield new Object[]{point, other, distance, height};
                }
            });
        }
        return inputs;
    }

    // Vectors whose length is within a few ulps of normalize's threshold, (double)1.0E-5F, either side of it, first along an
    // axis, where the length comes out exactly (the square root of a rounded square is the number again).
    private static List<Object[]> normalizeThresholds() {
        Random random = new Random(20);
        List<Object[]> inputs = new ArrayList<>();
        for (int ulps = -2; ulps <= 2; ulps++) {
            double length = 1.0E-5F;
            for (int step = ulps; step != 0; step -= Integer.signum(step)) {
                length = step > 0 ? Math.nextUp(length) : Math.nextDown(length);
            }
            for (Direction direction : Direction.values()) {
                inputs.add(new Object[]{direction.getUnitVec3().scale(length)});
            }
        }
        for (int i = 0; i < 500; i++) {
            Vec3 unit = Vec3.directionFromRotation((random.nextFloat() * 2.0F - 1.0F) * 90.0F, (random.nextFloat() * 2.0F - 1.0F) * 180.0F);
            double length = 1.0E-5F;
            for (int ulps = random.nextInt(9) - 4; ulps != 0; ulps -= Integer.signum(ulps)) {
                length = ulps > 0 ? Math.nextUp(length) : Math.nextDown(length);
            }
            inputs.add(new Object[]{new Vec3(unit.x * length, unit.y * length, unit.z * length)});
        }
        return inputs;
    }

    private static List<Object[]> normalizedThresholds() {
        Random random = new Random(21);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 500; i++) {
            float angle = random.nextFloat() * 2.0F * (float)Math.PI;
            float length = 1.0E-4F;
            for (int ulps = random.nextInt(9) - 4; ulps != 0; ulps -= Integer.signum(ulps)) {
                length = ulps > 0 ? Math.nextUp(length) : Math.nextDown(length);
            }
            inputs.add(new Object[]{new Vec2((float)Math.cos(angle) * length, (float)Math.sin(angle) * length)});
        }
        return inputs;
    }

    // The walk scenario's looks, and every whole x rotation (up and down) at y rotations around the circle.
    private static List<Object[]> lookRotations() {
        List<Object[]> inputs = new ArrayList<>();
        inputs.add(new Object[]{0.0F, 0.0F});
        inputs.add(new Object[]{30.0F, 90.0F});
        for (int xRot = -90; xRot <= 90; xRot++) {
            inputs.add(new Object[]{(float)xRot, (xRot + 90) * 2.0F - 180.0F});
        }
        return inputs;
    }

    // Every rotation of every whole step of a turn of 4 (a block's facing) and 16 (a sign's), and of none.
    private static List<Object[]> rotationSteps() {
        List<Object[]> inputs = new ArrayList<>();
        for (Rotation rotation : Rotation.values()) {
            for (int steps : new int[]{4, 16, 0}) {
                for (int index = -1; index <= steps; index++) {
                    inputs.add(new Object[]{rotation, index, steps});
                }
            }
        }
        return inputs;
    }

    // Every block whose coordinates are at BlockPos.asLong's packing limits (26 bits for x and z, 12 for y), or an int
    // past them, which wraps, or near zero.
    private static List<Object[]> packingLimits() {
        int[] horizontal = {-33554433, -33554432, -33554431, -1, 0, 1, 33554431, 33554432};
        int[] vertical = {-2049, -2048, -2047, -64, -1, 0, 1, 319, 2047, 2048};
        List<Object[]> inputs = new ArrayList<>();
        for (int x : horizontal) {
            for (int y : vertical) {
                for (int z : horizontal) {
                    inputs.add(new Object[]{x, y, z});
                }
            }
        }
        return inputs;
    }

    // Blocks packed with BlockPos.asLong: at the packing limits, around the walk scenario's spawn and anywhere a BlockPos
    // packs, and any longs. Kind 0 passes the long alone, 1 with each direction, 2 with steps, and 3 to a
    // MutableBlockPos's set.
    private static List<Object[]> packedInputs(final int kind) {
        Random random = new Random(30 + kind);
        List<Long> packed = new ArrayList<>();
        for (Object[] limits : packingLimits()) {
            packed.add(BlockPos.asLong((Integer)limits[0], (Integer)limits[1], (Integer)limits[2]));
        }
        for (int i = 0; i < 300; i++) {
            packed.add(CoreValues.randomBlockPos(random).asLong());
        }
        for (int i = 0; i < 60; i++) {
            packed.add(random.nextLong());
        }

        List<Object[]> inputs = new ArrayList<>();
        for (long node : packed) {
            switch (kind) {
                case 0 -> inputs.add(new Object[]{node});
                case 1 -> {
                    for (Direction direction : Direction.values()) {
                        inputs.add(new Object[]{node, direction});
                    }
                }
                case 2 -> inputs.add(new Object[]{node, random.nextInt(5) - 2, random.nextInt(5) - 2, random.nextInt(5) - 2});
                default -> inputs.add(new Object[]{CoreValues.randomBlockPos(random).mutable(), node});
            }
        }
        return inputs;
    }

    // Points on the bounds clampLocationWithin works out for a block, an ulp either side, inside and outside, for blocks
    // around the walk scenario's spawn and far out, where the lower bound's float sum rounds (and past 2^24, the block's
    // coordinate too).
    private static List<Object[]> clampBounds() {
        Random random = new Random(33);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 2000; i++) {
            BlockPos block = CoreValues.randomBlockPos(random);
            double[] point = new double[3];
            for (int axis = 0; axis < 3; axis++) {
                int coordinate = block.get(Direction.Axis.VALUES[axis]);
                double min = coordinate + 1.0E-5F;
                double max = coordinate + 1.0 - 1.0E-5F;
                point[axis] = switch (random.nextInt(9)) {
                    case 0 -> min;
                    case 1 -> Math.nextDown(min);
                    case 2 -> Math.nextUp(min);
                    case 3 -> max;
                    case 4 -> Math.nextDown(max);
                    case 5 -> Math.nextUp(max);
                    case 6 -> coordinate + 0.5;
                    case 7 -> coordinate - 3.0;
                    default -> coordinate + 3.0;
                };
            }
            inputs.add(new Object[]{block, new Vec3(point[0], point[1], point[2])});
        }
        return inputs;
    }

    // Reaches like the game's (up to 4 blocks each way, or 8 as one reach), and negative ones, with which the search
    // starts outside the reach or ends at once.
    private static List<Object[]> manhattanReaches(final int reaches) {
        Random random = new Random(34 + reaches);
        List<Object[]> inputs = new ArrayList<>();
        if (reaches == 1) {
            for (int reach = -2; reach <= 8; reach++) {
                for (int i = 0; i < 3; i++) {
                    inputs.add(new Object[]{CoreValues.randomBlockPos(random), reach});
                }
            }
            return inputs;
        }
        for (int x = -2; x <= 4; x++) {
            for (int y = -2; y <= 4; y++) {
                for (int z = -2; z <= 4; z++) {
                    inputs.add(new Object[]{CoreValues.randomBlockPos(random), x, y, z});
                }
            }
        }
        return inputs;
    }

    // The search visits at least one block per depth when every reach is non-negative, so it gets to the 512th block
    // quickly. Otherwise it may find none, and only stops past maxDepth, so that has to be small.
    private static boolean manhattanEnds(final Object[] args, final int maxDepth) {
        return (Integer)args[1] >= 0 && (Integer)args[2] >= 0 && (Integer)args[3] >= 0 || maxDepth <= 64;
    }

    // Boxes up to 9 blocks across around the walk scenario's spawn and anywhere, with their corners either way round (as
    // ints, a min above the max makes the width 0, which leaves nothing to visit, or negative), or as two BlockPos.
    private static List<Object[]> smallBoxes(final boolean blockPositions) {
        Random random = new Random(blockPositions ? 36 : 37);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 300; i++) {
            BlockPos a = CoreValues.randomBlockPos(random);
            BlockPos b = a.offset(random.nextInt(11) - 2, random.nextInt(11) - 2, random.nextInt(11) - 2);
            if (random.nextBoolean()) {
                BlockPos swap = a;
                a = b;
                b = swap;
            }
            inputs.add(blockPositions ? new Object[]{a, b} : new Object[]{a.getX(), a.getY(), a.getZ(), b.getX(), b.getY(), b.getZ()});
        }
        return inputs;
    }

    // Columns from a block up or down to a y up to 16 away, or at it.
    private static List<Object[]> neighborColumns() {
        Random random = new Random(38);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 300; i++) {
            BlockPos start = CoreValues.randomBlockPos(random);
            inputs.add(new Object[]{start.getX(), start.getY(), start.getZ(), start.getY() + random.nextInt(33) - 16});
        }
        return inputs;
    }

    // Every pair of directions (those on one axis throw) with radii up to 3, and none.
    private static List<Object[]> spirals() {
        Random random = new Random(39);
        List<Object[]> inputs = new ArrayList<>();
        for (Direction first : Direction.values()) {
            for (Direction second : Direction.values()) {
                for (int radius = -1; radius <= 3; radius++) {
                    inputs.add(new Object[]{CoreValues.randomBlockPos(random), radius, first, second});
                }
            }
        }
        return inputs;
    }

    // Boxes up to 5 blocks across with their corners either way round, walked along movements like an entity's, ones with
    // zero (of either sign) or NaN components, which pick the order and the start, and ties for axisStepOrder.
    private static List<Object[]> cornerWalks(final boolean blockPositions) {
        Random random = new Random(blockPositions ? 40 : 41);
        double[] components = {0.0, -0.0, 1.0, -1.0, 0.5, -0.5, Double.NaN};
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 500; i++) {
            BlockPos a = CoreValues.randomBlockPos(random);
            BlockPos b = a.offset(random.nextInt(9) - 4, random.nextInt(9) - 4, random.nextInt(9) - 4);
            Vec3 direction = switch (random.nextInt(3)) {
                case 0 -> CoreValues.randomVec3(random);
                case 1 -> new Vec3(components[random.nextInt(7)], components[random.nextInt(7)], components[random.nextInt(7)]);
                default -> {
                    double tie = random.nextDouble() - 0.5;
                    yield new Vec3(tie, random.nextDouble() - 0.5, random.nextBoolean() ? tie : -tie);
                }
            };
            inputs.add(
                blockPositions ? new Object[]{a, b, direction} : new Object[]{a.getX(), a.getY(), a.getZ(), b.getX(), b.getY(), b.getZ(), direction}
            );
        }
        return inputs;
    }

    // Draws up to 20 blocks from boxes up to 5 blocks across, or from a cube up to 4 blocks either way of a center, some
    // empty (a width of 0, which nextInt rejects, but only once a block is drawn).
    private static List<Object[]> randomBoxes(final boolean cube) {
        Random random = new Random(cube ? 42 : 43);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 300; i++) {
            long seed = random.nextLong();
            int limit = random.nextInt(22) - 1;
            BlockPos a = CoreValues.randomBlockPos(random);
            if (cube) {
                inputs.add(new Object[]{seed, limit, a, random.nextInt(6) - 1});
            } else {
                BlockPos b = a.offset(random.nextInt(6) - 1, random.nextInt(6) - 1, random.nextInt(6) - 1);
                inputs.add(new Object[]{seed, limit, a.getX(), a.getY(), a.getZ(), b.getX(), b.getY(), b.getZ()});
            }
        }
        return inputs;
    }

    // Each box against a copy of itself, and against a copy with one side an ulp away, or a zero or NaN of the other sign:
    // equals compares each side with Double.compare, which tells those apart (and any NaN from a number).
    private static List<Object[]> boxEqualsInputs() {
        Random random = new Random(50);
        List<Object> edges = CoreValues.BOX.edges();
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 400; i++) {
            AABB box = i < edges.size() ? (AABB)edges.get(i) : CoreValues.randomBox(random);
            inputs.add(new Object[]{box, box});
            double[] sides = sides(box);
            int side = random.nextInt(6);
            double value = sides[side];
            if (Double.isNaN(value)) {
                sides[side] = Double.doubleToRawLongBits(value) < 0 ? Double.NaN : Double.longBitsToDouble(0xfff8000000000000L);
            } else if (value == 0.0) {
                sides[side] = -value;
            } else {
                sides[side] = random.nextBoolean() ? Math.nextUp(value) : Math.nextDown(value);
            }
            inputs.add(new Object[]{box, new AABB(sides[0], sides[1], sides[2], sides[3], sides[4], sides[5])});
        }
        return inputs;
    }

    // Blocks whose coordinates are at the int limits, where the far side's int sum wraps, or near them or zero.
    private static List<Object[]> limitBlockPairs() {
        Random random = new Random(63);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 300; i++) {
            BlockPos a = limitBlock(random);
            inputs.add(new Object[]{a, random.nextBoolean() ? a : limitBlock(random)});
        }
        return inputs;
    }

    // Moves with zeros of both signs, NaN and small steps, on boxes with -0.0 and 0.0 sides, where whether a zero counts as
    // moving decides the sign of the side it moves.
    private static List<Object[]> signedZeroMoves(final boolean vec3) {
        double[] components = {0.0, -0.0, Double.NaN, 1.0, -1.0, 1.0E-300};
        List<AABB> boxes = List.of(
            new AABB(-0.0, -0.0, -0.0, 0.0, 0.0, 0.0), new AABB(-0.0, -0.0, -0.0, -0.0, -0.0, -0.0), new AABB(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            new AABB(-1.0, -0.0, 0.0, -0.0, 0.0, 1.0)
        );
        List<Object[]> inputs = new ArrayList<>();
        for (AABB box : boxes) {
            for (double x : components) {
                for (double y : components) {
                    for (double z : components) {
                        inputs.add(vec3 ? new Object[]{box, new Vec3(x, y, z)} : new Object[]{box, x, y, z});
                    }
                }
            }
        }
        return inputs;
    }

    // Boxes like the game's whose sides meet along an axis, overlap by an ulp, or are an ulp apart, and along another axis
    // too now and then: an entity on a block, against a wall or under a ceiling, and boxes side by side. Kind 0 passes
    // the other box (either one may be the receiver), 1 its sides, 2 two of its corners (either way round along each
    // axis), and 3 the block.
    private static List<Object[]> touchingBoxes(final int kind) {
        Random random = new Random(51 + kind);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 600; i++) {
            BlockPos pos = switch (random.nextInt(4)) {
                case 0 -> limitBlock(random);
                case 1 -> CoreValues.randomBlockPos(random);
                default -> nearSpawn(random);
            };
            AABB other = kind == 3 || random.nextBoolean() ? new AABB(pos) : CoreValues.randomBox(random);
            Vec3 center = other.getCenter();
            Vec3 feet = new Vec3(center.x + spread(random), other.minY + spread(random), center.z + spread(random));
            AABB box = CoreValues.randomEntityBox(random, feet);
            int axis = random.nextInt(3);
            box = meeting(box, other, axis, random);
            if (random.nextInt(3) == 0) {
                box = meeting(box, other, (axis + 1 + random.nextInt(2)) % 3, random);
            }

            double[] sides = sides(other);
            inputs.add(switch (kind) {
                case 0 -> random.nextBoolean() ? new Object[]{box, other} : new Object[]{other, box};
                case 1 -> new Object[]{box, sides[0], sides[1], sides[2], sides[3], sides[4], sides[5]};
                case 2 -> {
                    double[] a = new double[3];
                    double[] b = new double[3];
                    for (int j = 0; j < 3; j++) {
                        boolean swap = random.nextBoolean();
                        a[j] = swap ? sides[j + 3] : sides[j];
                        b[j] = swap ? sides[j] : sides[j + 3];
                    }
                    yield new Object[]{box, new Vec3(a[0], a[1], a[2]), new Vec3(b[0], b[1], b[2])};
                }
                default -> new Object[]{box, pos};
            });
        }
        return inputs;
    }

    // The box moved along the axis so one of its sides is on the other's opposite side, or an ulp either way. Its size
    // along the axis is kept as nearly as doubles allow.
    private static AABB meeting(final AABB box, final AABB other, final int axis, final Random random) {
        double[] sides = sides(box);
        double[] others = sides(other);
        double size = sides[axis + 3] - sides[axis];
        int ulps = random.nextInt(3) - 1;
        if (random.nextBoolean()) {
            sides[axis] = ulps(others[axis + 3], ulps);
            sides[axis + 3] = sides[axis] + size;
        } else {
            sides[axis + 3] = ulps(others[axis], ulps);
            sides[axis] = sides[axis + 3] - size;
        }
        return new AABB(sides[0], sides[1], sides[2], sides[3], sides[4], sides[5]);
    }

    // Points on a box's sides (contains takes its min sides in and its max sides out), an ulp either side, in the middle and
    // outside, for any box. Kind 0 passes a Vec3, 1 the coordinates.
    private static List<Object[]> facePoints(final int kind) {
        Random random = new Random(55 + kind);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 600; i++) {
            AABB box = CoreValues.randomBox(random);
            double[] sides = sides(box);
            double[] point = new double[3];
            for (int axis = 0; axis < 3; axis++) {
                double min = sides[axis];
                double max = sides[axis + 3];
                point[axis] = switch (random.nextInt(8)) {
                    case 0 -> min;
                    case 1 -> ulps(min, random.nextBoolean() ? 1 : -1);
                    case 2 -> max;
                    case 3 -> ulps(max, random.nextBoolean() ? 1 : -1);
                    case 4 -> min - 1.0;
                    case 5 -> max + 1.0;
                    default -> Mth.lerp(random.nextDouble(), min, max);
                };
            }
            inputs.add(kind == 0 ? new Object[]{box, new Vec3(point[0], point[1], point[2])} : new Object[]{box, point[0], point[1], point[2]});
        }
        return inputs;
    }

    // Rays like the game's picks and projectiles: from near a block along a look vector, as far as the block interaction
    // range (4.5) or the entity one (3) or 20 blocks, aimed at a box's center, a point inside it, a corner, an edge or a
    // side, or at a corner along a diagonal, where two sides are hit at the same scale. Also rays along an axis that pass a
    // side at clipPoint's 1.0E-7 margin, or an ulp either side of it, rays that start inside a box or on a side, end on a
    // side or move less than the 1.0E-7 that counts as moving along an axis, and rays of length 0. The boxes are a block's
    // shapes at the block. Kind 0 passes the box, 1 its sides, 2 the block's shapes relative to it and the block, and 3 a
    // ray's start and direction (Mth.rayIntersectsAABB).
    private static List<Object[]> clipRays(final int kind) {
        Random random = new Random(57 + kind);
        List<Object> shapes = CoreValues.BOX_LIST.edges();
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 2000; i++) {
            BlockPos pos = random.nextInt(4) == 0 ? CoreValues.randomBlockPos(random) : nearSpawn(random);
            @SuppressWarnings("unchecked")
            List<AABB> shape = switch (random.nextInt(3)) {
                case 0 -> (List<AABB>)shapes.get(1 + random.nextInt(shapes.size() - 1));
                case 1 -> List.of(CoreValues.blockShape(random));
                default -> List.of(CoreValues.blockShape(random), CoreValues.blockShape(random));
            };
            AABB box = shape.get(random.nextInt(shape.size())).move(pos);
            Vec3[] ray = ray(box, random);
            if (random.nextInt(8) == 0) {
                Object[] threshold = thresholdRay(random);
                pos = (BlockPos)threshold[0];
                shape = List.of(new AABB(0.0, 0.0, 0.0, 1.0, 1.0, 1.0));
                box = new AABB(pos);
                ray = new Vec3[]{(Vec3)threshold[1], (Vec3)threshold[2]};
            }
            double[] sides = sides(box);
            inputs.add(switch (kind) {
                case 0 -> new Object[]{box, ray[0], ray[1]};
                case 1 -> new Object[]{sides[0], sides[1], sides[2], sides[3], sides[4], sides[5], ray[0], ray[1]};
                case 2 -> new Object[]{random.nextInt(50) == 0 ? List.of() : shape, ray[0], ray[1], pos};
                default -> new Object[]{ray[0], ray[1].subtract(ray[0]), box};
            });
        }

        // Rays along an axis that pass one of the other axes' sides at clipPoint's 1.0E-7 margin, give or take two ulps,
        // where the order the margin is added in decides whether they hit.
        Random margins = new Random(70 + kind);
        for (int i = 0; i < 1000; i++) {
            BlockPos pos = margins.nextInt(4) == 0 ? CoreValues.randomBlockPos(margins) : nearSpawn(margins);
            AABB shape = CoreValues.blockShape(margins);
            AABB box = shape.move(pos);
            double[] sides = sides(box);
            int axis = margins.nextInt(3);
            int edge = (axis + 1 + margins.nextInt(2)) % 3;
            double[] point = new double[3];
            for (int other = 0; other < 3; other++) {
                point[other] = Mth.lerp(margins.nextDouble(), sides[other], sides[other + 3]);
            }
            point[edge] = margins.nextBoolean() ? ulps(sides[edge] - 1.0E-7, margins.nextInt(5) - 2) : ulps(sides[edge + 3] + 1.0E-7, margins.nextInt(5) - 2);
            double sign = margins.nextBoolean() ? 1.0 : -1.0;
            point[axis] = sign > 0.0 ? sides[axis] - 0.5 : sides[axis + 3] + 0.5;
            Vec3 from = new Vec3(point[0], point[1], point[2]);
            Vec3 to = from.with(Direction.Axis.VALUES[axis], point[axis] + sign * 2.0);
            inputs.add(switch (kind) {
                case 0 -> new Object[]{box, from, to};
                case 1 -> new Object[]{sides[0], sides[1], sides[2], sides[3], sides[4], sides[5], from, to};
                case 2 -> new Object[]{List.of(shape), from, to, pos};
                default -> new Object[]{from, to.subtract(from), box};
            });
        }
        return inputs;
    }

    // A ray that moves 1.0E-7 along an axis, an ulp more or an ulp less, through a whole block's side at 0, so it counts as
    // moving along the axis or doesn't: it starts and ends half of that either side of the side, which doubles hold exactly
    // near 0. Along the other axes it stays over the side, and now and then starts at -0.0, where the end's sign decides the
    // sign of the hit. Returns the block, the ray's start and its end.
    private static Object[] thresholdRay(final Random random) {
        int axis = random.nextInt(3);
        boolean positive = random.nextBoolean();
        double[] moves = {1.0E-7, Math.nextUp(1.0E-7), Math.nextDown(1.0E-7)};
        double half = moves[random.nextInt(moves.length)] / 2.0;
        BlockPos near = nearSpawn(random);
        int[] block = {near.getX(), near.getY(), near.getZ()};
        double[] from = new double[3];
        double[] to = new double[3];
        for (int other = 0; other < 3; other++) {
            if (other == axis) {
                block[other] = positive ? 0 : -1;
                from[other] = positive ? -half : half;
                to[other] = -from[other];
            } else if (random.nextInt(3) == 0) {
                double[] ends = {-0.0, 0.0, -Double.MIN_VALUE, Double.MIN_VALUE};
                block[other] = random.nextBoolean() ? 0 : -1;
                from[other] = -0.0;
                to[other] = ends[random.nextInt(ends.length)];
            } else {
                from[other] = block[other] + random.nextDouble();
                to[other] = random.nextBoolean() ? from[other] : from[other] + (random.nextDouble() - 0.5) * 0.5;
            }
        }
        return new Object[]{new BlockPos(block[0], block[1], block[2]), new Vec3(from[0], from[1], from[2]), new Vec3(to[0], to[1], to[2])};
    }

    // A ray's start and end around a box.
    private static Vec3[] ray(final AABB box, final Random random) {
        double[] sides = sides(box);
        double[] reaches = {4.5, 3.0, 20.0, 1.0};
        double reach = reaches[random.nextInt(reaches.length)];
        Vec3 look = Vec3.directionFromRotation((random.nextFloat() * 2.0F - 1.0F) * 90.0F, (random.nextFloat() * 2.0F - 1.0F) * 180.0F);
        switch (random.nextInt(8)) {
            case 0, 1 -> {
                Vec3 aim = aim(sides, random);
                Vec3 from = aim.subtract(look.scale(random.nextDouble() * reach));
                return new Vec3[]{from, from.add(look.scale(reach))};
            }
            case 2 -> {
                // At a corner along a diagonal, from a whole number of blocks away, so both sides' scales come out the same.
                double[] corner = new double[3];
                double[] diagonal = new double[3];
                for (int axis = 0; axis < 3; axis++) {
                    boolean max = random.nextBoolean();
                    corner[axis] = max ? sides[axis + 3] : sides[axis];
                    diagonal[axis] = axis == 2 && random.nextBoolean() ? 0.0 : max ? -1.0 : 1.0;
                }
                double distance = 1 + random.nextInt(3);
                Vec3 direction = new Vec3(diagonal[0], diagonal[1], diagonal[2]);
                Vec3 from = new Vec3(corner[0], corner[1], corner[2]).subtract(direction.scale(distance));
                return new Vec3[]{from, from.add(direction.scale(distance * 2.0))};
            }
            case 3 -> {
                // Along an axis, passing the other axes' sides at the margin, or an ulp either side of it.
                int axis = random.nextInt(3);
                double[] point = new double[3];
                for (int other = 0; other < 3; other++) {
                    point[other] = switch (random.nextInt(4)) {
                        case 0 -> ulps(sides[other] - 1.0E-7, random.nextInt(3) - 1);
                        case 1 -> ulps(sides[other + 3] + 1.0E-7, random.nextInt(3) - 1);
                        case 2 -> random.nextBoolean() ? sides[other] : sides[other + 3];
                        default -> Mth.lerp(random.nextDouble(), sides[other], sides[other + 3]);
                    };
                }
                double sign = random.nextBoolean() ? 1.0 : -1.0;
                point[axis] = sign > 0.0 ? sides[axis] - 1.0 - random.nextDouble() : sides[axis + 3] + 1.0 + random.nextDouble();
                Vec3 from = new Vec3(point[0], point[1], point[2]);
                return new Vec3[]{from, from.with(Direction.Axis.VALUES[axis], from.get(Direction.Axis.VALUES[axis]) + sign * reach)};
            }
            case 4 -> {
                // From inside, or from a side, into the box or out of it.
                Vec3 from = random.nextBoolean() ? aim(sides, random) : new Vec3(sides[0], Mth.lerp(0.5, sides[1], sides[4]), Mth.lerp(0.5, sides[2], sides[5]));
                return new Vec3[]{from, from.add(random.nextBoolean() ? look.scale(reach) : new Vec3(1.0, 0.0, 0.0))};
            }
            case 5 -> {
                // Ending on a side.
                Vec3 to = new Vec3(sides[0], Mth.lerp(random.nextDouble(), sides[1], sides[4]), Mth.lerp(random.nextDouble(), sides[2], sides[5]));
                return new Vec3[]{to.subtract(random.nextBoolean() ? new Vec3(1.0, 0.0, 0.0) : look.scale(reach)), to};
            }
            case 6 -> {
                // Moving about 1.0E-7 or less along some axes.
                double[] tiny = {1.0E-7, -1.0E-7, Math.nextUp(1.0E-7), -Math.nextUp(1.0E-7), Math.nextDown(1.0E-7), 0.0, -0.0};
                Vec3 from = aim(sides, random).subtract(look);
                double[] delta = {look.x * 2.0, look.y * 2.0, look.z * 2.0};
                for (int axis = 0; axis < 3; axis++) {
                    if (random.nextBoolean()) {
                        delta[axis] = tiny[random.nextInt(tiny.length)];
                    }
                }
                return new Vec3[]{from, from.add(delta[0], delta[1], delta[2])};
            }
            default -> {
                Vec3 at = random.nextBoolean() ? aim(sides, random) : CoreValues.randomVec3(random);
                return new Vec3[]{at, random.nextBoolean() ? at : CoreValues.randomVec3(random)};
            }
        }
    }

    // The box's center, a point inside it, one of its corners, a point on an edge, or a point on a side.
    private static Vec3 aim(final double[] sides, final Random random) {
        int kind = random.nextInt(5);
        double[] point = new double[3];
        int free = random.nextInt(3);
        for (int axis = 0; axis < 3; axis++) {
            double min = sides[axis];
            double max = sides[axis + 3];
            boolean onSide = kind == 2 || kind == 3 && axis != free || kind == 4 && axis == free;
            point[axis] = kind == 0 ? Mth.lerp(0.5, min, max) : onSide ? random.nextBoolean() ? min : max : Mth.lerp(random.nextDouble(), min, max);
        }
        return new Vec3(point[0], point[1], point[2]);
    }

    // Entities moving like the game's (walking, falling, a projectile's step, toward a box, or about 1.0E-7 along an axis)
    // past a fluid's box (a block with its fluid's height, the only list the game passes) or a block's shapes, starting or
    // ending inside one of them inflated by half the entity, passing through it, or missing it, and entities against one.
    private static List<Object[]> collisionPaths() {
        Random random = new Random(61);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 600; i++) {
            BlockPos pos = random.nextInt(4) == 0 ? CoreValues.randomBlockPos(random) : nearSpawn(random);
            List<AABB> boxes = new ArrayList<>();
            if (random.nextBoolean()) {
                boxes.add(new AABB(pos.getX(), pos.getY(), pos.getZ(), pos.getX() + 1, pos.getY() + (1 + random.nextInt(9)) / 9.0F, pos.getZ() + 1));
            } else {
                for (int count = 1 + random.nextInt(3); count > 0; count--) {
                    boxes.add(CoreValues.blockShape(random).move(pos));
                }
            }
            Vec3 feet = Vec3.atBottomCenterOf(pos).add(spread(random), spread(random), spread(random));
            AABB box = CoreValues.randomEntityBox(random, feet);
            AABB target = boxes.get(random.nextInt(boxes.size()));
            // Against the box, where its center is on the side of the box inflated by half of it, give or take 1.0E-7.
            if (random.nextInt(3) == 0) {
                box = meeting(box, target, random.nextInt(3), random);
            }
            double[] tiny = {1.0E-7, Math.nextDown(1.0E-7), 5.0E-8, 2.0E-7};
            Vec3 vector = switch (random.nextInt(6)) {
                case 0 -> new Vec3((random.nextDouble() * 2.0 - 1.0) * 0.3, random.nextBoolean() ? 0.0 : -0.0784000015258789, (random.nextDouble() * 2.0 - 1.0) * 0.3);
                case 1 -> new Vec3(0.0, -random.nextDouble() * 3.92, 0.0);
                case 2 -> Vec3.directionFromRotation((random.nextFloat() * 2.0F - 1.0F) * 90.0F, (random.nextFloat() * 2.0F - 1.0F) * 180.0F).scale(3.0);
                case 3 -> target.getCenter().subtract(box.getCenter()).scale(random.nextDouble() * 2.0);
                case 4 -> Vec3.ZERO.with(Direction.Axis.VALUES[random.nextInt(3)], tiny[random.nextInt(tiny.length)] * (random.nextBoolean() ? 1.0 : -1.0));
                default -> Vec3.ZERO;
            };
            inputs.add(new Object[]{box, vector, List.copyOf(boxes)});
        }

        // Moves from outside onto the min side of the box inflated by half the entity, which contains takes in and clip
        // doesn't (the move ends there, at a scale of 1). The center and the side are in the same binade, so the move
        // adds back to the side exactly.
        while (inputs.size() < 800) {
            BlockPos pos = nearSpawn(random);
            AABB target = random.nextBoolean() ? new AABB(pos) : CoreValues.blockShape(random).move(pos);
            Direction.Axis axis = Direction.Axis.VALUES[random.nextInt(3)];
            AABB box = CoreValues.randomEntityBox(random, Vec3.atBottomCenterOf(pos).add(spread(random), spread(random), spread(random)));
            AABB inflated = target.inflate(box.getXsize() * 0.5 - 1.0E-7, box.getYsize() * 0.5 - 1.0E-7, box.getZsize() * 0.5 - 1.0E-7);
            Vec3 center = inflated.getCenter();
            box = box.move(center.subtract(box.getCenter())).move(Vec3.ZERO.with(axis, -(inflated.max(axis) - inflated.min(axis)) * 0.5 - random.nextDouble() * 0.5));
            Vec3 from = box.getCenter();
            Vec3 vector = Vec3.ZERO.with(axis, inflated.min(axis) - from.get(axis));
            if (!inflated.contains(from) && from.add(vector).get(axis) == inflated.min(axis) && inflated.contains(from.add(vector))) {
                inputs.add(new Object[]{box, vector, List.of(target)});
            }
        }
        return inputs;
    }

    // Boxes up to 5 blocks across whose sides are on blocks' faces, an ulp or 1.0E-7 inside or outside them, or anywhere in
    // a block, like entities' boxes and the ones nextDeflated and deflate give. Kind 0 passes the box, 1 also a direction
    // to walk it in, like cornerWalks'.
    private static List<Object[]> blockAlignedBoxes(final int kind) {
        Random random = new Random(62 + kind);
        double[] components = {0.0, -0.0, 1.0, -1.0, 0.5, -0.5, Double.NaN};
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 500; i++) {
            BlockPos pos = random.nextBoolean() ? nearSpawn(random) : CoreValues.randomBlockPos(random);
            double[] sides = new double[6];
            for (int side = 0; side < 6; side++) {
                double face = pos.get(Direction.Axis.VALUES[side % 3]) + (side < 3 ? 0.0 : random.nextInt(6));
                sides[side] = switch (random.nextInt(6)) {
                    case 0 -> face + random.nextDouble();
                    case 1 -> face + 1.0E-7;
                    case 2 -> face - 1.0E-7;
                    case 3 -> ulps(face, random.nextBoolean() ? 1 : -1);
                    default -> face;
                };
            }
            AABB box = new AABB(sides[0], sides[1], sides[2], sides[3], sides[4], sides[5]);
            Vec3 direction = random.nextBoolean()
                ? CoreValues.randomVec3(random)
                : new Vec3(components[random.nextInt(7)], components[random.nextInt(7)], components[random.nextInt(7)]);
            inputs.add(kind == 0 ? new Object[]{box} : new Object[]{box, direction});
        }
        return inputs;
    }

    // A block with each coordinate at an int limit, next to one, or near zero.
    private static BlockPos limitBlock(final Random random) {
        int[] coordinates = {Integer.MAX_VALUE, Integer.MAX_VALUE - 1, Integer.MIN_VALUE, 0, -1};
        return new BlockPos(
            coordinates[random.nextInt(coordinates.length)], coordinates[random.nextInt(coordinates.length)], coordinates[random.nextInt(coordinates.length)]
        );
    }

    // A block within the walk scenario's render distance of its spawn, at a height near the ground.
    private static BlockPos nearSpawn(final Random random) {
        return new BlockPos(random.nextInt(129) - 64, random.nextInt(17) - 68, random.nextInt(129) - 64);
    }

    // Up to a block and a half either way, in sixteenths now and then, so boxes overlap, meet or miss.
    private static double spread(final Random random) {
        return random.nextBoolean() ? (random.nextInt(49) - 24) / 16.0 : (random.nextDouble() * 2.0 - 1.0) * 1.5;
    }

    private static double[] sides(final AABB box) {
        return new double[]{box.minX, box.minY, box.minZ, box.maxX, box.maxY, box.maxZ};
    }

    // The value moved by that many ulps, up or down.
    private static double ulps(final double value, final int ulps) {
        double moved = value;
        for (int step = ulps; step != 0; step -= Integer.signum(step)) {
            moved = step > 0 ? Math.nextUp(moved) : Math.nextDown(moved);
        }
        return moved;
    }

    private static double sign(final Random random) {
        return random.nextBoolean() ? 1.0 : -1.0;
    }

    // 1 to 2 times a power of two from 2^low to 2^high. Below 2^-1022 it rounds to a subnormal.
    private static double magnitude(final Random random, final int low, final int high) {
        return Math.scalb(1.0 + random.nextDouble(), random.nextInt(high - low + 1) + low);
    }

    // 1 plus or minus up to 2^-k, for k from 0 to 52.
    private static double nearOne(final Random random) {
        return 1.0 + (random.nextDouble() * 2.0 - 1.0) * Math.scalb(1.0, -random.nextInt(53));
    }

    // NaN payloads never change game behaviour, so a random NaN becomes the canonical one, like Values does.
    private static double randomBits(final Random random) {
        double value = Double.longBitsToDouble(random.nextLong());
        return Double.isNaN(value) ? Double.NaN : value;
    }
}
