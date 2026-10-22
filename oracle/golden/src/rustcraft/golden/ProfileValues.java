package rustcraft.golden;

import com.google.common.collect.ImmutableMultimap;
import com.google.common.collect.Multimap;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.mojang.authlib.GameProfile;
import com.mojang.authlib.properties.Property;
import com.mojang.authlib.properties.PropertyMap;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Base64;
import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.UUID;
import java.util.stream.Stream;
import net.minecraft.core.UUIDUtil;

/**
 * Arguments and values of authlib's profiles. A Property is a record, {@code [name, value, signature]}, and so is a
 * GameProfile, {@code [id, name, properties]}. A PropertyMap, and the Guava Multimap it's made from, is its
 * {@code asMap()}'s entries in order, {@code [[key, [property, …]], …]}. The multimaps are built like the game builds
 * them, with {@code ImmutableMultimap.builder()}, so they're ImmutableListMultimaps: each key in the order it was first
 * put, with its values in the order they were put.
 */
final class ProfileValues {
    private static final Values.Generator STRINGS = Values.generator(String.class);
    private static final Values.Generator UUIDS = Values.generator(UUID.class);

    // The scenario's player, and around readUtf(16)'s limit.
    static final Values.Generator PLAYER_NAMES = strings(
        "rustcraft", "", "a".repeat(16), "a".repeat(17), "é".repeat(16), "😀".repeat(8), "😀".repeat(8) + "a"
    );

    // A skin's textures, the base64 of what Mojang's session server answers, and a signature of the length its 4096-bit
    // key makes.
    private static final String TEXTURES = Base64.getEncoder().encodeToString(
        (
            "{\"timestamp\":1761091200000,\"profileId\":\"" + UUIDUtil.createOfflinePlayerUUID("rustcraft").toString().replace("-", "")
                + "\",\"profileName\":\"rustcraft\",\"textures\":{\"SKIN\":{\"url\":"
                + "\"http://textures.minecraft.net/texture/31f477eb1a7beee631c2ca64d06f8f68fa93a3386d04452ab27f43acdf1b60cb\"}}}"
        ).getBytes(StandardCharsets.UTF_8)
    );
    private static final String SIGNATURE = Base64.getEncoder().encodeToString(randomBytes(512, 512));

    // The skin's name, and around the reader's limit of 64 in characters of one, two and four bytes. Random names are
    // a few shared ones half the time, so a multimap's keys repeat.
    private static final List<Object> SHARED_NAMES = List.of("textures", "a", "b");
    private static final Values.Generator NAMES = new Values.Simple(
        List.of("textures", "", "a".repeat(64), "a".repeat(65), "é".repeat(64), "😀".repeat(32), "😀".repeat(32) + "a"),
        random -> random.nextBoolean() ? SHARED_NAMES.get(random.nextInt(SHARED_NAMES.size())) : STRINGS.random(random),
        STRINGS::encode
    );
    // Textures, and around the reader's limit of 32767.
    private static final Values.Generator VALUES = strings(TEXTURES, "", "a".repeat(32767), "a".repeat(32768));
    // None a quarter of the time, a signature, and around the reader's limit of 1024.
    private static final Values.Generator SIGNATURES = Codecs.nullableOf(
        strings(SIGNATURE, "", "a".repeat(1024), "a".repeat(1025), "😀".repeat(512), "😀".repeat(512) + "a")
    );
    private static final Values.Generator PROPERTY_FIELDS = Codecs.tupleOf(NAMES, VALUES, SIGNATURES);

    static final Values.Generator PROPERTIES = new Values.Simple(
        Stream.concat(PROPERTY_FIELDS.edges().stream().map(ProfileValues::property), Stream.of(new Property("textures", TEXTURES))).toList(),
        random -> property(PROPERTY_FIELDS.random(random)),
        property -> Values.encode(property, Property.class)
    );

    // Empty, each property edge alone, two under one name either way round, names that interleave (which the multimap
    // groups) either way round, a property twice, a key that isn't its property's name, and the most a profile's codec
    // writes (16) and one more. Random ones hold up to 4 properties, now and then up to 20, mostly under their names.
    static final Values.Generator MULTIMAPS = new Values.Simple(multimapEdges(), ProfileValues::randomMultimap, ProfileValues::encodeMultimap);

    static final Values.Generator PROPERTY_MAPS = new Values.Simple(
        Stream.concat(Stream.of(PropertyMap.EMPTY), MULTIMAPS.edges().stream().map(ProfileValues::propertyMap)).toList(),
        random -> propertyMap(MULTIMAPS.random(random)),
        ProfileValues::encodeMultimap
    );

    // Each field's edges among the first edges, then the offline player's profile, which an offline server sends, and a
    // signed skin's, which an online one does.
    private static final Values.Generator PROFILE_FIELDS = Codecs.tupleOf(UUIDS, PLAYER_NAMES, PROPERTY_MAPS);
    static final Values.Generator GAME_PROFILES = new Values.Simple(
        Stream.concat(
            PROFILE_FIELDS.edges().stream().map(ProfileValues::gameProfile),
            Stream.of(
                UUIDUtil.createOfflineProfile("rustcraft"),
                new GameProfile(
                    UUIDUtil.createOfflinePlayerUUID("rustcraft"), "rustcraft",
                    new PropertyMap(ImmutableMultimap.of("textures", new Property("textures", TEXTURES, SIGNATURE)))
                )
            )
        ).toList(),
        random -> gameProfile(PROFILE_FIELDS.random(random)),
        profile -> Values.encode(profile, GameProfile.class)
    );

    private ProfileValues() {
    }

    static Values.Generator generator(final Class<?> type) {
        if (type == Property.class) {
            return PROPERTIES;
        }
        if (type == PropertyMap.class) {
            return PROPERTY_MAPS;
        }
        if (type == Multimap.class) {
            return MULTIMAPS;
        }
        if (type == GameProfile.class) {
            return GAME_PROFILES;
        }
        return null;
    }

    static boolean canEncode(final Class<?> type) {
        return Multimap.class.isAssignableFrom(type);
    }

    static JsonElement encode(final Object value) {
        return encodeMultimap(value);
    }

    // A PropertyMap is a Multimap too, which forwards asMap() to the one it holds.
    private static JsonElement encodeMultimap(final Object multimap) {
        JsonArray entries = new JsonArray();
        for (Map.Entry<?, ? extends Collection<?>> entry : ((Multimap<?, ?>)multimap).asMap().entrySet()) {
            JsonArray pair = new JsonArray();
            pair.add(Values.encode(entry.getKey(), String.class));
            pair.add(Values.encode(entry.getValue(), Collection.class));
            entries.add(pair);
        }
        return entries;
    }

    private static List<Object> multimapEdges() {
        List<Object> edges = new ArrayList<>();
        edges.add(ImmutableMultimap.of());
        PROPERTIES.edges().forEach(property -> edges.add(byName(List.of((Property)property))));
        Property first = new Property("a", "1");
        Property second = new Property("a", "2", SIGNATURE);
        Property other = new Property("b", "3");
        edges.add(byName(List.of(first, second)));
        edges.add(byName(List.of(second, first)));
        edges.add(byName(List.of(first, other, second)));
        edges.add(byName(List.of(other, first, second)));
        edges.add(byName(List.of(first, first)));
        edges.add(ImmutableMultimap.of("textures", first));
        for (int count : new int[]{16, 17}) {
            List<Property> properties = new ArrayList<>();
            for (int i = 0; i < count; i++) {
                properties.add(new Property("p" + i % 5, Integer.toString(i), i % 2 == 0 ? SIGNATURE : null));
            }
            edges.add(byName(properties));
        }
        return edges;
    }

    private static Object randomMultimap(final Random random) {
        int count = random.nextInt(8) == 0 ? random.nextInt(21) : random.nextInt(5);
        ImmutableMultimap.Builder<String, Property> builder = ImmutableMultimap.builder();
        for (int i = 0; i < count; i++) {
            Property property = (Property)PROPERTIES.random(random);
            builder.put(random.nextInt(4) == 0 ? (String)NAMES.random(random) : property.name(), property);
        }
        return builder.build();
    }

    private static ImmutableMultimap<String, Property> byName(final List<Property> properties) {
        ImmutableMultimap.Builder<String, Property> builder = ImmutableMultimap.builder();
        properties.forEach(property -> builder.put(property.name(), property));
        return builder.build();
    }

    private static Object property(final Object spec) {
        Object[] fields = (Object[])spec;
        return new Property((String)fields[0], (String)fields[1], (String)fields[2]);
    }

    @SuppressWarnings("unchecked")
    private static Object propertyMap(final Object multimap) {
        return new PropertyMap((Multimap<String, Property>)multimap);
    }

    private static Object gameProfile(final Object spec) {
        Object[] fields = (Object[])spec;
        return new GameProfile((UUID)fields[0], (String)fields[1], (PropertyMap)fields[2]);
    }

    private static byte[] randomBytes(final long seed, final int length) {
        byte[] bytes = new byte[length];
        new Random(seed).nextBytes(bytes);
        return bytes;
    }

    private static Values.Generator strings(final String... edges) {
        return new Values.Simple(Arrays.asList((Object[])edges), STRINGS::random, STRINGS::encode);
    }
}
