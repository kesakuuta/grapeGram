mod canonical;
mod crypto;
mod pem;
mod timefmt;

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

const BADGES: [&str; 4] = ["grapegram", "founder", "developer", "contributor"];
const PRIVATE_FILE: &str = "grapegram_ed25519_private.pem";
const PUBLIC_FILE: &str = "grapegram_ed25519.pub.pem";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match run(&args) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            1
        }
    };
    if code != 0 {
        std::process::exit(code);
    }
}

fn run(args: &[String]) -> Res<i32> {
    if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h" | "help") {
        usage();
        return Ok(0);
    }
    let (opts, positional) = parse_args(&args[1..])?;
    match args[0].as_str() {
        "gen-keys" => cmd_gen_keys(&opts),
        "sign" => cmd_sign(&opts, &positional),
        "verify" => cmd_verify(&opts, &positional),
        "apply-change" => cmd_apply_change(&opts),
        other => Err(format!("unknown subcommand: {other}").into()),
    }
}

fn usage() {
    println!(
        "grape-badge-tools\n\
         \n\
         Usage:\n\
         \x20 grape-badge-tools gen-keys --out DIR\n\
         \x20 grape-badge-tools sign --key PRIVATE_PEM --out DIR JSON...\n\
         \x20 grape-badge-tools verify --pub PUB_PEM JSON SIGFILE\n\
         \x20 grape-badge-tools apply-change --root DIR\n\
         \n\
         apply-change reads ACTION, USER_ID, BADGE, REASON, OWNER_ID,\n\
         GRP_SIGNING_KEY_PEM and GITHUB_ACTOR from the environment."
    );
}

fn parse_args(args: &[String]) -> Res<(HashMap<String, String>, Vec<String>)> {
    let mut opts = HashMap::new();
    let mut positional = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let argument = &args[index];
        if let Some(rest) = argument.strip_prefix("--") {
            let (name, value) = match rest.split_once('=') {
                Some((name, value)) => (name, value.to_string()),
                None => {
                    index += 1;
                    if index >= args.len() {
                        return Err(format!("missing value for --{rest}").into());
                    }
                    (rest, args[index].clone())
                }
            };
            if name.is_empty() {
                return Err("empty option name".into());
            }
            if opts.insert(name.to_string(), value).is_some() {
                return Err(format!("duplicate option --{name}").into());
            }
        } else {
            positional.push(argument.clone());
        }
        index += 1;
    }
    Ok((opts, positional))
}

fn cmd_gen_keys(opts: &HashMap<String, String>) -> Res<i32> {
    let out = opts.get("out").ok_or("--out is required")?;
    let directory = Path::new(out);
    fs::create_dir_all(directory)?;
    fs::set_permissions(directory, fs::Permissions::from_mode(0o700))?;
    let private_path = directory.join(PRIVATE_FILE);
    let public_path = directory.join(PUBLIC_FILE);
    if private_path.exists() {
        return Err(format!("refusing to overwrite {}", private_path.display()).into());
    }
    if public_path.exists() {
        return Err(format!("refusing to overwrite {}", public_path.display()).into());
    }
    let seed = random_seed()?;
    let public = crypto::public_from_seed(&seed);
    write_new(&private_path, pem::encode_private(&seed).as_bytes(), 0o600)?;
    write_new(&public_path, pem::encode_public(&public).as_bytes(), 0o644)?;
    println!(
        "private: {}\npublic:  {}",
        private_path.display(),
        public_path.display()
    );
    println!("Commit only the public key; keep the private key in GRP_SIGNING_KEY_PEM.");
    Ok(0)
}

fn cmd_sign(opts: &HashMap<String, String>, files: &[String]) -> Res<i32> {
    let key_path = opts.get("key").ok_or("--key is required")?;
    let out = opts.get("out").ok_or("--out is required")?;
    if files.is_empty() {
        return Err("sign: at least one JSON file is required".into());
    }
    let seed = pem::private_seed(&fs::read_to_string(key_path)?)?;
    fs::create_dir_all(out)?;
    for file in files {
        let source = fs::read(file)?;
        let document: Value = serde_json::from_slice(&source)?;
        let payload = canonical::canonical_bytes(&document)
            .map_err(|error| format!("{file}: {error}"))?;
        let signature = crypto::sign(&payload, &seed);
        let name = Path::new(file)
            .file_name()
            .ok_or("input path has no file name")?;
        let destination = Path::new(out).join(format!("{}.sig", name.to_string_lossy()));
        let encoded = format!("{}\n", STANDARD.encode(signature));
        atomic_write(&destination, encoded.as_bytes())?;
        println!(
            "{file} -> {} ({} bytes, sha256:{})",
            destination.display(),
            signature.len(),
            hex(&Sha256::digest(&payload))
        );
    }
    Ok(0)
}

fn cmd_verify(opts: &HashMap<String, String>, files: &[String]) -> Res<i32> {
    let public_path = opts.get("pub").ok_or("--pub is required")?;
    if files.len() != 2 {
        return Err("usage: verify --pub PUB_PEM JSON SIGFILE".into());
    }
    let public = pem::public_bytes(&fs::read_to_string(public_path)?)?;
    let raw = fs::read(&files[0])?;
    let document: Value = serde_json::from_slice(&raw)?;
    let payload = canonical::canonical_bytes(&document)?;
    let signature_text = fs::read_to_string(&files[1])?;
    let decoded = STANDARD.decode(signature_text.trim())?;
    let signature: [u8; 64] = decoded
        .try_into()
        .map_err(|_| "signature must be exactly 64 bytes")?;
    let valid = crypto::verify(&payload, &public, &signature)
        || crypto::verify(&raw, &public, &signature);
    println!("sha256: {}", hex(&Sha256::digest(&payload)));
    println!("{}", if valid { "VALID" } else { "INVALID" });
    Ok(if valid { 0 } else { 1 })
}

fn cmd_apply_change(opts: &HashMap<String, String>) -> Res<i32> {
    let root = opts.get("root").ok_or("--root is required")?;
    let action = std::env::var("ACTION").unwrap_or_default();
    let id_raw = std::env::var("USER_ID").unwrap_or_default();
    let user_id = id_raw.trim();
    let badge_raw = std::env::var("BADGE").unwrap_or_default();
    let badge = badge_raw.trim().to_string();
    let reason = std::env::var("REASON").unwrap_or_default();
    let owner_id = std::env::var("OWNER_ID").unwrap_or_default().trim().to_string();
    let pem_text = std::env::var("GRP_SIGNING_KEY_PEM").unwrap_or_default();
    if action != "grant" && action != "revoke" {
        return Err("bad action".into());
    }
    if !valid_id(user_id) {
        return Err("bad user id".into());
    }
    if !BADGES.contains(&badge.as_str()) {
        return Err("bad badge".into());
    }
    if pem_text.is_empty() {
        return Err("GRP_SIGNING_KEY_PEM is not set".into());
    }
    let seed = pem::private_seed(&pem_text)?;

    let data = Path::new(root).join("data");
    let owners_document: Value =
        serde_json::from_slice(&fs::read(data.join("owner_users.json"))?)?;
    let owners = owners_document
        .get("owners")
        .and_then(Value::as_array)
        .ok_or("bad owner_users.json")?;
    let owner_list: Vec<String> = owners
        .iter()
        .map(|value| {
            value.as_str()
                .map(str::to_string)
                .ok_or("owner_users.json contains a non-string owner ID")
        })
        .collect::<Result<Vec<_>, _>>()?;
    if owner_list.len() != 2
            || owner_list.iter().any(|owner| !valid_id(owner))
            || owner_list[0] == owner_list[1] {
        return Err("owner_users.json must contain exactly two unique numeric IDs".into());
    }
    // OWNER_ID must come from the authenticated Telegram/admin caller.
    // Never allow an omitted identity to bypass the owner allow-list.
    if !is_authorized_owner(&owner_id, &owner_list) {
        return Err("OWNER_ID is missing or is not in owner_users.json".into());
    }

    let path = data.join("badge_users.json");
    let mut document: Value = serde_json::from_slice(&fs::read(&path)?)?;
    let now = timefmt::now_rfc3339();
    {
        let top = document
            .as_object_mut()
            .ok_or("badge_users.json must be an object")?;
        let badges = top
            .get_mut("badges")
            .and_then(Value::as_object_mut)
            .ok_or("badges must be an object")?;
        let entry = badges
            .entry(user_id.to_string())
            .or_insert_with(|| json!({"record_version": 0}));
        let entry = entry.as_object_mut().ok_or("bad badge record")?;
        let previous = entry
            .get("record_version")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        entry.insert("badge".into(), Value::String(badge.clone()));
        entry.insert("enabled".into(), json!(action == "grant"));
        entry.insert("updated_at".into(), Value::String(now.clone()));
        entry.insert("record_version".into(), json!(previous + 1));
        if action == "grant" && !entry.contains_key("granted_at") {
            entry.insert("granted_at".into(), Value::String(now.clone()));
        }
    }
    let version = {
        let top = document
            .as_object_mut()
            .ok_or("badge_users.json must be an object")?;
        let current = top
            .get("version")
            .and_then(Value::as_i64)
            .ok_or("bad version")?;
        top.insert("version".into(), json!(current + 1));
        top.insert("updated_at".into(), Value::String(now.clone()));
        current + 1
    };

    let payload = canonical::canonical_bytes(&document)?;
    atomic_write(&path, &payload)?;
    let signature = crypto::sign(&payload, &seed);
    let signature_directory = data.join("signatures");
    fs::create_dir_all(&signature_directory)?;
    let signature_path = signature_directory.join("badge_users.json.sig");
    let encoded = format!("{}\n", STANDARD.encode(signature));
    atomic_write(&signature_path, encoded.as_bytes())?;

    let actor = owner_id.clone();
    let record = json!({
        "action": &action,
        "target": user_id,
        "badge": badge.clone(),
        "reason": reason,
        "version": version,
        "at": now,
        "by": actor,
        "owners_configured": owner_list,
        "payload_sha256": hex(&Sha256::digest(&payload)),
    });
    let mut audit = OpenOptions::new()
        .create(true)
        .append(true)
        .open(data.join("audit.jsonl"))?;
    writeln!(audit, "{}", serde_json::to_string(&record)?)?;
    println!("{action} {badge} -> {user_id}, version {version}");
    Ok(0)
}

fn is_authorized_owner(owner_id: &str, owners: &[String]) -> bool {
    valid_id(owner_id) && owners.iter().any(|owner| owner == owner_id)
}

fn valid_id(id: &str) -> bool {
    (5..=20).contains(&id.len()) && id.bytes().all(|byte| byte.is_ascii_digit())
}

fn random_seed() -> Res<[u8; 32]> {
    let mut source = File::open("/dev/urandom")?;
    let mut seed = [0u8; 32];
    source.read_exact(&mut seed)?;
    Ok(seed)
}

fn write_new(path: &Path, contents: &[u8], mode: u32) -> Res<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(path)?;
    file.write_all(contents)?;
    file.sync_all()?;
    Ok(())
}

fn atomic_write(path: &Path, contents: &[u8]) -> Res<()> {
    let temporary = PathBuf::from(format!("{}.tmp", path.display()));
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&temporary)?;
        file.write_all(contents)?;
        file.sync_all()?;
    }
    fs::rename(&temporary, path)?;
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_validation_matches_python_regex() {
        assert!(valid_id("12345"));
        assert!(valid_id("7293365262"));
        assert!(valid_id(&"1".repeat(20)));
        assert!(!valid_id("1234"));
        assert!(!valid_id(&"1".repeat(21)));
        assert!(!valid_id("1234a"));
        assert!(!valid_id("12 45"));
    }

    #[test]
    fn owner_allowlist_rejects_missing_and_unknown_ids() {
        let owners = vec!["7293365262".to_string(), "960792706".to_string()];
        assert!(is_authorized_owner("7293365262", &owners));
        assert!(is_authorized_owner("960792706", &owners));
        assert!(!is_authorized_owner("", &owners));
        assert!(!is_authorized_owner("123456789", &owners));
        assert!(!is_authorized_owner("owner", &owners));
    }

    #[test]
    fn option_parser_handles_both_forms() {
        let args: Vec<String> = ["--out=data", "--root", "/repo", "a.json"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let (opts, positional) = parse_args(&args).unwrap();
        assert_eq!(opts.get("out").unwrap(), "data");
        assert_eq!(opts.get("root").unwrap(), "/repo");
        assert_eq!(positional, vec!["a.json"]);
    }

    #[test]
    fn hex_is_lowercase() {
        assert_eq!(hex(&[0x00, 0x0f, 0xff]), "000fff");
    }
}
