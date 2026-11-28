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
import java.util.Collections;
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
    private static final String CACHE_SIGNATURE = "badge_cache.json.sig";
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
        // badge-tools signs canonical JSON (sorted keys, compact separators), not
        // the pretty-printed bytes stored in Git. verify() accepts legacy raw
        // signatures too, preserving compatibility with the early prototype.
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
        writeCache(context, data, sig, version);
        return r;
    }

    private static boolean verify(String pem, byte[] data, byte[] sig) {
        // badge-tools signs canonical JSON; raw-byte verification remains as a
        // compatibility fallback for signatures created by the early prototype.
        if (sig == null || sig.length != 64 || pem == null || data == null) {
            return false;
        }
        String body = pem.replace("-----BEGIN PUBLIC KEY-----", "")
                .replace("-----END PUBLIC KEY-----", "").replaceAll("\\s", "");
        try {
            byte[] der = Base64.decode(body, Base64.DEFAULT);
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
            return verifyBytes(publicKey, canonicalJson(data), sig)
                    || verifyBytes(publicKey, data, sig);
        } catch (Throwable ignored) {
            return false;
        }
    }

    private static boolean verifyBytes(Ed25519PublicKeyParameters publicKey, byte[] payload, byte[] signature) {
        Ed25519Signer verifier = new Ed25519Signer();
        verifier.init(false, publicKey);
        verifier.update(payload, 0, payload.length);
        return verifier.verifySignature(signature);
    }

    private static byte[] canonicalJson(byte[] data) throws Exception {
        JSONObject document = new JSONObject(new String(data, StandardCharsets.UTF_8));
        StringBuilder out = new StringBuilder(data.length);
        appendCanonical(document, out, 0);
        return out.toString().getBytes(StandardCharsets.UTF_8);
    }

    private static void appendCanonical(Object value, StringBuilder out, int depth) throws Exception {
        if (depth > 128) {
            throw new IllegalArgumentException("JSON слишком глубокий");
        }
        if (value == null || value == JSONObject.NULL) {
            out.append("null");
        } else if (value instanceof JSONObject) {
            JSONObject object = (JSONObject) value;
            ArrayList<String> keys = new ArrayList<>();
            Iterator<String> iterator = object.keys();
            while (iterator.hasNext()) {
                keys.add(iterator.next());
            }
            Collections.sort(keys);
            out.append('{');
            for (int i = 0; i < keys.size(); i++) {
                if (i > 0) out.append(',');
                appendCanonicalString(keys.get(i), out);
                out.append(':');
                appendCanonical(object.get(keys.get(i)), out, depth + 1);
            }
            out.append('}');
        } else if (value instanceof org.json.JSONArray) {
            org.json.JSONArray array = (org.json.JSONArray) value;
            out.append('[');
            for (int i = 0; i < array.length(); i++) {
                if (i > 0) out.append(',');
                appendCanonical(array.get(i), out, depth + 1);
            }
            out.append(']');
        } else if (value instanceof String) {
            appendCanonicalString((String) value, out);
        } else if (value instanceof Boolean) {
            out.append(((Boolean) value) ? "true" : "false");
        } else if (value instanceof Number) {
            String number = value.toString();
            if (!number.matches("-?(0|[1-9][0-9]*)")) {
                throw new IllegalArgumentException("В подписанном JSON допустимы только целые числа");
            }
            out.append(number);
        } else {
            throw new IllegalArgumentException("Неподдерживаемое значение JSON");
        }
    }

    private static void appendCanonicalString(String value, StringBuilder out) {
        out.append('"');
        for (int i = 0; i < value.length(); i++) {
            char ch = value.charAt(i);
            switch (ch) {
                case '"': out.append("\\\""); break;
                case '\\': out.append("\\\\"); break;
                case '\b': out.append("\\b"); break;
                case '\f': out.append("\\f"); break;
                case '\n': out.append("\\n"); break;
                case '\r': out.append("\\r"); break;
                case '\t': out.append("\\t"); break;
                default:
                    if (ch < 0x20) {
                        out.append(String.format(java.util.Locale.US, "\\u%04x", (int) ch));
                    } else {
                        out.append(ch);
                    }
                    break;
            }
        }
        out.append('"');
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

    private static void writeCache(Context c, byte[] data, byte[] signature, int version) {
        File dataFile = cacheFile(c);
        File signatureFile = new File(c.getFilesDir(), CACHE_SIGNATURE);
        File dataTemp = new File(c.getFilesDir(), CACHE + ".tmp");
        File signatureTemp = new File(c.getFilesDir(), CACHE_SIGNATURE + ".tmp");
        try {
            try (FileOutputStream out = new FileOutputStream(dataTemp)) {
                out.write(data);
                out.getFD().sync();
            }
            try (FileOutputStream out = new FileOutputStream(signatureTemp)) {
                out.write(Base64.encode(signature, Base64.NO_WRAP));
                out.getFD().sync();
            }
            if (!dataTemp.renameTo(dataFile)) {
                throw new IllegalStateException("не удалось сохранить кэш");
            }
            if (!signatureTemp.renameTo(signatureFile)) {
                dataFile.delete();
                throw new IllegalStateException("не удалось сохранить подпись кэша");
            }
            GrapeConfig.setInt(c, CACHE_VER, version);
        } catch (Throwable ignored) {
            dataTemp.delete();
            signatureTemp.delete();
        }
    }

    private static Result readCache(Context c) {
        File f = cacheFile(c);
        File signatureFile = new File(c.getFilesDir(), CACHE_SIGNATURE);
        if (!f.isFile() || !signatureFile.isFile()) {
            // Old unsigned caches are deliberately rejected.
            return Result.fail("нет проверенного кэша");
        }
        try (FileInputStream in = new FileInputStream(f);
             FileInputStream signatureIn = new FileInputStream(signatureFile)) {
            byte[] data = readAll(in);
            byte[] signatureText = readAll(signatureIn);
            byte[] signature = Base64.decode(
                    new String(signatureText, StandardCharsets.US_ASCII).trim(), Base64.DEFAULT);
            if (!verify(GrapeConfig.PUBLIC_KEY_PEM, data, signature)) {
                return Result.fail("подпись кэша недействительна");
            }
            JSONObject doc = new JSONObject(new String(data, StandardCharsets.UTF_8));
            int version = doc.optInt("version", 0);
            if (version <= 0 || version < GrapeConfig.getInt(c, CACHE_VER, 0)) {
                return Result.fail("устаревший кэш");
            }
            List<Entry> list = new ArrayList<>();
            JSONObject badges = doc.optJSONObject("badges");
            if (badges == null) {
                return Result.fail("неверный формат кэша");
            }
            Iterator<String> it = badges.keys();
            while (it.hasNext()) {
                String uid = it.next();
                JSONObject rec = badges.optJSONObject(uid);
                if (rec != null) {
                    list.add(new Entry(uid, rec.optString("badge", "grapegram"),
                            rec.optBoolean("enabled", false)));
                }
            }
            return new Result(true, null, list, version);
        } catch (Throwable t) {
            return Result.fail("кэш повреждён");
        }
    }

    private static byte[] readAll(InputStream in) throws Exception {
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        byte[] buffer = new byte[8192];
        int count;
        while ((count = in.read(buffer)) != -1) {
            out.write(buffer, 0, count);
        }
        return out.toByteArray();
    }
}
