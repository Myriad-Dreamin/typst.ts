#!/usr/bin/env python3
"""Install CI binaries only after verifying their repository-pinned SHA-256."""

import hashlib
import io
import os
from pathlib import Path
import platform
import subprocess
import sys
import tarfile
import urllib.request
import zipfile


DIST = "https://github.com/Myriad-Dreamin/cargo-dist/releases/download/v0.32.0-tinymist.5"
TOOLS = {
    "fonts": (
        "https://github.com/Myriad-Dreamin/shiroa/releases/download/v0.1.0/charter-font-assets.tar.gz",
        "df658ec404512c540e475435e9143e6f6a866fac801e09878cbd1f6f8a4cc0d3",
        None,
    ),
    "wasm-pack": (
        "https://github.com/wasm-bindgen/wasm-pack/releases/download/v0.13.1/wasm-pack-v0.13.1-x86_64-unknown-linux-musl.tar.gz",
        "c539d91ccab2591a7e975bcf82c82e1911b03335c80aa83d67ad25ed2ad06539",
        "wasm-pack",
    ),
    "shiroa": (
        "https://github.com/Myriad-Dreamin/shiroa/releases/download/v0.3.1-rc2/shiroa-x86_64-unknown-linux-gnu.tar.gz",
        "120ee860aebb591561506d473c15db183c697f5d1bd7498fee3f7f0564465b02",
        "shiroa",
    ),
    "parse-changelog": (
        "https://github.com/taiki-e/parse-changelog/releases/download/v0.6.18/parse-changelog-x86_64-unknown-linux-musl.tar.gz",
        "e0f0aa87a94501111052dbb7f3c79b28b61ab421427641ba9ac4e4ae61d6eee9",
        "parse-changelog",
    ),
    "zizmor": (
        "https://github.com/zizmorcore/zizmor/releases/download/v1.22.0/zizmor-x86_64-unknown-linux-gnu.tar.gz",
        "f7633bbf155cddebd3fdfdcbfaa777a4e89e4da1bc719b08dfa9ae50e5daae46",
        "zizmor",
    ),
    "mold": (
        "https://github.com/rui314/mold/releases/download/v3.0.0/mold-3.0.0-x86_64-linux.tar.gz",
        "6c90d4a474c7c0409dfb575be03a5345878ac14fdba18de8b40fa58c60121189",
        "mold",
    ),
    "rustup": (
        "https://static.rust-lang.org/rustup/archive/1.28.2/x86_64-unknown-linux-gnu/rustup-init",
        "20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c",
        "rustup-init",
    ),
    "dist-Linux-X64": (
        f"{DIST}/cargo-dist-x86_64-unknown-linux-musl.tar.xz",
        "bfcedaf1819be34641075bc758f384eb8cdbfee915e6d8e32b3bc58bb4e579d4",
        "dist",
    ),
    "dist-Linux-ARM64": (
        f"{DIST}/cargo-dist-aarch64-unknown-linux-musl.tar.xz",
        "a974041e3ebd8c655783cce9385431366d6cb6abbb79bf490ccc97015c2cf498",
        "dist",
    ),
    "dist-macOS-X64": (
        f"{DIST}/cargo-dist-x86_64-apple-darwin.tar.xz",
        "0978bee48f8c617143f27a9238610d4d0cfba350bdd16e20f038d86a9d8f34da",
        "dist",
    ),
    "dist-macOS-ARM64": (
        f"{DIST}/cargo-dist-aarch64-apple-darwin.tar.xz",
        "0ebaf8838fd94f3adfc04cedc1d497bafb54c518ab1a5e45bf5a48c1bac7276c",
        "dist",
    ),
    "dist-Windows-X64": (
        f"{DIST}/cargo-dist-x86_64-pc-windows-msvc.zip",
        "22264af900175d58bc762db04d92eb9a4eab5d845e88e0c3e7859b825b8d42f7",
        "dist.exe",
    ),
}


def install(name, destination):
    if name == "dist":
        system = {"Darwin": "macOS"}.get(platform.system(), platform.system())
        arch = {"x86_64": "X64", "AMD64": "X64", "aarch64": "ARM64", "arm64": "ARM64"}[
            platform.machine()
        ]
        name = f"dist-{system}-{arch}"
    url, expected, binary = TOOLS[name]
    with urllib.request.urlopen(url, timeout=120) as response:
        data = response.read()
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError(f"SHA-256 mismatch for {url}")
    if name == "fonts":
        destination.mkdir(parents=True, exist_ok=True)
        with tarfile.open(fileobj=io.BytesIO(data)) as archive:
            for member in archive.getmembers():
                if member.isfile() and member.name.endswith((".ttf", ".otf")):
                    (destination / Path(member.name).name).write_bytes(
                        archive.extractfile(member).read()
                    )
        return destination
    if url.endswith(".zip"):
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            matches = [p for p in archive.namelist() if Path(p).name == binary]
            if len(matches) != 1:
                raise ValueError(f"Expected exactly one {binary} in {url}")
            data = archive.read(matches[0])
    elif ".tar." in url:
        with tarfile.open(fileobj=io.BytesIO(data)) as archive:
            matches = [p for p in archive.getmembers() if p.isfile() and Path(p.name).name == binary]
            if len(matches) != 1:
                raise ValueError(f"Expected exactly one {binary} in {url}")
            data = archive.extractfile(matches[0]).read()
    destination.mkdir(parents=True, exist_ok=True)
    target = destination / binary
    target.write_bytes(data)
    target.chmod(0o755)
    return target


if __name__ == "__main__":
    destination = (
        Path("assets/fonts") if sys.argv[1] == "fonts" else Path.home() / ".local" / "bin"
    )
    target = install(sys.argv[1], destination)
    if sys.argv[1] == "rustup":
        subprocess.run(
            [str(target), "-y", "--profile", "minimal", "--default-toolchain", "1.92.0"],
            check=True,
        )
        destination = Path.home() / ".cargo" / "bin"
    if sys.argv[1] != "fonts" and "GITHUB_PATH" in os.environ:
        with open(os.environ["GITHUB_PATH"], "a", encoding="utf-8") as path_file:
            path_file.write(f"{destination}\n")
