# Traffic Light Slash Command Alias (/dr)

Execute the corresponding action immediately in 1 step without reading source files or repository searches.

- **`/dr on`**: Run `powershell -NoProfile -Command "if (-not (Get-Process traffic-light -ErrorAction SilentlyContinue)) { Start-Process traffic-light; Start-Sleep -Milliseconds 400 }; curl.exe -s 'http://127.0.0.1:8765/session/on?session_id=claude-session&label=Claude&state=green'"`
- **`/dr yellow [msg]`**: Run `curl.exe -s "http://127.0.0.1:8765/state?session_id=claude-session&state=yellow&message=$ARG"`
- **`/dr green [msg]`**: Run `curl.exe -s "http://127.0.0.1:8765/state?session_id=claude-session&state=green&message=$ARG"`
- **`/dr red [msg]`**: Run `curl.exe -s "http://127.0.0.1:8765/state?session_id=claude-session&state=red&message=$ARG"`
- **`/dr off`**: Run `curl.exe -s "http://127.0.0.1:8765/session/off?session_id=claude-session"`

