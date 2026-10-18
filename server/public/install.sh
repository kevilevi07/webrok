#!/usr/bin/env bash
# webrok installer for macOS, Linux, and Termux (Android).
#
# Usage:
#   curl -sSL https://webrok.vercel.app/install.sh | bash
#   curl -sSL https://webrok.vercel.app/install.sh | bash -s -- <token>
#
# On Linux/Termux/macOS it builds the Rust CLI from source
# (so you need Rust: see the per-platform note below).

set -euo pipefail

BASE_URL="${WEBROK_BASE_URL:-https://webrok.vercel.app}"
INSTALL_DIR="$HOME/.webrok/bin"
BIN="$INSTALL_DIR/webrok"

# ----- platform detect -----
is_termux() { [ -n "${PREFIX:-}" ] && echo "$PREFIX" | grep -q "com.termux"; }

OS="$(uname -s 2>/dev/null || echo unknown)"
case "$OS" in
  Darwin) PLATFORM="macOS" ;;
  Linux)  PLATFORM="$(is_termux && echo Termux || echo Linux)" ;;
  *)      PLATFORM="$OS" ;;
esac

echo "webrok installer"
echo "  platform    : $PLATFORM"
echo "  install dir : $INSTALL_DIR"

mkdir -p "$INSTALL_DIR"

# ----- ensure cargo -----
if ! command -v cargo >/dev/null 2>&1; then
  if [ "$PLATFORM" = "Termux" ]; then
    cat <<EOF >&2

Error: Rust (cargo) is not installed in Termux.
Install it with:
  pkg update && pkg install rust

Then re-run:
  curl -sSL $BASE_URL/install.sh | bash
EOF
    exit 1
  fi
  cat <<EOF >&2

Error: cargo is required on $PLATFORM. Install Rust first:
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Then open a new shell and re-run:
  curl -sSL $BASE_URL/install.sh | bash
EOF
  exit 1
fi

# ----- build from source -----
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo
echo "Fetching source..."
curl -sSL "https://github.com/kevilevi07/webrok/archive/refs/heads/main.tar.gz" | tar -xz -C "$tmp"
SRC_DIR="$(find "$tmp" -maxdepth 1 -type d -name 'webrok-*' | head -1)"
if [ -z "$SRC_DIR" ]; then
  echo "Error: source archive layout unexpected." >&2
  exit 1
fi

echo "Compiling (first build takes ~2 min)..."
( cd "$SRC_DIR/cli" && cargo build --release )
cp "$SRC_DIR/cli/target/release/webrok" "$BIN"
chmod +x "$BIN"
echo "  built $BIN"

# ----- add to PATH in rc files that each shell actually sources -----
PATH_LINE='export PATH="$HOME/.webrok/bin:$PATH"'
# bash: .bashrc for interactive, .profile for login
# zsh:  .zshenv is sourced by every zsh (login, interactive, non-interactive)
RC_CANDIDATES="$HOME/.bashrc $HOME/.zshrc $HOME/.zshenv $HOME/.profile"
for rc in $RC_CANDIDATES; do
  # Only create .zshenv on demand; create nothing else.
  if [ ! -f "$rc" ] && [ "$rc" != "$HOME/.zshenv" ]; then
    continue
  fi
  if ! grep -Fqs "$PATH_LINE" "$rc" 2>/dev/null; then
    { printf '\n# webrok\n%s\n' "$PATH_LINE"; } >> "$rc"
    echo "  added PATH to $rc"
  fi
done

# Make it work in THIS invocation so --version works.
export PATH="$INSTALL_DIR:$PATH"

echo
"$BIN" --version || true

# ----- interactive login (prompt for token) -----
TOKEN="${1:-}"
if [ -z "$TOKEN" ]; then
  if [ -t 0 ] || [ -r /dev/tty ]; then
    echo
    echo "Get your token at $BASE_URL/dashboard  (sign up if you don't have one)."
    # read from /dev/tty so this works when the script is piped from curl
    if [ -r /dev/tty ]; then
      printf "Paste your webrok token (or press Enter to skip): "
      IFS= read -r TOKEN < /dev/tty || TOKEN=""
    else
      printf "Paste your webrok token (or press Enter to skip): "
      IFS= read -r TOKEN || TOKEN=""
    fi
  fi
fi

TOKEN="$(printf '%s' "${TOKEN:-}" | tr -d '[:space:]')"
if [ -n "$TOKEN" ]; then
  "$BIN" --server "$BASE_URL" login --token "$TOKEN" || echo "login failed; try: webrok --server $BASE_URL login"
else
  echo "Skipped. Log in later with:"
  echo "  webrok --server $BASE_URL login"
fi

cat <<EOF

Installed.

To use webrok in THIS shell now:
  export PATH="\$HOME/.webrok/bin:\$PATH"

Or open a new terminal. Then:
  webrok http <port> --name <app-name>
EOF
