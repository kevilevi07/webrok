#!/usr/bin/env bash
# webrok installer for macOS / Linux.
# Usage:  curl -sSL https://webrok.vercel.app/install.sh | bash
#
# NOTE: Windows is the only platform with a prebuilt binary right now.
#       On macOS/Linux you'll build from source (Rust required).

set -euo pipefail

BASE_URL="${WEBROK_BASE_URL:-https://webrok.vercel.app}"
INSTALL_DIR="$HOME/.webrok/bin"
BIN="$INSTALL_DIR/webrok"

echo "webrok installer"
echo "  install dir : $INSTALL_DIR"

mkdir -p "$INSTALL_DIR"

OS="$(uname -s)"
case "$OS" in
  Darwin|Linux)
    if ! command -v cargo >/dev/null 2>&1; then
      echo "Error: cargo is required on $OS. Install Rust: https://rustup.rs" >&2
      exit 1
    fi
    tmp="$(mktemp -d)"
    echo "Fetching source..."
    curl -sSL "https://github.com/kevilevi07/webrok/archive/refs/heads/main.tar.gz" | tar -xz -C "$tmp"
    (cd "$tmp"/*/cli && cargo build --release)
    cp "$tmp"/*/cli/target/release/webrok "$BIN"
    chmod +x "$BIN"
    rm -rf "$tmp"
    ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

# Add to PATH in common shell rc files
add_path_line='export PATH="$HOME/.webrok/bin:$PATH"'
for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
  if [ -f "$rc" ] && ! grep -Fqs "$add_path_line" "$rc"; then
    printf '\n# webrok\n%s\n' "$add_path_line" >> "$rc"
    echo "Added to PATH in $rc"
  fi
done

echo
echo "Installed at $BIN"
echo "Open a new terminal (or 'source ~/.bashrc'), then:"
echo "  1. Visit $BASE_URL/dashboard to copy your token"
echo "  2. webrok --server $BASE_URL login"
echo "  3. webrok http <port> --name <app-name>"
