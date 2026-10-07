<!-- generated-by: traffic-status -->
# Traffic Status Slash Command (/traffic)

Execute the corresponding action immediately in 1 step without reading source files or repository searches.

### Binary Resolution:
Locate and execute the `traffic-status` binary in this order:
1. `traffic-status` (if on PATH)
2. `%LOCALAPPDATA%\traffic-status\bin\traffic-status.exe` (Windows) or `~/.local/share/traffic-status/bin/traffic-status` (macOS/Linux)
3. `npx -y traffic-status`

### Actions:
- **`/traffic on`** (or `/traffic`):
  Starts the floating status daemon (if not running) and registers the active session.
  ```powershell
  traffic-status on "Claude"
  ```
- **`/traffic yellow [msg]`** (when thinking/running tools):
  ```powershell
  traffic-status yellow "$ARG"
  ```
- **`/traffic green [msg]`** (when ready/waiting for prompt):
  ```powershell
  traffic-status green "$ARG"
  ```
- **`/traffic red [msg]`** (when error/needs confirmation):
  ```powershell
  traffic-status red "$ARG"
  ```
- **`/traffic off`**:
  ```powershell
  traffic-status off
  ```
