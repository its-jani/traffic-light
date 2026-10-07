#!/usr/bin/env bash
# ==============================================================================
# Traffic Status - Complete Machine Uninstaller (Bash / POSIX)
# Removes Traffic Status from this machine (global configs, binaries, & daemons).
# ==============================================================================
echo "🚨 Removing Traffic Status from this machine..."

# 1. Stop any running traffic-status processes
echo "🛑 [1/4] Stopping any active Traffic Status instances..."
pkill -f traffic-status 2>/dev/null || true
echo "  ✅ Terminated running processes."

# 2. Remove installed binaries
echo "📦 [2/4] Removing installed binaries..."
rm -f "$HOME/.local/share/traffic-status/bin/traffic-status"
rm -f "$HOME/.cargo/bin/traffic-light" "$HOME/.cargo/bin/traffic-light.exe"
echo "  🗑️  Deleted stable binary"

# 3. Remove Global Claude Code commands
echo "🤖 [3/4] Removing Global Claude Code configs..."
rm -f "$HOME/.claude/commands/traffic.md"
echo "  🗑️  Deleted $HOME/.claude/commands/traffic.md"

# 4. Remove Global OpenCode plugins and commands (files only)
echo "⚡ [4/4] Removing Global OpenCode configs..."
for BASE in "$HOME/.config/opencode" "$HOME/.opencode"; do
    rm -f "$BASE/commands/traffic.md" "$BASE/plugins/traffic-status.js" "$BASE/plugins/traffic-light.js"
    echo "  🗑️  Cleaned $BASE"
done

echo ""
echo "✨ Traffic Status global uninstallation complete."
