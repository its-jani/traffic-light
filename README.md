# 🚦 Traffic Light

> A minimalist, non-Electron, Rust-based floating desktop traffic light for terminal-based AI coding agents (**OpenCode**, **Claude Code**, **Gemini CLI**, **Aider**, etc.).

Sitting **always-on-top** in a transparent, frameless floating widget, **Traffic Light** gives you instant peripheral awareness when your AI agent is thinking, requires confirmation/input, or has completed its work—even while you are in your browser or another app.

---

## ✨ Features

- **⚡ Near-Zero CPU Overhead**: Powered by native Rust (`egui` + `eframe` with Glow/Winit backend). Absolutely zero Electron bloat, idling at 0.0% CPU when sessions are static.
- **🪟 Sleek Glassmorphism Widget**: Frameless, transparent background, always-on-top, draggable anywhere on screen.
- **🚥 3 Clear Traffic States**:
  - 🟢 **Green**: Task completed / Idle / Ready for next prompt.
  - 🟡 **Yellow**: Thinking / Generating / Working (with gentle breathing glow pulse).
  - 🔴 **Red**: Error / Halted / Requires user input or feedback.
- **🔀 Multi-Session Support**: Maintains separate traffic lights for parallel terminal sessions or subagents with individual labels, elapsed timers, and dismiss actions.
- **🔌 Universal IPC Backend**: Async `tokio` TCP/HTTP server on port `8765`. Trigger state changes with standard `curl`, `nc`, PowerShell `Invoke-RestMethod`, or native shell wrappers.
- **🔄 Layout Options**: Easily toggle between **Horizontal** (↔) and **Vertical** (↕) traffic light arrangements and **Compact** (⇱/⇲) mode.

---

## 🚀 Quick Start

### 1. Build and Run

```bash
# Build release binary
cargo build --release

# Run the Traffic Light desktop daemon
cargo run --release
```

The floating traffic light widget will appear on your desktop, listening on `http://127.0.0.1:8765`.

---

## 🛠️ CLI & Agent Integration

### Bash / Zsh Integration

Source the provided `scripts/traffic.sh` in your `~/.bashrc` or `~/.zshrc`:

```bash
source /path/to/traffic-light/scripts/traffic.sh
```

#### Slash Commands & CLI Commands:

```bash
# 1. Register a new agent session
traffic on my-session "Claude Code #1"

# 2. Update states
traffic yellow "Generating code..."    # Yellow: Thinking / Working
traffic green "Task completed"        # Green: Ready for input
traffic red "Needs confirmation"      # Red: Halted / Requires feedback

# 3. Automatically wrap any long-running command
traffic wrap pytest tests/

# 4. Dismiss session
traffic off my-session
```

---

### PowerShell Integration (Windows)

Dot-source `scripts/traffic.ps1` in your PowerShell profile:

```powershell
. .\scripts\traffic.ps1

# Examples:
traffic on "backend-agent" "API Refactor"
traffic yellow "Running migration..."
traffic green "Done"
traffic red "Confirmation needed"
traffic off
```

---

## 📡 Direct HTTP / REST API (curl / nc)

You can send state updates from any programming language, webhook, or CLI tool:

### Update State (POST JSON)
```bash
curl -X POST http://127.0.0.1:8765/state \
  -H "Content-Type: application/json" \
  -d '{"session_id":"s1","state":"yellow","message":"Generating unit tests..."}'
```

### Update State (GET Query)
```bash
curl "http://127.0.0.1:8765/state?session_id=s1&state=green"
```

### Register Session
```bash
curl -X POST http://127.0.0.1:8765/session/on \
  -H "Content-Type: application/json" \
  -d '{"session_id":"s1","label":"OpenCode: Auth Feature","state":"green"}'
```

### Dismiss Session
```bash
curl -X POST http://127.0.0.1:8765/session/off \
  -H "Content-Type: application/json" \
  -d '{"session_id":"s1"}'
```

### Raw TCP (netcat)
```bash
echo '{"session_id":"s1","state":"yellow"}' | nc 127.0.0.1 8765
```

---

## 🤖 Hooking into Terminal AI Agents

### Claude Code / Gemini CLI / OpenCode Lifecycle Hook Example

Add this pattern to your agent's command wrapper or shell prompt hook:

```bash
# Before agent runs prompt:
curl -s -X POST http://127.0.0.1:8765/state \
  -d '{"session_id":"agent-main","state":"yellow","message":"Agent thinking..."}' >/dev/null 2>&1

# On user confirmation required / tool approval:
curl -s -X POST http://127.0.0.1:8765/state \
  -d '{"session_id":"agent-main","state":"red","message":"Waiting for approval"}' >/dev/null 2>&1

# When agent finishes and prompt is idle:
curl -s -X POST http://127.0.0.1:8765/state \
  -d '{"session_id":"agent-main","state":"green","message":"Ready for next prompt"}' >/dev/null 2>&1
```

---

## 📂 Project Architecture

```
traffic-light/
├── Cargo.toml          # Minimal Rust dependencies (eframe, tokio, serde)
├── src/
│   ├── main.rs         # Daemon entry point & thread coordinator
│   ├── types.rs        # State models (LightState, SessionInfo, IpcCommand)
│   ├── ipc.rs          # Tokio async TCP/HTTP REST IPC server
│   └── app.rs          # Eframe/egui GUI renderer with glassmorphism & glow
└── scripts/
    ├── traffic.sh      # Bash/Zsh CLI integration & slash command handler
    └── traffic.ps1     # PowerShell CLI integration
```

---

## 📜 License

MIT
