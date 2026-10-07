#!/usr/bin/env bash
# ==============================================================================
# Traffic Status - Standalone Installer for Linux/macOS
# Usage: curl -fsSL https://github.com/its-jani/traffic-status/releases/latest/download/install.sh | sh
# ==============================================================================
set -e

REPO="its-jani/traffic-status"
BASE_URL="https://github.com/$REPO/releases/latest/download"

# 1. Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        if [ "$ARCH" = "x86_64" ]; then
            TARGET="x86_64-unknown-linux-gnu"
        else
            echo "❌ Unsupported Linux architecture: $ARCH. Only x86_64 is supported."
            exit 1
        fi
        ;;
    Darwin)
        if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
            TARGET="aarch64-apple-darwin"
        elif [ "$ARCH" = "x86_64" ]; then
            TARGET="x86_64-apple-darwin"
        else
            echo "❌ Unsupported macOS architecture: $ARCH"
            exit 1
        fi
        ;;
    *)
        echo "❌ Unsupported operating system: $OS"
        exit 1
        ;;
esac

ARCHIVE_NAME="traffic-status-${TARGET}.tar.gz"
BIN_DIR="$HOME/.local/share/traffic-status/bin"
mkdir -p "$BIN_DIR"

TEMP_DIR="$(mktemp -d)"
cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

echo "🚦 Installing Traffic Status for $TARGET..."

# 2. Download archive and SHA256SUMS
echo "📥 Downloading $ARCHIVE_NAME..."
curl -fsSL "$BASE_URL/$ARCHIVE_NAME" -o "$TEMP_DIR/$ARCHIVE_NAME"
curl -fsSL "$BASE_URL/SHA256SUMS" -o "$TEMP_DIR/SHA256SUMS"

# 3. Verify SHA256 Checksum
echo "🔒 Verifying SHA-256 checksum..."
cd "$TEMP_DIR"
if command -v sha256sum >/dev/null 2>&1; then
    grep "$ARCHIVE_NAME" SHA256SUMS | sha256sum -c -
elif command -v shasum >/dev/null 2>&1; then
    grep "$ARCHIVE_NAME" SHA256SUMS | shasum -a 256 -c -
else
    echo "⚠️ Neither sha256sum nor shasum found; skipping checksum verification."
fi

# 4. Extract binary
echo "📦 Extracting binary..."
tar -xzf "$ARCHIVE_NAME" -C "$TEMP_DIR"

if [ ! -f "$TEMP_DIR/traffic-status" ]; then
    echo "❌ Binary not found in archive."
    exit 1
fi

cp "$TEMP_DIR/traffic-status" "$BIN_DIR/traffic-status"
chmod +x "$BIN_DIR/traffic-status"
echo "  ✅ Installed binary to $BIN_DIR/traffic-status"

# 5. Run global hooks installer
echo ""
echo "⚙️ Running initial global configuration..."
"$BIN_DIR/traffic-status" install --global

echo ""
echo "🎉 Traffic Status installation complete!"
if ! echo "$PATH" | grep -q "$BIN_DIR"; then
    echo "ℹ️ To run 'traffic-status' directly from your shell, add this line to your ~/.bashrc or ~/.zshrc:"
    echo "   export PATH=\"\$HOME/.local/share/traffic-status/bin:\$PATH\""
fi
