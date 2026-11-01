#!/usr/bin/env python3
"""Read the root Cargo package and reject tags for a different app version."""

import json
import os
from pathlib import Path
import tomllib

package = tomllib.loads(Path("Cargo.toml").read_text())["package"]
version = package["version"]
tag = f"refiner-v{version}"
requested = os.environ.get("RELEASE_TAG", "")
if requested and requested != tag:
    raise SystemExit(f"Release tag {requested!r} does not match Cargo.toml; expected {tag!r}")

result = {"version": version, "tag": tag}
if output := os.environ.get("GITHUB_OUTPUT"):
    with open(output, "a") as stream:
        for key, value in result.items():
            stream.write(f"{key}={value}\n")
print(json.dumps(result))
