#!/usr/bin/env bash
# ==============================================================================
# Traffic Light - Single Project Installer (Bash / POSIX)
# Installs Traffic Light slash commands & hooks strictly to one specified project.
# ==============================================================================
set -e

PROJECT_PATH="${1:-.}"
TARGET_DIR=$(cd "$PROJECT_PATH" && pwd)

echo "🚦 Installing Traffic Light for project: $TARGET_DIR"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 1. Claude Code Project Setup
mkdir -p "$TARGET_DIR/.claude/commands"
if [ -f "$SOURCE_ROOT/.claude/commands/traffic.md" ]; then
    cp "$SOURCE_ROOT/.claude/commands/traffic.md" "$TARGET_DIR/.claude/commands/traffic.md"
    echo "  ✅ Added Claude Code command: .claude/commands/traffic.md"
fi

# 2. OpenCode Project Setup
mkdir -p "$TARGET_DIR/.opencode/commands"
mkdir -p "$TARGET_DIR/.opencode/plugins"

if [ -f "$SOURCE_ROOT/.opencode/commands/traffic.md" ]; then
    cp "$SOURCE_ROOT/.opencode/commands/traffic.md" "$TARGET_DIR/.opencode/commands/traffic.md"
    echo "  ✅ Added OpenCode command: .opencode/commands/traffic.md"
fi

if [ -f "$SOURCE_ROOT/.opencode/plugins/traffic-light.js" ]; then
    cp "$SOURCE_ROOT/.opencode/plugins/traffic-light.js" "$TARGET_DIR/.opencode/plugins/traffic-light.js"
    echo "  ✅ Added OpenCode plugin: .opencode/plugins/traffic-light.js"
fi

echo ""
echo "✨ Project installation complete!"
echo "Scope: Project-only ($TARGET_DIR)"
echo "You can now use /traffic on, /traffic off, etc. inside this project."
