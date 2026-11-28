import base64
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
import zlib
import struct

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, ROOT)
sys.path.insert(0, os.path.join(ROOT, "scripts"))
sys.path.insert(0, os.path.join(ROOT, "backend"))

from canonical import canonical_bytes  # noqa: E402
import server  # noqa: E402

OWNERS = ["7293365262", "960792706"]
ID_RE = re.compile(r"^[0-9]{5,20}$")


def load(name):
    with open(os.path.join(ROOT, "data", name), "rb") as f:
        return json.load(f)


def make_key(tmp):
    priv = os.path.join(tmp, "k.pem")
    pub = os.path.join(tmp, "k.pub.pem")
    subprocess.run(["openssl", "genpkey", "-algorithm", "ED25519", "-out", priv],
                   check=True, capture_output=True)
    subprocess.run(["openssl", "pkey", "-in", priv, "-pubout", "-out", pub],
                   check=True, capture_output=True)
    return priv, pub


def sign(key, payload):
    with tempfile.NamedTemporaryFile(delete=False) as f:
        f.write(payload)
        p = f.name
    try:
        return subprocess.run(["openssl", "pkeyutl", "-sign", "-inkey", key,
                               "-rawin", "-in", p],
                              check=True, capture_output=True).stdout
    finally:
        os.unlink(p)


def verify(pub, payload, sig):
    a = b = None
    try:
        with tempfile.NamedTemporaryFile(delete=False) as f:
            f.write(payload)
            a = f.name
        with tempfile.NamedTemporaryFile(delete=False) as f:
            f.write(sig)
            b = f.name
        r = subprocess.run(["openssl", "pkeyutl", "-verify", "-pubin",
                            "-inkey", pub, "-rawin", "-in", a, "-sigfile", b],
                           capture_output=True)
        return r.returncode == 0
    finally:
        for p in (a, b):
            if p:
                os.unlink(p)


class DataTests(unittest.TestCase):
    def test_owners_exactly_two(self):
        d = load("owner_users.json")
        self.assertEqual(d["version"], 1)
        self.assertEqual(d["owners"], OWNERS)
        for o in d["owners"]:
            self.assertIsInstance(o, str)
            self.assertRegex(o, ID_RE)

    def test_badge_document(self):
        d = load("badge_users.json")
        self.assertEqual(d["version"], 1)
        self.assertIn("updated_at", d)
        allowed = {"grapegram", "founder", "developer", "contributor"}
        for uid, rec in d["badges"].items():
            self.assertRegex(uid, ID_RE)
            self.assertIn(rec["badge"], allowed)
            self.assertIsInstance(rec["enabled"], bool)
            self.assertGreaterEqual(rec["record_version"], 1)

    def test_badge_schema_allows_version_increments(self):
        with open(os.path.join(ROOT, "data", "schemas", "badge_users.schema.json"),
                  encoding="utf-8") as f:
            schema = json.load(f)
        version = schema["properties"]["version"]
        self.assertEqual(version.get("minimum"), 1)
        self.assertNotIn("const", version)

    def test_canonical_is_order_independent(self):
        a = {"b": 1, "a": {"y": 2, "x": [1, 2]}}
        b = {"a": {"x": [1, 2], "y": 2}, "b": 1}
        self.assertEqual(canonical_bytes(a), canonical_bytes(b))
        self.assertNotIn(b" ", canonical_bytes(a))

    def test_canonical_rejects_non_integer_numbers(self):
        with self.assertRaises(ValueError):
            canonical_bytes({"amount": 1.5})
        with self.assertRaises(ValueError):
            canonical_bytes({"amount": 1 << 64})
        self.assertEqual(canonical_bytes({"enabled": True}), b'{"enabled":true}')


class CryptoTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.priv, self.pub = make_key(self.tmp)
        with open(os.path.join(ROOT, "data", "badge_users.json"), "rb") as f:
            self.payload = f.read()
        self.doc = json.loads(self.payload)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def test_roundtrip(self):
        sig = sign(self.priv, self.payload)
        self.assertEqual(len(sig), 64)
        self.assertTrue(verify(self.pub, self.payload, sig))

    def test_signing_scripts_use_canonical_json(self):
        document = {"z": 2, "a": {"b": True, "a": "grapeGram"}}
        json_path = os.path.join(self.tmp, "sample.json")
        with open(json_path, "w", encoding="utf-8") as f:
            json.dump(document, f, indent=2)
        out_dir = os.path.join(self.tmp, "signatures")
        subprocess.run([
            sys.executable, os.path.join(ROOT, "scripts", "sign_data.py"),
            "--key", self.priv, "--out", out_dir, json_path
        ], check=True, capture_output=True)
        signature_path = os.path.join(out_dir, "sample.json.sig")
        with open(signature_path, "rb") as f:
            signature = base64.b64decode(f.read().strip())
        self.assertTrue(verify(self.pub, canonical_bytes(document), signature))
        subprocess.run([
            sys.executable, os.path.join(ROOT, "scripts", "verify_data.py"),
            "--pub", self.pub, json_path, signature_path
        ], check=True, capture_output=True)

    def test_tampered_document_fails(self):
        sig = sign(self.priv, self.payload)
        doc = json.loads(self.payload)
        doc["badges"]["123456789"] = {"badge": "grapegram", "enabled": True,
                                      "record_version": 1}
        tampered = json.dumps(doc, indent=2, sort_keys=True).encode()
        self.assertFalse(verify(self.pub, tampered, sig))

    def test_other_key_fails(self):
        sig = sign(self.priv, self.payload)
        other = os.path.join(self.tmp, "other.pem")
        subprocess.run(["openssl", "genpkey", "-algorithm", "ED25519",
                        "-out", other], check=True, capture_output=True)
        subprocess.run(["openssl", "pkey", "-in", other, "-pubout",
                        "-out", other + ".pub"], check=True, capture_output=True)
        self.assertFalse(verify(other + ".pub", self.payload, sig))

    def test_published_files_match_published_key(self):
        pub = os.path.join(ROOT, "data", "keys", "grapegram_ed25519.pub.pem")
        for name in ("badge_users.json", "owner_users.json"):
            with open(os.path.join(ROOT, "data", name), "rb") as f:
                payload = f.read()
            with open(os.path.join(ROOT, "data", "signatures",
                                   name + ".sig"), "rb") as f:
                sig = base64.b64decode(f.read().strip())
            canonical = canonical_bytes(json.loads(payload))
            self.assertTrue(verify(pub, canonical, sig) or verify(pub, payload, sig), name)

    def test_no_private_key_in_repo(self):
        marker = b"BEGIN " + b"PRIVATE" + b" KEY"
        for dirpath, dirnames, filenames in os.walk(ROOT):
            dirnames[:] = [d for d in dirnames if d not in (".git", "outbox", "rust", "target")]
            for fn in filenames:
                p = os.path.join(dirpath, fn)
                try:
                    with open(p, "rb") as f:
                        blob = f.read(8192)
                except OSError:
                    continue
                self.assertNotIn(marker, blob,
                                 f"private key material in {p}")


class StoreTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.data = os.path.join(self.tmp, "data")
        os.makedirs(self.data)
        for n in ("badge_users.json", "owner_users.json"):
            shutil.copy(os.path.join(ROOT, "data", n), self.data)
        self.priv, self.pub = make_key(self.tmp)
        os.environ["GRP_SIGNING_KEY_FILE"] = self.priv
        self.old_publisher = server.publish_badge_files
        server.publish_badge_files = lambda data, signature: None
        self.store = server.Store(self.data)

    def tearDown(self):
        server.publish_badge_files = self.old_publisher
        os.environ.pop("GRP_SIGNING_KEY_FILE", None)
        shutil.rmtree(self.tmp, ignore_errors=True)

    def test_owner_allowlist_requires_two_unique_numeric_ids(self):
        path = os.path.join(self.data, "owner_users.json")
        invalid_lists = [
            ["7293365262"],
            ["7293365262", "7293365262"],
            ["not-an-id", "960792706"],
            [None, "960792706"],
        ]
        for owners in invalid_lists:
            with self.subTest(owners=owners):
                with open(path, "w", encoding="utf-8") as f:
                    json.dump({"version": 1, "owners": owners}, f)
                with self.assertRaises(RuntimeError):
                    self.store.owners()
        shutil.copy(os.path.join(ROOT, "data", "owner_users.json"), path)

    def test_challenges_and_sessions_survive_restart(self):
        code = self.store.new_challenge()
        restored = server.Store(self.data)
        self.assertIn(code, restored.challenges)
        token = restored.bind(code, OWNERS[0])
        restored_again = server.Store(self.data)
        self.assertEqual(restored_again.require_owner(token), OWNERS[0])

    def test_challenge_single_use(self):
        code = self.store.new_challenge()
        token = self.store.bind(code, OWNERS[0])
        self.assertTrue(token)
        with self.assertRaises(PermissionError):
            self.store.bind(code, OWNERS[0])

    def test_expired_challenge(self):
        code = self.store.new_challenge()
        self.store.challenges[code]["expires"] = time.time() - 1
        with self.assertRaises(PermissionError):
            self.store.bind(code, OWNERS[0])

    def test_non_owner_cannot_bind(self):
        code = self.store.new_challenge()
        with self.assertRaises(PermissionError):
            self.store.bind(code, "111111111")

    def test_non_owner_cannot_grant(self):
        code = self.store.new_challenge()
        with self.assertRaises(PermissionError):
            self.store.bind(code, "111111111")
        with self.assertRaises(PermissionError):
            self.store.mutate("forged-token", "555555555", "grapegram", True)

    def test_grant_signs_and_increments(self):
        code = self.store.new_challenge()
        token = self.store.bind(code, OWNERS[0])
        before = load("badge_users.json")["version"]
        doc = self.store.mutate(token, "555555555", "grapegram", True, "test")
        self.assertEqual(doc["version"], before + 1)
        self.assertTrue(doc["badges"]["555555555"]["enabled"])
        with open(os.path.join(self.data, "signatures",
                               "badge_users.json.sig"), "rb") as f:
            sig = base64.b64decode(f.read().strip())
        with open(os.path.join(self.data, "badge_users.json"), "rb") as f:
            self.assertTrue(verify(self.pub, f.read(), sig))

    def test_revoke(self):
        code = self.store.new_challenge()
        token = self.store.bind(code, OWNERS[1])
        doc = self.store.mutate(token, OWNERS[0], "grapegram", False, "revoked")
        self.assertFalse(doc["badges"][OWNERS[0]]["enabled"])

    def test_bad_badge_rejected(self):
        code = self.store.new_challenge()
        token = self.store.bind(code, OWNERS[0])
        with self.assertRaises(ValueError):
            self.store.mutate(token, "555555555", "admin", True)

    def test_bad_id_rejected(self):
        code = self.store.new_challenge()
        token = self.store.bind(code, OWNERS[0])
        with self.assertRaises(ValueError):
            self.store.mutate(token, "not-an-id", "grapegram", True)

    def test_audit_written(self):
        code = self.store.new_challenge()
        token = self.store.bind(code, OWNERS[0])
        self.store.mutate(token, "555555555", "founder", True, "why")
        with open(os.path.join(self.data, "audit.jsonl"), encoding="utf-8") as f:
            rec = json.loads(f.readline())
        self.assertEqual(rec["by"], OWNERS[0])
        self.assertEqual(rec["target"], "555555555")
        self.assertEqual(rec["action"], "grant")


class IconTests(unittest.TestCase):
    def _decode(self, path):
        with open(path, "rb") as f:
            d = f.read()
        i, idat, w, h = 8, b"", 0, 0
        while i < len(d):
            ln = struct.unpack(">I", d[i:i + 4])[0]
            typ = d[i + 4:i + 8]
            data = d[i + 8:i + 8 + ln]
            i += 12 + ln
            if typ == b"IHDR":
                w, h, bd, ct = struct.unpack(">IIBB", data[:10])
            elif typ == b"IDAT":
                idat += data
        raw = zlib.decompress(idat)
        bpp, stride = 4, w * 4
        out, prev, p = bytearray(), bytearray(stride), 0
        for _ in range(h):
            ft = raw[p]
            p += 1
            line = bytearray(raw[p:p + stride])
            p += stride
            for x in range(stride):
                a = line[x - bpp] if x >= bpp else 0
                b = prev[x]
                c = prev[x - bpp] if x >= bpp else 0
                if ft == 1:
                    line[x] = (line[x] + a) & 255
                elif ft == 2:
                    line[x] = (line[x] + b) & 255
                elif ft == 3:
                    line[x] = (line[x] + (a + b) // 2) & 255
                elif ft == 4:
                    pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                    pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                    line[x] = (line[x] + pr) & 255
            out += line
            prev = line
        return w, h, out

    def test_nine_icons_named(self):
        with open(os.path.join(ROOT, "assets", "icons", "icons.json")) as f:
            meta = json.load(f)
        self.assertEqual(len(meta["icons"]), 9)
        names = [i["name"] for i in meta["icons"]]
        self.assertEqual(len(set(names)), 9)
        for icon in meta["icons"]:
            path = os.path.join(ROOT, "assets", "icons", icon["file"])
            self.assertTrue(os.path.isfile(path), icon["file"])
            w, h, px = self._decode(path)
            self.assertEqual((w, h), (512, 512))
            for x, y in ((0, 0), (511, 0), (0, 511), (511, 511)):
                self.assertEqual(px[(y * w + x) * 4 + 3], 0,
                                 f"{icon['file']} corner is not transparent")
            self.assertEqual(px[((256 * w) + 256) * 4 + 3], 255)


class SettingsRegressionTests(unittest.TestCase):
    def test_grapegram_hub_owns_custom_settings_and_icon_picker_is_nested(self):
        path = os.path.join(ROOT, "client", "overlay", "java", "app",
                            "grapegram", "GrapeSettingsActivity.java")
        with open(path, encoding="utf-8") as source_file:
            source = source_file.read()
        self.assertIn('title.setText("grapeGram")', source)
        self.assertIn('"Менеджер плагинов"', source)
        self.assertIn('"Бейджи grapeGram"', source)
        self.assertIn("new LiteModeSettingsActivity()", source)
        self.assertIn("new OpenExteraGeneralActivity()", source)
        self.assertNotIn("ID_ICONS", source)
        self.assertNotIn("Разрешения плагинов", source)
        self.assertNotIn("Журнал активности", source)

    def test_icon_picker_is_only_in_chat_settings(self):
        path = os.path.join(ROOT, "rust", "materializer", "src", "main.rs")
        with open(path, encoding="utf-8") as source_file:
            source = source_file.read()
        self.assertIn("appIconRow = addRow();", source)
        self.assertIn("new app.exteraless.appicons.AppIconsActivity()", source)
        self.assertIn("position == appIconRow", source)
        self.assertIn(r'icon.name().startsWith(\"GRAPE_\")', source)
        self.assertIn(r'icon.name().startsWith(\"CHERRY_\")', source)
        self.assertIn("org.bouncycastle:bcprov-jdk18on:1.83", source)

    def test_settings_deep_links_route_to_grapegram_hub(self):
        materializer_path = os.path.join(ROOT, "rust", "materializer", "src", "main.rs")
        with open(materializer_path, encoding="utf-8") as source_file:
            materializer = source_file.read()
        self.assertIn("patch_exact_count(", materializer)
        self.assertIn('"fragment = neko_fragment = new OpenExteraSettingsActivity();"', materializer)
        self.assertIn('"fragment = new app.grapegram.GrapeSettingsActivity();"', materializer)
        self.assertIn('"legacy settings deep links"', materializer)
        settings_path = os.path.join(ROOT, "client", "overlay", "java", "app",
                                     "grapegram", "GrapeSettingsActivity.java")
        with open(settings_path, encoding="utf-8") as source_file:
            settings = source_file.read()
        self.assertIn("public class GrapeSettingsActivity extends BaseFragment", settings)

    def test_profile_badge_particles_are_patched_once(self):
        path = os.path.join(ROOT, "rust", "materializer", "src", "main.rs")
        with open(path, encoding="utf-8") as source_file:
            source = source_file.read()
        self.assertEqual(source.count('"badge particles"'), 1)
        self.assertEqual(source.count('"grapeGram profile badge particles"'), 0)
        self.assertEqual(source.count('badgeDrawable.setParticles(true, animated);'), 1)

    def test_notification_icon_patch_replaces_method_without_format_fragility(self):
        path = os.path.join(ROOT, "rust", "materializer", "src", "main.rs")
        with open(path, encoding="utf-8") as source_file:
            source = source_file.read()
        self.assertIn("fn replace_java_method(", source)
        self.assertIn('"    private int getNotificationIconResId()"', source)
        self.assertIn("return R.drawable.grape_icon_original;", source)
        self.assertNotIn("let old_notification_icon = r#", source)

    def test_android_signature_verifier_has_raw_and_canonical_compatibility(self):
        path = os.path.join(ROOT, "client", "overlay", "java", "app",
                            "grapegram", "GrapeBadges.java")
        with open(path, encoding="utf-8") as source_file:
            source = source_file.read()
        self.assertIn("org.bouncycastle.crypto.signers.Ed25519Signer", source)
        self.assertIn("verifyBytes(publicKey, canonicalJson(data), sig)", source)
        self.assertIn("verifyBytes(publicKey, data, sig)", source)
        self.assertIn("for (int attempt = 0; attempt < 3; attempt++)", source)
        self.assertIn("c.setUseCaches(false)", source)
        self.assertIn('CACHE_SIGNATURE = "badge_cache.json.sig"', source)


class BadgeDisplayRegressionTests(unittest.TestCase):
    def test_signed_badges_are_wired_to_chat_header_and_profile_particles(self):
        path = os.path.join(ROOT, "rust", "materializer", "src", "main.rs")
        with open(path, encoding="utf-8") as source_file:
            source = source_file.read()
        self.assertIn("ChatAvatarContainer.java", source)
        self.assertIn("ChatMessageCell.java", source)
        self.assertIn("BadgesController.INSTANCE.getBadge(grapeBadgeUser)", source)
        self.assertIn("currentNameBadge = currentUser != null && !viaBot && !viaGuestBot", source)
        self.assertIn("currentNameBadgeDrawable.draw(canvas)", source)
        self.assertIn("currentNameBadgeDrawable.attach()", source)
        self.assertIn("currentNameBadgeDrawable.detach()", source)
        self.assertIn("currentNameBadgeDrawable.set((Drawable) null, false)", source)
        self.assertNotIn("grapeBadgeChat", source)
        self.assertIn("grapeBadgeDrawable.setParticles(true, animated)", source)
        self.assertIn("badgeDrawable.setParticles(true, animated)", source)


class BadgeBridgeRegressionTests(unittest.TestCase):
    def test_sync_removes_previously_synced_revoked_badges(self):
        path = os.path.join(ROOT, "client", "overlay", "java", "app",
                            "grapegram", "GrapeBadgeBridge.java")
        with open(path, encoding="utf-8") as source_file:
            source = source_file.read()
        self.assertIn('getStringSet(KEY_SYNCED_IDS', source)
        self.assertIn('getApiBadgeSource().put(id, null)', source)
        self.assertIn('prefs.edit().putStringSet(KEY_SYNCED_IDS, nextIds).apply()', source)

    def test_sync_does_not_apply_disabled_or_unknown_badges(self):
        path = os.path.join(ROOT, "client", "overlay", "java", "app",
                            "grapegram", "GrapeBadgeBridge.java")
        with open(path, encoding="utf-8") as source_file:
            source = source_file.read()
        self.assertIn('if (!entry.enabled || !isKnownBadge(entry.badge))', source)
        self.assertIn('"contributor".equals(badge)', source)

    def test_foreground_refresh_is_throttled(self):
        path = os.path.join(ROOT, "client", "overlay", "java", "app",
                            "grapegram", "GrapeBadgeBridge.java")
        with open(path, encoding="utf-8") as source_file:
            source = source_file.read()
        self.assertIn("refreshIfDue(Context context)", source)
        self.assertIn("REFRESH_INTERVAL_MS = 60L * 60L * 1000L", source)
        self.assertIn("KEY_LAST_REFRESH", source)

    def test_startup_refreshes_network_but_manual_sync_reuses_cache(self):
        bridge_path = os.path.join(ROOT, "client", "overlay", "java", "app",
                                   "grapegram", "GrapeBadgeBridge.java")
        with open(bridge_path, encoding="utf-8") as source_file:
            bridge = source_file.read()
        self.assertIn("GrapeBadges.load(appContext, true)", bridge)
        self.assertIn("syncCachedAsync(Context context)", bridge)
        settings_path = os.path.join(ROOT, "client", "overlay", "java", "app",
                                     "grapegram", "GrapeSettingsActivity.java")
        with open(settings_path, encoding="utf-8") as source_file:
            settings = source_file.read()
        self.assertIn("GrapeBadgeBridge.syncCachedAsync(context.getApplicationContext())", settings)

    def test_cache_is_signature_verified_before_use(self):
        path = os.path.join(ROOT, "client", "overlay", "java", "app",
                            "grapegram", "GrapeBadges.java")
        with open(path, encoding="utf-8") as source_file:
            source = source_file.read()
        self.assertIn('CACHE_SIGNATURE = "badge_cache.json.sig"', source)
        self.assertIn("if (!verify(GrapeConfig.PUBLIC_KEY_PEM, data, signature))", source)
        self.assertIn("parseDocument(data)", source)
        self.assertIn("withCacheBust(", source)
        self.assertIn("for (int attempt = 0; attempt < 3; attempt++)", source)

    def test_developer_badge_uses_developer_status(self):
        path = os.path.join(ROOT, "client", "overlay", "java", "app",
                            "grapegram", "GrapeBadgeBridge.java")
        with open(path, encoding="utf-8") as source_file:
            source = source_file.read()
        self.assertIn('ProfileStatus.DEVELOPER : ProfileStatus.SUPPORTER', source)


if __name__ == "__main__":
    unittest.main()
