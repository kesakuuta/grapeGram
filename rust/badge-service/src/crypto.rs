use std::env;

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

use crate::error::{bad_request, internal, ApiError};

const PEM_BEGIN: &str = "-----BEGIN PRIVATE KEY-----";
const PEM_END: &str = "-----END PRIVATE KEY-----";

pub fn signing_key_from_env() -> Result<SigningKey, ApiError> {
    let pem = env::var("GRP_SIGNING_KEY_PEM").unwrap_or_default();
    if pem.trim().is_empty() {
        return Err(internal("signing key is not configured"));
    }
    signing_key_from_pem(&pem.replace("\\n", "\n"))
}

pub fn signing_key_from_pem(pem: &str) -> Result<SigningKey, ApiError> {
    let start = pem
        .find(PEM_BEGIN)
        .ok_or_else(|| bad_request("PEM private key header not found"))?
        + PEM_BEGIN.len();
    let end = pem
        .find(PEM_END)
        .ok_or_else(|| bad_request("PEM private key footer not found"))?;
    if end < start {
        return Err(bad_request("invalid PEM private key"));
    }
    let body: String = pem[start..end]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let der = STANDARD.decode(body)?;
    if der.len() < 32 || der.first() != Some(&0x30) {
        return Err(bad_request("invalid PKCS8 Ed25519 key"));
    }
    let oid: &[u8] = &[0x06, 0x03, 0x2b, 0x65, 0x70];
    if !der.windows(5).any(|window| window == oid) {
        return Err(bad_request("key is not Ed25519"));
    }
    let seed: [u8; 32] = der[der.len() - 32..]
        .try_into()
        .map_err(|_| bad_request("invalid key seed"))?;
    Ok(SigningKey::from_bytes(&seed))
}

pub fn sign(payload: &[u8], key: &SigningKey) -> String {
    let signature: Signature = key.sign(payload);
    STANDARD.encode(signature.to_bytes())
}

pub fn verifying_key_from_pem(pem: &str) -> Result<VerifyingKey, ApiError> {
    let begin = "-----BEGIN PUBLIC KEY-----";
    let end = "-----END PUBLIC KEY-----";
    let start = pem.find(begin)
        .ok_or_else(|| internal("public key PEM header not found"))? + begin.len();
    let finish = pem.find(end)
        .ok_or_else(|| internal("public key PEM footer not found"))?;
    if finish < start {
        return Err(internal("invalid public key PEM"));
    }
    let body: String = pem[start..finish].chars().filter(|c| !c.is_whitespace()).collect();
    let der = STANDARD.decode(body).map_err(|_| internal("invalid public key base64"))?;
    let oid: &[u8] = &[0x06, 0x03, 0x2b, 0x65, 0x70];
    if der.len() < 32 || !der.windows(5).any(|window| window == oid) {
        return Err(internal("public key is not Ed25519"));
    }
    let raw: [u8; 32] = der[der.len() - 32..]
        .try_into()
        .map_err(|_| internal("invalid Ed25519 public key length"))?;
    VerifyingKey::from_bytes(&raw).map_err(|_| internal("invalid Ed25519 public key"))
}

pub fn verify(payload: &[u8], signature_b64: &str, key: &VerifyingKey) -> Result<(), ApiError> {
    let bytes = STANDARD.decode(signature_b64.trim())?;
    let array: [u8; 64] = bytes
        .try_into()
        .map_err(|_| bad_request("invalid signature length"))?;
    key.verify(payload, &Signature::from_bytes(&array))
        .map_err(|_| bad_request("invalid signature"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_sign_verify_and_tamper() {
        let value = serde_json::json!({"b": 2, "a": "привет"});
        let payload = crate::canonical::canonical_bytes(&value).unwrap();
        let seed = [0x11u8; 32];
        let key = SigningKey::from_bytes(&seed);
        let signature = sign(&payload, &key);
        assert!(verify(&payload, &signature, &key.verifying_key()).is_ok());

        let mut tampered = payload.clone();
        tampered[0] ^= 0x01;
        assert!(verify(&tampered, &signature, &key.verifying_key()).is_err());

        let other = SigningKey::from_bytes(&[0x22u8; 32]);
        assert!(verify(&payload, &signature, &other.verifying_key()).is_err());
    }

    #[test]
    fn parses_pkcs8_pem_seed() {
        let seed = [0x42u8; 32];
        let mut der = vec![
            0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22,
            0x04, 0x20,
        ];
        der.extend_from_slice(&seed);
        let body = STANDARD.encode(der);
        let pem = format!(
            "{PEM_BEGIN}\n{}\n{PEM_END}\n",
            body.as_bytes()
                .chunks(64)
                .map(|c| std::str::from_utf8(c).unwrap().to_string())
                .collect::<Vec<String>>()
                .join("\n")
        );
        let key = signing_key_from_pem(&pem).unwrap();
        assert_eq!(key.to_bytes(), seed);
    }

    #[test]
    fn rejects_pem_without_ed25519_oid() {
        let mut der = vec![0x30, 0x04, 0x02, 0x01, 0x00];
        der.extend_from_slice(&[0u8; 32]);
        let pem = format!("{PEM_BEGIN}\n{}\n{PEM_END}\n", STANDARD.encode(der));
        assert!(signing_key_from_pem(&pem).is_err());
    }
}
