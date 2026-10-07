#!/usr/bin/env bash
# ==============================================================================
# Traffic Status - Single Project Installer (Bash / POSIX)
# Installs Traffic Status slash commands & hooks strictly to one specified project.
# ==============================================================================
set -e

PROJECT_PATH="${1:-.}"
TARGET_DIR=$(cd "$PROJECT_PATH" && pwd)

echo "🚦 Installing Traffic Status for project: $TARGET_DIR"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 1. Claude Code Project Setup
mkdir -p "$TARGET_DIR/.claude/commands"
cat << 'EOF' > "$TARGET_DIR/.claude/commands/traffic.md"
<!-- generated-by: traffic-status -->
# Traffic Status Slash Command (/traffic)

Execute the corresponding action immediately in 1 step without reading source files or repository searches.

### Control & Auto-Start:

- **`/traffic on`** (or `/traffic`):
  Starts the floating status daemon (if not already running) and registers the active session.
  ```powershell
  traffic-status on "Claude"
  ```

- **`/traffic yellow [msg]`** (or when processing/thinking/running tools):
  ```powershell
  traffic-status yellow "$ARG"
  ```

- **`/traffic green [msg]`** (or when ready/waiting for prompt):
  ```powershell
  traffic-status green "$ARG"
  ```

- **`/traffic red [msg]`** (or when error/needs confirmation):
  ```powershell
  traffic-status red "$ARG"
  ```

- **`/traffic off`**:
  ```powershell
  traffic-status off
  ```
EOF
echo "  ✅ Added Claude Code command: .claude/commands/traffic.md"

# 2. OpenCode Project Setup
mkdir -p "$TARGET_DIR/.opencode/commands"
mkdir -p "$TARGET_DIR/.opencode/plugins"

cat << 'EOF' > "$TARGET_DIR/.opencode/commands/traffic.md"
<!-- generated-by: traffic-status -->
# Traffic Status Command (/traffic)

When `/traffic` is invoked in OpenCode, execute the corresponding action immediately in 1 step. **Do not search the repository, do not inspect source files, and do not run codebase searches.**

## Commands:
- `/traffic on` -> `traffic-status on "OpenCode"`
- `/traffic off` -> `traffic-status off`
- `/traffic yellow [message]` -> `traffic-status yellow "$ARG"`
- `/traffic green [message]` -> `traffic-status green "$ARG"`
- `/traffic red [message]` -> `traffic-status red "$ARG"`

### Automatic Lifecycle Rule:
- When thinking, compiling, or executing tools: send `yellow`.
- When an error occurs or waiting for user confirmation: send `red`.
- When task is completed and waiting for user prompt: send `green`.
EOF
echo "  ✅ Added OpenCode command: .opencode/commands/traffic.md"

if [ -f "$SOURCE_ROOT/.opencode/plugins/traffic-status.js" ]; then
    cp "$SOURCE_ROOT/.opencode/plugins/traffic-status.js" "$TARGET_DIR/.opencode/plugins/traffic-status.js"
    echo "  ✅ Added OpenCode plugin: .opencode/plugins/traffic-status.js"
fi

# Clean legacy project file
rm -f "$TARGET_DIR/.opencode/plugins/traffic-light.js"

echo ""
echo "✨ Project installation complete!"
echo "Scope: Project-only ($TARGET_DIR)"
echo "You can now use /traffic on, /traffic off, etc. inside this project."
