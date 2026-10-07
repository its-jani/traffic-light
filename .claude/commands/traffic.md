# Traffic Light Slash Command (/traffic)

Execute the corresponding action immediately in 1 step without reading source files or repository searches.

### Zero-Install Auto-Start & Control:

- **`/traffic on`** (or `/traffic`):
  Starts the floating Traffic Light daemon (if not already running) and registers the active session in Green state.
  ```powershell
  powershell -NoProfile -Command "$proc = Get-Process traffic-light -ErrorAction SilentlyContinue; if (-not $proc) { $candidates = @('traffic-light', (Join-Path $env:USERPROFILE '.cargo\bin\traffic-light.exe'), '.\target\release\traffic-light.exe', 'd:\traffic-light\target\release\traffic-light.exe'); foreach ($b in $candidates) { if ((Get-Command $b -ErrorAction SilentlyContinue) -or (Test-Path $b)) { Start-Process $b -WindowStyle Hidden; Start-Sleep -Milliseconds 450; break } } }; curl.exe -s 'http://127.0.0.1:8765/session/on?session_id=claude-session&label=Claude&state=green'"
  ```

- **`/traffic yellow [msg]`** (or when processing/thinking/running tools):
  ```bash
  curl.exe -s "http://127.0.0.1:8765/state?session_id=claude-session&state=yellow&message=$ARG"
  ```

- **`/traffic green [msg]`** (or when ready/waiting for prompt):
  ```bash
  curl.exe -s "http://127.0.0.1:8765/state?session_id=claude-session&state=green&message=$ARG"
  ```

- **`/traffic red [msg]`** (or when error/needs confirmation):
  ```bash
  curl.exe -s "http://127.0.0.1:8765/state?session_id=claude-session&state=red&message=$ARG"
  ```

- **`/traffic off`**:
  ```bash
  curl.exe -s "http://127.0.0.1:8765/session/off?session_id=claude-session"
  ```
