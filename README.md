# 🚦 traffic-status

> Minimalist floating desktop traffic status widget for AI coding agents (**Claude Code**, **OpenCode**).

Sitting **always-on-top** on your desktop, `traffic-status` provides immediate peripheral awareness of your AI agent's state without switching windows or reading terminal logs.

```
       ┌───────┐
       │ ⠿  — ✕│  <-- Drag anywhere / Minimize / Close
       │  (🔴) │  <-- Red: Needs Input / Confirmation / Error
       │  (🟡) │  <-- Yellow: Thinking / Generating / Tool Execution
       │  (🟢) │  <-- Green: Ready / Prompt Idle / Completed
       └───────┘
```

<!-- Screenshot Placeholder: Take a screenshot of the traffic-status widget floating over your code editor showing active agent session tabs and place it here as assets/screenshot.png -->

---

## ⚡ Installation

### Route A: npm / npx (Requires Node.js 18+)

Install globally with a single command (no Rust toolchain required):

```bash
npm i -g traffic-status && traffic-status install --global
```

Or run via `npx` (which copies the standalone binary to your local user directory):

```bash
npx traffic-status install --global
```

### Route B: Standalone Shell Scripts (Node-Free)

Downloads the prebuilt native binary from GitHub Releases with SHA-256 verification and runs global configuration.

**Windows (PowerShell):**
```powershell
irm https://github.com/its-jani/traffic-status/releases/latest/download/install.ps1 | iex
```

**macOS / Linux (Bash):**
```bash
curl -fsSL https://github.com/its-jani/traffic-status/releases/latest/download/install.sh | sh
```

---

## 🚀 Quick Start

Once installed, use `/traffic` in your AI coding assistant:

- **`/traffic on`** (or `/traffic`):
  Starts the background status daemon (if not already running) and activates the session indicator in 🟢 **Green** (Ready).
- **Automatic State Transitions**:
  - 🟡 **Yellow**: When processing, generating tokens, running tools, or executing commands.
  - 🔴 **Red**: When requiring user permission/confirmation, encountering errors, or blocked.
  - 🟢 **Green**: When the task completes and waits for your next input.
- **`/traffic off`**:
  Dismisses the active session from the widget.

---

## 🤖 Supported Agents

`traffic-status` provides first-party integration hooks for:

1. **Claude Code**: Native slash commands in `.claude/commands/traffic.md`.
2. **OpenCode**: Native slash command in `.opencode/commands/traffic.md` and lifecycle plugin in `.opencode/plugins/traffic-status.js`.

*For other agents (e.g. Aider, custom scripts, terminal loops), control the status directly via the [Native CLI](#-cli-reference) or [REST API](#-rest-api-reference).*

---

## 📦 Installation Scopes & File Safety

`traffic-status` supports both global (machine-wide) and per-project installation scopes:

| Scope | Command | Files Created / Managed |
| --- | --- | --- |
| **Global Scope** | `traffic-status install --global` | **Claude**: `~/.claude/commands/traffic.md`<br>**OpenCode**: `~/.config/opencode/` & `~/.opencode/` (commands & plugins)<br>**Binary**: `%LOCALAPPDATA%\traffic-status\bin\` (Windows) or `~/.local/share/traffic-status/bin/` (Unix) |
| **Project Scope** | `traffic-status install --project [PATH]` | `<project>/.claude/commands/traffic.md`<br>`<project>/.opencode/commands/traffic.md`<br>`<project>/.opencode/plugins/traffic-status.js` |

### Uninstallation Guarantee
- `traffic-status uninstall --global` and `traffic-status uninstall --project` **only** delete files containing the official `traffic-status` marker header (`<!-- generated-by: traffic-status -->` / `// generated-by: traffic-status`).
- It **never** wipes entire directories (such as `~/.config/opencode` or `~/.claude`), preserving all your custom configurations, settings, and other plugins.
- Use `--dry-run` to preview all file actions before executing.

---

## 💻 CLI Reference

```bash
# Launch GUI daemon
traffic-status

# Global & Project Setup
traffic-status install --global               # Install globally
traffic-status install --project ./my-project # Install into specific project
traffic-status uninstall --global             # Remove global hooks & binary
traffic-status uninstall --project ./my-proj  # Remove from specific project

# Diagnostics
traffic-status doctor                         # Inspect PATH, port, daemon, and hooks

# Direct Status Control
traffic-status on [LABEL]                     # Start/show session (default: CLI-Session)
traffic-status yellow [MESSAGE]               # Set state to Yellow (Working)
traffic-status green [MESSAGE]                # Set state to Green (Ready)
traffic-status red [MESSAGE]                  # Set state to Red (Blocked / Error)
traffic-status off                            # Dismiss session
traffic-status clear                          # Clear all active sessions

# Flags
traffic-status --help                         # Show usage documentation
traffic-status --version                      # Show current version
--dry-run                                     # Preview installer actions without writing files
--force, -f                                   # Overwrite files missing standard marker header
```

### Environment Variables
- `TRAFFIC_STATUS_PORT`: Custom port for the IPC HTTP server (default: `8765`).

---

## 📡 REST API Reference

The background daemon listens on `http://127.0.0.1:8765`. All state-mutating endpoints require `POST` with `Content-Type: application/json`.

### 1. Health & Ping
```http
GET /ping
```
Response:
```json
{"status":"ok","app":"traffic-status","version":"0.1.0"}
```

### 2. Update Status Light
```http
POST /state
Content-Type: application/json

{
  "session_id": "claude-1",
  "state": "yellow",
  "message": "Running integration tests..."
}
```

### 3. Session Lifecycle
```http
POST /session/on
Content-Type: application/json

{
  "session_id": "opencode-1",
  "label": "OpenCode",
  "state": "green"
}
```

```http
POST /session/off
Content-Type: application/json

{
  "session_id": "opencode-1"
}
```

```http
POST /clear
Content-Type: application/json

{}
```

---

## 🛡️ Security Architecture & Localhost Boundary

- **Strict Loopback Binding**: The REST IPC server binds exclusively to `127.0.0.1` and does not accept remote network traffic.
- **CSRF & Drive-By Protection**: All state mutations strictly require `POST` and `Content-Type: application/json`. `GET` requests to mutating endpoints are rejected with `405 Method Not Allowed`, preventing malicious websites in your browser from altering your widget state via simple requests or image tags.
- **Buffer Safety**: Request bodies are capped at 64 KB, and malformed JSON payloads return `400 Bad Request` without terminating or panicking the daemon.
- **Zero Telemetry**: `traffic-status` contains zero telemetry, tracking, or network calls to external servers.

---

## 🖥️ Platform Notes

### Windows
- **SmartScreen**: Unsigned binaries downloaded directly may trigger Windows SmartScreen. The npm route (`npm i -g traffic-status`) runs via Node launcher and is less prone to binary warnings *(untested across all Windows Defender configurations)*.

### macOS
- **Gatekeeper / Quarantine**: If downloading the standalone archive manually via browser, you may need to clear the quarantine attribute:
  ```bash
  xattr -d com.apple.quarantine ~/.local/share/traffic-status/bin/traffic-status
  ```

### Linux
- **Required System Libraries**:
  Building or running `egui` on Linux requires X11/GL development libraries:
  ```bash
  sudo apt-get install libxkbcommon-dev libx11-dev libxcb1-dev libgl1-mesa-dev libssl-dev
  ```
- **Wayland Limitations**:
  The Wayland protocol intentionally restricts client-side window positioning and always-on-top capabilities. If your Wayland compositor ignores always-on-top, run with the X11 backend:
  ```bash
  WINIT_UNIX_BACKEND=x11 traffic-status
  ```

---

## 🔧 Troubleshooting

| Issue | Cause | Solution |
| --- | --- | --- |
| `Port 8765 occupied` | Another process is using port 8765 | Set `TRAFFIC_STATUS_PORT=8766` in your environment or terminate the competing service. |
| `traffic-status: command not found` | Binary directory not in `PATH` | Run `traffic-status doctor`. Ensure `%LOCALAPPDATA%\traffic-status\bin` (Windows) or `~/.local/share/traffic-status/bin` (Linux/macOS) is in your system `PATH`. |
| Hooks not updating light | Daemon not running | Triggering `/traffic on` or `traffic-status on` automatically spawns the background daemon. |

---

## 🔄 Migration from `traffic-light`

If you used the earlier development version (`traffic-light`):
- `traffic-status install` and `traffic-status uninstall` automatically clean up legacy `traffic-light.js` plugin files and old `traffic-light` binaries from `~/.cargo/bin`.
- The slash command `/traffic` remains identical and now invokes the new `traffic-status` binary.

---

## 🔨 Build from Source

```bash
git clone https://github.com/its-jani/traffic-status.git
cd traffic-status
cargo build --release
./target/release/traffic-status install --global
```

---

## 📜 License

[MIT](LICENSE)
