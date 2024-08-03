

import subprocess
import sys

pub = ["cargo", "publish", *sys.argv[1:], "-p"]
feats = ["--features", "no-content-hint"]
subprocess.run([*pub, "reflexo"])
subprocess.run([*pub, "reflexo-vfs"])
subprocess.run([*pub, "reflexo-world"])
subprocess.run([*pub, "reflexo-typst2vec"])
subprocess.run([*pub, "reflexo-vec2bbox"])
subprocess.run([*pub, "reflexo-vec2canvas"])
subprocess.run([*pub, "reflexo-vec2dom"])
subprocess.run([*pub, "reflexo-vec2sema"])
subprocess.run([*pub, "reflexo-vec2svg", *feats])
subprocess.run([*pub, "reflexo-typst", *feats])
