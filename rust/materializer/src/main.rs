use serde::Deserialize;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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
const EXT_ICONS: [(&str, &str); 17] = [
    ("mole", "Mole"),
    ("echpochmak", "Echpochmak"),
    ("avatar", "Avatar"),
    ("candy", "Candy"),
    ("guitar", "Guitar"),
    ("frog", "Frog"),
    ("hamster", "Hamster"),
    ("sponge", "Sponge"),
    ("flappy", "Flappy"),
    ("nyan_cat", "Nyan Cat"),
    ("fish", "Fish"),
    ("psychosis", "Psychosis"),
    ("xp", "XP"),
    ("nasa", "NASA"),
    ("jellyfish", "Jellyfish"),
    ("human", "Human"),
    ("osu", "osu!"),
];
const CHERRY_ICONS: [(&str, &str, &str, &str, &str, &str); 18] = [
    ("cherry", "Cherrygram", "R.drawable.icon_background_default", "R.drawable.icon_foreground_cherry", "@drawable/icon_background_default", "@drawable/icon_foreground_cherry"),
    ("dark_cherry", "Тёмная вишня", "R.drawable.icon_background_dark", "R.drawable.icon_foreground_cherry", "@drawable/icon_background_dark", "@drawable/icon_foreground_cherry"),
    ("dark_cherry_bra", "Тёмная вишня (Bra)", "R.drawable.icon_background_dark", "R.drawable.icon_foreground_cherry_bra", "@drawable/icon_background_dark", "@drawable/icon_foreground_cherry_bra"),
    ("white_cherry", "Белая вишня", "R.drawable.icon_background_white", "R.drawable.icon_foreground_cherry_white", "@drawable/icon_background_white", "@drawable/icon_foreground_cherry_white"),
    ("white_cherry_bra", "Белая вишня (Bra)", "R.drawable.icon_background_white", "R.drawable.icon_foreground_cherry_white_bra", "@drawable/icon_background_white", "@drawable/icon_foreground_cherry_white_bra"),
    ("laguna_cherry", "Лагуна", "R.drawable.icon_background_laguna", "R.drawable.icon_foreground_cherry", "@drawable/icon_background_laguna", "@drawable/icon_foreground_cherry"),
    ("aqua_cherry", "Аква", "R.drawable.icon_background_aqua", "R.drawable.icon_foreground_cherry", "@drawable/icon_background_aqua", "@drawable/icon_foreground_cherry"),
    ("green_cherry", "Зелёная", "R.drawable.icon_background_green", "R.drawable.icon_foreground_cherry", "@drawable/icon_background_green", "@drawable/icon_foreground_cherry"),
    ("lavanda_cherry", "Лаванда", "R.drawable.icon_background_lavanda", "R.drawable.icon_foreground_cherry", "@drawable/icon_background_lavanda", "@drawable/icon_foreground_cherry"),
    ("violet_sunset_cherry", "Фиолетовый закат", "R.drawable.icon_background_violet_sunset", "R.drawable.icon_foreground_cherry", "@drawable/icon_background_violet_sunset", "@drawable/icon_foreground_cherry"),
    ("violet_sunset_cherry_bra", "Фиолетовый закат (Bra)", "R.drawable.icon_background_violet_sunset", "R.drawable.icon_foreground_cherry_bra", "@drawable/icon_background_violet_sunset", "@drawable/icon_foreground_cherry_bra"),
    ("sunset_cherry", "Закат", "R.drawable.icon_background_sunset", "R.drawable.icon_foreground_cherry", "@drawable/icon_background_sunset", "@drawable/icon_foreground_cherry"),
    ("sunrise_cherry", "Рассвет", "R.drawable.icon_background_sunrise", "R.drawable.icon_foreground_cherry", "@drawable/icon_background_sunrise", "@drawable/icon_foreground_cherry"),
    ("palms_cherry", "Пальмы", "R.drawable.icon_background_palms", "R.drawable.icon_foreground_cherry", "@drawable/icon_background_palms", "@drawable/icon_foreground_cherry"),
    ("palms_cherry_bra", "Пальмы (Bra)", "R.drawable.icon_background_palms", "R.drawable.icon_foreground_cherry_bra", "@drawable/icon_background_palms", "@drawable/icon_foreground_cherry_bra"),
    ("turbo_cherry", "Turbo Cherry", "R.drawable.icon_5_background_sa", "R.drawable.icon_foreground_cherry", "@drawable/icon_5_background_sa", "@drawable/icon_foreground_cherry"),
    ("nox_cherry", "Nox Cherry", "R.mipmap.icon_2_background_sa", "R.drawable.icon_foreground_cherry", "@mipmap/icon_2_background_sa", "@drawable/icon_foreground_cherry"),
    ("dark_ny", "Тёмная NY", "R.drawable.icon_background_dark_ny", "R.drawable.icon_foreground_cherry_bra", "@drawable/icon_background_dark_ny", "@drawable/icon_foreground_cherry_bra"),
];

#[derive(Deserialize)]
struct Config {
    data_base: String,
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

fn patch_once(text: &str, anchor: &str, replacement: &str, what: &str) -> Result<String, String> {
    let n = text.matches(anchor).count();
    if n != 1 {
        return Err(format!("{}: anchor found {} times, expected 1", what, n));
    }
    Ok(text.replacen(anchor, replacement, 1))
}

fn patch_exact_count(
    text: &str,
    anchor: &str,
    replacement: &str,
    expected: usize,
    what: &str,
) -> Result<String, String> {
    let n = text.matches(anchor).count();
    if n != expected {
        return Err(format!(
            "{}: anchor found {} times, expected {}",
            what, n, expected
        ));
    }
    Ok(text.replace(anchor, replacement))
}

fn replace_java_method(
    text: &str,
    signature: &str,
    replacement: &str,
    what: &str,
) -> Result<String, String> {
    let count = text.matches(signature).count();
    if count != 1 {
        return Err(format!(
            "{}: method signature found {} times, expected 1",
            what, count
        ));
    }
    let start = text.find(signature).unwrap();
    let open = text[start..]
        .find('{')
        .map(|offset| start + offset)
        .ok_or_else(|| format!("{}: method body not found", what))?;
    let mut depth = 0usize;
    let mut end = None;
    for (offset, ch) in text[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end = Some(open + offset + ch.len_utf8());
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end.ok_or_else(|| format!("{}: unterminated method body", what))?;
    let mut out = String::with_capacity(text.len() + replacement.len());
    out.push_str(&text[..start]);
    out.push_str(replacement);
    out.push_str(&text[end..]);
    Ok(out)
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
                .replace("Exteraless", "grapeGram")
                .replace("ExteraLess", "grapeGram")
                .replace("ExteraLESS", "grapeGram")
                .replace("EXTERALESS", "grapeGram"),
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
    let mut chars = key.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
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

fn alias_template(key: &str, icon_res: &str, enabled: bool) -> String {
    format!(
        r#"        <activity-alias
            android:enabled="{enabled}"
            android:name="org.telegram.messenger.GrapeIcon{camel}"
            android:targetActivity="org.telegram.ui.LaunchActivity"
            android:icon="{icon}"
            android:roundIcon="{icon}"
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
        icon = icon_res
    )
}

fn cherry_alias_template(key: &str, icon_res: &str) -> String {
    format!(
        r#"        <activity-alias
            android:enabled="false"
            android:name="org.telegram.messenger.GrapeCherryIcon{camel}"
            android:targetActivity="org.telegram.ui.LaunchActivity"
            android:icon="{icon}"
            android:roundIcon="{icon}"
            android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
                <category android:name="android.intent.category.MULTIWINDOW_LAUNCHER" />
            </intent-filter>
            <meta-data android:name="android.app.shortcuts" android:resource="@xml/shortcuts" />
        </activity-alias>

"#,
        camel = camel(key),
        icon = icon_res
    )
}

fn enum_entries() -> String {
    let mut entries = Vec::new();
    for (key, _) in ICONS.iter() {
        entries.push(format!(
            "        GRAPE_{}(\"GrapeIcon{}\", R.drawable.grape_icon_{}, R.drawable.app_icon_empty_foreground, R.string.GrapeIcon{})",
            key.to_uppercase(),
            camel(key),
            key,
            camel(key)
        ));
    }
    for (id, _) in EXT_ICONS.iter() {
        entries.push(format!(
            "        EXT_{}(\"GrapeIcon{}\", R.drawable.ext_{}_background, R.drawable.ext_{}_foreground, R.string.GrapeIcon{})",
            id.to_uppercase(),
            camel(id),
            id,
            id,
            camel(id)
        ));
    }
    for (key, _, bg, fg, _, _) in CHERRY_ICONS.iter() {
        entries.push(format!(
            "        CHERRY_{}(\"GrapeCherryIcon{}\", {}, {}, R.string.GrapeIcon{})",
            key.to_uppercase(),
            camel(key),
            bg,
            fg,
            camel(key)
        ));
    }
    entries.join(",\n")
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
    let data_base = env::var("GRP_DATA_BASE")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or(cfg.data_base);
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

    // GrapeBadges uses BC's lightweight Ed25519 verifier. One upstream
    // dependency already brings the legacy jdk15to18 artifact; exclude it to
    // prevent duplicate classes when adding the maintained jdk18on artifact.
    let mut build_gradle = read_file(&build_path)?;
    build_gradle = patch_once(
        &build_gradle,
        "configurations.configureEach {\n",
        "configurations.configureEach {\n    resolutionStrategy.dependencySubstitution {\n        substitute(module('org.bouncycastle:bcprov-jdk15to18')).using(module('org.bouncycastle:bcprov-jdk18on:1.83'))\n    }\n    exclude group: 'org.bouncycastle', module: 'bcprov-jdk15to18'\n",
        "exclude legacy Bouncy Castle provider",
    )?;
    build_gradle = patch_once(
        &build_gradle,
        "dependencies {\n",
        "dependencies {\n    implementation 'org.bouncycastle:bcprov-jdk18on:1.83'\n",
        "Ed25519 verifier dependency",
    )?;
    write_file(&build_path, &build_gradle)?;

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
        .replace("@PUBLIC_KEY_PEM@", &pem.replace('\n', "\n"));
    write_file(&gconf_path, &gconf)?;

    let row_anchor = "        items.add(UItem.asCustomShadow(topView, 200 - 12));\n";
    let row = format!(
        "{}        // grapeGram: единственный дополнительный пункт, первым в списке\n        items.add(SettingCell.Factory.of(7001, 0xFF7C4DFF, 0xFF5E35B1, R.drawable.grape_icon_original, \"Настройки grapeGram\", \"персонализация · функции · плагины\"));\n        items.add(UItem.asShadow(null));\n",
        row_anchor
    );
    let case_anchor = "            case -1: {\n                presentFragment(new OpenExteraSettingsActivity());\n                break;\n            }\n";
    let new_case = "            case 7001: {\n                presentFragment(new app.grapegram.GrapeSettingsActivity());\n                break;\n            }\n";
    let old_row = "        items.add(SettingCell.Factory.of(-1, 0xFF993C38, 0xFF993C38, R.drawable.exteraless_icon_tile, getString(R.string.OpenExteraPreferences), getString(R.string.OpenExteraInfo)));\n        items.add(UItem.asShadow(null));\n";
    let mut s = read_file(&settings_path)?;
    s = patch_once(&s, row_anchor, &row, "settings row")?;
    s = patch_once(&s, case_anchor, &format!("{}{}", new_case, case_anchor), "settings click")?;
    s = patch_once(&s, old_row, "        // exteraless entry moved into the grapeGram hub\n", "remove old top-level entry")?;
    s = patch_once(
        &s,
        "            case -1: {\n                presentFragment(new OpenExteraSettingsActivity());\n                break;\n            }\n",
        "            case -1: {\n                presentFragment(new app.grapegram.GrapeSettingsActivity());\n                break;\n            }\n",
        "settings legacy route",
    )?;
    write_file(&settings_path, &s)?;

    let launch_path = proj.join("src/main/java/org/telegram/ui/LaunchActivity.java");
    let mut launch = read_file(&launch_path)?;
    launch = patch_once(
        &launch,
        "                } else if (open_settings == 100) {\n                    fragment = new OpenExteraSettingsActivity();\n",
        "                } else if (open_settings == 100) {\n                    fragment = new app.grapegram.GrapeSettingsActivity();\n",
        "launch settings route",
    )?;
    launch = patch_once(
        &launch,
        "    protected void onResume() {\n        super.onResume();\n",
        "    protected void onResume() {\n        super.onResume();\n        app.grapegram.GrapeBadgeBridge.refreshIfDue(this);\n",
        "badge refresh on foreground",
    )?;
    write_file(&launch_path, &launch)?;

    let menu_path = proj.join("src/main/java/app/exteraless/drawer/MainMenuHelper.java");
    let mut menu = read_file(&menu_path)?;
    menu = patch_once(
        &menu,
        "                return new MenuItemInfo(R.drawable.exteraless_icon_tile, LocaleController.getString(R.string.OpenExteraPreferences),\n                        () -> fragment.presentFragment(new OpenExteraSettingsActivity()), null);\n",
        "                return new MenuItemInfo(R.drawable.grape_icon_original, LocaleController.getString(R.string.OpenExteraPreferences),\n                        () -> fragment.presentFragment(new app.grapegram.GrapeSettingsActivity()), null);\n",
        "drawer settings route",
    )?;
    menu = patch_once(
        &menu,
        "            case EXTERALESS_SETTINGS -> new MenuItemInfo(R.drawable.exteraless_icon_tile, LocaleController.getString(R.string.OpenExteraPreferences), null, null);",
        "            case EXTERALESS_SETTINGS -> new MenuItemInfo(R.drawable.grape_icon_original, LocaleController.getString(R.string.OpenExteraPreferences), null, null);",
        "drawer settings icon",
    )?;
    write_file(&menu_path, &menu)?;

    let settings_helper_path = proj.join("src/main/java/tw/nekomimi/nekogram/helpers/SettingsHelper.java");
    let mut settings_helper = read_file(&settings_helper_path)?;
    // Both the modern "settings" route and the legacy "exteraless" route
    // resolve to this upstream activity. GrapeSettingsActivity extends BaseFragment,
    // not BaseNekoSettingsActivity, so do not assign it to neko_fragment.
    settings_helper = patch_exact_count(
        &settings_helper,
        "fragment = neko_fragment = new OpenExteraSettingsActivity();",
        "fragment = new app.grapegram.GrapeSettingsActivity();",
        2,
        "legacy settings deep links",
    )?;
    write_file(&settings_helper_path, &settings_helper)?;

    let settings_registry_path = proj.join("src/main/java/app/exteraless/settings/utils/SettingsRegistry.java");
    let mut settings_registry = read_file(&settings_registry_path)?;
    settings_registry = patch_once(
        &settings_registry,
        "        icons.put(OpenExteraSettingsActivity.class, R.drawable.msg_settings);",
        "        icons.put(app.grapegram.GrapeSettingsActivity.class, R.drawable.grape_icon_original);",
        "grapeGram settings search icon",
    )?;
    write_file(&settings_registry_path, &settings_registry)?;

    let about_header_path = proj.join("src/main/java/app/exteraless/settings/AboutHeaderCell.java");
    let mut about_header = read_file(&about_header_path)?;
    about_header = patch_once(
        &about_header,
        "public static final int LOGO_BACKGROUND = 0xFFE83030;",
        "public static final int LOGO_BACKGROUND = 0xFF7956C8;",
        "grapeGram settings logo color",
    )?;
    about_header = patch_once(
        &about_header,
        "logo.setImageResource(R.drawable.exteraless_icon);",
        "logo.setImageResource(R.drawable.grape_icon_original);",
        "grapeGram settings logo",
    )?;
    write_file(&about_header_path, &about_header)?;

    let updater_path = proj.join("src/main/java/app/exteraless/updater/UpdaterSheet.java");
    let mut updater = read_file(&updater_path)?;
    updater = patch_once(
        &updater,
        "logo.setImageResource(R.drawable.exteraless_icon);",
        "logo.setImageResource(R.drawable.grape_icon_original);",
        "grapeGram updater logo",
    )?;
    write_file(&updater_path, &updater)?;

    let qr_path = proj.join("src/main/java/org/telegram/messenger/TelegramQRCodeWriter.java");
    let mut qr = read_file(&qr_path)?;
    qr = patch_once(
        &qr,
        "getResources().getDrawable(R.drawable.exteraless_icon)",
        "getResources().getDrawable(R.drawable.grape_icon_original)",
        "grapeGram QR logo",
    )?;
    write_file(&qr_path, &qr)?;

    let glyph_path = proj.join("src/main/java/app/exteraless/glyph/GlyphController.java");
    let mut glyph = read_file(&glyph_path)?;
    glyph = patch_once(
        &glyph,
        "context.getDrawable(R.drawable.exteraless_icon_monochrome)",
        "context.getDrawable(R.drawable.grape_icon_original)",
        "grapeGram glyph logo",
    )?;
    write_file(&glyph_path, &glyph)?;

    let plugin_log_path = proj.join("src/main/java/app/exteraless/plugins/PluginLog.java");
    let mut plugin_log = read_file(&plugin_log_path)?;
    plugin_log = patch_once(
        &plugin_log,
        "String header = \"exteraless \" + BuildConfig.BUILD_VERSION_STRING",
        "String header = \"grapeGram \" + BuildConfig.BUILD_VERSION_STRING",
        "grapeGram plugin log branding",
    )?;
    write_file(&plugin_log_path, &plugin_log)?;

    let theme_path = proj.join("src/main/java/org/telegram/ui/ThemeActivity.java");
    let mut theme = read_file(&theme_path)?;
    theme = patch_once(
        &theme,
        "            appIconHeaderRow = rowCount++;\n            appIconSelectorRow = rowCount++;\n            appIconShadowRow = rowCount++;\n",
        "            appIconHeaderRow = -1;\n            appIconSelectorRow = -1;\n            appIconShadowRow = -1;\n",
        "remove duplicate icon picker",
    )?;
    write_file(&theme_path, &theme)?;

    let other_path = proj.join("src/main/java/app/exteraless/settings/OpenExteraOtherActivity.java");
    let mut other = read_file(&other_path)?;
    other = patch_once(
        &other,
        "        nagramHeaderRow = addRow(\"nagramHeader\");\n        nagramSettingsRow = -1;\n        ayuMomentsRow = addRow(\"ayuMoments\");\n        nagramDividerRow = addRow();\n",
        "        nagramHeaderRow = -1;\n        nagramSettingsRow = -1;\n        ayuMomentsRow = -1;\n        nagramDividerRow = -1;\n",
        "remove duplicate ayumoments entry",
    )?;
    write_file(&other_path, &other)?;

    let chats_path = proj.join("src/main/java/app/exteraless/settings/OpenExteraChatsActivity.java");
    let mut chats = read_file(&chats_path)?;
    chats = patch_once(
        &chats,
        "        aiChatRow = addRow(\"aiChat\");\n",
        "        aiChatRow = -1;\n",
        "remove duplicate ai entry",
    )?;
    write_file(&chats_path, &chats)?;

    let profile_path = proj.join("src/main/java/org/telegram/ui/ProfileActivity.java");
    let mut profile = read_file(&profile_path)?;
    profile = patch_once(
        &profile,
        "        badgeDrawable.setParticles(false, animated);\n",
        "        badgeDrawable.setParticles(true, animated);\n",
        "badge particles",
    )?;
    write_file(&profile_path, &profile)?;

    // The chat-list title inherits an upstream default; migrate legacy names at display time.
    let typeface_path = proj.join("src/main/java/tw/nekomimi/nekogram/helpers/TypefaceHelper.java");
    let mut typeface = read_file(&typeface_path)?;
    typeface = patch_once(
        &typeface,
        "        String title = (String) NaConfig.INSTANCE.getCustomTitle().defaultValue;\n",
        "        String title = (String) NaConfig.INSTANCE.getCustomTitle().defaultValue;\n        if (title != null && (title.equalsIgnoreCase(\"exteraless\") || title.equalsIgnoreCase(\"ExteraLess\") || title.equalsIgnoreCase(\"NagramX\"))) {\n            title = \"grapeGram\";\n        }\n",
        "grapeGram default chat-list title",
    )?;
    typeface = patch_once(
        &typeface,
        "            String customTitle = NaConfig.INSTANCE.getCustomTitle().String();\n",
        "            String customTitle = NaConfig.INSTANCE.getCustomTitle().String();\n            if (customTitle != null && (customTitle.equalsIgnoreCase(\"exteraless\") || customTitle.equalsIgnoreCase(\"ExteraLess\") || customTitle.equalsIgnoreCase(\"NagramX\"))) {\n                customTitle = \"grapeGram\";\n            }\n",
        "grapeGram custom chat-list title",
    )?;
    write_file(&typeface_path, &typeface)?;

    // Notification branding must not fall back to ExteraLess assets.
    let notifications_path = proj.join("src/main/java/org/telegram/messenger/NotificationsController.java");
    let mut notifications = read_file(&notifications_path)?;
    notifications = replace_java_method(
        &notifications,
        "    private int getNotificationIconResId()",
        "    private int getNotificationIconResId() {\n        return R.drawable.grape_icon_original;\n    }",
        "grapeGram notification icon",
    )?;
    notifications = patch_once(
        &notifications,
        ".setSmallIcon(R.drawable.notification)",
        ".setSmallIcon(R.drawable.grape_icon_original)",
        "grapeGram wallet notification icon",
    )?;
    write_file(&notifications_path, &notifications)?;

    let notification_service_path = proj.join("src/main/java/org/telegram/messenger/NotificationsService.java");
    let mut notification_service = read_file(&notification_service_path)?;
    notification_service = patch_once(
        &notification_service,
        ".setSmallIcon(R.drawable.exteraless_notification)",
        ".setSmallIcon(R.drawable.grape_icon_original)",
        "grapeGram foreground notification icon",
    )?;
    write_file(&notification_service_path, &notification_service)?;

    let session_cell_path = proj.join("src/main/java/org/telegram/ui/Cells/SessionCell.java");
    let mut session_cell = read_file(&session_cell_path)?;
    session_cell = patch_once(
        &session_cell,
        "iconId = R.drawable.exteraless_notification;",
        "iconId = R.drawable.grape_icon_original;",
        "grapeGram session app icon",
    )?;
    write_file(&session_cell_path, &session_cell)?;

    // Surface verified grapeGram badges in Telegram's reusable dialog/member rows.
    let user_cell_path = proj.join("src/main/java/org/telegram/ui/Cells/UserCell.java");
    let mut user_cell = read_file(&user_cell_path)?;
    user_cell = patch_once(
        &user_cell,
        "import org.telegram.messenger.FileLog;\n",
        "import org.telegram.messenger.FileLog;\nimport com.exteragram.messenger.api.dto.BadgeDTO;\nimport com.exteragram.messenger.badges.BadgesController;\n",
        "UserCell badge imports",
    )?;
    user_cell = patch_once(
        &user_cell,
        "    private final AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable emojiStatus;\n",
        "    private final AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable emojiStatus;\n    private final AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable grapeBadge;\n",
        "UserCell badge drawable field",
    )?;
    user_cell = patch_once(
        &user_cell,
        "        emojiStatus = new AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable(nameTextView, dp(20));\n",
        "        emojiStatus = new AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable(nameTextView, dp(20));\n        grapeBadge = new AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable(nameTextView, dp(20));\n",
        "UserCell badge drawable init",
    )?;
    user_cell = patch_once(
        &user_cell,
        "        if (currentUser != null && MessagesController.getInstance(currentAccount).isPremiumUser(currentUser) && !MessagesController.getInstance(currentAccount).premiumFeaturesBlocked()) {\n",
        "        BadgeDTO grapeBadgeInfo = currentUser == null ? null : BadgesController.INSTANCE.getBadge(currentUser);\n        if (grapeBadgeInfo != null) {\n            grapeBadge.set(grapeBadgeInfo.getDocumentId(), false);\n            grapeBadge.setColor(Theme.getColor(Theme.key_chats_verifiedBackground, resourcesProvider));\n            grapeBadge.setParticles(true, false);\n            nameTextView.setRightDrawable(grapeBadge);\n            nameTextView.setRightDrawableTopPadding(-dp(0.5f));\n        } else if (currentUser != null && MessagesController.getInstance(currentAccount).isPremiumUser(currentUser) && !MessagesController.getInstance(currentAccount).premiumFeaturesBlocked()) {\n",
        "UserCell badge display",
    )?;
    user_cell = patch_once(
        &user_cell,
        "        if (id == NotificationCenter.emojiLoaded) {\n            nameTextView.invalidate();\n        }\n",
        "        if (id == NotificationCenter.emojiLoaded) {\n            nameTextView.invalidate();\n        } else if (id == NotificationCenter.updateInterfaces && account == currentAccount) {\n            int mask = args != null && args.length > 0 && args[0] instanceof Integer\n                    ? (Integer) args[0] : 0;\n            if ((mask & MessagesController.UPDATE_MASK_EMOJI_STATUS) != 0) {\n                update(0);\n            }\n        }\n",
        "UserCell badge refresh notification",
    )?;
    user_cell = patch_once(
        &user_cell,
        "        emojiStatus.attach();\n        botVerification.attach();\n",
        "        emojiStatus.attach();\n        botVerification.attach();\n        grapeBadge.attach();\n        NotificationCenter.getInstance(currentAccount).addObserver(this, NotificationCenter.updateInterfaces);\n",
        "UserCell badge attach",
    )?;
    user_cell = patch_once(
        &user_cell,
        "        emojiStatus.detach();\n        botVerification.detach();\n",
        "        emojiStatus.detach();\n        botVerification.detach();\n        grapeBadge.detach();\n        NotificationCenter.getInstance(currentAccount).removeObserver(this, NotificationCenter.updateInterfaces);\n",
        "UserCell badge detach",
    )?;
    write_file(&user_cell_path, &user_cell)?;

    // Surface the same signed badge in the one-to-one/group chat action-bar title.
    let chat_header_path = proj.join("src/main/java/org/telegram/ui/Components/ChatAvatarContainer.java");
    let mut chat_header = read_file(&chat_header_path)?;
    chat_header = patch_once(
        &chat_header,
        "import com.radolyn.ayugram.utils.LastSeenHelper;\n",
        "import com.radolyn.ayugram.utils.LastSeenHelper;\nimport com.exteragram.messenger.api.dto.BadgeDTO;\nimport com.exteragram.messenger.badges.BadgesController;\n",
        "chat header badge imports",
    )?;
    chat_header = patch_once(
        &chat_header,
        "    private final AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable botVerificationDrawable;\n",
        "    private final AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable botVerificationDrawable;\n    private final AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable grapeBadgeDrawable;\n",
        "chat header badge field",
    )?;
    chat_header = patch_once(
        &chat_header,
        "        botVerificationDrawable = new AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable(titleTextView, dp(17));\n",
        "        botVerificationDrawable = new AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable(titleTextView, dp(17));\n        grapeBadgeDrawable = new AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable(titleTextView, dp(24));\n",
        "chat header badge init",
    )?;
    chat_header = patch_once(
        &chat_header,
        "        titleTextView.setText(value);\n        titleTextView.setScrollNonFitText(scrollable || isCentered());\n",
        "        titleTextView.setText(value);\n        titleTextView.setScrollNonFitText(scrollable || isCentered());\n        TLRPC.User grapeBadgeUser = parentFragment == null ? null : parentFragment.getCurrentUser();\n        BadgeDTO grapeBadgeInfo = grapeBadgeUser == null ? null : BadgesController.INSTANCE.getBadge(grapeBadgeUser);\n",
        "chat header badge lookup",
    )?;
    chat_header = patch_once(
        &chat_header,
        "        if (premium || DialogObject.getEmojiStatusDocumentId(emojiStatus) != 0) {\n",
        "        if (grapeBadgeInfo != null) {\n            grapeBadgeDrawable.set(grapeBadgeInfo.getDocumentId(), animated);\n            grapeBadgeDrawable.setColor(getThemedColor(Theme.key_profile_verifiedBackground));\n            grapeBadgeDrawable.setParticles(true, animated);\n            titleTextView.setRightDrawable(grapeBadgeDrawable);\n            rightDrawableIsScamOrVerified = false;\n            rightDrawableContentDescription = \"grapeGram\";\n            if (isCentered()) {\n                titleTextView.setRightDrawable2(null);\n            }\n        } else if (premium || DialogObject.getEmojiStatusDocumentId(emojiStatus) != 0) {\n",
        "chat header badge display",
    )?;
    chat_header = patch_once(
        &chat_header,
        "            NotificationCenter.getGlobalInstance().addWeakObserver(this, NotificationCenter.emojiLoaded);\n",
        "            NotificationCenter.getGlobalInstance().addWeakObserver(this, NotificationCenter.emojiLoaded);\n            NotificationCenter.getInstance(currentAccount).addWeakObserver(this, NotificationCenter.updateInterfaces);\n",
        "chat header badge observer",
    )?;
    chat_header = patch_once(
        &chat_header,
        "            NotificationCenter.getGlobalInstance().removeObserver(this, NotificationCenter.emojiLoaded);\n",
        "            NotificationCenter.getGlobalInstance().removeObserver(this, NotificationCenter.emojiLoaded);\n            NotificationCenter.getInstance(currentAccount).removeObserver(this, NotificationCenter.updateInterfaces);\n",
        "chat header badge observer removal",
    )?;
    chat_header = patch_once(
        &chat_header,
        "        if (botVerificationDrawable != null) {\n            botVerificationDrawable.attach();\n        }\n",
        "        if (botVerificationDrawable != null) {\n            botVerificationDrawable.attach();\n        }\n        grapeBadgeDrawable.attach();\n",
        "chat header badge attach",
    )?;
    chat_header = patch_once(
        &chat_header,
        "        if (botVerificationDrawable != null) {\n            botVerificationDrawable.detach();\n        }\n",
        "        if (botVerificationDrawable != null) {\n            botVerificationDrawable.detach();\n        }\n        grapeBadgeDrawable.detach();\n",
        "chat header badge detach",
    )?;
    chat_header = patch_once(
        &chat_header,
        "        } else if (id == NotificationCenter.savedMessagesDialogsUpdate) {\n            updateSubtitle(true);\n        }\n",
        "        } else if (id == NotificationCenter.updateInterfaces && account == currentAccount && parentFragment != null) {\n            int mask = args != null && args.length > 0 && args[0] instanceof Integer\n                    ? (Integer) args[0] : 0;\n            if ((mask & MessagesController.UPDATE_MASK_EMOJI_STATUS) != 0) {\n                TLRPC.User user = parentFragment.getCurrentUser();\n                if (user != null) {\n                    setTitle(titleTextView.getText(), user.scam, user.fake, user.verified,\n                            MessagesController.getInstance(currentAccount).isPremiumUser(user)\n                                    && !MessagesController.getInstance(currentAccount).premiumFeaturesBlocked(),\n                            user.emoji_status, false);\n                }\n            }\n        } else if (id == NotificationCenter.savedMessagesDialogsUpdate) {\n            updateSubtitle(true);\n        }\n",
        "chat header badge refresh",
    )?;
    write_file(&chat_header_path, &chat_header)?;

    let nox = "NOX(\"NoxIcon\", R.mipmap.icon_2_background_sa, R.mipmap.icon_foreground_sa, R.string.AppIconNox);";
    let enum_replacement = format!(
        "{},\n{};",
        &nox[..nox.len() - 1],
        enum_entries()
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

    // The launcher picker exposes only grapeGram's nine curated icons. Legacy
    // exteraless aliases remain disabled and cannot leak into the client UI.
    let app_icons_path = proj.join("src/main/java/app/exteraless/appicons/AppIcons.java");
    let mut app_icons = read_file(&app_icons_path)?;
    app_icons = patch_once(
        &app_icons,
        "        for (LauncherIcon icon : LauncherIcon.values()) {\n            if (!icon.premium || !blocked || LauncherIconController.isEnabled(icon)) {\n                icons.add(icon);\n            }\n        }",
        "        for (LauncherIcon icon : LauncherIcon.values()) {\n            if ((icon.name().startsWith(\"GRAPE_\") || icon.name().startsWith(\"CHERRY_\"))\n                    && (!icon.premium || !blocked || LauncherIconController.isEnabled(icon))) {\n                icons.add(icon);\n            }\n        }",
        "grapeGram-only launcher icons",
    )?;
    write_file(&app_icons_path, &app_icons)?;

    let mut m = disable_alias(&manifest_before, "org.telegram.messenger.ExteralessIcon")?;
    let adaptive_dir = proj.join("src/main/res/mipmap-anydpi-v26");
    let legacy_icon_dir = proj.join("src/main/res/mipmap-anydpi");
    fs::create_dir_all(&adaptive_dir).map_err(|e| format!("create {}: {}", adaptive_dir.display(), e))?;
    fs::create_dir_all(&legacy_icon_dir).map_err(|e| format!("create {}: {}", legacy_icon_dir.display(), e))?;
    let mut cherry_aliases = String::new();
    for (key, _, _, _, bg, fg) in CHERRY_ICONS.iter() {
        let adaptive = format!(
            r#"<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="{bg}" />
    <foreground android:drawable="{fg}" />
</adaptive-icon>
"#,
            bg = bg,
            fg = fg
        );
        let legacy = format!(
            r#"<layer-list xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:drawable="{bg}" />
    <item android:drawable="{fg}" />
</layer-list>
"#,
            bg = bg,
            fg = fg
        );
        write_file(&adaptive_dir.join(format!("grape_cherry_icon_{}.xml", key)), &adaptive)?;
        write_file(&legacy_icon_dir.join(format!("grape_cherry_icon_{}.xml", key)), &legacy)?;
        cherry_aliases.push_str(&cherry_alias_template(key, &format!("@mipmap/grape_cherry_icon_{}", key)));
    }
    let aliases = ICONS
        .iter()
        .map(|(key, _)| alias_template(key, &format!("@drawable/grape_icon_{}", key), *key == "original"))
        .chain(EXT_ICONS.iter().map(|(id, _)| {
            alias_template(id, &format!("@mipmap/ext_icon_{}", id), false)
        }))
        .collect::<String>() + &cherry_aliases;
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
    let add = ICONS
        .iter()
        .map(|(key, name)| format!("    <string name=\"GrapeIcon{}\">{}</string>\n", camel(key), name))
        .chain(
            EXT_ICONS
                .iter()
                .map(|(id, name)| format!("    <string name=\"GrapeIcon{}\">{}</string>\n", camel(id), name)),
        )
        .chain(CHERRY_ICONS.iter().map(|(key, name, _, _, _, _)| {
            format!("    <string name=\"GrapeIcon{}\">{}</string>\n", camel(key), name)
        }))
        .collect::<String>();
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
    let preset_google_key = concat!("AIzaSyDLEeFI5", "OtFBwYBIoK_jj5m32rZK5CkCXA");
    google_translate = patch_once(
        &google_translate,
        &format!(".addQueryParameter(\"key\", \"{}\")", preset_google_key),
        ".addQueryParameter(\"key\", grapeTranslateKey)",
        "remove preset google api key",
    )?;
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

    let chats_path = proj.join("src/main/java/app/exteraless/settings/OpenExteraChatsActivity.java");
    let mut chats = read_file(&chats_path)?;
    chats = patch_once(&chats,
        "    private int linksHeaderRow = -1;",
        "    private int appIconRow = -1;\n    private int linksHeaderRow = -1;",
        "chats icon field")?;
    chats = patch_once(&chats,
        "        linksHeaderRow = addRow(\"linksHeader\");",
        "        appIconRow = addRow();\n        linksHeaderRow = addRow(\"linksHeader\");",
        "chats icon row")?;
    chats = patch_once(&chats,
        "        } else if (position == aiChatRow) {\n            presentFragment(new app.exteraless.ai.ui.AiSettingsActivity());\n            return;\n        }",
        "        } else if (position == appIconRow) {\n            presentFragment(new app.exteraless.appicons.AppIconsActivity());\n            return;\n        } else if (position == aiChatRow) {\n            presentFragment(new app.exteraless.ai.ui.AiSettingsActivity());\n            return;\n        }",
        "chats icon click")?;
    chats = patch_once(&chats,
        "            } else if (position == aiChatRow) {\n                cell.setTextAndIcon(getString(R.string.OEChatsAiChat), aiChatIcon(), true);",
        "            } else if (position == appIconRow) {\n                cell.setTextAndIcon(\"Иконка приложения\", appIconIcon(), true);\n                cell.setSubtitle(\"Оригинальная, Аметист, Мозаика и другие\");\n            } else if (position == aiChatRow) {\n                cell.setTextAndIcon(getString(R.string.OEChatsAiChat), aiChatIcon(), true);",
        "chats icon bind")?;
    chats = patch_once(&chats,
        "if (position == inlineMathSettingsRow || position == aiChatRow || position == chatSettingsRow || position == menusRow || position == mediaRow) {",
        "if (position == inlineMathSettingsRow || position == appIconRow || position == aiChatRow || position == chatSettingsRow || position == menusRow || position == mediaRow) {",
        "chats icon view type")?;
    if !chats.contains("private static int appIconIcon()") {
        let end = chats.rfind('}').ok_or("chats final brace")?;
        chats.insert_str(end, r#"
    private int appIconIcon() {
        return app.grapegram.GrapeIconResolver.getSelectedIcon(getContext());
    }
"#);
    }
    write_file(&chats_path, &chats)?;

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

    let chat_avatar_path = proj.join("src/main/java/org/telegram/ui/Components/ChatAvatarContainer.java");
    let mut chat_avatar = read_file(&chat_avatar_path)?;
    chat_avatar = patch_once(
        &chat_avatar,
        "import org.telegram.messenger.ChatObject;",
        "import com.exteragram.messenger.api.dto.BadgeDTO;\nimport com.exteragram.messenger.badges.BadgesController;\nimport org.telegram.messenger.ChatObject;",
        "grapeGram chat badge imports",
    )?;
    chat_avatar = patch_once(
        &chat_avatar,
        "    private final AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable botVerificationDrawable;",
        "    private final AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable botVerificationDrawable;\n    private AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable grapeBadgeDrawable;",
        "grapeGram chat badge drawable field",
    )?;
    chat_avatar = patch_once(
        &chat_avatar,
        "        checkActionBar(animated);\n    }\n\n    private Drawable emojiStatusDefaultDrawable;",
        r#"        // grapeGram: show the remotely signed badge in the chat title.
        // Keep Telegram's scam/verified and premium/emoji-status indicators intact.
        BadgeDTO grapeBadge = null;
        TLRPC.User grapeBadgeUser = parentFragment != null ? parentFragment.getCurrentUser() : null;
        if (parentFragment != null
                && parentFragment.getChatMode() == ChatActivity.MODE_DEFAULT
                && !parentFragment.isTopic
                && !scam && !fake && !verified
                && !premium && DialogObject.getEmojiStatusDocumentId(emojiStatus) == 0) {
            if (grapeBadgeUser != null) {
                grapeBadge = BadgesController.INSTANCE.getBadge(grapeBadgeUser);
            }
        }
        if (grapeBadge != null) {
            if (grapeBadgeDrawable == null) {
                grapeBadgeDrawable = new AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable(
                        titleTextView, dp(18), AnimatedEmojiDrawable.CACHE_TYPE_KEYBOARD);
                if (isAttachedToWindow()) {
                    grapeBadgeDrawable.attach();
                }
            }
            grapeBadgeDrawable.set(grapeBadge.getDocumentId(), animated);
            grapeBadgeDrawable.setParticles(true, animated);
            titleTextView.setRightDrawable(grapeBadgeDrawable);
            rightDrawableContentDescription = grapeBadge.getText();
            final BadgeDTO badgeForClick = grapeBadge;
            final TLRPC.User userForClick = grapeBadgeUser;
            titleTextView.setRightDrawableOnClick(v -> {
                if (userForClick != null) {
                    BadgesController.INSTANCE.showBadgeBulletin(
                            parentFragment, badgeForClick, userForClick, resourcesProvider, currentAccount);
                }
            });
        } else if (grapeBadgeDrawable != null) {
            grapeBadgeDrawable.set((Drawable) null, animated);
            if (titleTextView.getRightDrawable() == grapeBadgeDrawable) {
                titleTextView.setRightDrawable(null);
                rightDrawableContentDescription = null;
            }
        }
        if (grapeBadge == null) {
            titleTextView.setRightDrawableOnClick(null);
        }
        checkActionBar(animated);
    }

    private Drawable emojiStatusDefaultDrawable;"#,
        "grapeGram chat title badge",
    )?;
    chat_avatar = patch_once(
        &chat_avatar,
        "        if (emojiStatusDrawable != null) {\n            emojiStatusDrawable.attach();\n        }",
        "        if (emojiStatusDrawable != null) {\n            emojiStatusDrawable.attach();\n        }\n        if (grapeBadgeDrawable != null) {\n            grapeBadgeDrawable.attach();\n        }",
        "grapeGram chat badge attach",
    )?;
    chat_avatar = patch_once(
        &chat_avatar,
        "        if (emojiStatusDrawable != null) {\n            emojiStatusDrawable.detach();\n        }",
        "        if (emojiStatusDrawable != null) {\n            emojiStatusDrawable.detach();\n        }\n        if (grapeBadgeDrawable != null) {\n            grapeBadgeDrawable.detach();\n        }",
        "grapeGram chat badge detach",
    )?;
    write_file(&chat_avatar_path, &chat_avatar)?;

    let chat_message_path = proj.join("src/main/java/org/telegram/ui/Cells/ChatMessageCell.java");
    let mut chat_message = read_file(&chat_message_path)?;
    chat_message = patch_once(
        &chat_message,
        "import org.telegram.messenger.AndroidUtilities;",
        "import com.exteragram.messenger.api.dto.BadgeDTO;\nimport com.exteragram.messenger.badges.BadgesController;\nimport org.telegram.messenger.AndroidUtilities;",
        "grapeGram message badge imports",
    )?;
    chat_message = patch_once(
        &chat_message,
        "    public AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable currentNameEmojiStatusDrawable;",
        "    public AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable currentNameEmojiStatusDrawable;\n    private BadgeDTO currentNameBadge;\n    private AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable currentNameBadgeDrawable;",
        "grapeGram message badge fields",
    )?;
    chat_message = patch_once(
        &chat_message,
        "            currentUser = null;\n            currentChat = null;",
        "            currentUser = null;\n            currentChat = null;\n            currentNameBadge = null;\n            if (currentNameBadgeDrawable != null) {\n                currentNameBadgeDrawable.set((Drawable) null, false);\n            }",
        "grapeGram message badge reset",
    )?;
    chat_message = patch_once(
        &chat_message,
        "            currentNameStatus = null;\n            nameStatusSlug = null;\n            currentNameBotVerificationId = 0;",
        "            currentNameStatus = null;\n            nameStatusSlug = null;\n            currentNameBotVerificationId = 0;\n            currentNameBadge = currentUser != null && !viaBot && !viaGuestBot\n                    ? BadgesController.INSTANCE.getBadge(currentUser) : null;",
        "grapeGram message badge lookup",
    )?;
    chat_message = patch_once(
        &chat_message,
        "            if (currentNameBotVerificationId != 0) {\n                nameWidth -= dp(4 + 12 + 4);\n            }",
        "            if (currentNameBotVerificationId != 0) {\n                nameWidth -= dp(4 + 12 + 4);\n            }\n            if (currentNameBadge != null) {\n                nameWidth -= dp(4 + 18 + 4);\n            }",
        "grapeGram message badge width reservation",
    )?;
    chat_message = patch_once(
        &chat_message,
        "                    nameWidth = nameLayoutWidth = (int) Math.ceil(nameLayout.getLineWidth(0));",
        "                    nameWidth = nameLayoutWidth = (int) Math.ceil(nameLayout.getLineWidth(0));\n                    if (currentNameBadge != null) {\n                        nameWidth += dp(4 + 18 + 4);\n                    }",
        "grapeGram message badge measured width",
    )?;
    chat_message = patch_once(
        &chat_message,
        "            if (currentNameStatusDrawable != null) {\n                if (currentNameStatus == null) {\n                    currentNameStatusDrawable.set((Drawable) null, false);\n                } else if (currentNameStatus instanceof Long) {\n                    currentNameStatusDrawable.set((long) currentNameStatus, false);\n                } else if (currentNameStatus instanceof Drawable) {\n                    currentNameStatusDrawable.set((Drawable) currentNameStatus, false);\n                }\n            }",
        "            if (currentNameStatusDrawable != null) {\n                if (currentNameStatus == null) {\n                    currentNameStatusDrawable.set((Drawable) null, false);\n                } else if (currentNameStatus instanceof Long) {\n                    currentNameStatusDrawable.set((long) currentNameStatus, false);\n                } else if (currentNameStatus instanceof Drawable) {\n                    currentNameStatusDrawable.set((Drawable) currentNameStatus, false);\n                }\n            }\n            if (currentNameBadge != null) {\n                if (currentNameBadgeDrawable == null) {\n                    currentNameBadgeDrawable = new AnimatedEmojiDrawable.SwapAnimatedEmojiDrawable(this, true, dp(18));\n                    if (attachedToWindow) {\n                        currentNameBadgeDrawable.attach();\n                    }\n                }\n                currentNameBadgeDrawable.set(currentNameBadge.getDocumentId(), false);\n                currentNameBadgeDrawable.setParticles(true, false);\n            } else if (currentNameBadgeDrawable != null) {\n                currentNameBadgeDrawable.set((Drawable) null, false);\n            }",
        "grapeGram message badge drawable",
    )?;
    chat_message = patch_once(
        &chat_message,
        "        if (currentNameStatusDrawable != null) {\n            currentNameStatusDrawable.detach();\n        }",
        "        if (currentNameStatusDrawable != null) {\n            currentNameStatusDrawable.detach();\n        }\n        if (currentNameBadgeDrawable != null) {\n            currentNameBadgeDrawable.detach();\n        }",
        "grapeGram message badge detach",
    )?;
    chat_message = patch_once(
        &chat_message,
        "        if (currentNameStatusDrawable != null) {\n            currentNameStatusDrawable.attach();\n        }",
        "        if (currentNameStatusDrawable != null) {\n            currentNameStatusDrawable.attach();\n        }\n        if (currentNameBadgeDrawable != null) {\n            currentNameBadgeDrawable.attach();\n        }",
        "grapeGram message badge attach",
    )?;
    chat_message = patch_once(
        &chat_message,
        "if (super.verifyDrawable(who) || who == selectorDrawable[0]",
        "if (super.verifyDrawable(who) || who == currentNameBadgeDrawable || who == selectorDrawable[0]",
        "grapeGram message badge drawable verification",
    )?;
    chat_message = patch_once(
        &chat_message,
        "if ((currentNameStatusDrawable != null || currentNameEmojiStatusDrawable != null || topicButton != null",
        "if ((currentNameStatusDrawable != null || currentNameEmojiStatusDrawable != null || currentNameBadgeDrawable != null && currentNameBadge != null || topicButton != null",
        "grapeGram message badge draw condition",
    )?;
    chat_message = patch_once(
        &chat_message,
        "                currentNameStatusDrawable.draw(canvas);\n            }\n",
        "                currentNameStatusDrawable.draw(canvas);\n            }\n            if (currentNameBadgeDrawable != null && currentNameBadge != null) {\n                float badgeOffset = currentNameStatus != null ? dp(24) : dp(2);\n                float badgeX = Math.abs(nx) + (viaNameWidth > 0 ? viaNameWidth - dp(4 + 28) : nameLayoutWidth) + badgeOffset;\n                currentNameBadgeDrawable.setBounds(\n                        (int) badgeX,\n                        (int) (ny + nameLayout.getHeight() / 2 - dp(9)),\n                        (int) (badgeX + dp(18)),\n                        (int) (ny + nameLayout.getHeight() / 2 + dp(9)));\n                currentNameBadgeDrawable.setAlpha(255);\n                currentNameBadgeDrawable.draw(canvas);\n            }\n",
        "grapeGram message badge drawing",
    )?;
    chat_message = patch_once(
        &chat_message,
        "                    if (currentNameStatus != null && viaNameWidth <= 0) {\n                        nameEnd += dp(4 + 12 + 4);\n                    }",
        "                    if (currentNameStatus != null && viaNameWidth <= 0) {\n                        nameEnd += dp(4 + 12 + 4);\n                    }\n                    if (currentNameBadge != null) {\n                        nameEnd += dp(4 + 18 + 4);\n                    }",
        "grapeGram message badge compact admin spacing",
    )?;
    write_file(&chat_message_path, &chat_message)?;

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
            "verify aliases",
        )?;
    }
    expect_count(&manifest, "android:enabled=\"true\"\n            android:name=\"org.telegram.messenger.ExteralessIcon\"", 0, "verify old alias disabled")?;
    let app_tag = application_tag_text(&manifest, "verify application tag")?;
    expect_count(app_tag, "android:label=\"@string/AppName\"", 1, "verify label")?;
    expect_count(app_tag, "android:icon=\"@drawable/grape_icon_original\"", 1, "verify icon")?;
    expect_count(app_tag, "android:roundIcon=\"@drawable/grape_icon_original\"", 1, "verify roundIcon")?;
    let settings = read_file(&settings_path)?;
    expect_count(&settings, "case 7001:", 1, "verify settings case")?;
    for (key, _, _, _, _, _) in CHERRY_ICONS.iter() {
        expect_count(&manifest, &format!("android:name=\"org.telegram.messenger.GrapeCherryIcon{}\"", camel(key)), 1, "verify Cherrygram icon alias")?;
    }
    let launcher = read_file(&launcher_path)?;
    expect_count(&launcher, "GRAPE_ORIGINAL", 3, "verify launcher enum")?;
    let app_icons = read_file(&app_icons_path)?;
    expect_count(&app_icons, "(icon.name().startsWith(\"GRAPE_\") || icon.name().startsWith(\"CHERRY_\"))", 1, "verify grapeGram and Cherrygram icon picker")?;
    let typeface = read_file(&typeface_path)?;
    expect_count(&typeface, "customTitle = \"grapeGram\";", 1, "verify legacy custom chat title migration")?;
    let notifications = read_file(&notifications_path)?;
    expect_count(&notifications, "return R.drawable.grape_icon_original;", 1, "verify notification icon uses grapeGram")?;
    if notifications.contains("R.drawable.exteraless_notification") {
        return Err("verify: legacy notification icon remains".to_string());
    }
    let notification_service = read_file(&notification_service_path)?;
    expect_count(&notification_service, "setSmallIcon(R.drawable.grape_icon_original)", 1, "verify foreground notification icon")?;
    let session_cell = read_file(&session_cell_path)?;
    if session_cell.contains("R.drawable.exteraless_notification") {
        return Err("verify: legacy session icon remains".to_string());
    }
    let icon_resolver_path = proj.join("src/main/java/app/grapegram/GrapeIconResolver.java");
    let icon_resolver = read_file(&icon_resolver_path)?;
    expect_count(&icon_resolver, "getSelectedIcon", 1, "verify dynamic grapeGram icon resolver")?;
    expect_count(&icon_resolver, "GrapeCherryIcon", 18, "verify Cherrygram icon resolver aliases")?;
    let build_gradle = read_file(&build_path)?;
    expect_count(&build_gradle, "implementation 'org.bouncycastle:bcprov-jdk18on:1.83'", 1, "verify Ed25519 verifier dependency")?;
    expect_count(&build_gradle, "module: 'bcprov-jdk15to18'", 1, "verify legacy Bouncy Castle exclusion")?;
    expect_count(&build_gradle, "substitute(module('org.bouncycastle:bcprov-jdk15to18')).using(module('org.bouncycastle:bcprov-jdk18on:1.83'))", 1, "verify legacy Bouncy Castle module substitution")?;
    expect_count(&build_gradle, "exclude group: 'org.bouncycastle', module: 'bcprov-jdk15to18'", 1, "verify legacy Bouncy Castle exclusion")?;
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
        "applied: {} icon aliases, settings entry, enum, strings, package {}, data_base {}",
        ICONS.len() + EXT_ICONS.len(),
        PACKAGE, data_base
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
    fn patch_once_counts_anchors() {
        assert_eq!(patch_once("a X b", "X", "Y", "t").unwrap(), "a Y b");
        assert!(patch_once("a X X b", "X", "Y", "t").is_err());
        assert!(patch_once("a b", "X", "Y", "t").is_err());
    }

    #[test]
    fn rebrand_xml_strings_handles_legacy_casing_without_touching_resource_names() {
        let src = "<resources><string name=\"a\">ExteraLess</string><string name=\"b\">exteraless</string><string name=\"c\">Keep</string></resources>";
        let out = rebrand_xml_strings(src);
        assert!(out.contains(">grapeGram</string>"));
        assert!(out.contains("name=\"c\">Keep</string>"));
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
    fn templates_match_expected_names() {
        let enums = enum_entries();
        assert!(enums.contains("GRAPE_ORIGINAL(\"GrapeIconOriginal\""));
        assert!(enums.contains("GRAPE_SILK(\"GrapeIconSilk\""));
        assert_eq!(enums.matches("GRAPE_").count(), 9);
        assert_eq!(enums.matches("EXT_").count(), 17);
        assert_eq!(enums.lines().filter(|line| line.trim_start().starts_with("CHERRY_")).count(), 18);
        let alias = alias_template("original", "@drawable/grape_icon_original", true);
        assert!(alias.contains("android:enabled=\"true\""));
        assert!(alias.contains("org.telegram.messenger.GrapeIconOriginal"));
        assert!(alias.contains("@drawable/grape_icon_original"));
        let cherry_alias = cherry_alias_template("dark_cherry", "@mipmap/grape_cherry_icon_dark_cherry");
        assert!(cherry_alias.contains("GrapeCherryIconDark_cherry"));
        assert!(cherry_alias.contains("@mipmap/grape_cherry_icon_dark_cherry"));
    }
}
