#!/usr/bin/env bash
set -euo pipefail

for target in "$@"; do
  case "$target" in
    aarch64-unknown-linux-musl|loongarch64-unknown-linux-musl|riscv64gc-unknown-linux-musl|x86_64-unknown-linux-musl)
      sudo apt-get update
      sudo apt-get install --yes musl-tools=1.2.4-2
      ;;
  esac
  case "$target" in
    aarch64-unknown-linux-gnu|arm-unknown-linux-gnueabihf|arm-unknown-linux-musleabihf|armv7-unknown-linux-gnueabihf|armv7-unknown-linux-musleabihf|riscv64gc-unknown-linux-musl)
      python3 -m venv "$RUNNER_TEMP/ci-cross"
      "$RUNNER_TEMP/ci-cross/bin/python" -m pip install \
        --require-hashes --only-binary=:all: -r scripts/ci/requirements-cross.txt
      echo "$RUNNER_TEMP/ci-cross/bin" >> "$GITHUB_PATH"
      ;;
    loongarch64-unknown-linux-gnu|loongarch64-unknown-linux-musl)
      cargo install cross --git https://github.com/cross-rs/cross.git \
        --locked --rev 51f46f296253d8122c927c5bb933e3c4f27cc317
      ;;
  esac
done
