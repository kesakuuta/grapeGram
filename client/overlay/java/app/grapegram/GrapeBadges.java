package app.grapegram;

import android.content.Context;
import android.util.Base64;

import org.bouncycastle.jce.provider.BouncyCastleProvider;
import org.json.JSONArray;
import org.json.JSONObject;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.security.KeyFactory;
import java.security.PublicKey;
import java.security.Security;
import java.security.Signature;
import java.security.spec.X509EncodedKeySpec;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

/**
 * Fetches, verifies and caches the signed badge list.
 * Signature covers the exact bytes of badge_users.json (Ed25519, Android 13+).
 * Nothing is applied unless the signature is valid.
 */
public final class GrapeBadges {

    public static final class Entry {
        public final String userId;
        public final String badge;
        public final boolean enabled;

        Entry(String userId, String badge, boolean enabled) {
            this.userId = userId;
            this.badge = badge;
            this.enabled = enabled;
        }
    }

    public static final class Result {
        public final boolean ok;
        public final String error;
        public final List<Entry> entries;
        public final int version;

        private Result(boolean ok, String error, List<Entry> entries, int version) {
            this.ok = ok;
            this.error = error;
            this.entries = entries;
            this.version = version;
        }

        public static Result fail(String error) {
            return new Result(false, error, new ArrayList<Entry>(), 0);
        }
    }

    private static final String CACHE = "badge_cache.json";
    private static final String CACHE_VER = "badge_cache_version";

    private GrapeBadges() {
    }

    public static Result load(Context context, boolean forceNetwork) {
        Result cached = readCache(context);
        if (!forceNetwork && cached.ok) {
            return cached;
        }
        Result fresh;
        try {
            fresh = fetch(context);
        } catch (Throwable t) {
            fresh = Result.fail(t.getClass().getSimpleName());
        }
        if (fresh.ok) {
            return fresh;
        }
        return cached.ok ? cached : fresh;
    }

    public static Entry find(Context context, String userId) {
        Result r = readCache(context);
        if (!r.ok || userId == null) {
            return null;
        }
        for (Entry e : r.entries) {
            if (userId.equals(e.userId) && e.enabled) {
                return e;
            }
        }
        return null;
    }

    private static Result fetch(Context context) throws Exception {
        byte[] data = get(GrapeConfig.DATA_BASE + "badge_users.json");
        byte[] sig = Base64.decode(
                new String(get(GrapeConfig.DATA_BASE + "signatures/badge_users.json.sig"),
                        StandardCharsets.US_ASCII).trim(), Base64.DEFAULT);
        JSONObject doc = new JSONObject(new String(data, StandardCharsets.UTF_8));
        if (!verify(GrapeConfig.PUBLIC_KEY_PEM, canonicalize(doc).getBytes(StandardCharsets.UTF_8), sig)) {
            return Result.fail("подпись недействительна");
        }
        int version = doc.optInt("version", 0);
        int cachedVersion = GrapeConfig.getInt(context, CACHE_VER, 0);
        if (version < cachedVersion) {
            return Result.fail("устаревшая версия данных");
        }
        List<Entry> list = new ArrayList<>();
        JSONObject badges = doc.optJSONObject("badges");
        if (badges != null) {
            Iterator<String> it = badges.keys();
            while (it.hasNext()) {
                String uid = it.next();
                JSONObject rec = badges.optJSONObject(uid);
                if (rec != null) {
                    list.add(new Entry(uid, rec.optString("badge", "grapegram"),
                            rec.optBoolean("enabled", false)));
                }
            }
        }
        Result r = new Result(true, null, list, version);
        writeCache(context, data, version);
        return r;
    }

    private static boolean verify(String pem, byte[] data, byte[] sig) throws Exception {
        Security.addProvider(new BouncyCastleProvider());
        String body = pem.replace("-----BEGIN PUBLIC KEY-----", "")
                .replace("-----END PUBLIC KEY-----", "").replaceAll("\\s", "");
        PublicKey key = KeyFactory.getInstance("Ed25519", "BC")
                .generatePublic(new X509EncodedKeySpec(Base64.decode(body, Base64.DEFAULT)));
        Signature s = Signature.getInstance("Ed25519", "BC");
        s.initVerify(key);
        s.update(data);
        return s.verify(sig);
    }

    /** JCS-like canonical form: sorted keys, no whitespace, literal UTF-8. */
    private static String canonicalize(Object value) {
        if (value == null) {
            return "null";
        }
        if (value instanceof String) {
            return quote((String) value);
        }
        if (value instanceof Boolean || value instanceof Number) {
            return value.toString();
        }
        if (value instanceof JSONObject) {
            JSONObject object = (JSONObject) value;
            TreeMap<String, String> parts = new TreeMap<>();
            Iterator<String> keys = object.keys();
            while (keys.hasNext()) {
                String key = keys.next();
                parts.put(key, canonicalize(object.opt(key)));
            }
            StringBuilder out = new StringBuilder("{");
            boolean first = true;
            for (Map.Entry<String, String> entry : parts.entrySet()) {
                if (!first) {
                    out.append(',');
                }
                first = false;
                out.append(quote(entry.getKey())).append(':').append(entry.getValue());
            }
            return out.append('}').toString();
        }
        if (value instanceof JSONArray) {
            JSONArray array = (JSONArray) value;
            StringBuilder out = new StringBuilder("[");
            for (int i = 0; i < array.length(); i++) {
                if (i > 0) {
                    out.append(',');
                }
                out.append(canonicalize(array.opt(i)));
            }
            return out.append(']').toString();
        }
        return "null";
    }

    private static String quote(String value) {
        StringBuilder out = new StringBuilder(value.length() + 2).append('"');
        for (int i = 0; i < value.length(); i++) {
            char c = value.charAt(i);
            switch (c) {
                case '"': out.append("\\\""); break;
                case '\\': out.append("\\\\"); break;
                case '\b': out.append("\\b"); break;
                case '\f': out.append("\\f"); break;
                case '\n': out.append("\\n"); break;
                case '\r': out.append("\\r"); break;
                case '\t': out.append("\\t"); break;
                default:
                    if (c < 0x20) {
                        out.append(String.format("\\u%04x", (int) c));
                    } else {
                        out.append(c);
                    }
            }
        }
        return out.append('"').toString();
    }

    private static byte[] get(String url) throws Exception {
        HttpURLConnection c = (HttpURLConnection) new URL(url).openConnection();
        c.setConnectTimeout(10000);
        c.setReadTimeout(15000);
        c.setRequestProperty("User-Agent", "grapeGram/" + GrapeConfig.VERSION);
        try {
            if (c.getResponseCode() != 200) {
                throw new IllegalStateException("HTTP " + c.getResponseCode());
            }
            try (InputStream in = c.getInputStream()) {
                ByteArrayOutputStream out = new ByteArrayOutputStream();
                byte[] buf = new byte[8192];
                int n;
                while ((n = in.read(buf)) > 0) {
                    out.write(buf, 0, n);
                }
                return out.toByteArray();
            }
        } finally {
            c.disconnect();
        }
    }

    private static File cacheFile(Context c) {
        return new File(c.getFilesDir(), CACHE);
    }

    private static void writeCache(Context c, byte[] data, int version) {
        try (FileOutputStream out = new FileOutputStream(cacheFile(c))) {
            out.write(data);
            GrapeConfig.setInt(c, CACHE_VER, version);
        } catch (Throwable ignored) {
        }
    }

    private static Result readCache(Context c) {
        File f = cacheFile(c);
        if (!f.isFile()) {
            return Result.fail("нет данных");
        }
        try (FileInputStream in = new FileInputStream(f)) {
            ByteArrayOutputStream out = new ByteArrayOutputStream();
            byte[] buf = new byte[8192];
            int n;
            while ((n = in.read(buf)) > 0) {
                out.write(buf, 0, n);
            }
            JSONObject doc = new JSONObject(new String(out.toByteArray(), StandardCharsets.UTF_8));
            List<Entry> list = new ArrayList<>();
            JSONObject badges = doc.optJSONObject("badges");
            if (badges != null) {
                Iterator<String> it = badges.keys();
                while (it.hasNext()) {
                    String uid = it.next();
                    JSONObject rec = badges.optJSONObject(uid);
                    if (rec != null) {
                        list.add(new Entry(uid, rec.optString("badge", "grapegram"),
                                rec.optBoolean("enabled", false)));
                    }
                }
            }
            return new Result(true, null, list, doc.optInt("version", 0));
        } catch (Throwable t) {
            return Result.fail("кэш повреждён");
        }
    }
}
