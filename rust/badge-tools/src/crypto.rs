use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

pub fn sign(message: &[u8], seed: &[u8; 32]) -> [u8; 64] {
    SigningKey::from_bytes(seed).sign(message).to_bytes()
}

pub fn verify(message: &[u8], public: &[u8; 32], signature: &[u8; 64]) -> bool {
    match VerifyingKey::from_bytes(public) {
        Ok(key) => key.verify(message, &Signature::from_bytes(signature)).is_ok(),
        Err(_) => false,
    }
}

pub fn public_from_seed(seed: &[u8; 32]) -> [u8; 32] {
    SigningKey::from_bytes(seed).verifying_key().to_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed() -> [u8; 32] {
        [7u8; 32]
    }

    #[test]
    fn sign_verify_roundtrip() {
        let message = b"{\"a\":1,\"b\":2}";
        let signature = sign(message, &seed());
        assert_eq!(signature.len(), 64);
        assert!(verify(message, &public_from_seed(&seed()), &signature));
    }

    #[test]
    fn tampered_message_fails() {
        let signature = sign(b"{\"a\":1}", &seed());
        assert!(!verify(b"{\"a\":2}", &public_from_seed(&seed()), &signature));
    }

    #[test]
    fn wrong_public_key_fails() {
        let signature = sign(b"payload", &seed());
        assert!(!verify(b"payload", &[9u8; 32], &signature));
    }

    #[test]
    fn deterministic_for_same_seed_and_message() {
        assert_eq!(sign(b"payload", &seed()), sign(b"payload", &seed()));
    }
}
