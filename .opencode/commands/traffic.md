<!-- generated-by: traffic-status -->
# Traffic Status Command (/traffic)

When `/traffic` is invoked in OpenCode, execute the corresponding action immediately in 1 step without searching the codebase or reading project files.

### Binary Resolution:
Locate and execute the `traffic-status` binary in this order:
1. `traffic-status` (if on PATH)
2. `%LOCALAPPDATA%\traffic-status\bin\traffic-status.exe` (Windows) or `~/.local/share/traffic-status/bin/traffic-status` (macOS/Linux)
3. `npx -y traffic-status`

### Commands:
- `/traffic on` -> `traffic-status on "OpenCode"`
- `/traffic off` -> `traffic-status off`
- `/traffic yellow [message]` -> `traffic-status yellow "$ARG"`
- `/traffic green [message]` -> `traffic-status green "$ARG"`
- `/traffic red [message]` -> `traffic-status red "$ARG"`

### Automatic Lifecycle Rule:
- When thinking, compiling, or executing tools: send `yellow`.
- When an error occurs or waiting for user confirmation: send `red`.
- When task is completed and waiting for user prompt: send `green`.
