use serde::Deserialize;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const MARK: &str = "GrapeIconOriginal";
const PACKAGE: &str = "app.grapegram";
const APP_NAME: &str = "grapeGram";
const ICONS: [(&str, &str); 9] = [
    ("original", "Оригинальная"),
    ("amethyst", "Аметист"),
    ("mosaic", "Мозаика"),
    ("onyx", "Оникс"),
    ("silk", "Лиловый шёлк"),
    ("neon", "Неон"),
    ("orbit", "Орбита"),
    ("graphite", "Графит"),
    ("violet", "Фиалка"),
];

const CHERRY_ICONS: [(&str, &str, &str, &str, &str); 18] = [
    ("cherry", "Cherrygram · Вишня", "R.drawable.icon_background_default", "R.drawable.icon_foreground_cherry", "icon_launcher_cherry"),
    ("dark_cherry", "Cherrygram · Тёмная", "R.drawable.icon_background_dark", "R.drawable.icon_foreground_cherry", "icon_launcher_cherry_dark"),
    ("dark_cherry_bra", "Cherrygram · Тёмная Bra", "R.drawable.icon_background_dark", "R.drawable.icon_foreground_cherry_bra", "icon_launcher_cherry_dark_bra"),
    ("white_cherry", "Cherrygram · Белая", "R.drawable.icon_background_white", "R.drawable.icon_foreground_cherry_white", "icon_launcher_cherry_white"),
    ("white_cherry_bra", "Cherrygram · Белая Bra", "R.drawable.icon_background_white", "R.drawable.icon_foreground_cherry_white_bra", "icon_launcher_cherry_white_bra"),
    ("laguna_cherry", "Cherrygram · Лагуна", "R.drawable.icon_background_laguna", "R.drawable.icon_foreground_cherry", "icon_launcher_cherry_laguna"),
    ("aqua_cherry", "Cherrygram · Аква", "R.drawable.icon_background_aqua", "R.drawable.icon_foreground_cherry", "icon_launcher_cherry_aqua"),
    ("green_cherry", "Cherrygram · Зелёная", "R.drawable.icon_background_green", "R.drawable.icon_foreground_cherry", "icon_launcher_cherry_green"),
    ("lavanda_cherry", "Cherrygram · Лаванда", "R.drawable.icon_background_lavanda", "R.drawable.icon_foreground_cherry", "icon_launcher_cherry_lavanda"),
    ("violet_sunset_cherry", "Cherrygram · Фиолетовый закат", "R.drawable.icon_background_violet_sunset", "R.drawable.icon_foreground_cherry", "icon_launcher_cherry_violet_sunset"),
    ("violet_sunset_cherry_bra", "Cherrygram · Фиолетовый закат Bra", "R.drawable.icon_background_violet_sunset", "R.drawable.icon_foreground_cherry_bra", "icon_launcher_cherry_violet_sunset_bra"),
    ("sunset_cherry", "Cherrygram · Закат", "R.drawable.icon_background_sunset", "R.drawable.icon_foreground_cherry", "icon_launcher_cherry_sunset"),
    ("sunrise_cherry", "Cherrygram · Восход", "R.drawable.icon_background_sunrise", "R.drawable.icon_foreground_cherry", "icon_launcher_cherry_sunrise"),
    ("palms_cherry", "Cherrygram · Пальмы", "R.drawable.icon_background_palms", "R.drawable.icon_foreground_cherry", "icon_launcher_cherry_palms"),
    ("palms_cherry_bra", "Cherrygram · Пальмы Bra", "R.drawable.icon_background_palms", "R.drawable.icon_foreground_cherry_bra", "icon_launcher_cherry_palms_bra"),
    ("turbo_cherry", "Cherrygram · Turbo", "R.drawable.icon_5_background_sa", "R.drawable.icon_foreground_cherry", "icon_launcher_cherry_turbo"),
    ("nox_cherry", "Cherrygram · Nox", "R.mipmap.icon_2_background_sa", "R.drawable.icon_foreground_cherry", "icon_launcher_cherry_nox"),
    ("dark_ny", "Cherrygram · Новогодняя", "R.drawable.icon_background_dark_ny", "R.drawable.icon_foreground_cherry_bra", "icon_launcher_cherry_dark_ny"),
];

const EXTERA_ICONS: [(&str, &str, &str, &str); 31] = [
    ("winter", "exteraGram · Winter", "R.mipmap.ic_launcher_winter_background", "R.mipmap.ic_launcher_foreground"),
    ("monet", "exteraGram · Monet", "R.drawable.ic_launcher_bg_monet", "R.drawable.ic_foreground_monet"),
    ("orbit", "exteraGram · Orbit", "R.drawable.ic_launcher_bg_default", "R.mipmap.ic_launcher_orbit_foreground"),
    ("editor", "exteraGram · Editor", "R.mipmap.ic_launcher_editor_background", "R.mipmap.ic_launcher_editor_foreground"),
    ("mole", "exteraGram · Mole", "R.mipmap.ic_launcher_mole_background", "R.mipmap.ic_launcher_mole_foreground"),
    ("echpochmak", "exteraGram · Echpochmak", "R.mipmap.ic_launcher_echpochmak_background", "R.mipmap.ic_launcher_echpochmak_foreground"),
    ("avatar", "exteraGram · Avatar", "R.mipmap.ic_launcher_avatar_background", "R.mipmap.ic_launcher_avatar_foreground"),
    ("candy", "exteraGram · Candy", "R.mipmap.ic_launcher_candy_background", "R.mipmap.ic_launcher_candy_foreground"),
    ("guitar", "exteraGram · Guitar", "R.mipmap.ic_launcher_guitar_background", "R.mipmap.ic_launcher_guitar_foreground"),
    ("sus", "exteraGram · Sus", "R.drawable.ic_launcher_bg_sus", "R.mipmap.ic_launcher_sus_foreground"),
    ("frog", "exteraGram · Frog", "R.mipmap.ic_launcher_frog_background", "R.mipmap.ic_launcher_frog_foreground"),
    ("hamster", "exteraGram · Hamster", "R.mipmap.ic_launcher_hamster_background", "R.mipmap.ic_launcher_hamster_foreground"),
    ("sponge", "exteraGram · Sponge", "R.mipmap.ic_launcher_sponge_background", "R.mipmap.ic_launcher_sponge_foreground"),
    ("invincible", "exteraGram · Invincible", "R.mipmap.ic_launcher_invincible_background", "R.mipmap.ic_launcher_invincible_foreground"),
    ("flappy", "exteraGram · Flappy", "R.mipmap.ic_launcher_flappy_background", "R.mipmap.ic_launcher_flappy_foreground"),
    ("nyan_cat", "exteraGram · Nyan Cat", "R.mipmap.ic_launcher_nyan_cat_background", "R.mipmap.ic_launcher_nyan_cat_foreground"),
    ("ice_age", "exteraGram · Ice Age", "R.mipmap.ic_launcher_ice_age_background", "R.mipmap.ic_launcher_ice_age_foreground"),
    ("fish", "exteraGram · Fish", "R.mipmap.ic_launcher_fish_background", "R.mipmap.ic_launcher_fish_foreground"),
    ("psychosis", "exteraGram · Psychosis", "R.mipmap.ic_launcher_psychosis_background", "R.mipmap.ic_launcher_psychosis_foreground"),
    ("xp", "exteraGram · XP", "R.mipmap.ic_launcher_xp_background", "R.mipmap.ic_launcher_xp_foreground"),
    ("nasa", "exteraGram · NASA", "R.mipmap.ic_launcher_nasa_background", "R.mipmap.ic_launcher_nasa_foreground"),
    ("cyberpunk", "exteraGram · Cyberpunk", "R.drawable.ic_launcher_bg_cyberpunk", "R.mipmap.ic_launcher_cyberpunk_foreground"),
    ("space", "exteraGram · Space", "R.mipmap.ic_launcher_space_background", "R.mipmap.ic_launcher_space_foreground"),
    ("jellyfish", "exteraGram · Jellyfish", "R.mipmap.ic_launcher_jellyfish_background", "R.mipmap.ic_launcher_jellyfish_foreground"),
    ("human", "exteraGram · Human", "R.mipmap.ic_launcher_human_background", "R.mipmap.ic_launcher_human_foreground"),
    ("sunset", "exteraGram · Sunset", "R.mipmap.ic_launcher_sunset_background", "R.mipmap.ic_launcher_sunset_foreground"),
    ("amethyst", "exteraGram · Amethyst", "R.mipmap.ic_launcher_amethyst_background", "R.mipmap.ic_launcher_amethyst_foreground"),
    ("aurora", "exteraGram · Aurora", "R.mipmap.ic_launcher_aurora_background", "R.mipmap.ic_launcher_aurora_foreground"),
    ("osu", "exteraGram · osu!", "R.mipmap.ic_launcher_osu_background", "R.mipmap.ic_launcher_osu_foreground"),
    ("google", "exteraGram · Google", "R.drawable.ic_launcher_bg_white", "R.mipmap.ic_launcher_google_foreground"),
    ("google26", "exteraGram · Google ’26", "R.drawable.ic_launcher_bg_white", "R.mipmap.ic_launcher_google26_foreground"),
];

#[derive(Deserialize)]
struct Config {
    #[serde(default)]
    base_repo: String,
    #[serde(default)]
    base_commit: String,
    data_base: String,
    #[serde(default)]
    badge_service: String,
}

fn resolve_expected_base_commit(config_pin: &str, env_pin: Option<&str>) -> Result<String, String> {
    let config_pin = config_pin.trim();
    let env_pin = env_pin.unwrap_or("").trim();
    if !config_pin.is_empty() && !env_pin.is_empty() && config_pin != env_pin {
        return Err(format!("client/config.json base_commit ({config_pin}) does not match BASE_COMMIT ({env_pin})"));
    }
    let expected = if !env_pin.is_empty() { env_pin } else { config_pin };
    if expected.len() != 40 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("a pinned 40-character BASE_COMMIT is required".to_string());
    }
    Ok(expected.to_ascii_lowercase())
}

fn verify_base_checkout(root: &Path, config: &Config) -> Result<(), String> {
    if !config.base_repo.is_empty() && config.base_repo != "exteraless/exteraless" {
        return Err(format!("unexpected base_repo in client/config.json: {}", config.base_repo));
    }
    let expected = resolve_expected_base_commit(
        &config.base_commit,
        env::var("BASE_COMMIT").ok().as_deref(),
    )?;
    let head = Command::new("git")
        .arg("-C").arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| format!("cannot inspect base checkout HEAD: {e}"))?;
    if !head.status.success() {
        return Err("base checkout is not a readable git repository".to_string());
    }
    let actual = String::from_utf8_lossy(&head.stdout).trim().to_ascii_lowercase();
    if actual != expected {
        return Err(format!("base checkout commit mismatch: expected {expected}, got {actual}"));
    }

    let submodules = Command::new("git")
        .arg("-C").arg(root)
        .args(["submodule", "status", "--recursive"])
        .output()
        .map_err(|e| format!("cannot inspect base submodules: {e}"))?;
    if !submodules.status.success() {
        return Err("cannot verify base submodule state".to_string());
    }
    for line in String::from_utf8_lossy(&submodules.stdout).lines() {
        match line.chars().next() {
            Some(' ') => {}
            // The upstream .gitmodules intentionally disables the huge TC39
            // test262 checkout. Its SHA is still recorded in the parent gitlink.
            Some('-') if line.split_whitespace().nth(1)
                == Some("TMessagesProj/jni/twidget/widget-sandbox/packages/ios/Sources/CQuickJS/quickjs/test262") => {}
            Some('+') | Some('-') | Some('U') => {
                return Err(format!("base submodule is not at its pinned gitlink: {line}"));
            }
            _ => return Err(format!("unrecognised git submodule status: {line}")),
        }
    }
    Ok(())
}

struct Args {
    root: PathBuf,
    config: Option<PathBuf>,
}

fn usage() -> &'static str {
    "usage: grape-materializer --root <base-checkout> [--config <client/config.json>]"
}

fn parse_args() -> Result<Args, String> {
    let mut root = None;
    let mut config = None;
    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--root" => root = Some(PathBuf::from(it.next().ok_or("--root requires a path")?)),
            "--config" => config = Some(PathBuf::from(it.next().ok_or("--config requires a path")?)),
            "-h" | "--help" => {
                println!("{}", usage());
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument '{}'; {}", other, usage())),
        }
    }
    Ok(Args {
        root: root.ok_or_else(|| format!("--root is required; {}", usage()))?,
        config,
    })
}

fn find_repo_root(extra: Option<&Path>) -> Option<PathBuf> {
    let mut bases = Vec::new();
    if let Some(p) = extra {
        bases.push(p.to_path_buf());
    }
    if let Ok(v) = env::var("GRP_ROOT") {
        bases.push(PathBuf::from(v));
    }
    if let Ok(p) = env::current_dir() {
        bases.push(p);
    }
    if let Ok(p) = env::current_exe() {
        if let Some(parent) = p.parent() {
            bases.push(parent.to_path_buf());
        }
    }
    for base in bases {
        let mut cur = base;
        loop {
            if cur.join("client").join("overlay").join("java").is_dir()
                && cur.join("client").join("overlay").join("res").is_dir()
            {
                return Some(cur);
            }
            if !cur.pop() {
                break;
            }
        }
    }
    None
}

fn read_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("read {}: {}", path.display(), e))
}

fn write_file(path: &Path, text: &str) -> Result<(), String> {
    fs::write(path, text).map_err(|e| format!("write {}: {}", path.display(), e))
}

fn copy_dir(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("create {}: {}", dst.display(), e))?;
    let entries =
        fs::read_dir(src).map_err(|e| format!("read dir {}: {}", src.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read dir {}: {}", src.display(), e))?;
        let kind = entry
            .file_type()
            .map_err(|e| format!("stat {}: {}", entry.path().display(), e))?;
        let target = dst.join(entry.file_name());
        if kind.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(&entry.path(), &target)
                .map_err(|e| format!("copy {} -> {}: {}", entry.path().display(), target.display(), e))?;
        }
    }
    Ok(())
}

/// Overlay launcher artwork is WebP while some upstream mipmap folders still
/// contain PNGs with the same Android resource name. Android's resource merger
/// treats those as duplicate resources even though the file extensions differ.
/// Prefer the overlay WebP for launcher resources and remove only the colliding
/// upstream PNGs (never touch unrelated drawable/mipmap assets).
fn remove_duplicate_launcher_pngs(res_root: &Path) -> Result<usize, String> {
    let mut removed = 0usize;
    let entries = fs::read_dir(res_root)
        .map_err(|e| format!("read resource root {}: {}", res_root.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read resource root entry: {}", e))?;
        let folder = entry.file_name().to_string_lossy().into_owned();
        if !folder.starts_with("mipmap-") || !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let dir = entry.path();
        let files = fs::read_dir(&dir)
            .map_err(|e| format!("read mipmap dir {}: {}", dir.display(), e))?;
        let mut webp_stems = std::collections::HashSet::new();
        let mut png_paths = Vec::new();
        for file in files {
            let file = file.map_err(|e| format!("read mipmap file: {}", e))?;
            if !file.file_type().map_err(|e| e.to_string())?.is_file() {
                continue;
            }
            let path = file.path();
            let stem = path.file_stem().and_then(|v| v.to_str()).unwrap_or("");
            match path.extension().and_then(|v| v.to_str()) {
                Some("webp") if stem.starts_with("ic_launcher") => {
                    webp_stems.insert(stem.to_string());
                }
                Some("png") if stem.starts_with("ic_launcher") => {
                    png_paths.push((stem.to_string(), path));
                }
                _ => {}
            }
        }
        for (stem, png) in png_paths {
            if webp_stems.contains(&stem) {
                fs::remove_file(&png)
                    .map_err(|e| format!("remove duplicate launcher PNG {}: {}", png.display(), e))?;
                removed += 1;
            }
        }
    }
    Ok(removed)
}

fn patch_once(text: &str, anchor: &str, replacement: &str, what: &str) -> Result<String, String> {
    let n = text.matches(anchor).count();
    if n != 1 {
        return Err(format!("{}: anchor found {} times, expected 1", what, n));
    }
    Ok(text.replacen(anchor, replacement, 1))
}

/// Replaces the old app name inside `<string>` text nodes only, so resource names
/// such as `exteraless_icon_tile` and `@drawable/...` references stay intact.
fn rebrand_xml_strings(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    loop {
        let idx = match [rest.find("<string "), rest.find("<string>")]
            .into_iter()
            .flatten()
            .min()
        {
            Some(v) => v,
            None => {
                out.push_str(rest);
                break;
            }
        };
        out.push_str(&rest[..idx]);
        let after = &rest[idx..];
        let gt = match after.find('>') {
            Some(g) => g,
            None => {
                out.push_str(after);
                break;
            }
        };
        let open_end = idx + gt + 1;
        out.push_str(&rest[idx..open_end]);
        let tail = &rest[open_end..];
        let close = match tail.find("</string>") {
            Some(c) => c,
            None => {
                out.push_str(tail);
                break;
            }
        };
        let content = &tail[..close];
        out.push_str(
            &content
                .replace("exteraless", "grapeGram")
                .replace("Exteraless", "grapeGram"),
        );
        out.push_str("</string>");
        rest = &tail[close + "</string>".len()..];
    }
    out
}

fn expect_count(text: &str, needle: &str, expected: usize, what: &str) -> Result<(), String> {
    let n = text.matches(needle).count();
    if n != expected {
        return Err(format!("{}: '{}' found {} times, expected {}", what, needle, n, expected));
    }
    Ok(())
}

fn camel(key: &str) -> String {
    key.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

fn disable_alias(text: &str, alias: &str) -> Result<String, String> {
    let name_anchor = format!("android:name=\"{}\"", alias);
    expect_count(text, &name_anchor, 1, "disable alias")?;
    let pos = text.find(&name_anchor).unwrap();
    let before = &text[..pos];
    let enabled_anchor = "android:enabled=\"true\"";
    let epos = before
        .rfind(enabled_anchor)
        .ok_or_else(|| format!("disable alias {}: no enabled=true before name", alias))?;
    let mut out = String::with_capacity(text.len());
    out.push_str(&before[..epos]);
    out.push_str("android:enabled=\"false\"");
    out.push_str(&before[epos + enabled_anchor.len()..]);
    out.push_str(&text[pos..]);
    Ok(out)
}

fn application_tag_text<'a>(text: &'a str, what: &str) -> Result<&'a str, String> {
    expect_count(text, "<application", 1, what)?;
    let start = text.find("<application").unwrap();
    let rel = text[start..]
        .find('>')
        .ok_or_else(|| format!("{}: <application tag not closed", what))?;
    Ok(&text[start..start + rel + 1])
}

fn patch_application_tag(
    text: &str,
    pairs: &[(&str, &str)],
    what: &str,
) -> Result<String, String> {
    let tag_span = application_tag_text(text, what)?;
    let start = text.find(tag_span).unwrap();
    let end = start + tag_span.len();
    let mut tag = tag_span.to_string();
    for (anchor, replacement) in pairs {
        tag = patch_once(&tag, anchor, replacement, what)?;
    }
    let mut out = String::with_capacity(text.len() + 64);
    out.push_str(&text[..start]);
    out.push_str(&tag);
    out.push_str(&text[end..]);
    Ok(out)
}

fn set_xml_string(text: &str, name: &str, value: &str) -> (String, usize) {
    let open = format!("<string name=\"{}\">", name);
    let close = "</string>";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut count = 0usize;
    while let Some(pos) = rest.find(&open) {
        out.push_str(&rest[..pos + open.len()]);
        rest = &rest[pos + open.len()..];
        match rest.find(close) {
            Some(end) => {
                out.push_str(value);
                out.push_str(close);
                rest = &rest[end + close.len()..];
                count += 1;
            }
            None => {
                out.push_str(rest);
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    (out, count)
}

fn set_gradle_property(text: &str, key: &str, value: &str) -> Result<String, String> {
    let prefix = format!("{}=", key);
    let mut out = String::with_capacity(text.len());
    let mut count = 0usize;
    for line in text.split_inclusive('\n') {
        if line.trim_start().starts_with(&prefix) {
            let indent_len = line.len() - line.trim_start().len();
            out.push_str(&line[..indent_len]);
            out.push_str(&prefix);
            out.push_str(value);
            if line.ends_with('\n') {
                out.push('\n');
            }
            count += 1;
        } else {
            out.push_str(line);
        }
    }
    if count != 1 {
        return Err(format!("gradle property {}: found {} times, expected 1", key, count));
    }
    Ok(out)
}

fn get_gradle_property(text: &str, key: &str) -> Result<String, String> {
    let prefix = format!("{}=", key);
    let mut found = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            found.push(rest.trim().to_string());
        }
    }
    if found.len() != 1 {
        return Err(format!("gradle property {}: found {} times, expected 1", key, found.len()));
    }
    Ok(found.remove(0))
}

fn set_nested_string(root: &mut Value, path: &[&str], value: &str) -> Result<(), String> {
    let mut cur = root;
    for key in &path[..path.len() - 1] {
        cur = cur
            .get_mut(*key)
            .ok_or_else(|| format!("json path missing object '{}'", key))?;
    }
    let obj = cur
        .as_object_mut()
        .ok_or_else(|| format!("json path '{}' is not an object", path[path.len() - 1]))?;
    obj.insert(path[path.len() - 1].to_string(), Value::String(value.to_string()));
    Ok(())
}

fn nested_string(root: &Value, path: &[&str]) -> Option<String> {
    let mut cur = root;
    for key in path {
        cur = cur.get(*key)?;
    }
    cur.as_str().map(|s| s.to_string())
}

fn alias_template(key: &str, enabled: bool) -> String {
    format!(
        r#"        <activity-alias
            android:enabled="{enabled}"
            android:name="org.telegram.messenger.GrapeIcon{camel}"
            android:targetActivity="org.telegram.ui.LaunchActivity"
            android:icon="@drawable/grape_icon_{key}"
            android:roundIcon="@drawable/grape_icon_{key}"
            android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
                <category android:name="android.intent.category.MULTIWINDOW_LAUNCHER" />
            </intent-filter>
            <meta-data android:name="android.app.shortcuts" android:resource="@xml/shortcuts" />
        </activity-alias>

"#,
        enabled = if enabled { "true" } else { "false" },
        camel = camel(key),
        key = key
    )
}

fn enum_entries() -> String {
    ICONS
        .iter()
        .map(|(key, _)| {
            format!(
                "        GRAPE_{}(\"GrapeIcon{}\", R.drawable.grape_icon_{}, R.drawable.app_icon_empty_foreground, R.string.GrapeIcon{})",
                key.to_uppercase(),
                camel(key),
                key,
                camel(key)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

fn upstream_alias_template(prefix: &str, key: &str, resource: &str) -> String {
    format!(
        r#"        <activity-alias
            android:enabled="false"
            android:name="org.telegram.messenger.{prefix}{camel}"
            android:targetActivity="org.telegram.ui.LaunchActivity"
            android:icon="@mipmap/{resource}"
            android:roundIcon="@mipmap/{resource}"
            android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
                <category android:name="android.intent.category.MULTIWINDOW_LAUNCHER" />
            </intent-filter>
            <meta-data android:name="android.app.shortcuts" android:resource="@xml/shortcuts" />
        </activity-alias>

"#,
        prefix = prefix,
        camel = camel(key),
        resource = resource
    )
}

fn cherry_enum_entries() -> String {
    CHERRY_ICONS
        .iter()
        .map(|(key, _, background, foreground, _)| {
            format!(
                "        CHERRY_{}(\"CherryIcon{}\", {}, {}, R.string.CherryIcon{})",
                key.to_uppercase(),
                camel(key),
                background,
                foreground,
                camel(key)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

fn extera_enum_entries() -> String {
    EXTERA_ICONS
        .iter()
        .map(|(key, _, background, foreground)| {
            format!(
                "        EXTERA_{}(\"ExteraIcon{}\", {}, {}, R.string.ExteraIcon{})",
                key.to_uppercase(),
                camel(key),
                background,
                foreground,
                camel(key)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

fn upstream_aliases() -> String {
    let cherry = CHERRY_ICONS
        .iter()
        .map(|(key, _, _, _, resource)| upstream_alias_template("CherryIcon", key, resource))
        .collect::<String>();
    let extera = EXTERA_ICONS
        .iter()
        .map(|(key, _, _, _)| upstream_alias_template("ExteraIcon", key, &format!("ic_launcher_{}", key)))
        .collect::<String>();
    format!("{}{}", cherry, extera)
}

fn upstream_enum_entries() -> String {
    format!("{},\n{}", cherry_enum_entries(), extera_enum_entries())
}

fn upstream_string_entries() -> String {
    let cherry = CHERRY_ICONS
        .iter()
        .map(|(key, name, _, _, _)| {
            format!("    <string name=\"CherryIcon{}\">{}</string>\n", camel(key), name)
        })
        .collect::<String>();
    let extera = EXTERA_ICONS
        .iter()
        .map(|(key, name, _, _)| {
            format!("    <string name=\"ExteraIcon{}\">{}</string>\n", camel(key), name)
        })
        .collect::<String>();
    format!("{}{}", cherry, extera)
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    let extra_base = args.config.as_ref().and_then(|p| p.parent()).map(|p| p.to_path_buf());
    let repo_root = find_repo_root(extra_base.as_deref())
        .ok_or_else(|| "cannot find repo root (looked for client/overlay from --config, cwd and exe); set GRP_ROOT".to_string())?;
    let config_path = args
        .config
        .unwrap_or_else(|| repo_root.join("client").join("config.json"));
    let cfg: Config = serde_json::from_str(&read_file(&config_path)?)
        .map_err(|e| format!("parse {}: {}", config_path.display(), e))?;
    verify_base_checkout(&args.root, &cfg)?;
    let data_base = env::var("GRP_DATA_BASE")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or(cfg.data_base);
    let badge_service = env::var("GRP_BADGE_SERVICE")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or(cfg.badge_service);
    let client_dir = {
        let parent = config_path.parent().unwrap_or(Path::new(".")).to_path_buf();
        if parent.join("overlay").join("java").is_dir() {
            parent
        } else {
            repo_root.join("client")
        }
    };

    let proj = args.root.join("TMessagesProj");
    if !proj.is_dir() {
        return Err(format!("{} not found; pass a clean base checkout as --root", proj.display()));
    }
    let manifest_path = proj.join("src/main/AndroidManifest.xml");
    let settings_path = proj.join("src/main/java/org/telegram/ui/SettingsActivity.java");
    let launcher_path = proj.join("src/main/java/org/telegram/ui/LauncherIconController.java");
    let strings_path = proj.join("src/main/res/values/strings.xml");
    let gconf_path = proj.join("src/main/java/app/grapegram/GrapeConfig.java");
    let props_path = args.root.join("gradle.properties");
    let gs_path = proj.join("google-services.json");
    let build_path = proj.join("build.gradle");
    for p in [
        &manifest_path,
        &settings_path,
        &launcher_path,
        &strings_path,
        &gs_path,
        &build_path,
        &props_path,
    ] {
        if !p.is_file() {
            return Err(format!("missing {}", p.display()));
        }
    }

    let manifest_before = read_file(&manifest_path)?;
    if manifest_before.contains(MARK) {
        println!("already applied; nothing to do");
        return Ok(());
    }

    copy_dir(&client_dir.join("overlay/java"), &proj.join("src/main/java"))?;
    copy_dir(&client_dir.join("overlay/res"), &proj.join("src/main/res"))?;
    let removed_launcher_pngs = remove_duplicate_launcher_pngs(&proj.join("src/main/res"))?;
    println!("removed {} upstream PNGs shadowed by overlay launcher WebP assets", removed_launcher_pngs);

    let pem_path = client_dir
        .parent()
        .unwrap_or(Path::new("."))
        .join("data/keys/grapegram_ed25519.pub.pem");
    let pem = match env::var("GRP_PUBLIC_KEY_PEM") {
        Ok(v) => v,
        Err(_) => read_file(&pem_path)?.trim().to_string(),
    };
    let gconf = read_file(&gconf_path)?
        .replace("@DATA_BASE@", &data_base)
        .replace("@BADGE_SERVICE@", &badge_service)
        .replace("@PUBLIC_KEY_PEM@", &pem.replace('\n', "\\n"));
    write_file(&gconf_path, &gconf)?;

    let row_anchor = "        items.add(UItem.asCustomShadow(topView, 200 - 12));\n";
    let row = format!(
        "{}        // grapeGram: единственный дополнительный пункт, первым в списке\n        items.add(SettingCell.Factory.of(7001, 0xFF7C4DFF, 0xFF5E35B1, R.drawable.grape_icon_original, \"Настройки grapeGram\", \"Cherrygram · exteraGram · exteraless\"));\n        items.add(UItem.asShadow(null));\n",
        row_anchor
    );
    let case_anchor = "            case -1: {\n                presentFragment(new OpenExteraSettingsActivity());\n                break;\n            }\n";
    let new_case = "            case 7001: {\n                presentFragment(new app.grapegram.GrapeSettingsActivity());\n                break;\n            }\n";
    let old_row = "        items.add(SettingCell.Factory.of(-1, 0xFF993C38, 0xFF993C38, R.drawable.exteraless_icon_tile, getString(R.string.OpenExteraPreferences), getString(R.string.OpenExteraInfo)));\n        items.add(UItem.asShadow(null));\n";
    let mut s = read_file(&settings_path)?;
    s = patch_once(&s, row_anchor, &row, "settings row")?;
    s = patch_once(&s, case_anchor, &format!("{}{}", new_case, case_anchor), "settings click")?;
    s = patch_once(&s, old_row, "        // exteraless entry moved into the grapeGram hub\n", "remove old top-level entry")?;
    write_file(&settings_path, &s)?;

    let nox = "NOX(\"NoxIcon\", R.mipmap.icon_2_background_sa, R.mipmap.icon_foreground_sa, R.string.AppIconNox);";
    let enum_replacement = format!(
        "{},\n{},\n{};",
        &nox[..nox.len() - 1],
        enum_entries(),
        upstream_enum_entries()
    );
    let mut l = read_file(&launcher_path)?;
    l = patch_once(&l, nox, &enum_replacement, "icon enum")?;
    l = patch_once(
        &l,
        "        setIcon(LauncherIcon.EXTERALESS);",
        "        setIcon(LauncherIcon.GRAPE_ORIGINAL);",
        "default icon",
    )?;
    l = patch_once(
        &l,
        "|| i == PackageManager.COMPONENT_ENABLED_STATE_DEFAULT && icon == LauncherIcon.EXTERALESS;",
        "|| i == PackageManager.COMPONENT_ENABLED_STATE_DEFAULT && icon == LauncherIcon.GRAPE_ORIGINAL;",
        "isEnabled default",
    )?;
    write_file(&launcher_path, &l)?;

    let app_icons_path = proj.join("src/main/java/app/exteraless/appicons/AppIcons.java");
    let mut app_icons = read_file(&app_icons_path)?;
    app_icons = patch_once(
        &app_icons,
        "return LauncherIcon.EXTERALESS;",
        "return LauncherIcon.GRAPE_ORIGINAL;",
        "default current icon fallback",
    )?;
    app_icons = patch_once(
        &app_icons,
        "import android.graphics.Color;",
        "import android.graphics.Color;\nimport android.os.Build;",
        "adaptive icon API import",
    )?;
    app_icons = patch_once(
        &app_icons,
        "        for (LauncherIcon icon : LauncherIcon.values()) {\n            if (!icon.premium || !blocked || LauncherIconController.isEnabled(icon)) {",
        "        for (LauncherIcon icon : LauncherIcon.values()) {\n            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O\n                    && (icon.name().startsWith(\"CHERRY_\") || icon.name().startsWith(\"EXTERA_\"))) {\n                continue;\n            }\n            if (!icon.premium || !blocked || LauncherIconController.isEnabled(icon)) {",
        "adaptive icon API availability",
    )?;
    write_file(&app_icons_path, &app_icons)?;

    let mut m = disable_alias(&manifest_before, "org.telegram.messenger.ExteralessIcon")?;
    let aliases = format!(
        "{}{}",
        ICONS
            .iter()
            .map(|(key, _)| alias_template(key, *key == "original"))
            .collect::<String>(),
        upstream_aliases()
    );
    m = patch_once(&m, "    </application>", &format!("{}    </application>", aliases), "manifest aliases")?;
    m = patch_application_tag(
        &m,
        &[
            ("android:icon=\"@mipmap/ic_launcher_exteraless\"", "android:icon=\"@drawable/grape_icon_original\""),
            ("android:label=\"@string/NagramX\"", "android:label=\"@string/AppName\""),
            ("android:roundIcon=\"@mipmap/ic_launcher_exteraless_round\"", "android:roundIcon=\"@drawable/grape_icon_original\""),
        ],
        "manifest application tag",
    )?;
    m = patch_once(&m, "        tools:replace=\"android:supportsRtl\"\n", "", "supportsRtl warning")?;
    m = m
        .replace("@mipmap/ic_launcher_exteraless_round", "@drawable/grape_icon_original")
        .replace("@mipmap/ic_launcher_exteraless", "@drawable/grape_icon_original")
        .replace("vnd.com.exteraless.app.android", "vnd.app.grapegram.android")
        .replace("\"com.exteraless.app\"", "\"app.grapegram\"");
    write_file(&manifest_path, &m)?;

    let st = read_file(&strings_path)?;
    let grape_strings = ICONS
        .iter()
        .map(|(key, name)| format!("    <string name=\"GrapeIcon{}\">{}</string>\n", camel(key), name))
        .collect::<String>();
    let add = format!("{}{}", grape_strings, upstream_string_entries());
    let (st_new, n) = set_xml_string(&st, "AppName", APP_NAME);
    if n != 1 {
        return Err(format!("strings AppName: found {} times, expected 1", n));
    }
    let st_new = patch_once(&st_new, "</resources>", &format!("{}</resources>", add), "strings")?;
    write_file(&strings_path, &st_new)?;

    let res_dir = proj.join("src/main/res");
    for entry in fs::read_dir(&res_dir).map_err(|e| format!("read dir {}: {}", res_dir.display(), e))? {
        let entry = entry.map_err(|e| format!("read dir {}: {}", res_dir.display(), e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !(name == "values" || name.starts_with("values-")) || !entry.path().is_dir() {
            continue;
        }
        let path = entry.path().join("strings.xml");
        if !path.is_file() {
            continue;
        }
        let mut text = read_file(&path)?;
        let (updated, n) = set_xml_string(&text, "AppName", APP_NAME);
        if n > 0 {
            if n != 1 {
                return Err(format!("{}: AppName found {} times, expected at most 1", path.display(), n));
            }
            text = updated;
        }
        let (updated, n) = set_xml_string(&text, "Page1Title", APP_NAME);
        if n > 0 {
            text = updated;
        }
        let (updated, n) = set_xml_string(&text, "Page1Message",
            "The world\\'s **fastest** messaging app. It is free and secure.");
        if n > 0 {
            text = updated;
        }
        write_file(&path, &text)?;
    }

    for entry in fs::read_dir(&res_dir).map_err(|e| format!("read dir {}: {}", res_dir.display(), e))? {
        let entry = entry.map_err(|e| format!("read dir {}: {}", res_dir.display(), e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !(name == "values" || name.starts_with("values-")) || !entry.path().is_dir() {
            continue;
        }
        for file in fs::read_dir(entry.path())
            .map_err(|e| format!("read dir {}: {}", entry.path().display(), e))?
        {
            let file = file.map_err(|e| format!("read dir {}: {}", entry.path().display(), e))?;
            let path = file.path();
            if path.extension().and_then(|e| e.to_str()) != Some("xml") || !path.is_file() {
                continue;
            }
            let original = read_file(&path)?;
            let mut text = original.replace(">exteraless<", ">grapeGram<");
            text = rebrand_xml_strings(&text);
            if text != original {
                write_file(&path, &text)?;
            }
        }
    }

    let xml_dir = res_dir.join("xml");
    if xml_dir.is_dir() {
        for file in fs::read_dir(&xml_dir)
            .map_err(|e| format!("read dir {}: {}", xml_dir.display(), e))?
        {
            let file = file.map_err(|e| format!("read dir {}: {}", xml_dir.display(), e))?;
            let path = file.path();
            if path.extension().and_then(|e| e.to_str()) != Some("xml") || !path.is_file() {
                continue;
            }
            let original = read_file(&path)?;
            let text = original
                .replace("@mipmap/ic_launcher_exteraless_round", "@drawable/grape_icon_original")
                .replace("@mipmap/ic_launcher_exteraless", "@drawable/grape_icon_original")
                .replace("vnd.com.exteraless.app.android", "vnd.app.grapegram.android")
                .replace("\"com.exteraless.app\"", "\"app.grapegram\"");
            if text != original {
                write_file(&path, &text)?;
            }
        }
    }

    let props = set_gradle_property(&read_file(&props_path)?, "APP_PACKAGE", PACKAGE)?;
    write_file(&props_path, &props)?;

    let gs_text = read_file(&gs_path)?;
    let mut gs: Value = serde_json::from_str(&gs_text)
        .map_err(|e| format!("parse {}: {}", gs_path.display(), e))?;
    let clients = gs
        .get_mut("client")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "google-services.json: client array missing".to_string())?;
    if clients.len() != 1 {
        return Err(format!("google-services.json: expected 1 client, found {}", clients.len()));
    }
    let mut dup = clients[0].clone();
    set_nested_string(
        &mut dup,
        &["client_info", "android_client_info", "package_name"],
        PACKAGE,
    )?;
    clients.push(dup);
    write_file(&gs_path, &format!("{}\n", serde_json::to_string_pretty(&gs).map_err(|e| e.to_string())?))?;

    let mut build = read_file(&build_path)?;
    build = patch_once(
        &build,
        "    signingConfigs {\n        release {\n",
        "    signingConfigs {\n        release {\n            storeType 'PKCS12'\n",
        "signingConfigs.release",
    )?;
    build = patch_once(
        &build,
        "dependencies {\n",
        "dependencies {\n    implementation 'org.bouncycastle:bcprov-jdk15to18:1.78'\n",
        "bouncycastle dependency",
    )?;
    build = patch_once(&build, "            version '3.11'", "            version = '3.11'", "chaquopy version assignment")?;
    build = patch_once(&build, "        aidl true", "        aidl = true", "aidl assignment")?;
    build = patch_once(
        &build,
        "            String gramName = 'exteraless'",
        "            String gramName = 'grapeGram'",
        "apk file name",
    )?;
    write_file(&build_path, &build)?;

    let settings_root = args.root.join("settings.gradle");
    let mut settings_root_text = read_file(&settings_root)?;
    settings_root_text = patch_once(
        &settings_root_text,
        "maven { url \"https://maven.aliucord.com/releases\" }",
        "maven { url = uri(\"https://maven.aliucord.com/releases\") }",
        "maven url assignment",
    )?;
    write_file(&settings_root, &settings_root_text)?;

    let pg_path = proj.join("proguard-rules.pro");
    let mut pg = read_file(&pg_path)?;
    if !pg.contains("grapegram: keep Ed25519 providers") {
        pg.push_str("\n# grapegram: keep Ed25519 providers (BC registers them by string name)\n");
        pg.push_str("-keep class org.bouncycastle.jcajce.provider.asymmetric.edec.** { *; }\n");
        pg.push_str("-keep class org.bouncycastle.jce.provider.BouncyCastleProvider { *; }\n");
        pg.push_str("-keepclasseswithmembers,includedescriptorclasses class app.grapegram.GrapeBadges {\n    native <methods>;\n}\n");
        pg.push_str("# grapegram: optional javax.script engine service is referenced by a dependency but not bundled\n");
        pg.push_str("-dontwarn javax.script.**\n");
        write_file(&pg_path, &pg)?;
    }

    let locale_path = proj.join("src/main/java/org/telegram/messenger/LocaleController.java");
    let mut locale = read_file(&locale_path)?;
    locale = patch_once(&locale,
        "return ApplicationLoader.applicationContext.getString(R.string.OpenExtera);",
        "return \"grapeGram\";", "app name")?;
    locale = patch_once(&locale, "return \"exteraless\";", "return \"grapeGram\";", "app name fallback")?;
    write_file(&locale_path, &locale)?;

    let intro_path = proj.join("src/main/java/org/telegram/ui/IntroActivity.java");
    let mut intro = read_file(&intro_path)?;
    intro = patch_once(&intro, "R.drawable.telegram_logo", "R.drawable.grape_icon_original", "intro wordmark")?;
    intro = patch_once(&intro,
        "new int[]{R.drawable.exteraless_icon_background, R.drawable.exteraless_icon_foreground}",
        "new int[]{R.drawable.grape_icon_original}", "intro icon")?;
    write_file(&intro_path, &intro)?;

    let google_translate_path = proj
        .join("src/main/java/tw/nekomimi/nekogram/translate/source/GoogleTranslator.kt");
    let mut google_translate = read_file(&google_translate_path)?;
    google_translate = patch_once(
        &google_translate,
        "        val urlBuilder = HttpUrl.Builder().scheme(\"https\").host(\"translate-pa.googleapis.com\")",
        "        val grapeTranslateKey = tw.nekomimi.nekogram.NekoConfig.googleCloudTranslateKey.String()\n        if (grapeTranslateKey.isEmpty()) {\n            error(\"Google Translate API key is not set\")\n        }\n        val urlBuilder = HttpUrl.Builder().scheme(\"https\").host(\"translate-pa.googleapis.com\")",
        "google translate key check",
    )?;
    // Remove the bundled upstream key without copying its value into grapeGram.
    // Match the single key parameter by syntax and preserve the original indentation.
    let key_parameter = ".addQueryParameter(\"key\",";
    let key_lines: Vec<&str> = google_translate
        .lines()
        .filter(|line| line.contains(key_parameter))
        .collect();
    if key_lines.len() != 1 {
        return Err(format!(
            "google translate key parameter: found {}, expected exactly 1",
            key_lines.len()
        ));
    }
    let old_key_line = key_lines[0];
    let indentation = &old_key_line[..old_key_line.len() - old_key_line.trim_start().len()];
    let replacement_key_line = format!(
        "{}.addQueryParameter(\"key\", grapeTranslateKey)",
        indentation
    );
    google_translate = google_translate.replacen(old_key_line, &replacement_key_line, 1);
    write_file(&google_translate_path, &google_translate)?;

    let translator_path = proj
        .join("src/main/java/tw/nekomimi/nekogram/translate/Translator.kt");
    let mut translator = read_file(&translator_path)?;
    translator = patch_once(
        &translator,
        "            Locale(args[0])",
        "            Locale.forLanguageTag(args[0])",
        "translator locale one",
    )?;
    translator = patch_once(
        &translator,
        "            Locale(args[0], args[1])",
        "            Locale.forLanguageTag(args[0] + \"-\" + args[1])",
        "translator locale two",
    )?;
    write_file(&translator_path, &translator)?;

    let login_path = proj.join("src/main/java/org/telegram/ui/LoginActivity.java");
    let mut login = read_file(&login_path)?;
    login = patch_once(&login,
        "    public void onResume() {\n        super.onResume();\n        if (newAccount) {",
        "    public void onResume() {\n        super.onResume();\n        app.grapegram.GrapeApiGate.maybeAsk(this);\n        if (newAccount) {",
        "mandatory api keys")?;
    write_file(&login_path, &login)?;

    let firebase_path = proj.join("src/main/java/org/telegram/messenger/FirebasePhoneNumberVerificationHelper.java");
    let mut firebase = read_file(&firebase_path)?;
    firebase = patch_once(&firebase, "req.api_id = BuildVars.APP_ID;",
        "req.api_id = tw.nekomimi.nekogram.NekoXConfig.currentAppId();", "firebase api id")?;
    firebase = patch_once(&firebase, "req.api_hash = BuildVars.APP_HASH;",
        "req.api_hash = tw.nekomimi.nekogram.NekoXConfig.currentAppHash();", "firebase api hash")?;
    write_file(&firebase_path, &firebase)?;

    let dialog_cell_path = proj.join("src/main/java/org/telegram/ui/Cells/DialogCell.java");
    let mut dialog_cell = read_file(&dialog_cell_path)?;
    dialog_cell = patch_once(
        &dialog_cell,
        "    private final AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable emojiStatus;",
        "    private final AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable emojiStatus;\n    private app.grapegram.GrapeBadgeDrawable grapeBadgeDrawable;",
        "dialog cell badge field",
    )?;
    dialog_cell = patch_once(
        &dialog_cell,
        r#"        emojiStatusView = new View(context) {
            @Override
            protected void onDraw(@NonNull Canvas canvas) {
                emojiStatus.setBounds(0, 0, getWidth(), getHeight());
                emojiStatus.draw(canvas);
            }
        };"#,
        r#"        emojiStatusView = new View(context) {
            @Override
            protected void onDraw(@NonNull Canvas canvas) {
                if (grapeBadgeDrawable != null) {
                    grapeBadgeDrawable.setBounds(0, 0, getWidth(), getHeight());
                    grapeBadgeDrawable.draw(canvas);
                } else {
                    emojiStatus.setBounds(0, 0, dp(22), dp(22));
                    emojiStatus.draw(canvas);
                }
            }
        };"#,
        "dialog cell badge drawing",
    )?;
    dialog_cell = patch_once(
        &dialog_cell,
        "        emojiStatusView.layout(0, 0, dp(22), dp(22));",
        "        emojiStatusView.layout(0, 0, dp(42), dp(22));",
        "dialog cell badge view width",
    )?;
    dialog_cell = patch_once(
        &dialog_cell,
        "        drawPremium = false;\n        drawForwardIcon = false;",
        "        drawPremium = false;\n        if (grapeBadgeDrawable != null) {\n            grapeBadgeDrawable.setCallback(null);\n        }\n        grapeBadgeDrawable = null;\n        drawForwardIcon = false;",
        "dialog cell badge reset",
    )?;
    dialog_cell = patch_once(
        &dialog_cell,
        r#"                        drawPremium = MessagesController.getInstance(currentAccount).isPremiumUser(user) && UserConfig.getInstance(currentAccount).clientUserId != user.id && user.id != 0;
                        if (drawPremium) {
                            Long emojiStatusId = UserObject.getEmojiStatusDocumentId(user);
                            emojiStatus.center = LocaleController.isRTL;
                            if (emojiStatusId != null) {
                                nameLayoutEllipsizeByGradient = true;
                                emojiStatus.set(emojiStatusId, false);
                                emojiStatus.setParticles(DialogObject.isEmojiStatusCollectible(user.emoji_status), false);
                            } else {
                                nameLayoutEllipsizeByGradient = true;
                                emojiStatus.set(PremiumGradient.getInstance().premiumStarDrawableMini, false);
                                emojiStatus.setParticles(false, false);
                            }
                        }"#,
        r#"                        boolean userPremium = MessagesController.getInstance(currentAccount).isPremiumUser(user)
                                && UserConfig.getInstance(currentAccount).clientUserId != user.id && user.id != 0;
                        app.grapegram.GrapeBadges.Entry grapeBadge = user.id == 0 ? null
                                : app.grapegram.GrapeBadges.find(getContext(), String.valueOf(user.id));
                        drawPremium = userPremium || grapeBadge != null;
                        if (drawPremium) {
                            Long emojiStatusId = UserObject.getEmojiStatusDocumentId(user);
                            emojiStatus.center = LocaleController.isRTL;
                            if (emojiStatusId != null) {
                                nameLayoutEllipsizeByGradient = true;
                                emojiStatus.set(emojiStatusId, false);
                                emojiStatus.setParticles(DialogObject.isEmojiStatusCollectible(user.emoji_status), false);
                            } else if (userPremium) {
                                nameLayoutEllipsizeByGradient = true;
                                emojiStatus.set(PremiumGradient.getInstance().premiumStarDrawableMini, false);
                                emojiStatus.setParticles(false, false);
                            } else {
                                emojiStatus.set((android.graphics.drawable.Drawable) null, false);
                                emojiStatus.setParticles(false, false);
                            }
                            if (grapeBadge != null) {
                                android.graphics.drawable.Drawable grapeIcon = getContext().getDrawable(R.drawable.grape_icon_original);
                                grapeBadgeDrawable = new app.grapegram.GrapeBadgeDrawable(
                                        (emojiStatusId != null || userPremium) ? emojiStatus : null,
                                        grapeIcon,
                                        app.grapegram.GrapeBadgeDrawable.colorForBadge(grapeBadge.badge,
                                                org.telegram.ui.ActionBar.Theme.isCurrentThemeDark() ? grapeBadge.colorDark : grapeBadge.colorLight),
                                        dp(18), dp(2), LocaleController.isRTL,
                                        (int) (grapeBadge.alpha * 255), grapeBadge.particles, grapeBadge.animation);
                                grapeBadgeDrawable.setCallback(emojiStatusView);
                            }
                        }"#,
        "dialog cell signed badge lookup",
    )?;
    dialog_cell = patch_once(
        &dialog_cell,
        "if (drawPremium && emojiStatus.getDrawable() != null) {\n            int w = dp(6 + 24 + 6);",
        "if (drawPremium && (emojiStatus.getDrawable() != null || grapeBadgeDrawable != null)) {\n            int w = dp(6 + (grapeBadgeDrawable != null ? 42 : 24) + 6);",
        "dialog cell badge reserve width",
    )?;
    dialog_cell = dialog_cell.replace("w += dp(6 + 24 + 6);", "w += dp(6 + (grapeBadgeDrawable != null ? 42 : 24) + 6);");
    dialog_cell = dialog_cell.replace("int w = dp(6 + 24 + 6);", "int w = dp(6 + (grapeBadgeDrawable != null ? 42 : 24) + 6);");
    dialog_cell = dialog_cell.replace("- dp(24));", "- dp(grapeBadgeDrawable != null ? 42 : 24));");
    dialog_cell = patch_once(
        &dialog_cell,
        r#"                        emojiStatus.setBounds(
                            nameMuteLeft - dp(2),
                            y - dp(4),
                            nameMuteLeft + dp(20),
                            y - dp(4) + dp(22)
                        );
                        emojiStatus.draw(canvas);"#,
        r#"                        if (grapeBadgeDrawable != null) {
                            grapeBadgeDrawable.setBounds(
                                    nameMuteLeft - dp(2), y - dp(4),
                                    nameMuteLeft + dp(40), y - dp(4) + dp(22));
                            grapeBadgeDrawable.draw(canvas);
                        } else {
                            emojiStatus.setBounds(
                                nameMuteLeft - dp(2),
                                y - dp(4),
                                nameMuteLeft + dp(20),
                                y - dp(4) + dp(22)
                            );
                            emojiStatus.draw(canvas);
                        }"#,
        "dialog cell badge draw in transition",
    )?;
    write_file(&dialog_cell_path, &dialog_cell)?;

    let chat_avatar_path = proj.join("src/main/java/org/telegram/ui/Components/ChatAvatarContainer.java");
    let mut chat_avatar = read_file(&chat_avatar_path)?;
    chat_avatar = patch_once(
        &chat_avatar,
        "        titleTextView.setText(value);\n        titleTextView.setScrollNonFitText(scrollable || isCentered());\n        rightDrawableIsScam = false;",
        "        titleTextView.setText(value);\n        titleTextView.setScrollNonFitText(scrollable || isCentered());\n        if (titleTextView.getRightDrawable2() instanceof app.grapegram.GrapeBadgeDrawable) {\n            titleTextView.setRightDrawable2(((app.grapegram.GrapeBadgeDrawable) titleTextView.getRightDrawable2()).getPrimaryDrawable());\n        }\n        rightDrawableIsScam = false;",
        "chat header unwrap previous grapeGram badge",
    )?;
    chat_avatar = patch_once(
        &chat_avatar,
        "        checkActionBar(animated);\n    }\n\n    private Drawable emojiStatusDefaultDrawable;",
        "        TLRPC.User grapeUser = parentFragment != null ? parentFragment.getCurrentUser() : null;\n        app.grapegram.GrapeBadges.Entry grapeBadge = grapeUser == null ? null\n                : app.grapegram.GrapeBadges.find(getContext(), String.valueOf(grapeUser.id));\n        if (grapeBadge != null && !scam && !fake) {\n            Drawable existingStatus = titleTextView.getRightDrawable2();\n            Drawable grapeIcon = getContext().getDrawable(R.drawable.grape_icon_original);\n            titleTextView.setRightDrawable2(new app.grapegram.GrapeBadgeDrawable(\n                    existingStatus, grapeIcon,\n                    app.grapegram.GrapeBadgeDrawable.colorForBadge(grapeBadge.badge,\n                            Theme.isCurrentThemeDark() ? grapeBadge.colorDark : grapeBadge.colorLight),\n                    AndroidUtilities.dp(18), AndroidUtilities.dp(2), LocaleController.isRTL,\n                    (int) (grapeBadge.alpha * 255), grapeBadge.particles, grapeBadge.animation));\n            rightDrawableIsScamOrVerified = true;\n            rightDrawable2ContentDescription = \"grapeGram badge\";\n        }\n        checkActionBar(animated);\n    }\n\n    private Drawable emojiStatusDefaultDrawable;",
        "chat header grapeGram badge",
    )?;
    write_file(&chat_avatar_path, &chat_avatar)?;

    let profile_activity_path = proj.join("src/main/java/org/telegram/ui/ProfileActivity.java");
    let mut profile_activity = read_file(&profile_activity_path)?;
    profile_activity = patch_once(
        &profile_activity,
        "    private Drawable getBadgeDrawable(BadgeDTO badge, boolean animated) {\n        if (badgeDrawable == null) {",
        "    private Drawable getBadgeDrawable(BadgeDTO badge, boolean animated) {\n        if (badge != null && \"grapeGram\".equals(badge.getText()) && getDialogId() > 0) {\n            android.content.Context badgeContext = getParentActivity();\n            if (badgeContext != null) {\n                app.grapegram.GrapeBadges.Entry entry = app.grapegram.GrapeBadges.find(badgeContext, String.valueOf(getDialogId()));\n                if (entry != null) {\n                    android.graphics.drawable.Drawable icon = badgeContext.getDrawable(R.drawable.grape_icon_original);\n                    return new app.grapegram.GrapeBadgeDrawable(null, icon,\n                            app.grapegram.GrapeBadgeDrawable.colorForBadge(entry.badge,\n                                    Theme.isCurrentThemeDark() ? entry.colorDark : entry.colorLight),\n                            AndroidUtilities.dp(24), 0, false, (int) (entry.alpha * 255),\n                            entry.particles, entry.animation);\n                }\n            }\n        }\n        if (badgeDrawable == null) {",
        "profile header grapeGram badge styling",
    )?;
    write_file(&profile_activity_path, &profile_activity)?;

    let theme_activity_path = proj.join("src/main/java/org/telegram/ui/ThemeActivity.java");
    let mut theme_activity = read_file(&theme_activity_path)?;
    theme_activity = patch_once(
        &theme_activity,
        "            appIconHeaderRow = rowCount++;\n            appIconSelectorRow = rowCount++;\n            appIconShadowRow = rowCount++;\n",
        "",
        "move app icon picker out of theme settings",
    )?;
    write_file(&theme_activity_path, &theme_activity)?;

    let chats_activity_path = proj.join("src/main/java/app/exteraless/settings/OpenExteraChatsActivity.java");
    let mut chats_activity = read_file(&chats_activity_path)?;
    chats_activity = patch_once(
        &chats_activity,
        "    private int chatsHeaderRow = -1;\n",
        "    private int chatsHeaderRow = -1;\n    private int appIconRow = -1;\n",
        "chat settings app icon row field",
    )?;
    chats_activity = patch_once(
        &chats_activity,
        "        chatsHeaderRow = addRow(\"chatsHeader\");\n        quickTransitionGroupRow = addRow(\"quickTransition\");",
        "        chatsHeaderRow = addRow(\"chatsHeader\");\n        appIconRow = addRow(\"appIcon\");\n        quickTransitionGroupRow = addRow(\"quickTransition\");",
        "chat settings app icon row placement",
    )?;
    chats_activity = patch_once(
        &chats_activity,
        "        } else if (position == chatSettingsRow) {\n            presentFragment(new ThemeActivity(ThemeActivity.THEME_TYPE_BASIC));\n            return;\n        } else if (position == aiChatRow) {",
        "        } else if (position == chatSettingsRow) {\n            presentFragment(new ThemeActivity(ThemeActivity.THEME_TYPE_BASIC));\n            return;\n        } else if (position == appIconRow) {\n            presentFragment(new app.exteraless.appicons.AppIconsActivity());\n            return;\n        } else if (position == aiChatRow) {",
        "chat settings app icon navigation",
    )?;
    chats_activity = patch_once(
        &chats_activity,
        "            } else if (position == chatSettingsRow) {\n                cell.setTextAndIcon(getString(R.string.OEChatsChatSettings), R.drawable.msg_discussion, false);\n                cell.setSubtitle(getString(R.string.OEChatsChatSettingsInfo));\n            }",
        "            } else if (position == chatSettingsRow) {\n                cell.setTextAndIcon(getString(R.string.OEChatsChatSettings), R.drawable.msg_discussion, false);\n                cell.setSubtitle(getString(R.string.OEChatsChatSettingsInfo));\n            } else if (position == appIconRow) {\n                cell.setTextAndIcon(\"Иконка приложения\", R.drawable.grape_icon_original, true);\n                cell.setSubtitle(\"58 вариантов · grapeGram, Cherrygram, exteraGram\");\n            }",
        "chat settings app icon row binding",
    )?;
    chats_activity = patch_once(
        &chats_activity,
        "position == inlineMathSettingsRow || position == aiChatRow || position == chatSettingsRow || position == menusRow || position == mediaRow",
        "position == inlineMathSettingsRow || position == aiChatRow || position == chatSettingsRow || position == appIconRow || position == menusRow || position == mediaRow",
        "chat settings app icon text cell type",
    )?;
    write_file(&chats_activity_path, &chats_activity)?;

    let user_cell_path = proj.join("src/main/java/org/telegram/ui/Cells/UserCell.java");
    let mut user_cell = read_file(&user_cell_path)?;
    user_cell = patch_once(
        &user_cell,
        r#"        } else {
            nameTextView.setRightDrawable(null);
            nameTextView.setRightDrawableTopPadding(0);
        }
        if (currentStatus != null) {"#,
        r#"        } else {
            nameTextView.setRightDrawable(null);
            nameTextView.setRightDrawableTopPadding(0);
        }
        app.grapegram.GrapeBadges.Entry grapeBadge = currentUser == null ? null
                : app.grapegram.GrapeBadges.find(getContext(), String.valueOf(currentUser.id));
        if (grapeBadge != null) {
            android.graphics.drawable.Drawable grapeIcon = getContext().getDrawable(R.drawable.grape_icon_original);
            nameTextView.setRightDrawable2(new app.grapegram.GrapeBadgeDrawable(
                    null, grapeIcon,
                    app.grapegram.GrapeBadgeDrawable.colorForBadge(grapeBadge.badge,
                            org.telegram.ui.ActionBar.Theme.isCurrentThemeDark() ? grapeBadge.colorDark : grapeBadge.colorLight),
                    dp(14), 0, false, (int) (grapeBadge.alpha * 255),
                    grapeBadge.particles, grapeBadge.animation));
        } else {
            nameTextView.setRightDrawable2(null);
        }
        if (currentStatus != null) {"#,
        "user cell grapeGram badge",
    )?;
    write_file(&user_cell_path, &user_cell)?;


    // Encrypt AI provider keys at rest in Android Keystore. Use Rust raw strings
    // so the anchors contain actual line breaks and Java quotes without double escaping.
    let ai_config_path = proj.join("src/main/java/app/exteraless/ai/AiConfig.java");
    let mut ai_config = read_file(&ai_config_path)?;
    ai_config = patch_once(
        &ai_config,
        "import app.exteraless.ai.data.Suggestions;",
        "import app.exteraless.ai.data.Suggestions;\nimport app.grapegram.GrapeSecretStore;",
        "AI key encryption import",
    )?;
    ai_config = patch_once(
        &ai_config,
        r##"    public static ArrayList<Service> getServices() {
        ArrayList<Service> services = read(KEY_SERVICES,
                new TypeToken<ArrayList<Service>>() {}.getType());
        boolean changed = false;
        for (Service service : services) {
            if (service.ensureId()) {
                changed = true;
            }
        }
        if (changed) {
            saveServices(services);
        }
        return services;
    }"##,
        r##"    public static ArrayList<Service> getServices() {
        String stored = getPreferences().getString(KEY_SERVICES, null);
        if (TextUtils.isEmpty(stored)) {
            return new ArrayList<>();
        }
        try {
            String protectedJson = GrapeSecretStore.encryptJsonKeys(stored);
            String readableJson = GrapeSecretStore.decryptJsonKeys(protectedJson);
            if (!protectedJson.equals(stored)) {
                getEditor().putString(KEY_SERVICES, protectedJson).apply();
            }
            ArrayList<Service> services = GSON.fromJson(readableJson,
                    new TypeToken<ArrayList<Service>>() {}.getType());
            if (services == null) {
                services = new ArrayList<>();
            }
            boolean changed = false;
            for (Service service : services) {
                if (service.ensureId()) {
                    changed = true;
                }
            }
            if (changed) {
                saveServices(services);
            }
            return services;
        } catch (Exception e) {
            FileLog.e("AiConfig: cannot decrypt provider keys", e);
            return new ArrayList<>();
        }
    }"##,
        "encrypt AI service keys and migrate legacy plaintext",
    )?;
    ai_config = patch_once(
        &ai_config,
        r##"    public static void saveServices(ArrayList<Service> services) {
        getEditor().putString(KEY_SERVICES, GSON.toJson(services)).apply();
    }"##,
        r##"    public static void saveServices(ArrayList<Service> services) {
        String serialized = GSON.toJson(services);
        String protectedJson = GrapeSecretStore.encryptJsonKeys(serialized);
        getEditor().putString(KEY_SERVICES, protectedJson).apply();
    }"##,
        "persist encrypted AI service keys",
    )?;
    write_file(&ai_config_path, &ai_config)?;

    let plugins_path = proj.join("src/main/java/app/exteraless/plugins/ui/PluginsActivity.java");
    let mut plugins = read_file(&plugins_path)?;
    plugins = patch_once(&plugins,
        "    private static final int MENU_INFO = 1;",
        "    private static final int MENU_INFO = 1;\n    private static final int MENU_LOG = 2;",
        "plugins log menu id")?;
    plugins = patch_once(&plugins,
        "                } else if (id == MENU_INFO) {\n                    presentFragment(new PluginsInfoActivity());",
        "                } else if (id == MENU_INFO) {\n                    presentFragment(new PluginsInfoActivity());\n                } else if (id == MENU_LOG) {\n                    showLogPicker();",
        "plugins log menu click")?;
    plugins = patch_once(&plugins,
        "        actionBar.createMenu().addItem(MENU_INFO, R.drawable.msg_info);",
        "        actionBar.createMenu().addItem(MENU_INFO, R.drawable.msg_info);\n        actionBar.createMenu().addItem(MENU_LOG, R.drawable.msg_log);",
        "plugins log menu button")?;
    if !plugins.contains("private void showLogPicker()") {
        let end = plugins.rfind('}').ok_or("plugins final brace")?;
        plugins.insert_str(end, r#"
    private void showLogPicker() {
        final android.content.Context context = getParentActivity();
        if (context == null) {
            return;
        }
        final java.util.List<app.exteraless.plugins.Plugin> list =
                app.exteraless.plugins.PluginsController.getInstance().getPluginsSnapshot();
        if (list == null || list.isEmpty()) {
            new org.telegram.ui.ActionBar.AlertDialog.Builder(context)
                    .setTitle("Журнал активности")
                    .setMessage("Установленных плагинов нет")
                    .setPositiveButton(org.telegram.messenger.LocaleController.getString(R.string.OK), null)
                    .show();
            return;
        }
        final CharSequence[] names = new CharSequence[list.size()];
        for (int i = 0; i < list.size(); i++) {
            names[i] = list.get(i).getDisplayName();
        }
        new org.telegram.ui.ActionBar.AlertDialog.Builder(context)
                .setTitle("Журнал активности")
                .setItems(names, (dialog, which) ->
                        presentFragment(new app.exteraless.plugins.ui.PluginActivityLogActivity(list.get(which).id)))
                .setNegativeButton(org.telegram.messenger.LocaleController.getString(R.string.Cancel), null)
                .show();
    }
"#);
    }
    write_file(&plugins_path, &plugins)?;

    let app_path = proj.join("src/main/java/org/telegram/messenger/ApplicationLoader.java");
    let mut app = read_file(&app_path)?;
    app = patch_once(&app,
        "        super.onCreate();\n\n        // The widget engine's isolated process",
        "        super.onCreate();\n        app.grapegram.GrapeBadgeBridge.syncAsync(getApplicationContext());\n\n        // The widget engine's isolated process",
        "badge bridge")?;
    write_file(&app_path, &app)?;

    let manifest = read_file(&manifest_path)?;
    expect_count(&manifest, MARK, 1, "verify alias original")?;
    for (key, _) in ICONS.iter() {
        expect_count(
            &manifest,
            &format!("android:name=\"org.telegram.messenger.GrapeIcon{}\"", camel(key)),
            1,
            "verify grape aliases",
        )?;
    }
    for (key, _, _, _, _) in CHERRY_ICONS.iter() {
        expect_count(
            &manifest,
            &format!("android:name=\"org.telegram.messenger.CherryIcon{}\"", camel(key)),
            1,
            "verify Cherrygram aliases",
        )?;
    }
    for (key, _, _, _) in EXTERA_ICONS.iter() {
        expect_count(
            &manifest,
            &format!("android:name=\"org.telegram.messenger.ExteraIcon{}\"", camel(key)),
            1,
            "verify exteraGram aliases",
        )?;
    }
    expect_count(&manifest, "android:enabled=\"true\"\n            android:name=\"org.telegram.messenger.ExteralessIcon\"", 0, "verify old alias disabled")?;
    let app_tag = application_tag_text(&manifest, "verify application tag")?;
    expect_count(app_tag, "android:label=\"@string/AppName\"", 1, "verify label")?;
    expect_count(app_tag, "android:icon=\"@drawable/grape_icon_original\"", 1, "verify icon")?;
    expect_count(app_tag, "android:roundIcon=\"@drawable/grape_icon_original\"", 1, "verify roundIcon")?;
    let settings = read_file(&settings_path)?;
    expect_count(&settings, "case 7001:", 1, "verify settings case")?;
    let launcher = read_file(&launcher_path)?;
    expect_count(&launcher, "GRAPE_ORIGINAL", 3, "verify launcher enum")?;
    let props = read_file(&props_path)?;
    if get_gradle_property(&props, "APP_PACKAGE")? != PACKAGE {
        return Err("verify: APP_PACKAGE is not app.grapegram".to_string());
    }
    let build = read_file(&build_path)?;
    expect_count(&build, "defaultConfig.applicationId = APP_PACKAGE", 1, "verify applicationId")?;
    expect_count(&build, "storeType 'PKCS12'", 1, "verify signing storeType")?;
    let gs: Value = serde_json::from_str(&read_file(&gs_path)?)
        .map_err(|e| format!("verify google-services.json: {}", e))?;
    let clients = gs.get("client").and_then(Value::as_array).ok_or("verify: client array missing")?;
    if clients.len() != 2 {
        return Err(format!("verify: expected 2 google-services clients, found {}", clients.len()));
    }
    let hits = clients
        .iter()
        .filter(|c| {
            nested_string(c, &["client_info", "android_client_info", "package_name"]).as_deref()
                == Some(PACKAGE)
        })
        .count();
    if hits != 1 {
        return Err(format!("verify: expected 1 client with package {}, found {}", PACKAGE, hits));
    }

    println!(
        "applied: {} icon aliases (grapeGram {}, Cherrygram {}, exteraGram {}), settings entry, enum, strings, package {}, data_base {}",
        ICONS.len() + CHERRY_ICONS.len() + EXTERA_ICONS.len(),
        ICONS.len(),
        CHERRY_ICONS.len(),
        EXTERA_ICONS.len(),
        PACKAGE,
        data_base
    );
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("grape-materializer: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_base_commit_requires_exact_sha_and_detects_env_mismatch() {
        let pin = "31a35e4683fb420ee782cdd32f36aea812bf756c";
        assert_eq!(resolve_expected_base_commit(pin, None).unwrap(), pin);
        assert!(resolve_expected_base_commit(pin, Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")).is_err());
        assert!(resolve_expected_base_commit("", None).is_err());
        assert!(resolve_expected_base_commit("main", None).is_err());
    }

    #[test]
    fn removes_only_duplicate_launcher_pngs_shadowed_by_webp() {
        let root = env::temp_dir().join(format!("grape-materializer-icons-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let mipmap = root.join("mipmap-hdpi");
        fs::create_dir_all(&mipmap).unwrap();
        fs::write(mipmap.join("ic_launcher_foreground.png"), b"png").unwrap();
        fs::write(mipmap.join("ic_launcher_foreground.webp"), b"webp").unwrap();
        fs::write(mipmap.join("other_icon.png"), b"png").unwrap();
        fs::write(mipmap.join("other_icon.webp"), b"webp").unwrap();

        assert_eq!(remove_duplicate_launcher_pngs(&root).unwrap(), 1);
        assert!(!mipmap.join("ic_launcher_foreground.png").exists());
        assert!(mipmap.join("ic_launcher_foreground.webp").exists());
        assert!(mipmap.join("other_icon.png").exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn patch_once_counts_anchors() {
        assert_eq!(patch_once("a X b", "X", "Y", "t").unwrap(), "a Y b");
        assert!(patch_once("a X X b", "X", "Y", "t").is_err());
        assert!(patch_once("a b", "X", "Y", "t").is_err());
    }

    #[test]
    fn set_xml_string_replaces_value_only() {
        let src = "<resources>\n    <string name=\"AppName\">old</string>\n    <string name=\"Other\">keep</string>\n</resources>";
        let (out, n) = set_xml_string(src, "AppName", "grapeGram");
        assert_eq!(n, 1);
        assert!(out.contains("<string name=\"AppName\">grapeGram</string>"));
        assert!(out.contains("<string name=\"Other\">keep</string>"));
    }

    #[test]
    fn gradle_property_roundtrip() {
        let src = "APP_VERSION_CODE=7303\nAPP_PACKAGE=com.exteraless.app\nGLYPH_API_KEY=test\n";
        let out = set_gradle_property(src, "APP_PACKAGE", "app.grapegram").unwrap();
        assert_eq!(get_gradle_property(&out, "APP_PACKAGE").unwrap(), "app.grapegram");
        assert!(out.contains("APP_VERSION_CODE=7303\n"));
        assert!(set_gradle_property("A=1\nA=2\n", "A", "x").is_err());
    }

    #[test]
    fn alias_patch_targets_named_block() {
        let src = "<activity-alias\n            android:enabled=\"true\"\n            android:name=\"org.telegram.messenger.ExteralessIcon\">";
        let out = disable_alias(src, "org.telegram.messenger.ExteralessIcon").unwrap();
        assert!(out.contains("android:enabled=\"false\"\n            android:name=\"org.telegram.messenger.ExteralessIcon\""));
        assert!(disable_alias("nothing", "org.telegram.messenger.ExteralessIcon").is_err());
    }

    #[test]
    fn application_tag_patch_is_scoped() {
        let src = "<manifest>\n    <application\n        android:icon=\"@mipmap/ic_launcher_exteraless\"\n        android:label=\"@string/NagramX\"\n        android:roundIcon=\"@mipmap/ic_launcher_exteraless_round\"\n    >\n        <intent-filter android:icon=\"@mipmap/ic_launcher_exteraless\" />\n    </application>\n</manifest>";
        let out = patch_application_tag(
            src,
            &[
                ("android:icon=\"@mipmap/ic_launcher_exteraless\"", "android:icon=\"@drawable/grape_icon_original\""),
                ("android:label=\"@string/NagramX\"", "android:label=\"@string/AppName\""),
                ("android:roundIcon=\"@mipmap/ic_launcher_exteraless_round\"", "android:roundIcon=\"@drawable/grape_icon_original\""),
            ],
            "t",
        ).unwrap();
        assert!(out.contains("android:label=\"@string/AppName\""));
        assert!(out.contains("<intent-filter android:icon=\"@mipmap/ic_launcher_exteraless\" />"));
    }

    #[test]
    fn upstream_icon_catalogs_have_unique_components_and_resources() {
        assert!(CHERRY_ICONS.len() >= 18);
        assert_eq!(EXTERA_ICONS.len(), 31);
        let cherry = cherry_enum_entries();
        let extera = extera_enum_entries();
        assert_eq!(cherry.lines().count(), CHERRY_ICONS.len());
        assert_eq!(extera.lines().count(), EXTERA_ICONS.len());
        assert!(upstream_aliases().contains("org.telegram.messenger.CherryIconDarkCherry"));
        assert!(upstream_aliases().contains("org.telegram.messenger.ExteraIconNyanCat"));
        assert!(upstream_string_entries().contains("name=\"ExteraIconGoogle26\""));
    }

    #[test]
    fn templates_match_expected_names() {
        let enums = enum_entries();
        assert!(enums.contains("GRAPE_ORIGINAL(\"GrapeIconOriginal\""));
        assert!(enums.contains("GRAPE_SILK(\"GrapeIconSilk\""));
        assert_eq!(enums.matches("GRAPE_").count(), 9);
        let alias = alias_template("original", true);
        assert!(alias.contains("android:enabled=\"true\""));
        assert!(alias.contains("org.telegram.messenger.GrapeIconOriginal"));
        assert!(alias.contains("@drawable/grape_icon_original"));
    }
}
