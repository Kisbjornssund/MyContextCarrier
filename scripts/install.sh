#!/usr/bin/env sh
# MyContextPort installer
# Usage: curl -fsSL https://mycontextport.dev/install.sh | sh
set -e

REPO="Kisbjornssund/MyContextPort"
BINARY="mycontextport"
INSTALL_DIR="/usr/local/bin"

# ── Helpers ──────────────────────────────────────────────────────────────────

say()  { printf '\033[1;32m==>\033[0m %s\n' "$*"; }
die()  { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || die "required tool not found: $1"; }

# ── Platform detection ────────────────────────────────────────────────────────

detect_target() {
    OS=$(uname -s)
    ARCH=$(uname -m)

    case "$OS" in
        Linux)
            case "$ARCH" in
                x86_64)         echo "mycontextport-linux-x86_64" ;;
                aarch64|arm64)  echo "mycontextport-linux-aarch64" ;;
                *)              die "Unsupported Linux architecture: $ARCH" ;;
            esac
            ;;
        Darwin)
            case "$ARCH" in
                x86_64)         echo "mycontextport-macos-x86_64" ;;
                arm64|aarch64)  echo "mycontextport-macos-aarch64" ;;
                *)              die "Unsupported macOS architecture: $ARCH" ;;
            esac
            ;;
        *)
            die "Unsupported OS: $OS. Download manually from https://github.com/$REPO/releases"
            ;;
    esac
}

# ── Fetch latest release version ─────────────────────────────────────────────

fetch_latest_version() {
    need curl
    VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
        | grep '"tag_name"' \
        | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
    if [ -z "$VERSION" ]; then
        die "Could not determine latest release. Check https://github.com/$REPO/releases"
    fi
    echo "$VERSION"
}

# ── Download and verify ───────────────────────────────────────────────────────

download() {
    ARTIFACT="$1"
    VERSION="$2"
    BASE_URL="https://github.com/$REPO/releases/download/$VERSION"
    TMP_DIR=$(mktemp -d)

    say "Downloading MyContextPort $VERSION ($ARTIFACT)..."
    curl -fsSL "$BASE_URL/$ARTIFACT"        -o "$TMP_DIR/$BINARY"
    curl -fsSL "$BASE_URL/$ARTIFACT.sha256" -o "$TMP_DIR/$ARTIFACT.sha256"

    # Verify checksum
    say "Verifying checksum..."
    EXPECTED=$(cat "$TMP_DIR/$ARTIFACT.sha256" | awk '{print $1}')
    if command -v sha256sum >/dev/null 2>&1; then
        ACTUAL=$(sha256sum "$TMP_DIR/$BINARY" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
        ACTUAL=$(shasum -a 256 "$TMP_DIR/$BINARY" | awk '{print $1}')
    else
        say "Warning: no sha256 tool found, skipping checksum verification"
        ACTUAL="$EXPECTED"
    fi

    if [ "$ACTUAL" != "$EXPECTED" ]; then
        rm -rf "$TMP_DIR"
        die "Checksum mismatch — download may be corrupt. Expected: $EXPECTED  Got: $ACTUAL"
    fi

    chmod +x "$TMP_DIR/$BINARY"
    echo "$TMP_DIR"
}

# ── Install ───────────────────────────────────────────────────────────────────

install_binary() {
    TMP_DIR="$1"

    if [ -w "$INSTALL_DIR" ]; then
        mv "$TMP_DIR/$BINARY" "$INSTALL_DIR/$BINARY"
    else
        say "Installing to $INSTALL_DIR (requires sudo)..."
        sudo mv "$TMP_DIR/$BINARY" "$INSTALL_DIR/$BINARY"
    fi

    rm -rf "$TMP_DIR"
}

# ── Main ──────────────────────────────────────────────────────────────────────

main() {
    need curl

    ARTIFACT=$(detect_target)
    VERSION=$(fetch_latest_version)
    TMP_DIR=$(download "$ARTIFACT" "$VERSION")
    install_binary "$TMP_DIR"

    say "MyContextPort $VERSION installed to $INSTALL_DIR/$BINARY"
    "$INSTALL_DIR/$BINARY" --version

    cat <<EOF

Next steps:
  mycontextport init       Set up your local context store
  mycontextport --help     See all commands
  mycontextport mcp serve  Start the MCP server for AI tool integration

Documentation: https://docs.mycontextport.dev
EOF
}

main "$@"
