import base64
import json
import os
import shutil
import subprocess
import tempfile
import unittest

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCRIPT = os.path.join(ROOT, "scripts", "apply_change.py")


def run_openssl(args):
    return subprocess.run(args, check=True, capture_output=True)


def sign(private_key, payload):
    import tempfile as _tempfile
    with _tempfile.NamedTemporaryFile(delete=False) as f:
        f.write(payload)
        path = f.name
    try:
        return run_openssl([
            "openssl", "pkeyutl", "-sign", "-inkey", private_key,
            "-rawin", "-in", path
        ]).stdout
    finally:
        os.unlink(path)


class ApplyChangeSecurityTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.root = os.path.join(self.tmp, "repo")
        self.data = os.path.join(self.root, "data")
        os.makedirs(os.path.join(self.data, "signatures"))
        os.makedirs(os.path.join(self.data, "keys"))

        self.private_key = os.path.join(self.tmp, "private.pem")
        self.public_key = os.path.join(self.data, "keys", "grapegram_ed25519.pub.pem")
        run_openssl(["openssl", "genpkey", "-algorithm", "ED25519",
                     "-out", self.private_key])
        run_openssl(["openssl", "pkey", "-in", self.private_key,
                     "-pubout", "-out", self.public_key])

        for name in ("owner_users.json", "badge_users.json"):
            shutil.copy(os.path.join(ROOT, "data", name), os.path.join(self.data, name))
            with open(os.path.join(self.data, name), "rb") as f:
                payload = f.read()
            signature = sign(self.private_key, payload)
            with open(os.path.join(self.data, "signatures", name + ".sig"), "wb") as f:
                f.write(base64.b64encode(signature) + b"\\n")

        with open(self.private_key, "r", encoding="ascii") as f:
            self.private_pem = f.read()

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def invoke(self, owner_id):
        env = os.environ.copy()
        env.update({
            "ACTION": "grant",
            "USER_ID": "555555555",
            "BADGE": "founder",
            "REASON": "security test",
            "OWNER_ID": owner_id,
            "GRP_SIGNING_KEY_PEM": self.private_pem,
        })
        return subprocess.run(
            ["python3", SCRIPT, "--root", self.root],
            cwd=ROOT, env=env, capture_output=True, text=True
        )

    def test_signed_owner_can_grant_and_new_data_is_signed(self):
        result = self.invoke("7293365262")
        self.assertEqual(result.returncode, 0, result.stderr)
        with open(os.path.join(self.data, "badge_users.json"), encoding="utf-8") as f:
            document = json.load(f)
        self.assertTrue(document["badges"]["555555555"]["enabled"])
        self.assertEqual(document["badges"]["555555555"]["badge"], "founder")
        with open(os.path.join(self.data, "badge_users.json"), "rb") as f:
            payload = f.read()
        with open(os.path.join(self.data, "signatures", "badge_users.json.sig"), "rb") as f:
            signature = base64.b64decode(f.read().strip(), validate=True)
        self.assertEqual(len(signature), 64)
        self.assertTrue(sign(self.private_key, payload) != signature)  # Ed25519 signatures are deterministic

    def test_unlisted_owner_is_rejected(self):
        before = open(os.path.join(self.data, "badge_users.json"), "rb").read()
        result = self.invoke("111111111")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("signed owner allowlist", result.stderr)
        self.assertEqual(open(os.path.join(self.data, "badge_users.json"), "rb").read(), before)

    def test_tampered_owner_allowlist_is_rejected(self):
        path = os.path.join(self.data, "owner_users.json")
        with open(path, encoding="utf-8") as f:
            document = json.load(f)
        document["owners"].append("111111111")
        with open(path, "w", encoding="utf-8") as f:
            json.dump(document, f)
        result = self.invoke("111111111")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("owner_users.json: signature verification failed", result.stderr)


if __name__ == "__main__":
    unittest.main()
