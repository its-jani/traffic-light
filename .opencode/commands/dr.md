# Traffic Light Alias (/dr)

Alias for `/traffic`. Execute the corresponding curl action immediately in 1 step without reading source files or repository searches.

- `/dr on`: `curl -s -X POST http://127.0.0.1:8765/session/on -H "Content-Type: application/json" -d "{\"session_id\":\"active-session\",\"label\":\"OpenCode\",\"state\":\"green\"}"`
- `/dr off`: `curl -s -X POST http://127.0.0.1:8765/session/off -H "Content-Type: application/json" -d "{\"session_id\":\"active-session\"}"`
- `/dr yellow`: `curl -s -X POST http://127.0.0.1:8765/state -H "Content-Type: application/json" -d "{\"session_id\":\"active-session\",\"state\":\"yellow\"}"`
- `/dr green`: `curl -s -X POST http://127.0.0.1:8765/state -H "Content-Type: application/json" -d "{\"session_id\":\"active-session\",\"state\":\"green\"}"`
- `/dr red`: `curl -s -X POST http://127.0.0.1:8765/state -H "Content-Type: application/json" -d "{\"session_id\":\"active-session\",\"state\":\"red\"}"`

