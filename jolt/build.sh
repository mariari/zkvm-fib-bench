#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REV="aec34d10a6b5e96e698ee933d61a7c32a5a9b678"
TOOLCHAIN="$ROOT/.toolchain"

if [[ ! -x "$TOOLCHAIN/bin/jolt" ]]; then
    cargo +1.95.0 install --git https://github.com/a16z/jolt --rev "$REV" jolt \
        --bin jolt --root "$TOOLCHAIN"
fi

( cd "$ROOT" && cargo build --release --locked )
