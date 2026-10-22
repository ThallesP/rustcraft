package rustcraft.golden;

import java.io.IOException;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.List;
import net.minecraft.network.ProtocolInfo;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.Packet;
import net.minecraft.network.protocol.PacketType;

/**
 * Covers a class of protocols (HandshakeProtocols): its constants like a StaticSuite's, a template as its details and a
 * bound protocol as its id and flow (see PacketValues), and each bound protocol's codec, which gets codec cases named
 * {@code FIELD.codec()} (see Codecs). Their values are any of the packets the template with the protocol's id and flow
 * lists. A codec is skipped when one of those packets has no generator yet, naming its type.
 */
final class ProtocolSuite implements Suite {
    private final Class<?> type;

    ProtocolSuite(final Class<?> type) {
        this.type = type;
    }

    @Override
    public String name() {
        return this.type.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        StaticSuite.writeFields(this.type, writer);

        List<ProtocolInfo.DetailsProvider> templates = new ArrayList<>();
        List<Field> protocols = new ArrayList<>();
        for (Field field : this.constants()) {
            Object value = get(field);
            if (value instanceof ProtocolInfo.DetailsProvider template) {
                templates.add(template);
            } else if (value instanceof ProtocolInfo<?>) {
                protocols.add(field);
            }
        }

        for (Field field : protocols) {
            ProtocolInfo<?> protocol = (ProtocolInfo<?>)get(field);
            ProtocolInfo.Details details = templates.stream()
                .map(ProtocolInfo.DetailsProvider::details)
                .filter(candidate -> candidate.id() == protocol.id() && candidate.flow() == protocol.flow())
                .reduce((first, second) -> {
                    throw new IllegalStateException(this.type.getName() + " has two templates for " + field.getName());
                })
                .orElseThrow(() -> new IllegalStateException(this.type.getName() + " has no template for " + field.getName()));

            List<PacketType<?>> types = new ArrayList<>();
            details.listPackets((type, networkId) -> types.add(type));
            List<Values.Generator> packets = new ArrayList<>();
            String name = field.getName() + ".codec()";
            for (PacketType<?> type : types) {
                Values.Generator generator = PacketValues.packetsOfType(type);
                if (generator == null) {
                    writer.skipped(name, "no generator for packet " + type);
                    break;
                }
                packets.add(generator);
            }
            if (packets.size() == types.size()) {
                StreamCodec<?, ?> codec = protocol.codec();
                Codecs.writeCodec(
                    writer, name, new Values.Generator[0], leading -> codec,
                    new Codecs.Codec(NetworkValues.BUFFER, PacketValues.anyOf(packets), Packet.class), spec -> true
                );
            }
        }
    }

    // Its public constants, by name.
    private List<Field> constants() {
        return Arrays.stream(this.type.getFields())
            .filter(field -> Modifier.isStatic(field.getModifiers()) && Modifier.isFinal(field.getModifiers()))
            .filter(field -> field.getDeclaringClass() == this.type)
            .sorted(Comparator.comparing(Field::getName))
            .toList();
    }

    private static Object get(final Field field) {
        try {
            return field.get(null);
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
    }
}
