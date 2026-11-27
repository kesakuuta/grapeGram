use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD};
use base64::Engine as _;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::jboolean;
use jni::JNIEnv;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[no_mangle]
pub extern "system" fn Java_app_grapegram_GrapeBadges_nativeVerify(
    mut env: JNIEnv,
    _class: JClass,
    payload: JByteArray,
    sig_b64: JString,
    pub_pem: JString,
) -> jboolean {
    let ok = catch_unwind(AssertUnwindSafe(|| {
        let data = match env.convert_byte_array(payload) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };
        let sig_java = match env.get_string(&sig_b64) {
            Ok(value) => value,
            Err(_) => return false,
        };
        let pem_java = match env.get_string(&pub_pem) {
            Ok(value) => value,
            Err(_) => return false,
        };
        let sig_text: String = sig_java.into();
        let pem_text: String = pem_java.into();
        let signature = match decode_base64(sig_text.trim()) {
            Some(bytes) => bytes,
            None => return false,
        };
        let public = match public_key_bytes(&pem_text) {
            Some(bytes) => bytes,
            None => return false,
        };
        verify_bytes(&data, &signature, &public)
    }));
    if matches!(ok, Ok(true)) {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "system" fn Java_app_grapegram_GrapeBadges_nativeAvailable(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    1
}

fn verify_bytes(payload: &[u8], signature: &[u8], public: &[u8; 32]) -> bool {
    if signature.len() != 64 {
        return false;
    }
    let mut sig_bytes = [0u8; 64];
    sig_bytes.copy_from_slice(signature);
    match VerifyingKey::from_bytes(public) {
        Ok(key) => key.verify(payload, &Signature::from_bytes(&sig_bytes)).is_ok(),
        Err(_) => false,
    }
}

fn decode_base64(text: &str) -> Option<Vec<u8>> {
    if let Ok(bytes) = STANDARD.decode(text) {
        return Some(bytes);
    }
    STANDARD_NO_PAD.decode(text).ok()
}

fn public_key_bytes(pem: &str) -> Option<[u8; 32]> {
    let cleaned: String = pem
        .replace("-----BEGIN PUBLIC KEY-----", "")
        .replace("-----END PUBLIC KEY-----", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let der = decode_base64(&cleaned)?;
    if der.len() < 32 {
        return None;
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&der[der.len() - 32..]);
    Some(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn seed() -> [u8; 32] {
        [7u8; 32]
    }

    fn public_pem(public: &[u8; 32]) -> String {
        let mut der = vec![
            0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
        ];
        der.extend_from_slice(public);
        format!(
            "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
            STANDARD.encode(&der)
        )
    }

    #[test]
    fn sign_verify_roundtrip() {
        let signing = SigningKey::from_bytes(&seed());
        let public = signing.verifying_key().to_bytes();
        let payload = b"{\"badges\":{\"123\":{\"badge\":\"grapegram\",\"enabled\":true}},\"version\":1}";
        let signature = signing.sign(payload).to_bytes();
        assert!(verify_bytes(payload, &signature, &public));
        assert_eq!(public_key_bytes(&public_pem(&public)), Some(public));
        let signature_b64 = STANDARD.encode(signature);
        assert_eq!(decode_base64(&signature_b64), Some(signature.to_vec()));
    }

    #[test]
    fn bad_signature_is_rejected() {
        let signing = SigningKey::from_bytes(&seed());
        let public = signing.verifying_key().to_bytes();
        let signature = signing.sign(b"original").to_bytes();
        assert!(!verify_bytes(b"tampered", &signature, &public));
        assert!(!verify_bytes(b"original", &signature[..63], &public));
        let mut flipped = signature;
        flipped[0] ^= 0x01;
        assert!(!verify_bytes(b"original", &flipped, &public));
    }

    #[test]
    fn signature_without_padding_is_accepted() {
        let signature = [3u8; 64];
        let padded = STANDARD.encode(signature);
        let unpadded = padded.trim_end_matches('=');
        assert_eq!(decode_base64(unpadded), Some(signature.to_vec()));
    }

    #[test]
    fn malformed_pem_and_short_der_are_rejected() {
        assert_eq!(public_key_bytes("not a pem"), None);
        assert_eq!(public_key_bytes(&STANDARD.encode([1u8; 10])), None);
    }
}
