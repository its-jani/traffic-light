# 🚦 Traffic Light

> A minimalist, vertical floating desktop traffic light for AI coding agents (**OpenCode**, **Claude Code**, **Gemini CLI**, **Aider**).

Sitting **always-on-top** on your screen, **Traffic Light** gives you instant peripheral awareness of your AI agent's state without switching windows or reading terminal logs.

```
       ┌───────┐
       │ ⠿  — ✕│  <-- Drag anywhere / Minimize / Close
       │  (🔴) │  <-- Red: Needs Input / Error / Halted
       │  (🟡) │  <-- Yellow: Thinking / Working (Breathing Pulse)
       │  (🟢) │  <-- Green: Ready / Done / Idle
       └───────┘
```

---

## ⚡ Key Features

- **🚥 Authentic Vertical Housing**: Sleek, narrow physical traffic light with hood visors and glowing neon halos.
- **📌 Always-on-Top & Draggable**: Floats over all windows (VS Code, browser, terminal) and can be dragged anywhere on screen.
- **📑 Multi-Session Vertical Tabs**: Run multiple agents simultaneously and switch between numbered tabs (`1`, `2`...) with one click.
- **🔌 Zero Configuration IPC**: Listens on `http://127.0.0.1:8765` for instant state updates.
- **⚡ 0.0% Idle CPU**: Native Rust (`egui`/`eframe`) with Glow backend — no Electron bloat.

---

## 🚀 1-Step Installation & Running

### Build and Run Daemon (24/7 Background)

```bash
# Build & run directly
cargo run --release
```

To install globally as a native binary:
```bash
cargo install --path .
traffic-light
```

---

## 💬 Hands-Free Workflow & Commands

### Simple User Commands
You only ever need two commands:
- `/traffic on` (or `/dr on`) — Attach/show the traffic light for your current session.
- `/traffic off` (or `/dr off`) — Dismiss/hide the traffic light session.

### 🤖 100% Automatic Color Transitions
You never need to toggle colors manually. The widget transitions automatically in the background:
- 🟢 **Green (Idle / Ready)**: Session is active and waiting for your prompt.
- 🟡 **Yellow (Breathing Pulse / Working)**: Automatically triggers when the AI is processing, thinking, or running tools.
- 🔴 **Red (Action Required)**: Automatically turns Red if the background session requires user input, confirmation, or encounters an error.
- 🟢 **Green (Done)**: Automatically switches back to Green when the task is completed and ready for the next prompt.

### 🔧 CLI & Hook Integration
**PowerShell (Windows):**
```powershell
. .\scripts\traffic.ps1
```

**Bash / Zsh (Linux / macOS):**
```bash
source scripts/traffic.sh
```

---

## 📡 REST API (curl / Webhooks)

```bash
# Set light state
curl -X POST http://127.0.0.1:8765/state \
  -H "Content-Type: application/json" \
  -d '{"session_id":"agent-1","state":"yellow","message":"Generating unit tests..."}'

# Register / Dismiss session
curl -X POST http://127.0.0.1:8765/session/on -d '{"session_id":"agent-2","label":"Claude #2"}'
curl -X POST http://127.0.0.1:8765/session/off -d '{"session_id":"agent-2"}'
```

---

## 📜 License

MIT
