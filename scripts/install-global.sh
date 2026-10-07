#!/usr/bin/env bash
# ==============================================================================
# Traffic Light - Global System-Wide Installer (Bash / POSIX)
# Installs Traffic Light globally across the entire machine for all current and future projects.
# ==============================================================================
set -e

echo "🚦 Installing Traffic Light Globally (All Projects)..."

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 1. Build and install native binary via cargo
echo "📦 [1/3] Building & installing native binary (cargo install)..."
cd "$SOURCE_ROOT"
if command -v cargo >/dev/null 2>&1; then
    cargo install --path . --force
    echo "  ✅ Binary installed to ~/.cargo/bin/traffic-light"
else
    echo "  ⚠️ Cargo not found in PATH. Ensure cargo is installed."
fi

# 2. Global Claude Code Setup (~/.claude/commands/traffic.md)
echo "🤖 [2/3] Configuring Global Claude Code commands..."
mkdir -p "$HOME/.claude/commands"
if [ -f "$SOURCE_ROOT/.claude/commands/traffic.md" ]; then
    cp "$SOURCE_ROOT/.claude/commands/traffic.md" "$HOME/.claude/commands/traffic.md"
    echo "  ✅ Global Claude Code command installed: $HOME/.claude/commands/traffic.md"
fi

# 3. Global OpenCode Setup (~/.config/opencode & ~/.opencode)
echo "⚡ [3/3] Configuring Global OpenCode plugins & commands..."
for BASE in "$HOME/.config/opencode" "$HOME/.opencode"; do
    mkdir -p "$BASE/commands" "$BASE/plugins"
    if [ -f "$SOURCE_ROOT/.opencode/commands/traffic.md" ]; then
        cp "$SOURCE_ROOT/.opencode/commands/traffic.md" "$BASE/commands/traffic.md"
    fi
    if [ -f "$SOURCE_ROOT/.opencode/plugins/traffic-light.js" ]; then
        cp "$SOURCE_ROOT/.opencode/plugins/traffic-light.js" "$BASE/plugins/traffic-light.js"
    fi
    echo "  ✅ Global OpenCode plugin & commands installed in: $BASE"
done

echo ""
echo "🎉 Global Installation Complete!"
echo "Scope: Entire Laptop / All Projects (Current and Future)"
echo "You can now run '/traffic on' from any folder or project in Claude Code or OpenCode."
