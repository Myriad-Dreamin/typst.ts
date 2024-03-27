

import subprocess

pub = ["cargo", "publish", "-p"]
feats = ["--features", "no-content-hint"]
subprocess.run([*pub, "reflexo"])
subprocess.run([*pub, "reflexo-vec2canvas"])
subprocess.run([*pub, "typst-ts-core", *feats])
subprocess.run([*pub, "typst-ts-svg-exporter", *feats])
subprocess.run([*pub, "typst-ts-compiler", *feats])
