#!/usr/bin/env python3
"""Materialise grapeGram from a pinned base fork + this overlay.

The repository stays small: CI (or a developer) points this script at a clean
checkout of the base fork and it applies every integration point in one pass.

Usage:
  python3 client/apply.py --root /path/to/base-checkout
  python3 client/apply.py --root /tmp/build --download   # fetch pinned tarball

Every patch is anchored to exact source text and fails loudly if the base
changed, instead of silently producing a broken build.
"""
import argparse
import json
import os
import re
import shutil
import sys
import xml.etree.ElementTree as ET

HERE = os.path.dirname(os.path.abspath(__file__))
OVERLAY = os.path.join(HERE, "overlay")
MARK = "GrapeIconOriginal"

ICONS = [
    ("original", "Оригинальная"),
    ("amethyst", "Аметист"),
    ("mosaic", "Мозаика"),
    ("onyx", "Оникс"),
    ("silk", "Лиловый шёлк"),
    ("neon", "Неон"),
    ("orbit", "Орбита"),
    ("graphite", "Графит"),
    ("violet", "Фиалка"),
]

ALIAS_TEMPLATE = """        <activity-alias
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

"""

ENUM_TEMPLATE = """        GRAPE_{upper}("GrapeIcon{camel}", R.drawable.grape_icon_{key}, R.drawable.app_icon_empty_foreground, R.string.GrapeIcon{camel})"""


def die(msg):
    sys.exit("apply.py: " + msg)


def read(path):
    with open(path, "r", encoding="utf-8") as f:
        return f.read()


def write(path, text):
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


def patch_once(text, anchor, replacement, what):
    n = text.count(anchor)
    if n != 1:
        die(f"{what}: anchor found {n} times, expected 1")
    return text.replace(anchor, replacement)


def run(cmd, cwd=None, timeout=3600):
    print("+", " ".join(cmd))
    r = subprocess.run(cmd, cwd=cwd, timeout=timeout)
    if r.returncode != 0:
        die("command failed: " + " ".join(cmd))


def download_base(root, cfg):
    """Pinned commit + all submodules (tarballs do not include submodules)."""
    url = f"https://github.com/{cfg['base_repo']}.git"
    commit = cfg["base_commit"]
    os.makedirs(root, exist_ok=True)
    if not os.path.isdir(os.path.join(root, ".git")):
        run(["git", "init", "-q", root])
        run(["git", "-C", root, "remote", "add", "origin", url])
    run(["git", "-C", root, "fetch", "--depth", "1", "origin", commit])
    run(["git", "-C", root, "checkout", "-q", "--force", "FETCH_HEAD"])
    try:
        run(["git", "-C", root, "submodule", "update", "--init", "--recursive",
             "--depth", "1"])
    except SystemExit:
        print("shallow submodules failed, retrying full depth")
        run(["git", "-C", root, "submodule", "update", "--init", "--recursive"])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--config", default=os.path.join(HERE, "config.json"))
    ap.add_argument("--download", action="store_true")
    a = ap.parse_args()
    cfg = json.load(open(a.config))
    cfg["data_base"] = os.environ.get("GRP_DATA_BASE", cfg["data_base"])
    root = os.path.abspath(a.root)
    proj = os.path.join(root, "TMessagesProj")
    if a.download and not os.path.isdir(proj):
        os.makedirs(root, exist_ok=True)
        download_base(root, cfg)
    if not os.path.isdir(proj):
        die(f"{proj} not found; pass a base checkout or --download")

    manifest = os.path.join(proj, "src", "main", "AndroidManifest.xml")
    settings = os.path.join(proj, "src", "main", "java", "org", "telegram",
                            "ui", "SettingsActivity.java")
    launcher = os.path.join(proj, "src", "main", "java", "org", "telegram",
                            "ui", "LauncherIconController.java")
    strings = os.path.join(proj, "src", "main", "res", "values", "strings.xml")

    for p in (manifest, settings, launcher, strings):
        if not os.path.isfile(p):
            die(f"missing {p}")

    if MARK in read(manifest):
        print("already applied; nothing to do")
        return 0

    # 1. overlay sources and resources
    shutil.copytree(os.path.join(OVERLAY, "java"), os.path.join(proj, "src", "main", "java"),
                    dirs_exist_ok=True)
    shutil.copytree(os.path.join(OVERLAY, "res"), os.path.join(proj, "src", "main", "res"),
                    dirs_exist_ok=True)

    # 2. config placeholders
    gconf = os.path.join(proj, "src", "main", "java", "app", "grapegram",
                         "GrapeConfig.java")
    pem_path = os.path.join(HERE, "..", "data", "keys",
                            "grapegram_ed25519.pub.pem")
    pem = read(pem_path).strip() if os.path.isfile(pem_path) else ""
    src = read(gconf)
    src = src.replace("@DATA_BASE@", cfg["data_base"])
    src = src.replace("@PUBLIC_KEY_PEM@", pem.replace("\n", "\\n"))
    write(gconf, src)

    # 3. settings entry (first item) + click handler
    s = read(settings)
    row_anchor = "        items.add(UItem.asCustomShadow(topView, 200 - 12));\n"
    row = (row_anchor
           + "        // grapeGram: единственный дополнительный пункт, первым в списке\n"
           + "        items.add(SettingCell.Factory.of(7001, 0xFF7C4DFF, 0xFF5E35B1, "
             "R.drawable.grape_icon_original, \"Настройки grapeGram\", "
             "\"Cherrygram · exteraGram · exteraless\"));\n"
           + "        items.add(UItem.asShadow(null));\n")
    s = patch_once(s, row_anchor, row, "settings row")
    case_anchor = ("            case -1: {\n"
                   "                presentFragment(new OpenExteraSettingsActivity());\n"
                   "                break;\n"
                   "            }\n")
    case = ("            case 7001: {\n"
            "                presentFragment(new app.grapegram.GrapeSettingsActivity());\n"
            "                break;\n"
            "            }\n") + case_anchor
    s = patch_once(s, case_anchor, case, "settings click")
    old_row = ("        items.add(SettingCell.Factory.of(-1, 0xFF993C38, 0xFF993C38, "
               "R.drawable.exteraless_icon_tile, getString(R.string.OpenExteraPreferences), "
               "getString(R.string.OpenExteraInfo)));\n"
               "        items.add(UItem.asShadow(null));\n")
    s = patch_once(s, old_row,
                   "        // exteraless entry moved into the grapeGram hub\n",
                   "remove old top-level entry")
    write(settings, s)

    # 4. launcher icon enum + defaults
    l = read(launcher)
    nox = ('NOX("NoxIcon", R.mipmap.icon_2_background_sa, '
           'R.mipmap.icon_foreground_sa, R.string.AppIconNox);')
    entries = ",\n".join(
        ENUM_TEMPLATE.format(upper=k.upper(), camel=k.capitalize(), key=k)
        for k, _ in ICONS) + ";"
    l = patch_once(l, nox, nox[:-1] + ",\n" + entries, "icon enum")
    l = patch_once(l, "        setIcon(LauncherIcon.EXTERALESS);",
                   "        setIcon(LauncherIcon.GRAPE_ORIGINAL);", "default icon")
    l = patch_once(l,
                   "|| i == PackageManager.COMPONENT_ENABLED_STATE_DEFAULT && icon == LauncherIcon.EXTERALESS;",
                   "|| i == PackageManager.COMPONENT_ENABLED_STATE_DEFAULT && icon == LauncherIcon.GRAPE_ORIGINAL;",
                   "isEnabled default")
    write(launcher, l)

    # 5. manifest aliases, base alias off
    m = read(manifest)
    m2 = re.sub(r'(android:enabled=")true("\s*\n\s*android:name="org\.telegram\.messenger\.ExteralessIcon")',
                r"\1false\2", m, count=1)
    if m2 == m:
        die("could not disable ExteralessIcon alias")
    aliases = "".join(
        ALIAS_TEMPLATE.format(enabled="true" if k == "original" else "false",
                              camel=k.capitalize(), key=k)
        for k, _ in ICONS)
    m2 = patch_once(m2, "    </application>", aliases + "    </application>", "manifest aliases")
    write(manifest, m2)
    ET.parse(manifest)

    # 6. strings
    st = read(strings)
    add = "".join(f'    <string name="GrapeIcon{k.capitalize()}">{name}</string>\n'
                  for k, name in ICONS)
    st = patch_once(st, "</resources>", add + "</resources>", "strings")
    write(strings, st)

    # 7. verify
    tree = ET.parse(manifest)
    ns = "{http://schemas.android.com/apk/res/android}"
    found = [e.get(ns + "name") for e in tree.iter("activity-alias")
             if (e.get(ns + "name") or "").startswith("org.telegram.messenger.GrapeIcon")]
    if len(found) != 9:
        die(f"expected 9 grape aliases, found {len(found)}")
    if "case 7001:" not in read(settings):
        die("settings click handler missing")
    if "GRAPE_ORIGINAL" not in read(launcher):
        die("launcher enum missing")
    print(f"applied: 9 icon aliases, settings entry, enum, strings; base={cfg['base_commit'][:8]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
