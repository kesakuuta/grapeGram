import importlib.util
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "client" / "check_branding.py"
SPEC = importlib.util.spec_from_file_location("check_branding", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class BrandingCheckTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        (self.root / "src/main/java").mkdir(parents=True)
        (self.root / "src/main/kotlin").mkdir(parents=True)
        (self.root / "src/main/res/values").mkdir(parents=True)

    def tearDown(self):
        self.temp.cleanup()

    def test_allows_upstream_namespaces_and_legacy_migration_keys(self):
        source = self.root / "src/main/java/Legacy.java"
        source.write_text(
            'String a = "app.exteraless.plugins";\n'
            'String b = "exteraless_plugins";\n'
            'String c = "exteralessFaveStickers";\n'
            'String d = "ExteralessIcon";\n',
            encoding="utf-8",
        )
        self.assertEqual(MODULE.check_sources(self.root), [])

    def test_rejects_user_facing_legacy_string(self):
        source = self.root / "src/main/java/Brand.java"
        source.write_text('String label = "exteraless";\n', encoding="utf-8")
        errors = MODULE.check_sources(self.root)
        self.assertEqual(len(errors), 1)
        self.assertIn("Brand.java:1", errors[0])

    def test_rejects_legacy_resource_text(self):
        xml = self.root / "src/main/res/values/strings.xml"
        xml.write_text(
            '<resources><string name="AppName">exteraless</string></resources>',
            encoding="utf-8",
        )
        errors = MODULE.check_resources(self.root)
        self.assertEqual(len(errors), 1)
        self.assertIn("AppName", errors[0])


if __name__ == "__main__":
    unittest.main()
