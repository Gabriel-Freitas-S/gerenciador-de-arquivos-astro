#!/usr/bin/env bash
set -euo pipefail

tauri_version="${1:-^2}"

die() {
    echo "[setup-tauri] $*" >&2
    exit 1
}

if ! command -v cargo >/dev/null 2>&1; then
    die "cargo não foi encontrado no PATH. Instale o Rust via https://rustup.rs/."
fi

if ! command -v cargo-binstall >/dev/null 2>&1; then
    cat <<'EOF'
cargo-binstall não foi encontrado.
Instale com:
  curl -L https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
Depois execute este script novamente.
EOF
    exit 1
fi

echo "Instalando tauri-cli (${tauri_version}) via cargo-binstall..."
cargo binstall "tauri-cli@${tauri_version}" --secure --no-confirm
