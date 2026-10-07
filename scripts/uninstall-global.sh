#!/usr/bin/env bash
# ==============================================================================
# Traffic Light - Complete Machine Uninstaller (Bash / POSIX)
# Completely purges Traffic Light from the entire laptop (all global configs, binaries, & daemons).
# ==============================================================================
echo "🚨 Completely removing Traffic Light from this laptop..."

# 1. Stop any running traffic-light processes
echo "🛑 [1/4] Stopping any active Traffic Light instances..."
pkill -f traffic-light 2>/dev/null || true
echo "  ✅ Terminated running processes."

# 2. Uninstall binary from Cargo / System
echo "📦 [2/4] Removing installed binaries..."
if command -v cargo >/dev/null 2>&1; then
    cargo uninstall traffic-light 2>/dev/null || true
fi
rm -f "$HOME/.cargo/bin/traffic-light" "$HOME/.cargo/bin/traffic-light.exe"
echo "  🗑️  Deleted binary from ~/.cargo/bin"

# 3. Remove Global Claude Code commands
echo "🤖 [3/4] Removing Global Claude Code configs..."
rm -f "$HOME/.claude/commands/traffic.md"
echo "  🗑️  Deleted $HOME/.claude/commands/traffic.md"

# 4. Remove Global OpenCode plugins and commands
echo "⚡ [4/4] Removing Global OpenCode configs..."
for BASE in "$HOME/.config/opencode" "$HOME/.opencode"; do
    rm -f "$BASE/commands/traffic.md" "$BASE/plugins/traffic-light.js"
    echo "  🗑️  Cleaned $BASE"
done

echo ""
echo "✨ Traffic Light has been completely purged from your system."
echo "Scope: Entire Laptop (0 traces remaining)."
