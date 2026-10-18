#!/usr/bin/env bash
# webrok installer — downloads a prebuilt binary. No Rust required.
#
# Usage:
#   curl -sSL https://webrok.vercel.app/install.sh | bash
#   curl -sSL https://webrok.vercel.app/install.sh | bash -s -- <token>

set -euo pipefail

BASE_URL="${WEBROK_BASE_URL:-https://webrok.vercel.app}"
RELEASE_URL="${WEBROK_RELEASE_URL:-https://github.com/kevilevi07/webrok/releases/latest/download}"
INSTALL_DIR="$HOME/.webrok/bin"
BIN="$INSTALL_DIR/webrok"

# ----- platform detect -----
is_termux() { [ -n "${PREFIX:-}" ] && echo "$PREFIX" | grep -q "com.termux"; }

OS="$(uname -s 2>/dev/null || echo unknown)"
ARCH="$(uname -m 2>/dev/null || echo unknown)"

case "$OS/$ARCH" in
  Linux/x86_64|Linux/amd64)                ASSET="webrok-linux-x64"   ; PLATFORM="Linux x64" ;;
  Linux/aarch64|Linux/arm64)               ASSET="webrok-linux-arm64" ; PLATFORM="Linux ARM64" ;;
  Darwin/x86_64)                           ASSET="webrok-darwin-x64"  ; PLATFORM="macOS Intel" ;;
  Darwin/arm64)                            ASSET="webrok-darwin-arm64"; PLATFORM="macOS Apple Silicon" ;;
  *)                                       ASSET=""                   ; PLATFORM="$OS/$ARCH" ;;
esac

if is_termux; then
  case "$ARCH" in
    aarch64|arm64)  ASSET="webrok-linux-arm64" ; PLATFORM="Termux ARM64" ;;
    x86_64|amd64)   ASSET="webrok-linux-x64"   ; PLATFORM="Termux x64" ;;
  esac
fi

echo "webrok installer"
echo "  platform    : $PLATFORM"
echo "  install dir : $INSTALL_DIR"

if [ -z "$ASSET" ]; then
  echo >&2
  echo "Error: no prebuilt binary for $OS/$ARCH." >&2
  echo "Open an issue: https://github.com/kevilevi07/webrok/issues" >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"

# ----- download -----
URL="$RELEASE_URL/$ASSET"
echo
echo "Downloading $URL ..."
if command -v curl >/dev/null 2>&1; then
  curl -fSL --progress-bar "$URL" -o "$BIN"
elif command -v wget >/dev/null 2>&1; then
  wget -q --show-progress -O "$BIN" "$URL"
else
  echo "Error: need curl or wget." >&2
  exit 1
fi
chmod +x "$BIN"
echo "  saved $BIN"

# ----- PATH setup -----
PATH_LINE='export PATH="$HOME/.webrok/bin:$PATH"'
# bash: .bashrc for interactive, .profile for login
# zsh:  .zshenv is sourced by every zsh (login, interactive, non-interactive)
RC_CANDIDATES="$HOME/.bashrc $HOME/.zshrc $HOME/.zshenv $HOME/.profile"
for rc in $RC_CANDIDATES; do
  if [ ! -f "$rc" ] && [ "$rc" != "$HOME/.zshenv" ]; then continue; fi
  if ! grep -Fqs "$PATH_LINE" "$rc" 2>/dev/null; then
    printf '\n# webrok\n%s\n' "$PATH_LINE" >> "$rc"
    echo "  added PATH to $rc"
  fi
done

# Make it work in THIS invocation.
export PATH="$INSTALL_DIR:$PATH"
"$BIN" --version || true

# ----- interactive login -----
TOKEN="${1:-}"
if [ -z "$TOKEN" ] && [ -r /dev/tty ]; then
  echo
  echo "Get your token at $BASE_URL/dashboard  (sign up if you don't have one)."
  printf "Paste your webrok token (or press Enter to skip): "
  IFS= read -r TOKEN < /dev/tty || TOKEN=""
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
