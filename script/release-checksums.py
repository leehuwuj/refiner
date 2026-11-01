#!/usr/bin/env python3
"""Require every native platform's assets before generating release checksums."""

import hashlib
import os
from pathlib import Path

version = os.environ["RELEASE_VERSION"]
dist = Path("dist")
expected = {
    f"Refiner_{version}_{platform}_{arch}.{extension}"
    for platform, arch, extensions in [
        ("macos", "arm64", ("dmg", "zip")),
        ("macos", "x64", ("dmg", "zip")),
        ("windows", "x64", ("zip",)),
        ("windows", "arm64", ("zip",)),
    ]
    for extension in extensions
}
actual = {path.name for path in dist.iterdir() if path.is_file() and path.name != "SHA256SUMS"}
if actual != expected:
    raise SystemExit(
        f"Incomplete release: missing={sorted(expected - actual)}, unexpected={sorted(actual - expected)}"
    )

lines = []
for name in sorted(expected):
    path = dist / name
    if path.stat().st_size == 0:
        raise SystemExit(f"Empty release asset: {name}")
    with path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    lines.append(f"{digest}  {name}\n")
(dist / "SHA256SUMS").write_text("".join(lines))
print(f"Verified {len(expected)} native release assets and wrote dist/SHA256SUMS")
