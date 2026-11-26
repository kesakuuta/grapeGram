use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;

const PRIVATE_LABEL: &str = "PRIVATE KEY";
const PUBLIC_LABEL: &str = "PUBLIC KEY";

const PKCS8_PREFIX: [u8; 16] = [
    0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04,
    0x20,
];
const SPKI_PREFIX: [u8; 12] = [
    0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
];

pub fn private_seed(pem: &str) -> Result<[u8; 32], String> {
    let der = decode(pem, PRIVATE_LABEL)?;
    if der.len() != 48 || der[..16] != PKCS8_PREFIX {
        return Err("expected an unencrypted PKCS#8 Ed25519 key".to_string());
    }
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&der[16..]);
    Ok(seed)
}

pub fn public_bytes(pem: &str) -> Result<[u8; 32], String> {
    let der = decode(pem, PUBLIC_LABEL)?;
    if der.len() != 44 || der[..12] != SPKI_PREFIX {
        return Err("expected an SPKI Ed25519 public key".to_string());
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&der[12..]);
    Ok(bytes)
}

pub fn encode_private(seed: &[u8; 32]) -> String {
    let mut der = Vec::with_capacity(48);
    der.extend_from_slice(&PKCS8_PREFIX);
    der.extend_from_slice(seed);
    armor(PRIVATE_LABEL, &der)
}

pub fn encode_public(bytes: &[u8; 32]) -> String {
    let mut der = Vec::with_capacity(44);
    der.extend_from_slice(&SPKI_PREFIX);
    der.extend_from_slice(bytes);
    armor(PUBLIC_LABEL, &der)
}

fn decode(pem: &str, label: &str) -> Result<Vec<u8>, String> {
    let begin = format!("-----BEGIN {label}-----");
    let end = format!("-----END {label}-----");
    let mut inside = false;
    let mut body = String::new();
    for line in pem.lines() {
        let line = line.trim();
        if line == begin {
            if inside {
                return Err("duplicate PEM begin marker".to_string());
            }
            inside = true;
        } else if line == end {
            if !inside {
                return Err("PEM end marker without begin marker".to_string());
            }
            inside = false;
        } else if inside {
            if line.is_empty() {
                continue;
            }
            if line.contains(':') || line.starts_with("Proc-Type") || line.starts_with("DEK-Info") {
                return Err("encrypted or headered PEM keys are not supported".to_string());
            }
            if line.starts_with("-----") {
                return Err("unexpected PEM marker".to_string());
            }
            body.push_str(line);
        }
    }
    if inside {
        return Err("missing PEM end marker".to_string());
    }
    if body.is_empty() {
        return Err("empty PEM body".to_string());
    }
    STANDARD.decode(&body).map_err(|error| format!("invalid base64 in PEM: {error}"))
}

fn armor(label: &str, der: &[u8]) -> String {
    let encoded = STANDARD.encode(der);
    let mut out = format!("-----BEGIN {label}-----\n");
    for chunk in encoded.as_bytes().chunks(64) {
        out.push_str(std::str::from_utf8(chunk).expect("base64 is ascii"));
        out.push('\n');
    }
    out.push_str(&format!("-----END {label}-----\n"));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_seed() -> [u8; 32] {
        [7u8; 32]
    }

    #[test]
    fn private_pem_roundtrip() {
        let seed = dummy_seed();
        let pem = encode_private(&seed);
        assert!(pem.contains(PRIVATE_LABEL));
        assert_eq!(private_seed(&pem).unwrap(), seed);
    }

    #[test]
    fn public_pem_roundtrip() {
        let bytes = [11u8; 32];
        let pem = encode_public(&bytes);
        assert!(pem.contains(PUBLIC_LABEL));
        assert_eq!(public_bytes(&pem).unwrap(), bytes);
    }

    #[test]
    fn public_pem_from_repository_layout_is_accepted() {
        let pem = "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEA0NUFLJbh/MBnvBcgdZ+0g4Rix9ioDcnyg1jSkLuuHfo=\n-----END PUBLIC KEY-----\n";
        assert_eq!(public_bytes(pem).unwrap().len(), 32);
    }

    #[test]
    fn rejects_wrong_label_and_encrypted_headers() {
        let pem = encode_public(&[1u8; 32]);
        assert!(private_seed(&pem).is_err());
        let encrypted = pem.replace(
            &format!("-----BEGIN {PUBLIC_LABEL}-----"),
            &format!("-----BEGIN {PUBLIC_LABEL}-----\nProc-Type: 4,ENCRYPTED"),
        );
        assert!(public_bytes(&encrypted).is_err());
    }
}
