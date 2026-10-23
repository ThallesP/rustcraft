package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Locale;
import java.util.Random;
import java.util.Set;
import net.minecraft.util.Util;

/**
 * {@code java.net.URI}, which a text component's {@code open_url} click event holds: {@code ExtraCodecs.UNTRUSTED_URI}
 * decodes its string with {@code Util.parseAndValidateUntrustedUri}, which is {@code new URI(string)} with an http or
 * https scheme, and fails with the exception's message. For each input string, {@code new URI(s)}'s components
 * ({@code parse}) or its exception's message ({@code parseError}), and {@code parseAndValidateUntrustedUri}'s outcome;
 * the characters each place in a URI takes, over every char outside the surrogates ({@code acceptsCharacter}); and
 * {@code hashCode}, {@code equals} and {@code compareTo} of parsed URIs, which ignore the case of schemes, hosts and
 * escapes' hex digits.
 *
 * <p>The golden files are UTF-8, so no input holds a lone surrogate: inputs are cut and spliced at code points.
 */
final class UriSuite implements Suite {
    private static final int RANDOM_URLS = 600;
    private static final int MUTATIONS = 1100;
    private static final int RANDOM_STRINGS = 500;
    private static final int RANDOM_PAIRS = 700;

    @Override
    public String name() {
        return URI.class.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Random random = new Random(53);
        List<String> valid = new ArrayList<>();
        for (String input : inputs(random)) {
            JsonArray args = strings(input);
            try {
                writer.result("parse", args, components(new URI(input)), null);
                valid.add(input);
            } catch (URISyntaxException e) {
                writer.result("parseError", args, new JsonPrimitive(e.getMessage()), null);
            }

            JsonArray outcome = new JsonArray();
            try {
                URI uri = Util.parseAndValidateUntrustedUri(input);
                outcome.add("success");
                outcome.add(uri.toString());
            } catch (URISyntaxException e) {
                outcome.add("error");
                outcome.add(e.getMessage());
            }
            writer.result("Util.parseAndValidateUntrustedUri", args, outcome, null);
        }

        for (String[] place : CHARACTER_PLACES) {
            writer.result("acceptsCharacter", strings(place[0], place[1]), acceptedCharacters(place[0], place[1]), null);
        }

        List<String[]> pairs = pairs(random, valid);
        Set<String> hashed = new LinkedHashSet<>(valid);
        for (String[] pair : pairs) {
            hashed.add(pair[0]);
            hashed.add(pair[1]);
        }
        for (String input : hashed) {
            writer.result("hashCode()I", strings(input), new JsonPrimitive(parse(input).hashCode()), null);
        }
        for (String[] pair : pairs) {
            URI a = parse(pair[0]);
            URI b = parse(pair[1]);
            writer.result("equals(Ljava/lang/Object;)Z", strings(pair[0], pair[1]), new JsonPrimitive(a.equals(b)), null);
            writer.result("compareTo(Ljava/net/URI;)I", strings(pair[0], pair[1]), new JsonPrimitive(a.compareTo(b)), null);
        }
    }

    // [scheme, rawSchemeSpecificPart, rawAuthority, rawUserInfo, host, port, rawPath, rawQuery, rawFragment, toString,
    // isAbsolute, isOpaque, [schemeSpecificPart, authority, userInfo, path, query, fragment]]
    private static JsonArray components(final URI uri) {
        JsonArray components = strings(
            uri.getScheme(), uri.getRawSchemeSpecificPart(), uri.getRawAuthority(), uri.getRawUserInfo(), uri.getHost()
        );
        components.add(uri.getPort());
        for (String part : new String[]{uri.getRawPath(), uri.getRawQuery(), uri.getRawFragment(), uri.toString()}) {
            components.add(nullable(part));
        }
        components.add(uri.isAbsolute());
        components.add(uri.isOpaque());
        components.add(strings(uri.getSchemeSpecificPart(), uri.getAuthority(), uri.getUserInfo(), uri.getPath(), uri.getQuery(), uri.getFragment()));
        return components;
    }

    // The chars c (outside the surrogates) for which prefix + c + suffix parses, as [first, last] ranges.
    private static JsonArray acceptedCharacters(final String prefix, final String suffix) {
        JsonArray ranges = new JsonArray();
        int first = -1;
        int last = -1;
        for (int c = 0; c <= 0xffff; c++) {
            if (Character.isSurrogate((char)c) || !parses(prefix + (char)c + suffix)) {
                continue;
            }
            if (first < 0 || c != last + 1) {
                if (first >= 0) {
                    ranges.add(range(first, last));
                }
                first = c;
            }
            last = c;
        }
        if (first >= 0) {
            ranges.add(range(first, last));
        }
        return ranges;
    }

    private static JsonArray range(final int first, final int last) {
        JsonArray range = new JsonArray();
        range.add(first);
        range.add(last);
        return range;
    }

    // The edges, every special string in every place, three characters at every position of two URLs, then random URLs,
    // mutations of the URLs so far, and random strings.
    private static List<String> inputs(final Random random) {
        Set<String> inputs = new LinkedHashSet<>(EDGES);
        for (String special : SPECIALS) {
            for (String[] place : PLACES) {
                inputs.add(place[0] + special + place[1]);
            }
        }
        for (String base : List.of("https://user:pass@www.example.com:8080/path/to/page.html?query=1&b=2#frag", "mailto:someone@example.com?subject=hi#top")) {
            for (String inserted : List.of(" ", "%", "<")) {
                for (int i = 0; i <= base.length(); i++) {
                    inputs.add(base.substring(0, i) + inserted + base.substring(i));
                }
            }
        }
        List<String> urls = new ArrayList<>(EDGES);
        for (int i = 0; i < RANDOM_URLS; i++) {
            String url = randomUrl(random);
            inputs.add(url);
            urls.add(url);
        }
        for (int i = 0; i < MUTATIONS; i++) {
            inputs.add(mutate(random, urls.get(random.nextInt(urls.size()))));
        }
        for (int i = 0; i < RANDOM_STRINGS; i++) {
            inputs.add(randomString(random));
        }
        return new ArrayList<>(inputs);
    }

    // The edge pairs both ways round, then random parsed strings against themselves (parsed again), their case flipped,
    // their escapes' case changed, a mutation of them or another parsed string.
    private static List<String[]> pairs(final Random random, final List<String> valid) {
        List<String[]> pairs = new ArrayList<>();
        for (String[] pair : EQUALITY_EDGES) {
            if (parses(pair[0]) && parses(pair[1])) {
                pairs.add(pair);
                pairs.add(new String[]{pair[1], pair[0]});
            }
        }
        for (int i = 0; i < RANDOM_PAIRS; i++) {
            String a = valid.get(random.nextInt(valid.size()));
            String b = switch (random.nextInt(6)) {
                case 0 -> a;
                case 1, 2 -> flipCase(random, a);
                case 3 -> escapesCase(a, random.nextBoolean());
                case 4 -> mutate(random, a);
                default -> valid.get(random.nextInt(valid.size()));
            };
            if (parses(b)) {
                pairs.add(random.nextBoolean() ? new String[]{a, b} : new String[]{b, a});
            }
        }
        return pairs;
    }

    private static String randomUrl(final Random random) {
        StringBuilder url = new StringBuilder();
        boolean opaque = false;
        boolean relative = random.nextInt(10) == 0;
        if (!relative) {
            String scheme = pick(random, SCHEMES);
            url.append(scheme).append(':');
            opaque = OPAQUE_SCHEMES.contains(scheme.toLowerCase(Locale.ROOT)) ? random.nextInt(5) != 0 : random.nextInt(12) == 0;
        }
        if (opaque) {
            url.append(part(random, URIC, 1 + random.nextInt(16)));
        } else {
            if (random.nextInt(8) != 0) {
                url.append("//");
                if (random.nextInt(5) == 0) {
                    url.append(part(random, USER_INFO, random.nextInt(8))).append('@');
                }
                url.append(randomHost(random));
                if (random.nextInt(4) == 0) {
                    url.append(':').append(pick(random, PORTS));
                }
            }
            for (int segments = random.nextInt(5); segments > 0; segments--) {
                url.append('/').append(part(random, PATH, random.nextInt(10)));
            }
            if (random.nextInt(3) == 0) {
                url.append('?').append(part(random, URIC, random.nextInt(14)));
            }
        }
        if (random.nextInt(4) == 0) {
            url.append('#').append(part(random, URIC, random.nextInt(10)));
        }
        return url.toString();
    }

    private static String randomHost(final Random random) {
        return switch (random.nextInt(8)) {
            case 0, 1, 2 -> {
                StringBuilder host = new StringBuilder();
                for (int labels = 1 + random.nextInt(4); labels > 0; labels--) {
                    host.append(part(random, LABEL, 1 + random.nextInt(8)));
                    if (labels > 1 || random.nextInt(8) == 0) {
                        host.append('.');
                    }
                }
                yield host.toString();
            }
            case 3 -> {
                StringBuilder host = new StringBuilder();
                int parts = random.nextInt(8) == 0 ? 3 + random.nextInt(3) : 4;
                for (int i = 0; i < parts; i++) {
                    if (i > 0) {
                        host.append('.');
                    }
                    host.append(octet(random));
                }
                yield host.toString();
            }
            case 4, 5 -> ipv6(random);
            case 6 -> part(random, REG_NAME, 1 + random.nextInt(8));
            default -> pick(random, List.of("1", "123", "0", "a", "Z", "localhost", "a1", "1a", "-", "_"));
        };
    }

    // Mostly 0 to 255, sometimes past it, with leading zeros, or more digits than a byte has.
    private static String octet(final Random random) {
        return switch (random.nextInt(10)) {
            case 0 -> Integer.toString(256 + random.nextInt(800));
            case 1 -> "0".repeat(1 + random.nextInt(3)) + random.nextInt(256);
            case 2 -> Integer.toString(random.nextInt(100000));
            default -> Integer.toString(random.nextInt(256));
        };
    }

    // Up to nine groups of hex digits (some too long), compressed somewhere or not, sometimes ending in an IPv4 address,
    // sometimes with a scope id.
    private static String ipv6(final Random random) {
        StringBuilder host = new StringBuilder("[");
        int groups = random.nextInt(10);
        int compressedAt = random.nextBoolean() ? random.nextInt(groups + 1) : -1;
        for (int i = 0; i < groups; i++) {
            if (i == compressedAt) {
                host.append("::");
            } else if (i > 0) {
                host.append(':');
            }
            host.append(part(random, HEX, random.nextInt(12) == 0 ? 5 : 1 + random.nextInt(4)));
        }
        if (compressedAt == groups) {
            host.append("::");
        }
        if (random.nextInt(5) == 0) {
            if (groups > 0 && compressedAt != groups) {
                host.append(':');
            }
            for (int i = 0; i < 4; i++) {
                if (i > 0) {
                    host.append('.');
                }
                host.append(octet(random));
            }
        }
        if (random.nextInt(6) == 0) {
            host.append('%').append(part(random, SCOPE_ID, random.nextInt(6)));
        }
        return host.append(']').toString();
    }

    // One to three changes, at code points: a code point deleted, a token inserted or put in a code point's place, a stretch
    // repeated, an ASCII letter's case flipped, or the rest cut off.
    private static String mutate(final Random random, final String string) {
        StringBuilder s = new StringBuilder(string);
        for (int changes = 1 + random.nextInt(3); changes > 0; changes--) {
            int count = s.codePointCount(0, s.length());
            int at = s.offsetByCodePoints(0, random.nextInt(count + 1));
            int next = at < s.length() ? s.offsetByCodePoints(at, 1) : at;
            switch (random.nextInt(6)) {
                case 0 -> s.delete(at, next);
                case 1 -> s.insert(at, pick(random, TOKENS));
                case 2 -> s.replace(at, next, pick(random, TOKENS));
                case 3 -> {
                    int end = s.offsetByCodePoints(at, random.nextInt(count - s.codePointCount(0, at) + 1));
                    s.insert(end, s.substring(at, end));
                }
                case 4 -> {
                    if (next > at && s.charAt(at) < 0x80 && Character.isLetter(s.charAt(at))) {
                        char c = s.charAt(at);
                        s.setCharAt(at, Character.isUpperCase(c) ? Character.toLowerCase(c) : Character.toUpperCase(c));
                    }
                }
                default -> s.setLength(at);
            }
        }
        return s.toString();
    }

    private static String randomString(final Random random) {
        StringBuilder s = new StringBuilder();
        for (int length = random.nextInt(20); length > 0; length--) {
            s.append(pick(random, TOKENS));
        }
        return s.toString();
    }

    // Each ASCII letter's case flipped with a chance of one in two.
    private static String flipCase(final Random random, final String s) {
        StringBuilder flipped = new StringBuilder(s);
        for (int i = 0; i < flipped.length(); i++) {
            char c = flipped.charAt(i);
            if (c < 0x80 && Character.isLetter(c) && random.nextBoolean()) {
                flipped.setCharAt(i, Character.isUpperCase(c) ? Character.toLowerCase(c) : Character.toUpperCase(c));
            }
        }
        return flipped.toString();
    }

    // The two characters after each '%' in upper or lower case.
    private static String escapesCase(final String s, final boolean upper) {
        StringBuilder changed = new StringBuilder(s);
        for (int i = 0; i + 2 < changed.length(); i++) {
            if (changed.charAt(i) == '%') {
                for (int j = i + 1; j <= i + 2; j++) {
                    char c = changed.charAt(j);
                    if (c < 0x80) {
                        changed.setCharAt(j, upper ? Character.toUpperCase(c) : Character.toLowerCase(c));
                    }
                }
            }
        }
        return changed.toString();
    }

    // Tokens from the list, each a character, an escape or a non-ASCII character.
    private static String part(final Random random, final List<String> tokens, final int length) {
        StringBuilder part = new StringBuilder();
        for (int i = 0; i < length; i++) {
            part.append(pick(random, tokens));
        }
        return part.toString();
    }

    private static String pick(final Random random, final List<String> list) {
        return list.get(random.nextInt(list.size()));
    }

    private static boolean parses(final String s) {
        try {
            new URI(s);
            return true;
        } catch (URISyntaxException e) {
            return false;
        }
    }

    private static URI parse(final String s) {
        try {
            return new URI(s);
        } catch (URISyntaxException e) {
            throw new IllegalStateException(e);
        }
    }

    private static JsonElement nullable(final String s) {
        return s == null ? JsonNull.INSTANCE : new JsonPrimitive(s);
    }

    private static JsonArray strings(final String... strings) {
        JsonArray array = new JsonArray();
        for (String s : strings) {
            array.add(nullable(s));
        }
        return array;
    }

    private static List<String> tokens(final String characters, final String... more) {
        List<String> tokens = new ArrayList<>();
        characters.codePoints().forEach(c -> tokens.add(Character.toString(c)));
        tokens.addAll(List.of(more));
        return List.copyOf(tokens);
    }

    private static String[] concat(final String[] first, final String[] second) {
        String[] all = new String[first.length + second.length];
        System.arraycopy(first, 0, all, 0, first.length);
        System.arraycopy(second, 0, all, first.length, second.length);
        return all;
    }

    private static final String ALPHANUM = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    private static final String MARK = "-_.!~*'()";
    private static final String[] ESCAPES = {"%41", "%2f", "%2F", "%20", "%25", "%C3%A9", "%e2%82%ac", "%ff", "%C0%80", "%E2%82"};
    private static final String[] OTHERS = {"é", "日本", "😀", "\u00a1", "\ufffd"};
    private static final List<String> LABEL = tokens("abcxyzABCXYZ0129---");
    private static final List<String> HEX = tokens("0123456789abcdefABCDEF");
    private static final List<String> SCOPE_ID = tokens("abcXYZ019_.-");
    private static final List<String> USER_INFO = tokens(ALPHANUM + MARK + ";:&=+$,", concat(ESCAPES, OTHERS));
    private static final List<String> REG_NAME = tokens(ALPHANUM + MARK + "$,;:@&=+", concat(ESCAPES, OTHERS));
    private static final List<String> PATH = tokens(ALPHANUM + MARK + ":@&=+$,;", concat(ESCAPES, OTHERS));
    private static final List<String> URIC = tokens(ALPHANUM + MARK + ";/?:@&=+$,[]", concat(ESCAPES, OTHERS));
    // What random strings and mutations are made of: delimiters, characters of each class, escapes and broken ones, and
    // characters no component takes.
    private static final List<String> TOKENS = tokens(
        "::://///??##[[]]@@%%abcdefxyzABCDEFXYZ0123456789-_.!~*'();&=+$, <>\"{}|\\^`\t\u007f\u0085\u00a0\u2028\u3000é😀",
        "%41", "%4", "%zz", "%e2%82%ac", "%E2%82", "::1", "1.2.3.4", "//", "http:", "https://", "[::1]", ":80", "%eth0", "\u0000"
    );

    private static final List<String> SCHEMES = List.of(
        "http", "https", "HTTP", "HTTPS", "Http", "hTtPs", "ftp", "file", "ws", "wss", "git+ssh", "a.b-c+d", "s", "x1",
        "mailto", "urn", "tel", "data", "javascript", "about", "news", "MAILTO", "minecraft", "steam"
    );
    private static final Set<String> OPAQUE_SCHEMES = Set.of("mailto", "urn", "tel", "data", "javascript", "about", "news");
    private static final List<String> PORTS = List.of(
        "", "0", "1", "80", "443", "8080", "25565", "65535", "65536", "007", "2147483647", "2147483648", "99999999999", "1a", "٣"
    );

    // Where SPECIALS go: [before, after].
    private static final String[][] PLACES = {
        {"", "http://h/"}, {"ht", "tp://h/"}, {"http", "://h/"}, {"http:", "//h/"}, {"http:/", "/h/"}, {"http://", "h/"},
        {"http://u", "@h/"}, {"http://h", "/"}, {"http://h", ".com/"}, {"http://h:", "80/"}, {"http://h:80", "/"},
        {"http://h/", "p"}, {"http://h/p?", "q"}, {"http://h/p#", "f"}, {"mailto:", "a@b"}, {"mailto:a", "@b"}, {"", ""},
        {"a", "b"}, {"http://[::1", "]/"}, {"http://[fe80::1%", "eth0]/"}, {"http://1.2.3.4", "/"}, {"//", "h/p"}
    };
    private static final List<String> SPECIALS = List.of(
        " ", "  ", "<", ">", "\"", "{", "}", "|", "\\", "^", "`", "\t", "\n", "\r", "\u0000", "\u0001", "\u001f", "\u007f",
        "\u0080", "\u0081", "\u0085", "\u009f", "\u00a0", "\u00a1", "é", "€", "\u1680", "\u2000", "\u200b", "\u2028",
        "\u2029", "\u202f", "\u3000", "\ufeff", "\uffff", "😀", "\ud800\udc00", "\udbff\udfff", "%", "%2", "%2g", "%25",
        "%C3%A9", "[", "]", "@", ":", "#", "?", "/", ";", "-", "_", "."
    );
    // Where acceptsCharacter puts each char: a scheme's first character and the rest, an opaque part's, a relative path's
    // alone and after a slash, a path's, a query's, a fragment's, user info's, an authority's alone, a host's or registry
    // name's, a port's, a scope id's, an IPv6 address's and after an IPv4 address.
    private static final String[][] CHARACTER_PLACES = {
        {"", ":x"}, {"s", ":x"}, {"s:", ""}, {"s:x", ""}, {"", ""}, {"/", ""}, {"s://h/", ""}, {"s://h/?", ""}, {"s://h/#", ""},
        {"s://", "@h/"}, {"s://", "/"}, {"s://h", "/"}, {"s://h:", "/"}, {"s://h:8", "/"}, {"s://[::1%e", "]/"},
        {"s://[::", "]/"}, {"s://1.2.3.4", "/"}
    };

    private static final List<String> EDGES = List.of(
        // Empty strings and delimiters alone
        "", " ", "  ", "#", "?", "?#", "#?", "##", "??", "/", "//", "///", "////", ":", "::", "a:", ":a", "a:b", "a:/",
        "a://", "a:///", "a:?", "a:#", ":/", "://", "//#", "//?", "//?#", "//a", "//@", "//:", "//[", "//]", "%", "%%", "%2",
        "%20", "[", "]", "@",
        // http and https
        "http:", "http:/", "http://", "http:///", "http:///a", "http://#", "http://?", "http://h", "http://h/", "http://h:",
        "http://h:/", "http://h:0", "http://h:80", "http://h:65535", "http://h:65536", "http://h:2147483647",
        "http://h:2147483648", "http://h:4294967296", "http://h:99999999999999999999", "http://h:00000000000000000080",
        "http://h:-1", "http://h:+1", "http://h:1a", "http://h:a", "http://h::80", "http://h:80:80", "http://h:٣",
        "http://h:8 0", "https://example.com", "https://www.example.com/", "HTTPS://WWW.EXAMPLE.COM/",
        "HtTp://Example.Com/Path", "http://example.com/path/to/page.html?query=1&b=2#frag",
        "https://minecraft.net/en-us/article/minecraft-java-edition", "https://aka.ms/MinecraftEULA",
        "https://help.minecraft.net/hc/en-us/articles/4408950195341", "http://localhost:25565",
        "https://user:pass@host.example:8443/a/b/c?x=y#z", "https://example.com/?", "https://example.com/#",
        "https://example.com/?#", "https://example.com?q", "https://example.com#f", "http://a.b.c.d.e.f.g/",
        "http://xn--bcher-kva.example/", "http://example.com./", "http://example.com../", "http://.example.com/",
        "http://-example.com/", "http://example-.com/", "http://ex-ample.com/", "http://ex--ample.com/",
        "http://example.c-m/", "http://example.-com/", "http://example.1com/", "http://example.com1/",
        "http://1example.com/", "http://123/", "http://123.com/", "http://a.123/", "http://a_b.com/", "http://a~b/",
        "http://a!b/", "http://a$b/", "http://a,b/", "http://a;b/", "http://a=b/", "http://a+b/", "http://a&b/",
        "http://a'b/", "http://a(b)/", "http://a*b/", "http://a%41b/", "http://%41/", "http://a%zz/", "http://a%4/",
        "http://a%/", "http://exa mple.com/", "http://example.com /", "http:// example.com/", "http://example.com/ ",
        " http://example.com/", "http://example.com/a b", "http://example.com/a  b  c  ", "  http://a  b  ",
        // User info
        "http://@h/", "http://u@h/", "http://u:p@h/", "http://u:p:q@h/", "http://u@@h/", "http://u@h@x/", "http://u%41@h/",
        "http://u%4@h/", "http://u%zz@h/", "http://u;p@h/", "http://u&p=q+r$s,t@h/", "http://u!~*'()@h/", "http://ü@h/",
        "http://u p@h/", "http://u[@h/", "http://u]@h/", "http://:@h/", "http://u@/", "http://u@:80/", "http://u@h:80/",
        "http://u@[::1]/", "http://u@1.2.3.4/", "http://u@h_h/", "http://user@example.com:8080/path",
        // IPv4 addresses
        "http://1.2.3.4/", "http://1.2.3.4", "http://1.2.3.4:80/", "http://0.0.0.0/", "http://255.255.255.255/",
        "http://256.1.1.1/", "http://1.256.1.1/", "http://1.1.1.256/", "http://1.2.3/", "http://1.2/", "http://1/",
        "http://1.2.3.4.5/", "http://1.2.3.4./", "http://.1.2.3.4/", "http://1..2.3/", "http://01.02.03.004/",
        "http://001.002.003.004/", "http://0000000255.1.1.1/", "http://0000000256.1.1.1/", "http://1000.1.1.1/",
        "http://999.1.1.1/", "http://1.2.3.4a/", "http://1.2.3.a/", "http://a.1.2.3/", "http://1.2.3.4:/",
        "http://1.2.3.4:x/", "http://1.2.3.4x:80/", "http://127.0.0.1:25565", "http://192.168.0.1/admin",
        "http://1.2.3.4%41/", "http://1.2.3.04/", "http://1.2.3.400/", "http://1.2.3.4.com/", "http://1.2.3.4-/",
        // IPv6 addresses
        "http://[::1]/", "http://[::1]", "http://[::1]:8080/", "http://[::]/", "http://[::]:0", "http://[1:2:3:4:5:6:7:8]/",
        "http://[1:2:3:4:5:6:7]/", "http://[1:2:3:4:5:6:7:8:9]/", "http://[1::8]/", "http://[1:2:3:4:5:6:7::]/",
        "http://[1:2:3:4:5:6::]/", "http://[::2:3:4:5:6:7:8]/", "http://[::1:2:3:4:5:6:7:8]/", "http://[1::2::3]/",
        "http://[1:::2]/", "http://[:1]/", "http://[1:]/", "http://[:]/", "http://[:::]/", "http://[]/", "http://[/",
        "http://[::1/", "http://::1]/", "http://[::1]]/", "http://[[::1]/", "http://[::1]x/", "http://[::1]:/",
        "http://[::1]:x/", "http://[::1]::80/", "http://[12345::1]/", "http://[1234::1]/", "http://[g::1]/",
        "http://[G::1]/", "http://[::ABCD:ef01]/", "http://[::1.2.3.4]/", "http://[::ffff:1.2.3.4]/",
        "http://[::ffff:1.2.3.4:5]/", "http://[1:2:3:4:5:6:1.2.3.4]/", "http://[1:2:3:4:5:6:7:1.2.3.4]/",
        "http://[1:2:3:4:5:1.2.3.4]/", "http://[1::1.2.3.4]/", "http://[1::2:1.2.3.4]/", "http://[::1.2.3]/",
        "http://[::1.2.3.4.5]/", "http://[::256.1.1.1]/", "http://[::1.2.3.256]/", "http://[1.2.3.4]/",
        "http://[::1.2.3.4:5]/", "http://[1:2:3:4:5:6:7:8:1.2.3.4]/", "http://[::1:1.2.3.4]/", "http://[::01.002.3.4]/",
        "http://[fe80::1%eth0]/", "http://[fe80::1%25eth0]/", "http://[fe80::1%]/", "http://[fe80::1%%]/",
        "http://[fe80::1%eth-0]/", "http://[fe80::1%eth_0.1]/", "http://[fe80::1%1]/", "http://[%eth0]/",
        "http://[::1%eth0]:80/p", "http://[fe80::1%eth0]x/", "http://[fe80::1%é]/", "http://[fe80::1%eth0%1]/",
        "http://[1:2:3:4:5:6:7:8%0]/", "http://[1:2:3:4:5:6:7%0]/", "http://u@[::1]:1/", "http://[::1]@h/",
        "http://[FE80::A]/", "http://[::1] /", "http://[ ::1]/", "http://[::1 ]/", "http://[fe80::1%eth0]/%41",
        // A port only fails as itself where the authority can't be a registry name instead, like after an IPv6 address
        "http://[::1]:2147483647/", "http://[::1]:2147483648/", "http://[::1]:99999999999/", "http://[::1]:65536/",
        "http://[::1]:٣/", "http://[::1]:1a/", "http://u@[::1]:0080",
        // Registry-based authorities and other schemes
        "s://a_b/", "s://a_b:80/", "s://h:99999999999/", "s://-/", "s://a b/", "s://é/", "s://h:é/", "s://h:80é/",
        "s://ü.example/", "s://日本.jp/", "s://h/é", "s://😀/", "s://h/😀", "s://h?😀", "s://h#😀",
        "ftp://ftp.example.com/pub/file.txt", "file:///etc/passwd", "file:/etc/passwd", "file://localhost/etc/passwd",
        "file:///C:/Windows", "file://C:/Windows", "file:C:/Windows", "jar:file:/a.jar!/b/c.class", "ws://h/socket",
        "wss://h:443/", "git+ssh://git@github.com/a/b.git", "x-mc-link://store/path", "minecraft://connect/?a=b|c",
        "a+b-c.d://h/", "a.b:c", "a-b:c", "a+b:c", "A1:c", "1a:c", "+a:c", "-a:c", ".a:c", "a_b:c", "a b:c", "é:c",
        "a%41:c", "a:b:c", "a::", ":::", "a:/b", "a:b/c", "a:%", "a:%41", "a:%4", "a:%zz", "a:é", "a: ", "a: b",
        // Opaque URIs
        "mailto:", "mailto:a@b.c", "mailto:a@b.c?subject=hi", "mailto:a@b.c#f", "mailto:#f", "mailto:a#", "mailto:a#f#g",
        "mailto:a b", "mailto:[a]", "mailto:a%2fb", "mailto:%41", "MAILTO:A@B.C", "urn:isbn:0451450523",
        "urn:ietf:rfc:2648", "urn:uuid:6e8bc430-9c3a-11d9-9669-0800200c9a66", "urn:", "urn:a", "urn:a:b:c",
        "news:comp.lang.java", "tel:+1-816-555-1212", "data:text/plain;base64,SGVsbG8=", "data:,Hello%2C%20World!",
        "javascript:alert(1)", "javascript:alert('x')", "javascript:void(0)", "about:blank", "about:", "chrome://settings",
        "ms-settings:", "steam://run/1", "s:?", "s:??", "s:#", "s:a?b#c", "s:a/b", "s:[::1]", "s:%", "s:{}", "s:a|b",
        // Relative references
        "a", "a/b", "/a", "/a/b", "../a", "./a", "..", ".", "a?b", "a#b", "a?b#c", "?q", "#f", "?q#f", "//h", "//h/p",
        "//h:80", "//u@h:80/p?q#f", "///p", "//[::1]", "/p?q", "a/b:c", "./a:b", "a/:b", "//h/p:q", "p%20q", "p%2", "p%",
        "%41", "%E2%82%AC", "%e2%82%ac", "%zz", "%4g", "%g4", "%%41", "%41%", "é", "😀", "a😀b", "日本語", " a", "a ", "a b",
        // Escapes, well-formed or not, and what they decode to, malformed UTF-8 included
        "http://h/%41", "http://h/%4", "http://h/%", "http://h/%zz", "http://h/%4g", "http://h/%%41", "http://h/a%2Fb",
        "http://h/a%2fb", "http://h/?%41", "http://h/#%41", "http://h/%e2%82%ac", "http://h/%E2%82%AC", "http://h/%E2%82",
        "http://h/%E2", "http://h/%C0%80", "http://h/%C3%A9", "http://h/%c3%a9", "http://h/%ff", "http://h/%80",
        "http://h/%F0%9F%98%80", "http://h/%F0%9F%98", "http://h/%ED%A0%80", "http://h/%ED%A0%80%ED%B0%80",
        "http://h/%F4%90%80%80", "http://h/%E0%80%80", "http://h/%00", "http://h/%41%42%43", "http://h/%41b%42",
        "http://h/%25", "http://h/%2525", "http://u%C3%A9@h/", "http://h/?q=%5B%5D", "http://h/?[%41]", "http://h/#[%41]",
        "s://h/?q=[%41]#[%41]", "s://h/?[%41]%42]%43", "s:[%41]", "s:[%41]#[%41]", "s://a%41b/", "s://a%41b:80/",
        "http://[::1]/%41?%42#%43", "http://h/%41é%42", "http://h/é%C3%A9😀%F0%9F%98%80",
        // Characters URIs don't take, and some they take in some places
        "http://h/<>", "http://h/\"", "http://h/{}", "http://h/|", "http://h/\\", "http://h/^", "http://h/`", "http://h/\t",
        "http://h/\n", "http://h/\u0000", "http://h/\u007f", "http://h/\u0080", "http://h/\u0085", "http://h/\u009f",
        "http://h/\u00a0", "http://h/\u00a1", "http://h/\u2028", "http://h/\u3000", "http://h/\u200b", "http://h/\ufeff",
        "http://h/\uffff", "http://h/[", "http://h/]", "http://h/a[b]", "http://h/?a[b]", "http://h/#a[b]", "http://h/##",
        "http://h/?#?#",
        // Long ones
        "http://" + "a".repeat(300) + ".com/", "http://h/" + "a/".repeat(200), "http://h/?" + "%41".repeat(100)
    );

    // Pairs that differ in case (schemes and hosts compare without it, escapes' hex digits too, the rest with it), in a
    // port that's empty or not there, or in the kind of authority. Pairs where either doesn't parse are left out.
    private static final String[][] EQUALITY_EDGES = {
        {"HTTP://Example.COM/", "http://example.com/"}, {"http://h/%2f", "http://h/%2F"},
        {"http://h/a%2fb?q=%2f#%2f", "http://h/a%2Fb?q=%2F#%2F"}, {"http://U@h/", "http://u@h/"},
        {"http://u%2f@h/", "http://u%2F@h/"}, {"mailto:A@B", "mailto:a@b"}, {"MAILTO:a%2f", "mailto:a%2F"},
        {"urn:%41", "urn:%41"}, {"http://h:/", "http://h/"}, {"http://h:80/", "http://h/"}, {"http://h:80/", "http://h:080/"},
        {"http://[::A]/", "http://[::a]/"}, {"http://[fe80::1%ETH0]/", "http://[fe80::1%eth0]/"},
        {"http://a_b/", "http://A_B/"}, {"http://a_b%2f/", "http://a_b%2F/"}, {"http://h:99999999999/", "http://H:99999999999/"},
        {"//h/p", "//H/p"}, {"/p", "/P"}, {"#f", "#F"}, {"?%2f", "?%2F"}, {"http://h/p", "http://h/p#"},
        {"http://h/p", "http://h/p?"}, {"http://h", "http://h/"}, {"file:///a", "file:/a"}, {"s:/a", "s:a"},
        {"s:a", "s:a#"}, {"s:a", "a"}, {"//h", "///"}, {"http://h/%41", "http://h/A"}, {"http://1.2.3.4/", "http://01.2.3.4/"},
        {"http://h/%e2%82%ac", "http://h/€"}, {"", "#"}, {"", "?"}, {"http://h:2147483647/", "http://h:-1/"},
        {"http://h:2147483647/", "http://h/"}, {"http://a%2Fb/", "http://a%2fb:/"}, {"http://h./", "http://h/"},
        {"http://u@h:1/", "http://u@h:01/"}, {"http://h/%41%42", "http://h/%41B"}, {"s:%41", "s:%61"}
    };
}
