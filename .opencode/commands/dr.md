# Traffic Light Alias (/dr)

Alias for `/traffic`. Execute the corresponding action immediately in 1 step without reading source files or repository searches.

- `/dr on`: `powershell -NoProfile -Command "if (-not (Get-Process traffic-light -ErrorAction SilentlyContinue)) { Start-Process traffic-light; Start-Sleep -Milliseconds 400 }; curl.exe -s 'http://127.0.0.1:8765/session/on?session_id=active-session&label=OpenCode&state=green'"`
- `/dr off`: `curl.exe -s "http://127.0.0.1:8765/session/off?session_id=active-session"`
- `/dr yellow`: `curl.exe -s "http://127.0.0.1:8765/state?session_id=active-session&state=yellow"`
- `/dr green`: `curl.exe -s "http://127.0.0.1:8765/state?session_id=active-session&state=green"`
- `/dr red`: `curl.exe -s "http://127.0.0.1:8765/state?session_id=active-session&state=red"`

