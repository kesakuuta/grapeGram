import os
import shutil
import subprocess
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
APPLY = os.path.join(ROOT, "client", "apply.py")
NS = "{http://schemas.android.com/apk/res/android}"

MANIFEST = """<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application>
        <activity-alias
            android:enabled="true"
            android:name="org.telegram.messenger.ExteralessIcon"
            android:targetActivity="org.telegram.ui.LaunchActivity" />
    </application>
</manifest>
"""

SETTINGS = """class SettingsActivity {
    private void fillItems(ArrayList<UItem> items, UniversalAdapter adapter) {
        items.add(UItem.asCustomShadow(topView, 200 - 12));
        accountNumbers.clear();
    }
    private void onClick(int id) {
        switch (id) {
            case -1: {
                presentFragment(new OpenExteraSettingsActivity());
                break;
            }
        }
    }
    private void old() {
        items.add(SettingCell.Factory.of(-1, 0xFF993C38, 0xFF993C38, R.drawable.exteraless_icon_tile, getString(R.string.OpenExteraPreferences), getString(R.string.OpenExteraInfo)));
        items.add(UItem.asShadow(null));
    }
}
"""

LAUNCHER = """public class LauncherIconController {
    public static void tryFixLauncherIconIfNeeded() {
        setIcon(LauncherIcon.EXTERALESS);
    }
    public static boolean isEnabled(LauncherIcon icon) {
        return i == PackageManager.COMPONENT_ENABLED_STATE_ENABLED
                || i == PackageManager.COMPONENT_ENABLED_STATE_DEFAULT && icon == LauncherIcon.EXTERALESS;
    }
    public enum LauncherIcon {
        NOX("NoxIcon", R.mipmap.icon_2_background_sa, R.mipmap.icon_foreground_sa, R.string.AppIconNox);
    }
}
"""

STRINGS = """<resources>
    <string name="x">x</string>
</resources>
"""


class OverlayTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        main = os.path.join(self.tmp, "TMessagesProj", "src", "main")
        os.makedirs(os.path.join(main, "java", "org", "telegram", "ui"))
        os.makedirs(os.path.join(main, "res", "values"))
        self.manifest = os.path.join(main, "AndroidManifest.xml")
        self.settings = os.path.join(main, "java", "org", "telegram", "ui",
                                     "SettingsActivity.java")
        self.launcher = os.path.join(main, "java", "org", "telegram", "ui",
                                     "LauncherIconController.java")
        self.strings = os.path.join(main, "res", "values", "strings.xml")
        for path, text in ((self.manifest, MANIFEST), (self.settings, SETTINGS),
                           (self.launcher, LAUNCHER), (self.strings, STRINGS)):
            with open(path, "w", encoding="utf-8") as f:
                f.write(text)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def run_apply(self):
        return subprocess.run([sys.executable, APPLY, "--root", self.tmp],
                              capture_output=True, text=True)

    def test_apply_and_idempotent(self):
        r = self.run_apply()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("9 icon aliases", r.stdout)

        tree = ET.parse(self.manifest)
        aliases = [e.get(NS + "name") for e in tree.iter("activity-alias")
                   if (e.get(NS + "name") or "").startswith(
                       "org.telegram.messenger.GrapeIcon")]
        self.assertEqual(len(aliases), 9)
        base_alias = [e for e in tree.iter("activity-alias")
                      if e.get(NS + "name", "").endswith("ExteralessIcon")][0]
        self.assertEqual(base_alias.get(NS + "enabled"), "false")

        with open(self.settings, encoding="utf-8") as f:
            s = f.read()
        self.assertIn("case 7001:", s)
        self.assertIn("app.grapegram.GrapeSettingsActivity", s)
        self.assertNotIn("exteraless_icon_tile", s)
        self.assertLess(s.index("case 7001:"), s.index("case -1:"))

        with open(self.launcher, encoding="utf-8") as f:
            l = f.read()
        self.assertIn("GRAPE_VIOLET", l)
        self.assertIn("setIcon(LauncherIcon.GRAPE_ORIGINAL)", l)
        self.assertIn("icon == LauncherIcon.GRAPE_ORIGINAL", l)

        with open(self.strings, encoding="utf-8") as f:
            self.assertEqual(f.read().count("GrapeIcon"), 9)

        with open(os.path.join(self.tmp, "TMessagesProj", "src", "main",
                               "java", "app", "grapegram",
                               "GrapeSettingsActivity.java")) as f:
            self.assertIn("class GrapeSettingsActivity", f.read())

        r2 = self.run_apply()
        self.assertEqual(r2.returncode, 0, r2.stderr)
        self.assertIn("already applied", r2.stdout)

    def test_missing_anchor_fails(self):
        with open(self.settings, "w", encoding="utf-8") as f:
            f.write("class SettingsActivity {}\n")
        r = self.run_apply()
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("anchor", r.stderr)


if __name__ == "__main__":
    unittest.main()
