#!/usr/bin/env sh
set -eu

command -v cargo >/dev/null 2>&1 || {
  echo "erro: cargo não está instalado" >&2
  exit 127
}

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
