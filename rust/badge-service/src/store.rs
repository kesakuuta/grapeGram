use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use serde_json::{json, Value};

use crate::canonical::{json_line, pretty_bytes};
use crate::crypto::{sign, signing_key_from_env};
use crate::error::{bad_request, forbidden, internal, ApiError};

const ALLOWED_BADGES: [&str; 4] = ["grapegram", "founder", "developer", "contributor"];
const CHALLENGE_TTL: f64 = 300.0;
const SESSION_TTL: f64 = 3600.0;

struct Challenge {
    expires: f64,
    used: bool,
    poll_token: String,
    session: Option<String>,
}

struct Session {
    owner: String,
    expires: f64,
}

struct Inner {
    challenges: HashMap<String, Challenge>,
    sessions: HashMap<String, Session>,
}

pub struct Store {
    data_dir: PathBuf,
    inner: Mutex<Inner>,
}

fn now_seconds() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

fn random_bytes(length: usize) -> Result<Vec<u8>, ApiError> {
    let mut file = File::open("/dev/urandom")?;
    let mut bytes = vec![0u8; length];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn random_hex(length: usize) -> Result<String, ApiError> {
    let bytes = random_bytes(length)?;
    Ok(bytes.iter().map(|b| format!("{b:02X}")).collect())
}

fn random_token() -> Result<String, ApiError> {
    Ok(URL_SAFE_NO_PAD.encode(random_bytes(32)?))
}

fn id_is_valid(id: &str) -> bool {
    (5..=20).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_digit())
}

fn validate_style(style: &Value) -> Result<Value, ApiError> {
    let object = style
        .as_object()
        .ok_or_else(|| bad_request("style must be an object"))?;
    let mut clean = serde_json::Map::new();
    for (key, value) in object {
        match key.as_str() {
            "color_light" | "color_dark" => {
                let color = value
                    .as_str()
                    .ok_or_else(|| bad_request("badge color must be a string"))?;
                let bytes = color.as_bytes();
                if (bytes.len() != 7 && bytes.len() != 9)
                    || bytes.first() != Some(&b'#')
                    || !bytes[1..].iter().all(u8::is_ascii_hexdigit)
                {
                    return Err(bad_request("invalid badge color: expected #RRGGBB or #AARRGGBB"));
                }
                clean.insert(key.clone(), Value::String(color.to_ascii_uppercase()));
            }
            "alpha" => {
                let alpha = value
                    .as_f64()
                    .ok_or_else(|| bad_request("alpha must be a number between 0 and 1"))?;
                if !(0.0..=1.0).contains(&alpha) {
                    return Err(bad_request("alpha must be a number between 0 and 1"));
                }
                clean.insert(key.clone(), json!(alpha));
            }
            "particles" => {
                let particles = value
                    .as_bool()
                    .ok_or_else(|| bad_request("particles must be boolean"))?;
                clean.insert(key.clone(), Value::Bool(particles));
            }
            "animation" => {
                let animation = value
                    .as_str()
                    .ok_or_else(|| bad_request("animation must be a string"))?;
                if !["none", "pulse", "sparkle", "glow"].contains(&animation) {
                    return Err(bad_request("unknown badge animation"));
                }
                clean.insert(key.clone(), Value::String(animation.to_string()));
            }
            _ => return Err(bad_request("unknown badge style field")),
        }
    }
    Ok(Value::Object(clean))
}

fn timestamp() -> String {
    let seconds = now_seconds() as i64;
    let days = seconds.div_euclid(86_400);
    let daytime = seconds.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = if month <= 2 { year + 1 } else { year };
    format!(
        "{year:04}-{:02}-{day:02}T{:02}:{:02}:{:02}Z",
        month,
        daytime / 3600,
        (daytime % 3600) / 60,
        daytime % 60
    )
}

impl Store {
    pub fn new(data_dir: PathBuf) -> Self {
        Store {
            data_dir,
            inner: Mutex::new(Inner {
                challenges: HashMap::new(),
                sessions: HashMap::new(),
            }),
        }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Inner>, ApiError> {
        self.inner
            .lock()
            .map_err(|_| internal("store lock is poisoned"))
    }

    pub fn owners(&self) -> Result<Vec<String>, ApiError> {
        let path = self.data_dir.join("owner_users.json");
        let doc: Value = serde_json::from_slice(&fs::read(path)?)?;
        let owners = doc
            .get("owners")
            .and_then(Value::as_array)
            .ok_or_else(|| internal("owner_users.json has no owners array"))?;
        Ok(owners
            .iter()
            .filter_map(|value| value.as_str().map(str::to_string))
            .collect())
    }

    pub fn new_challenge(&self) -> Result<String, ApiError> {
        self.new_challenge_pair().map(|(code, _)| code)
    }

    pub fn new_challenge_pair(&self) -> Result<(String, String), ApiError> {
        let code = random_hex(4)?;
        let poll_token = random_token()?;
        let mut inner = self.lock()?;
        let now = now_seconds();
        inner.challenges.retain(|_, value| value.expires > now);
        inner.challenges.insert(
            code.clone(),
            Challenge {
                expires: now + CHALLENGE_TTL,
                used: false,
                poll_token: poll_token.clone(),
                session: None,
            },
        );
        Ok((code, poll_token))
    }

    pub fn bind(&self, code: &str, telegram_id: &str) -> Result<String, ApiError> {
        let owners = self.owners()?;
        let token = random_token()?;
        let mut inner = self.lock()?;
        let now = now_seconds();
        let challenge = inner
            .challenges
            .get_mut(code)
            .ok_or_else(|| forbidden("challenge is invalid or expired"))?;
        if challenge.used || challenge.expires < now {
            return Err(forbidden("challenge is invalid or expired"));
        }
        if !owners.iter().any(|owner| owner == telegram_id) {
            return Err(forbidden("not an owner"));
        }
        challenge.used = true;
        challenge.session = Some(token.clone());
        inner.sessions.insert(
            token.clone(),
            Session {
                owner: telegram_id.to_string(),
                expires: now + SESSION_TTL,
            },
        );
        Ok(token)
    }

    pub fn poll_challenge(&self, code: &str, poll_token: &str) -> Result<Option<String>, ApiError> {
        let mut inner = self.lock()?;
        let challenge = inner
            .challenges
            .get(code)
            .ok_or_else(|| forbidden("challenge is invalid or expired"))?;
        if challenge.expires < now_seconds() || challenge.poll_token != poll_token {
            return Err(forbidden("challenge is invalid or expired"));
        }
        let session = challenge.session.clone();
        if session.is_some() {
            inner.challenges.remove(code);
        }
        Ok(session)
    }

    pub fn require_owner(&self, token: &str) -> Result<String, ApiError> {
        let owners = self.owners()?;
        let inner = self.lock()?;
        let session = inner
            .sessions
            .get(token)
            .ok_or_else(|| forbidden("no valid session"))?;
        if session.expires < now_seconds() {
            return Err(forbidden("no valid session"));
        }
        if !owners.iter().any(|owner| owner == &session.owner) {
            return Err(forbidden("not an owner"));
        }
        Ok(session.owner.clone())
    }

    pub fn mutate(
        &self,
        token: &str,
        target: &str,
        badge: &str,
        enabled: bool,
        reason: &str,
        style: Option<&Value>,
    ) -> Result<u64, ApiError> {
        let owner = self.require_owner(token)?;
        if !id_is_valid(target) {
            return Err(bad_request("bad user id"));
        }
        if !ALLOWED_BADGES.contains(&badge) {
            return Err(bad_request("unknown badge"));
        }

        let path = self.data_dir.join("badge_users.json");
        let mut doc: Value = serde_json::from_slice(&fs::read(&path)?)?;
        let now = timestamp();
        let version = doc.get("version").and_then(Value::as_u64).unwrap_or(1) + 1;

        let mut badges = doc.get("badges").cloned().unwrap_or_else(|| json!({}));
        let mut entry = badges
            .get(target)
            .cloned()
            .unwrap_or_else(|| json!({"record_version": 0}));
        let record_version = entry
            .get("record_version")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            + 1;
        let entry_object = entry
            .as_object_mut()
            .ok_or_else(|| internal("badge entry is not an object"))?;
        entry_object.insert("badge".to_string(), Value::String(badge.to_string()));
        entry_object.insert("enabled".to_string(), Value::Bool(enabled));
        entry_object.insert("updated_at".to_string(), Value::String(now.clone()));
        entry_object.insert("record_version".to_string(), json!(record_version));
        if enabled && !entry_object.contains_key("granted_at") {
            entry_object.insert("granted_at".to_string(), Value::String(now.clone()));
        }
        if let Some(style) = style {
            entry_object.insert("style".to_string(), validate_style(style)?);
        }
        let style_record = entry_object.get("style").cloned().unwrap_or_else(|| json!({}));
        badges
            .as_object_mut()
            .ok_or_else(|| internal("badges is not an object"))?
            .insert(target.to_string(), entry);

        let doc_object = doc
            .as_object_mut()
            .ok_or_else(|| internal("badge_users.json is not an object"))?;
        doc_object.insert("badges".to_string(), badges);
        doc_object.insert("version".to_string(), json!(version));
        doc_object.insert("updated_at".to_string(), Value::String(now.clone()));

        write_atomic(&path, &doc)?;
        let payload = fs::read(&path)?;
        let signature = sign(&payload, &signing_key_from_env()?);
        let signatures = self.data_dir.join("signatures");
        fs::create_dir_all(&signatures)?;
        let mut signature_file = File::create(signatures.join("badge_users.json.sig"))?;
        signature_file.write_all(signature.as_bytes())?;
        signature_file.write_all(b"\n")?;

        let action = if enabled { "grant" } else { "revoke" };
        self.audit(&json!({
            "action": action,
            "by": owner,
            "target": target,
            "badge": badge,
            "reason": reason,
            "style": style_record,
            "version": version,
            "at": now
        }))?;
        crate::github::publish_if_configured(
            &self.data_dir,
            &["badge_users.json", "signatures/badge_users.json.sig"],
        );
        Ok(version)
    }

    fn audit(&self, record: &Value) -> Result<(), ApiError> {
        let line = json_line(record)?;
        let path = self.data_dir.join("audit.jsonl");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        file.write_all(&line)?;
        file.write_all(b"\n")?;
        Ok(())
    }
}

fn write_atomic(path: &Path, value: &Value) -> Result<(), ApiError> {
    let temporary = path.with_extension("json.tmp");
    let mut bytes = pretty_bytes(value)?;
    bytes.push(b'\n');
    let mut file = File::create(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::process;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_store(name: &str) -> (Store, PathBuf) {
        let suffix = COUNTER.fetch_add(1, Ordering::Relaxed);
        let directory = env::temp_dir().join(format!(
            "grape-badge-test-{}-{suffix}-{name}",
            process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join("owner_users.json"),
            r#"{"version":1,"owners":["7293365262","960792706"]}"#,
        )
        .unwrap();
        fs::write(
            directory.join("badge_users.json"),
            r#"{"version":1,"updated_at":"2026-01-01T00:00:00Z","badges":{}}"#,
        )
        .unwrap();
        (Store::new(directory.clone()), directory)
    }

    fn cleanup(directory: &Path) {
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn challenge_is_single_use() {
        let (store, directory) = test_store("single-use");
        let code = store.new_challenge().unwrap();
        assert_eq!(code.len(), 8);
        assert!(store.bind(&code, "7293365262").is_ok());
        let second = store.bind(&code, "7293365262");
        assert!(matches!(second, Err(ApiError::Forbidden(_))));
        cleanup(&directory);
    }

    #[test]
    fn app_can_poll_owner_binding_without_exposing_session() {
        let (store, directory) = test_store("poll-binding");
        let (code, poll_token) = store.new_challenge_pair().unwrap();
        assert_eq!(store.poll_challenge(&code, &poll_token).unwrap(), None);
        assert!(matches!(
            store.poll_challenge(&code, "wrong-token"),
            Err(ApiError::Forbidden(_))
        ));
        let session = store.bind(&code, "960792706").unwrap();
        assert_eq!(store.poll_challenge(&code, &poll_token).unwrap(), Some(session));
        assert!(matches!(
            store.poll_challenge(&code, &poll_token),
            Err(ApiError::Forbidden(_))
        ));
        cleanup(&directory);
    }

    #[test]
    fn non_owner_is_rejected_and_challenge_stays_unused() {
        let (store, directory) = test_store("non-owner");
        let code = store.new_challenge().unwrap();
        let rejected = store.bind(&code, "123456789");
        assert!(matches!(rejected, Err(ApiError::Forbidden(_))));
        assert!(store.bind(&code, "960792706").is_ok());
        assert!(matches!(
            store.require_owner("not-a-session"),
            Err(ApiError::Forbidden(_))
        ));
        cleanup(&directory);
    }

    #[test]
    fn expired_challenge_and_session_are_rejected() {
        let (store, directory) = test_store("expiry");
        let code = store.new_challenge().unwrap();
        {
            let mut inner = store.lock().unwrap();
            inner.challenges.get_mut(&code).unwrap().expires = now_seconds() - 1.0;
            inner.sessions.insert(
                "expired-token".to_string(),
                Session {
                    owner: "7293365262".to_string(),
                    expires: now_seconds() - 1.0,
                },
            );
        }
        assert!(matches!(
            store.bind(&code, "7293365262"),
            Err(ApiError::Forbidden(_))
        ));
        assert!(matches!(
            store.require_owner("expired-token"),
            Err(ApiError::Forbidden(_))
        ));
        cleanup(&directory);
    }

    #[test]
    fn mutate_validates_and_persists_badge_style() {
        let (store, directory) = test_store("style");
        {
            let mut inner = store.lock().unwrap();
            inner.sessions.insert(
                "session".to_string(),
                Session {
                    owner: "7293365262".to_string(),
                    expires: now_seconds() + SESSION_TTL,
                },
            );
        }
        let style = json!({
            "color_light": "#AA22CC",
            "color_dark": "#FF112233",
            "alpha": 0.75,
            "particles": true,
            "animation": "sparkle"
        });
        store.mutate("session", "555555555", "founder", true, "style", Some(&style)).unwrap();
        let doc: Value = serde_json::from_slice(&fs::read(directory.join("badge_users.json")).unwrap()).unwrap();
        assert_eq!(doc["badges"]["555555555"]["style"]["color_light"], "#AA22CC");
        assert_eq!(doc["badges"]["555555555"]["style"]["animation"], "sparkle");
        assert!(matches!(
            validate_style(&json!({"color_light": "purple"})),
            Err(ApiError::BadRequest(_))
        ));
        assert!(matches!(
            validate_style(&json!({"alpha": 2.0})),
            Err(ApiError::BadRequest(_))
        ));
        assert!(matches!(
            validate_style(&json!({"animation": "explode"})),
            Err(ApiError::BadRequest(_))
        ));
        cleanup(&directory);
    }

    #[test]
    fn mutate_validates_target_and_badge() {
        let (store, directory) = test_store("validation");
        {
            let mut inner = store.lock().unwrap();
            inner.sessions.insert(
                "session".to_string(),
                Session {
                    owner: "7293365262".to_string(),
                    expires: now_seconds() + SESSION_TTL,
                },
            );
        }
        assert!(matches!(
            store.mutate("session", "123", "grapegram", true, "", None),
            Err(ApiError::BadRequest(_))
        ));
        assert!(matches!(
            store.mutate("session", "1234567", "unknown", true, "", None),
            Err(ApiError::BadRequest(_))
        ));
        assert!(matches!(
            store.mutate("other", "1234567", "grapegram", true, "", None),
            Err(ApiError::Forbidden(_))
        ));
        cleanup(&directory);
    }
}
