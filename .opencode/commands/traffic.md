# Traffic Light Command (/traffic)

When `/traffic` is invoked in OpenCode, execute the corresponding action immediately in 1 step. **Do not search the repository, do not inspect source files, and do not run codebase searches.**

## Zero-Install Actions:

### 1. `/traffic on` (or `/traffic`)
Auto-starts the floating widget daemon in the background if not running, and registers this session:
```powershell
powershell -NoProfile -Command "$proc = Get-Process traffic-light -ErrorAction SilentlyContinue; if (-not $proc) { $candidates = @('traffic-light', (Join-Path $env:USERPROFILE '.cargo\bin\traffic-light.exe'), '.\target\release\traffic-light.exe', 'd:\traffic-light\target\release\traffic-light.exe'); foreach ($b in $candidates) { if ((Get-Command $b -ErrorAction SilentlyContinue) -or (Test-Path $b)) { Start-Process $b -WindowStyle Hidden; Start-Sleep -Milliseconds 450; break } } }; curl.exe -s 'http://127.0.0.1:8765/session/on?session_id=opencode-session&label=OpenCode&state=green'"
```
Output: `Traffic Light activated (Green) 🟢`

### 2. `/traffic off`
Dismisses this session from the widget:
```bash
curl.exe -s "http://127.0.0.1:8765/session/off?session_id=opencode-session"
```
Output: `Traffic Light dismissed ⚪`

### 3. State Updates:
- `/traffic yellow [message]` -> `curl.exe -s "http://127.0.0.1:8765/state?session_id=opencode-session&state=yellow&message=$ARG"`
- `/traffic green [message]` -> `curl.exe -s "http://127.0.0.1:8765/state?session_id=opencode-session&state=green&message=$ARG"`
- `/traffic red [message]` -> `curl.exe -s "http://127.0.0.1:8765/state?session_id=opencode-session&state=red&message=$ARG"`

### 4. Automatic Lifecycle Rule:
- When thinking, compiling, or executing tools: send `state=yellow`.
- When an error occurs or waiting for user confirmation: send `state=red`.
- When task is completed and waiting for user prompt: send `state=green`.
