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

    def test_badge_schema_allows_monotonic_versions(self):
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
            self.assertTrue(verify(pub, payload, sig), name)

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
        self.store = server.Store(self.data)

    def tearDown(self):
        os.environ.pop("GRP_SIGNING_KEY_FILE", None)
        shutil.rmtree(self.tmp, ignore_errors=True)

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

    def test_app_can_poll_owner_binding_without_exposing_session(self):
        code, poll_token = self.store.new_challenge_pair()
        self.assertIsNone(self.store.poll_challenge(code, poll_token))
        with self.assertRaises(PermissionError):
            self.store.poll_challenge(code, "wrong-token")
        session = self.store.bind(code, OWNERS[1])
        self.assertEqual(self.store.poll_challenge(code, poll_token), session)
        with self.assertRaises(PermissionError):
            self.store.poll_challenge(code, poll_token)


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

    def test_badge_style_is_validated_persisted_and_audited(self):
        code = self.store.new_challenge()
        token = self.store.bind(code, OWNERS[0])
        style = {
            "color_light": "#AA22CC",
            "color_dark": "#FF112233",
            "alpha": 0.75,
            "particles": True,
            "animation": "sparkle",
        }
        doc = self.store.mutate(token, "555555555", "founder", True,
                                "style test", style=style)
        self.assertEqual(doc["badges"]["555555555"]["style"], {
            "color_light": "#AA22CC",
            "color_dark": "#FF112233",
            "alpha": 0.75,
            "particles": True,
            "animation": "sparkle",
        })
        with open(os.path.join(self.data, "audit.jsonl"), encoding="utf-8") as f:
            rec = json.loads(f.readline())
        self.assertEqual(rec["style"]["animation"], "sparkle")

    def test_invalid_badge_style_rejected(self):
        with self.assertRaises(ValueError):
            server.validate_badge_style({"color_light": "purple"})
        with self.assertRaises(ValueError):
            server.validate_badge_style({"alpha": 1.5})
        with self.assertRaises(ValueError):
            server.validate_badge_style({"animation": "explode"})
        with self.assertRaises(ValueError):
            server.validate_badge_style({"particles": "yes"})

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

    def test_upstream_icon_catalog_is_complete_and_unique(self):
        with open(os.path.join(ROOT, "assets", "icons", "upstream_catalog.json"),
                  encoding="utf-8") as f:
            catalog = json.load(f)
        self.assertEqual(len(catalog["icons"]), 49)
        ids = [(item["source"], item["id"]) for item in catalog["icons"]]
        self.assertEqual(len(ids), len(set(ids)))
        names = [item["name"] for item in catalog["icons"]]
        self.assertEqual(len(names), len(set(names)))
        self.assertEqual(sum(item["source"] == "Cherrygram" for item in catalog["icons"]), 18)
        self.assertEqual(sum(item["source"] == "exteraGram" for item in catalog["icons"]), 31)
        self.assertEqual(catalog["sources"]["Cherrygram"]["commit"],
                         "9e1c901e6fbca5a9a346683b91c6d529376f5304")
        self.assertEqual(catalog["sources"]["exteraGram"]["commit"],
                         "b4d852f48f2734a0bee1a74a0939be621612262a")

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


if __name__ == "__main__":
    unittest.main()
