#!/usr/bin/env python3
"""Reject floating manifest dependencies and GitHub Actions references."""

import json
from pathlib import Path
import re
import subprocess
import tomllib


VERSION = r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?"
SHA = r"[0-9a-f]{40}"
DIGEST = r"@sha256:[0-9a-f]{64}"
errors = []
tracked = subprocess.check_output(["git", "ls-files", "-z"], text=True).split("\0")
manifests = {Path(p) for p in tracked if Path(p).name in ("package.json", "Cargo.toml")}
manifests.add(Path("scripts/ci/package.json"))

for path in sorted(manifests):
    if path.name == "package.json":
        package = json.loads(path.read_text())
        for field in ("dependencies", "devDependencies", "optionalDependencies", "peerDependencies", "resolutions"):
            for name, version in package.get(field, {}).items():
                if version.startswith(("link:", "file:", "workspace:")):
                    continue
                if not re.fullmatch(VERSION, version):
                    errors.append(f"{path}: {field}.{name} must use an exact version: {version}")
    else:
        manifest = tomllib.loads(path.read_text())
        tables = [manifest, manifest.get("workspace", {})]
        tables.extend(manifest.get("target", {}).values())
        tables.extend({"dependencies": patch} for patch in manifest.get("patch", {}).values())
        for table in tables:
            for field in ("dependencies", "dev-dependencies", "build-dependencies"):
                for name, value in table.get(field, {}).items():
                    if isinstance(value, str):
                        value = {"version": value}
                    if value.get("workspace") or ("path" in value and "version" not in value):
                        continue
                    if "git" in value:
                        if not re.fullmatch(SHA, value.get("rev", "")):
                            errors.append(f"{path}: {name} must use a full Git revision")
                    elif not re.fullmatch("=" + VERSION, value.get("version", "")):
                        errors.append(f"{path}: {name} must use an exact Cargo version")

for path in sorted(Path(".github").rglob("*")):
    if path.suffix not in (".yml", ".yaml"):
        continue
    for number, line in enumerate(path.read_text().splitlines(), 1):
        image = re.match(r"\s*(?:image|docker):\s+['\"]?([^\s'\"$]+)", line)
        if image and not re.search(DIGEST + "$", image[1]):
            errors.append(f"{path}:{number}: container must use an immutable digest")
        node = re.match(r"\s*node-version:\s+['\"]?([^\s'\"$]+)", line)
        if node and not re.fullmatch(VERSION, node[1]):
            errors.append(f"{path}:{number}: Node.js must use an exact release")
        match = re.match(r"\s*(?:-\s*)?uses:\s+(\S+)", line)
        if not match or match[1].startswith("./"):
            continue
        if not re.fullmatch(r"[^@\s]+@" + SHA, match[1]):
            errors.append(f"{path}:{number}: action must use a full commit SHA")

for path in (Path("Cross.toml"), Path("dist-workspace.toml"), Path("Dockerfile")):
    for number, line in enumerate(path.read_text().splitlines(), 1):
        image = re.search(r'\bimage\s*=\s*"([^"]+)"', line)
        base = re.match(r"FROM\s+(\S+)", line)
        if (image or base) and not re.search(DIGEST + "$", (image or base)[1]):
            errors.append(f"{path}:{number}: container must use an immutable digest")

if errors:
    raise SystemExit("\n".join(errors))
print("Dependency manifests, CI actions, Node.js releases and containers are pinned.")
