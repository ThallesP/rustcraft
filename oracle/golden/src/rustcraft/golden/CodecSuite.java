package rustcraft.golden;

import java.io.IOException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.function.Predicate;

/**
 * Covers a class whose members hand out codecs and functions rather than values (ByteBufCodecs, StreamCodec, ByIdMap).
 * Its constants and codec fields are written like StaticSuite.writeFields does, and a static method that makes a codec
 * or IntFunction from arguments that can be generated gets codec or apply cases (see Codecs), with those arguments first.
 * A method taking codecs or functions gets cases for the instances listed with codec() or function(), each named by the
 * method's descriptor and a label for what it was given: {@code optional(…)Lnet/minecraft/network/codec/StreamCodec;[VAR_INT]}.
 * Other methods are checked like a StaticSuite's, or skipped.
 */
final class CodecSuite implements Suite {
    private final Class<?> type;
    private final Map<String, List<Instance>> instances = new LinkedHashMap<>();
    private final Map<String, Predicate<Object[]>> decodeDomains = new HashMap<>();
    private final Map<String, List<Object[]>> extraInputs = new HashMap<>();

    // A codec (codec != null) or IntFunction (functionResult != null) a method made from what the label names.
    private record Instance(
        String label, Values.Generator[] leading, Codecs.Factory factory, Codecs.Codec codec, Class<?> functionResult
    ) {
    }

    CodecSuite(final Class<?> type) {
        this.type = type;
    }

    /** A codec the method made from what the label names, and from generated leading arguments, if any. */
    CodecSuite codec(
        final String method, final String label, final Codecs.Factory factory, final Codecs.Codec codec, final Values.Generator... leading
    ) {
        this.instances.computeIfAbsent(method, m -> new ArrayList<>()).add(new Instance(label, leading, factory, codec, null));
        return this;
    }

    /** An IntFunction the method made from what the label names, and from generated leading arguments, if any. */
    CodecSuite function(
        final String method, final String label, final Codecs.Factory factory, final Class<?> valueType, final Values.Generator... leading
    ) {
        this.instances.computeIfAbsent(method, m -> new ArrayList<>()).add(new Instance(label, leading, factory, null, valueType));
        return this;
    }

    /**
     * More buffers for a codec constant N to decode under {@code N.decode}, like InstanceSuite.extraInputs, for fields
     * its writer refuses but its reader doesn't.
     */
    CodecSuite extraInputs(final String member, final List<Object[]> inputs) {
        this.extraInputs.put(member, inputs);
        return this;
    }

    /**
     * Drops decode inputs, each the leading arguments and the buffer, like StaticSuite.where: a method's by its
     * descriptor, and a codec constant N's under {@code N.decode}.
     */
    CodecSuite whereDecode(final String method, final Predicate<Object[]> domain) {
        this.decodeDomains.put(method, domain);
        return this;
    }

    @Override
    public String name() {
        return this.type.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Set<String> written = new HashSet<>(StaticSuite.writeFields(this.type, writer, this.extraInputs, this.decodeDomains));

        List<String> plain = new ArrayList<>();
        for (Method method : this.methods()) {
            String descriptor = Descriptors.of(method);
            written.add(descriptor);
            Predicate<Object[]> domain = this.decodeDomains.getOrDefault(descriptor, spec -> true);
            List<Instance> listed = this.instances.get(descriptor);
            if (listed != null) {
                for (Instance instance : listed) {
                    String name = descriptor + "[" + instance.label() + "]";
                    if (instance.codec() != null) {
                        Codecs.writeCodec(writer, name, instance.leading(), instance.factory(), instance.codec(), domain);
                    } else {
                        Codecs.writeFunction(writer, name, instance.leading(), instance.factory(), instance.functionResult());
                    }
                }
                continue;
            }

            boolean isStatic = Modifier.isStatic(method.getModifiers());
            Values.Generator[] leading = Arrays.stream(method.getParameterTypes()).map(Values::generator).toArray(Values.Generator[]::new);
            boolean generated = Arrays.stream(leading).allMatch(generator -> generator != null);
            Codecs.Codec codec = Codecs.codecOf(method.getGenericReturnType());
            Class<?> function = Codecs.functionOf(method.getGenericReturnType());
            String reason = StaticSuite.unsupportedReason(method);
            if (isStatic && generated && codec != null) {
                Codecs.writeCodec(writer, descriptor, leading, args -> method.invoke(null, args), codec, domain);
            } else if (isStatic && generated && function != null) {
                Codecs.writeFunction(writer, descriptor, leading, args -> method.invoke(null, args), function);
            } else if (isStatic && reason == null) {
                plain.add(descriptor);
            } else if (!isStatic) {
                writer.skipped(descriptor, "no generator for " + this.type.getName());
            } else if (!generated) {
                Class<?> missing = Arrays.stream(method.getParameterTypes()).filter(parameter -> Values.generator(parameter) == null).findFirst().orElseThrow();
                writer.skipped(descriptor, "no generator for " + missing.getName());
            } else {
                String codecReason = Codecs.unsupportedReason(method.getGenericReturnType());
                writer.skipped(descriptor, codecReason != null ? codecReason : reason);
            }
        }
        if (!plain.isEmpty()) {
            new StaticSuite(this.type).only(plain.toArray(String[]::new)).write(writer);
        }

        Set<String> missing = new HashSet<>(this.instances.keySet());
        missing.addAll(this.decodeDomains.keySet());
        missing.addAll(this.extraInputs.keySet());
        missing.removeAll(written);
        if (!missing.isEmpty()) {
            throw new IllegalStateException(this.type.getName() + " has no public methods " + missing);
        }
    }

    // Public static methods, and the default methods of an interface.
    private List<Method> methods() {
        return Arrays.stream(this.type.getDeclaredMethods())
            .filter(method -> Modifier.isPublic(method.getModifiers()))
            .filter(method -> !method.isSynthetic() && !method.isBridge())
            .sorted(Comparator.comparing(Descriptors::of))
            .toList();
    }
}
