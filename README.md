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

## 📁 Project Structure

```
traffic-light/
├── .claude/
│   └── commands/
│       └── traffic.md            # Claude Code slash command (/traffic)
├── .opencode/
│   ├── commands/
│   │   └── traffic.md            # OpenCode slash command (/traffic)
│   └── plugins/
│       └── traffic-light.js      # OpenCode automatic lifecycle plugin
├── scripts/
│   ├── install-project.ps1       # Install traffic light to a specific project (PowerShell)
│   ├── install-project.sh        # Install traffic light to a specific project (Bash)
│   ├── install-global.ps1        # Install traffic light globally across machine (PowerShell)
│   ├── install-global.sh         # Install traffic light globally across machine (Bash)
│   ├── uninstall-project.ps1     # Remove traffic light from a specific project (PowerShell)
│   ├── uninstall-project.sh      # Remove traffic light from a specific project (Bash)
│   ├── uninstall-global.ps1      # Completely purge traffic light from entire laptop (PowerShell)
│   ├── uninstall-global.sh       # Completely purge traffic light from entire laptop (Bash)
│   ├── traffic.ps1               # Interactive CLI helper for Windows PowerShell
│   └── traffic.sh                # Interactive CLI helper for Linux/macOS
├── src/
│   ├── main.rs                   # App entrypoint & CLI dispatcher
│   ├── cli.rs                    # Native CLI installer & subcommand handler
│   ├── app.rs                    # UI layout, eframe/egui rendering, tab bar & physics
│   ├── ipc.rs                    # Tokio HTTP REST server (port 8765)
│   └── types.rs                  # Session, state, and IPC command data models
├── Cargo.toml                    # Rust crate configuration
└── README.md                     # Documentation
```

---

## ⚡ Zero-Install / On-Demand Usage

You **do not need to manually install anything** if you already have the repository or pre-built binary. Claude Code and OpenCode can trigger and auto-start the traffic light on demand via slash commands:

### In Claude Code or OpenCode:
- Type **`/traffic on`** (or `/traffic`):
  - Automatically starts the floating traffic light daemon in the background (if not already running).
  - Registers the session and turns the indicator 🟢 **Green** (Ready).
- **Automatic Transitions**:
  - 🟡 **Yellow**: When processing, thinking, compiling, or invoking tools.
  - 🔴 **Red**: When requiring user permission, confirmation, or when a tool encounters an error.
  - 🟢 **Green**: When task finishes and waits for your next prompt.
- Type **`/traffic off`**:
  - Dismisses the session from the widget.

---

## 🛠️ Installation Commands

### 1. Install for a Single Project Only (Project Scope)
Installs `/traffic` commands and automated hooks strictly inside one specific project. Only sessions inside that project will see the traffic light.

**PowerShell (Windows):**
```powershell
# In project folder:
.\scripts\install-project.ps1

# Or target a specific folder:
.\scripts\install-project.ps1 -ProjectPath "C:\path\to\your\project"
```

**Bash / macOS / Linux:**
```bash
# In project folder:
./scripts/install-project.sh

# Or target a specific folder:
./scripts/install-project.sh /path/to/your/project
```

---

### 2. Install Globally (Entire Laptop / All Projects)
Installs the native binary to your PATH (`~/.cargo/bin`) and configures global Claude Code (`~/.claude/commands`) and OpenCode (`~/.config/opencode`) settings so `/traffic` works in **any project, now and in the future**.

**PowerShell (Windows):**
```powershell
.\scripts\install-global.ps1
```

**Bash / macOS / Linux:**
```bash
./scripts/install-global.sh
```

---

## 🗑️ Uninstallation Commands

### 1. Delete from a Specific Project
Removes the Traffic Light commands and plugin from that specific project without affecting any other projects or global configs.

**PowerShell (Windows):**
```powershell
.\scripts\uninstall-project.ps1

# Or target a specific folder:
.\scripts\uninstall-project.ps1 -ProjectPath "C:\path\to\your\project"
```

**Bash / macOS / Linux:**
```bash
./scripts/uninstall-project.sh /path/to/your/project
```

---

### 2. Delete Entirely from the Laptop (Complete Purge)
Completely removes Traffic Light from the entire machine:
- Stops all running background daemons.
- Removes binary from `~/.cargo/bin`.
- Deletes global Claude Code commands (`~/.claude/commands/traffic.md`).
- Deletes global OpenCode plugins & commands (`~/.config/opencode/`, `~/.opencode/`).
- Leaves **zero traces** on your system.

**PowerShell (Windows):**
```powershell
.\scripts\uninstall-global.ps1
```

**Bash / macOS / Linux:**
```bash
./scripts/uninstall-global.sh
```

---

## 💻 Native CLI Commands (Compiled Binary)

If you have built `traffic-light`:

```bash
# Launch GUI daemon
traffic-light

# Install commands
traffic-light install --project ./my-app    # Project scope
traffic-light install --global             # Global scope

# Uninstall commands
traffic-light uninstall --project ./my-app  # Project scope
traffic-light uninstall --global            # Machine-wide purge

# Control light state directly
traffic-light on "My Session"
traffic-light yellow "Generating code..."
traffic-light green "Ready"
traffic-light red "Error occurred"
traffic-light off
```

---

## 📡 REST API (curl / Webhooks)

```bash
# State update
curl -X POST http://127.0.0.1:8765/state \
  -H "Content-Type: application/json" \
  -d '{"session_id":"agent-1","state":"yellow","message":"Generating unit tests..."}'

# Session on/off
curl -X POST http://127.0.0.1:8765/session/on -d '{"session_id":"agent-1","label":"Claude","state":"green"}'
curl -X POST http://127.0.0.1:8765/session/off -d '{"session_id":"agent-1"}'
```

---

## 📤 Git & GitHub Push Commands

To commit and push all changes to GitHub:

```bash
# 1. Check status
git status

# 2. Stage all files
git add .

# 3. Commit with a clear message
git commit -m "feat: add project & global install/uninstall scripts and zero-install slash commands"

# 4. Push to remote GitHub repository
git push origin main
```

---

## 📜 License

MIT
