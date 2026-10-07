#!/usr/bin/env bash
# ==============================================================================
# Traffic Status - Single Project Uninstaller (Bash / POSIX)
# Removes Traffic Status slash commands & hooks strictly from one specified project.
# ==============================================================================
set -e

PROJECT_PATH="${1:-.}"
TARGET_DIR=$(cd "$PROJECT_PATH" && pwd)

echo "🗑️  Removing Traffic Status from project: $TARGET_DIR"

# 1. Remove Claude Code command
if [ -f "$TARGET_DIR/.claude/commands/traffic.md" ]; then
    rm -f "$TARGET_DIR/.claude/commands/traffic.md"
    echo "  🗑️  Removed .claude/commands/traffic.md"
    rmdir "$TARGET_DIR/.claude/commands" 2>/dev/null || true
    rmdir "$TARGET_DIR/.claude" 2>/dev/null || true
fi

# 2. Remove OpenCode command & plugin
if [ -f "$TARGET_DIR/.opencode/commands/traffic.md" ]; then
    rm -f "$TARGET_DIR/.opencode/commands/traffic.md"
    echo "  🗑️  Removed .opencode/commands/traffic.md"
    rmdir "$TARGET_DIR/.opencode/commands" 2>/dev/null || true
fi

if [ -f "$TARGET_DIR/.opencode/plugins/traffic-status.js" ]; then
    rm -f "$TARGET_DIR/.opencode/plugins/traffic-status.js"
    echo "  🗑️  Removed .opencode/plugins/traffic-status.js"
    rmdir "$TARGET_DIR/.opencode/plugins" 2>/dev/null || true
fi

rm -f "$TARGET_DIR/.opencode/plugins/traffic-light.js"
rmdir "$TARGET_DIR/.opencode/plugins" 2>/dev/null || true
rmdir "$TARGET_DIR/.opencode" 2>/dev/null || true

echo ""
echo "✅ Project uninstallation complete!"
echo "Traffic Status has been removed from $TARGET_DIR."
