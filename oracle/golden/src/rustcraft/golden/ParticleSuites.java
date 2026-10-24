package rustcraft.golden;

import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.lang.reflect.ParameterizedType;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import net.minecraft.core.Holder;
import net.minecraft.core.particles.ParticleType;
import net.minecraft.core.particles.ParticleTypes;
import net.minecraft.core.particles.SimpleParticleType;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.sounds.SoundEvent;
import net.minecraft.sounds.SoundEvents;

/**
 * The particle types ({@code ParticleTypes}, in {@code BuiltInRegistries.PARTICLE_TYPE}'s order) and the sound events
 * ({@code SoundEvents}' fields) the level's animations make. See TESTING.md ("The level's animations").
 */
final class ParticleSuites {
    private ParticleSuites() {
    }

    static List<Suite> suites() {
        return List.of(new ParticleTypesSuite(), new SoundEventsSuite());
    }

    /**
     * {@code types}: each id → {@code [key, overrideLimiter, options]}, the options being the simple name of the class its
     * {@code ParticleTypes} field's type argument names ({@code Simple} for a {@code SimpleParticleType}).
     */
    static final class ParticleTypesSuite implements Suite {
        @Override
        public String name() {
            return ParticleTypes.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Map<ParticleType<?>, String> options = new HashMap<>();
            try {
                for (Field field : ParticleTypes.class.getFields()) {
                    if (Modifier.isStatic(field.getModifiers()) && field.get(null) instanceof ParticleType<?> type) {
                        options.put(
                            type, type instanceof SimpleParticleType
                                ? "Simple"
                                : ((Class<?>)((ParameterizedType)field.getGenericType()).getActualTypeArguments()[0]).getSimpleName()
                        );
                    }
                }
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
            for (ParticleType<?> type : BuiltInRegistries.PARTICLE_TYPE) {
                writer.result(
                    "types", ChunkSuites.array(BuiltInRegistries.PARTICLE_TYPE.getId(type)),
                    ChunkSuites.array(BuiltInRegistries.PARTICLE_TYPE.getKey(type).toString(), type.getOverrideLimiter(), options.get(type)), null
                );
            }
        }
    }

    /** Each {@code SoundEvents} field holding a sound event or its holder → {@code [location, fixedRange]} (a float's bits, or null). */
    static final class SoundEventsSuite implements Suite {
        @Override
        public String name() {
            return SoundEvents.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            try {
                for (Field field : SoundEvents.class.getFields()) {
                    if (!Modifier.isStatic(field.getModifiers())) {
                        continue;
                    }
                    SoundEvent event = switch (field.get(null)) {
                        case SoundEvent sound -> sound;
                        case Holder<?> holder when holder.value() instanceof SoundEvent sound -> sound;
                        default -> null;
                    };
                    if (event != null) {
                        Optional<Float> range = event.fixedRange();
                        writer.field(
                            field.getName(),
                            ChunkSuites.array(event.location().toString(), range.map(r -> Values.encode(r, float.class)).orElse(com.google.gson.JsonNull.INSTANCE))
                        );
                    }
                }
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }
    }
}
