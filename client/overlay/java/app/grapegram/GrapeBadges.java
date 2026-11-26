package app.grapegram;

import android.content.Context;
import android.os.Build;
import android.util.Base64;

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
import java.security.Signature;
import java.security.spec.X509EncodedKeySpec;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

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
        if (Build.VERSION.SDK_INT < 33) {
            return Result.fail("Ed25519 требует Android 13+");
        }
        byte[] data = get(GrapeConfig.DATA_BASE + "badge_users.json");
        byte[] sig = Base64.decode(
                new String(get(GrapeConfig.DATA_BASE + "signatures/badge_users.json.sig"),
                        StandardCharsets.US_ASCII).trim(), Base64.DEFAULT);
        if (!verify(GrapeConfig.PUBLIC_KEY_PEM, data, sig)) {
            return Result.fail("подпись недействительна");
        }
        JSONObject doc = new JSONObject(new String(data, StandardCharsets.UTF_8));
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
        String body = pem.replace("-----BEGIN PUBLIC KEY-----", "")
                .replace("-----END PUBLIC KEY-----", "").replaceAll("\\s", "");
        PublicKey key = KeyFactory.getInstance("Ed25519")
                .generatePublic(new X509EncodedKeySpec(Base64.decode(body, Base64.DEFAULT)));
        Signature s = Signature.getInstance("Ed25519");
        s.initVerify(key);
        s.update(data);
        return s.verify(sig);
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
