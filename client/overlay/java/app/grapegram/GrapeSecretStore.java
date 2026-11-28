package app.grapegram;

import android.security.keystore.KeyGenParameterSpec;
import android.security.keystore.KeyProperties;
import android.util.Base64;

import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.security.KeyStore;

import javax.crypto.Cipher;
import javax.crypto.KeyGenerator;
import javax.crypto.SecretKey;
import javax.crypto.spec.GCMParameterSpec;

/**
 * Encrypts AI provider keys before they are persisted in the AI service JSON.
 * AES-256-GCM keys are generated and kept by Android Keystore; the key material
 * never enters app preferences or the grapeGram source tree.
 */
public final class GrapeSecretStore {
    private static final String ALIAS = "grapegram.ai.provider.keys.v1";
    private static final String PREFIX = "ggk1:";
    private static final int IV_BYTES = 12;
    private static final int TAG_BITS = 128;

    private GrapeSecretStore() {
    }

    public static String encryptJsonKeys(String rawJson) {
        return transform(rawJson, true);
    }

    public static String decryptJsonKeys(String rawJson) {
        return transform(rawJson, false);
    }

    private static String transform(String rawJson, boolean encrypt) {
        try {
            JSONArray services = new JSONArray(rawJson);
            for (int i = 0; i < services.length(); i++) {
                JSONObject service = services.optJSONObject(i);
                if (service == null || !service.has("key") || service.isNull("key")) {
                    continue;
                }
                String value = service.optString("key", "");
                if (value.isEmpty()) {
                    continue;
                }
                if (encrypt) {
                    if (!value.startsWith(PREFIX)) {
                        service.put("key", PREFIX + encrypt(value));
                    }
                } else if (value.startsWith(PREFIX)) {
                    service.put("key", decrypt(value.substring(PREFIX.length())));
                }
            }
            return services.toString();
        } catch (Exception e) {
            throw new IllegalStateException("Cannot protect AI provider keys", e);
        }
    }

    private static synchronized SecretKey getOrCreateKey() throws Exception {
        KeyStore store = KeyStore.getInstance("AndroidKeyStore");
        store.load(null);
        java.security.Key existing = store.getKey(ALIAS, null);
        if (existing instanceof SecretKey) {
            return (SecretKey) existing;
        }
        KeyGenerator generator = KeyGenerator.getInstance(
                KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore");
        generator.init(new KeyGenParameterSpec.Builder(ALIAS,
                KeyProperties.PURPOSE_ENCRYPT | KeyProperties.PURPOSE_DECRYPT)
                .setKeySize(256)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setRandomizedEncryptionRequired(true)
                .setUserAuthenticationRequired(false)
                .build());
        return generator.generateKey();
    }

    private static String encrypt(String plaintext) throws Exception {
        Cipher cipher = Cipher.getInstance("AES/GCM/NoPadding");
        cipher.init(Cipher.ENCRYPT_MODE, getOrCreateKey());
        byte[] iv = cipher.getIV();
        byte[] encrypted = cipher.doFinal(plaintext.getBytes(StandardCharsets.UTF_8));
        ByteBuffer payload = ByteBuffer.allocate(iv.length + encrypted.length);
        payload.put(iv);
        payload.put(encrypted);
        return Base64.encodeToString(payload.array(), Base64.NO_WRAP);
    }

    private static String decrypt(String encoded) throws Exception {
        byte[] payload = Base64.decode(encoded, Base64.NO_WRAP);
        if (payload.length <= IV_BYTES) {
            throw new IllegalArgumentException("Invalid encrypted AI key");
        }
        ByteBuffer buffer = ByteBuffer.wrap(payload);
        byte[] iv = new byte[IV_BYTES];
        buffer.get(iv);
        byte[] encrypted = new byte[buffer.remaining()];
        buffer.get(encrypted);
        Cipher cipher = Cipher.getInstance("AES/GCM/NoPadding");
        cipher.init(Cipher.DECRYPT_MODE, getOrCreateKey(), new GCMParameterSpec(TAG_BITS, iv));
        return new String(cipher.doFinal(encrypted), StandardCharsets.UTF_8);
    }
}
