#!/usr/bin/env python3
"""Reject user-facing grapeGram strings and storage literals branded as exteraless.

Package/class namespaces and explicitly retained migration keys are allowlisted:
they are compatibility identifiers, not visible branding or new storage names.
"""
from __future__ import annotations

import argparse
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

STRING_LITERAL = re.compile(r'"(?:\\.|[^"\\])*"')
ALLOWED_EXACT = {
    "exteraless_plugins",          # one-time migration source
    "exteraless_plugin_grants",    # one-time migration source
    "exteraless_badges",           # one-time migration source
    "exteralessfavestickers",      # legacy backup JSON key; keep restore compatibility
    "exteralessicon",              # disabled upstream activity-alias component
}


def allowed_compatibility_literal(value: str) -> bool:
    lowered = value.lower()
    if lowered in ALLOWED_EXACT:
        return True
    if "app.exteraless." in lowered:
        return True  # upstream Java/Kotlin namespace must remain binary-compatible
    if lowered == "org.telegram.messenger.exteralessicon":
        return True  # disabled upstream launcher alias
    return False


def check_sources(root: Path) -> list[str]:
    errors: list[str] = []
    for source_root in (root / "src/main/java", root / "src/main/kotlin"):
        if not source_root.is_dir():
            continue
        for path in source_root.rglob("*"):
            if not path.is_file() or path.suffix not in (".java", ".kt"):
                continue
            try:
                source = path.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            for match in STRING_LITERAL.finditer(source):
                literal = match.group(0)
                value = literal[1:-1]
                if "exteraless" not in value.lower():
                    continue
                if allowed_compatibility_literal(value):
                    continue
                line = source.count("\n", 0, match.start()) + 1
                errors.append(f"{path.relative_to(root)}:{line}: legacy branding/path literal {literal}")
    return errors


def check_resources(root: Path) -> list[str]:
    errors: list[str] = []
    values_root = root / "src/main/res"
    if not values_root.is_dir():
        return errors
    for path in values_root.glob("values*/*.xml"):
        try:
            tree = ET.parse(path)
        except ET.ParseError as exc:
            errors.append(f"{path.relative_to(root)}: malformed XML: {exc}")
            continue
        for element in tree.iter():
            if list(element):
                continue
            value = (element.text or "").strip()
            if "exteraless" in value.lower():
                errors.append(
                    f"{path.relative_to(root)}: resource text still contains legacy branding: {value!r}"
                )
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("project", type=Path, help="materialized TMessagesProj directory")
    args = parser.parse_args()
    root = args.project.resolve()
    errors = check_sources(root) + check_resources(root)
    if errors:
        print("grapeGram branding check FAILED:", file=sys.stderr)
        for error in errors:
            print(f" - {error}", file=sys.stderr)
        return 1
    print("grapeGram branding check passed (legacy package names and migration keys are allowlisted)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
