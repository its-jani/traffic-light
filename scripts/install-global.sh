#!/usr/bin/env bash
# ==============================================================================
# Traffic Status - Global System-Wide Installer (Bash / POSIX)
# Installs Traffic Status globally across the entire machine for all current and future projects.
# ==============================================================================
set -e

echo "🚦 Installing Traffic Status Globally (All Projects)..."

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 1. Install native binary to ~/.local/share/traffic-status/bin
STABLE_BIN_DIR="$HOME/.local/share/traffic-status/bin"
mkdir -p "$STABLE_BIN_DIR"

if [ -f "$SOURCE_ROOT/target/release/traffic-status" ]; then
    cp "$SOURCE_ROOT/target/release/traffic-status" "$STABLE_BIN_DIR/traffic-status"
    chmod +x "$STABLE_BIN_DIR/traffic-status"
    echo "  ✅ Installed binary to $STABLE_BIN_DIR/traffic-status"
elif command -v cargo >/dev/null 2>&1; then
    (cd "$SOURCE_ROOT" && cargo build --release)
    cp "$SOURCE_ROOT/target/release/traffic-status" "$STABLE_BIN_DIR/traffic-status"
    chmod +x "$STABLE_BIN_DIR/traffic-status"
    echo "  ✅ Built and installed binary to $STABLE_BIN_DIR/traffic-status"
fi

# 2. Global Claude Code Setup (~/.claude/commands/traffic.md)
echo "🤖 [2/3] Configuring Global Claude Code commands..."
mkdir -p "$HOME/.claude/commands"
cat << 'EOF' > "$HOME/.claude/commands/traffic.md"
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
echo "  ✅ Global Claude Code command installed: $HOME/.claude/commands/traffic.md"

# 3. Global OpenCode Setup (~/.config/opencode & ~/.opencode)
echo "⚡ [3/3] Configuring Global OpenCode plugins & commands..."
for BASE in "$HOME/.config/opencode" "$HOME/.opencode"; do
    mkdir -p "$BASE/commands" "$BASE/plugins"
    cat << 'EOF' > "$BASE/commands/traffic.md"
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
    if [ -f "$SOURCE_ROOT/.opencode/plugins/traffic-status.js" ]; then
        cp "$SOURCE_ROOT/.opencode/plugins/traffic-status.js" "$BASE/plugins/traffic-status.js"
    fi
    # Clean legacy plugin
    rm -f "$BASE/plugins/traffic-light.js"
    echo "  ✅ Global OpenCode plugin & commands installed in: $BASE"
done

# Clean legacy traffic-light binary if present
rm -f "$HOME/.cargo/bin/traffic-light"

echo ""
echo "🎉 Global Installation Complete!"
echo "Scope: Entire Laptop / All Projects (Current and Future)"
echo "You can now run '/traffic on' from any folder or project in Claude Code or OpenCode."
