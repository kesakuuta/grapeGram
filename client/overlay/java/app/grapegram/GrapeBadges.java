package app.grapegram;

import android.content.Context;
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
import org.bouncycastle.crypto.params.Ed25519PublicKeyParameters;
import org.bouncycastle.crypto.signers.Ed25519Signer;
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
        byte[] data = get(GrapeConfig.DATA_BASE + "badge_users.json");
        byte[] sig = Base64.decode(
                new String(get(GrapeConfig.DATA_BASE + "signatures/badge_users.json.sig"),
                        StandardCharsets.US_ASCII).trim(), Base64.DEFAULT);
        JSONObject doc = new JSONObject(new String(data, StandardCharsets.UTF_8));
        // Signing tools sign the raw file bytes, not a re-serialized document.
        if (!verify(GrapeConfig.PUBLIC_KEY_PEM, data, sig)) {
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

    private static boolean verify(String pem, byte[] data, byte[] sig) {
        // Use Bouncy Castle's lightweight Ed25519 implementation directly.
        // Android provider availability differs between OEM builds; relying on
        // JCA provider names can silently reject valid signatures on some devices.
        if (sig == null || sig.length != 64 || pem == null) {
            return false;
        }
        String body = pem.replace("-----BEGIN PUBLIC KEY-----", "")
                .replace("-----END PUBLIC KEY-----", "").replaceAll("\\s", "");
        try {
            byte[] der = Base64.decode(body, Base64.DEFAULT);
            // SubjectPublicKeyInfo for Ed25519 is 12 bytes of DER header + 32 raw key bytes.
            if (der.length != 44
                    || der[0] != 0x30 || der[1] != 0x2a
                    || der[2] != 0x30 || der[3] != 0x05
                    || der[4] != 0x06 || der[5] != 0x03
                    || der[6] != 0x2b || der[7] != 0x65
                    || der[8] != 0x70 || der[9] != 0x03
                    || der[10] != 0x21 || der[11] != 0x00) {
                return false;
            }
            byte[] rawKey = new byte[32];
            System.arraycopy(der, 12, rawKey, 0, rawKey.length);
            Ed25519PublicKeyParameters publicKey =
                    new Ed25519PublicKeyParameters(rawKey, 0);
            Ed25519Signer verifier = new Ed25519Signer();
            verifier.init(false, publicKey);
            verifier.update(data, 0, data.length);
            return verifier.verifySignature(sig);
        } catch (Throwable ignored) {
            return false;
        }
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
