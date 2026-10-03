# Traffic Light Slash Command Alias (/dr)

Execute the corresponding action immediately in 1 step without reading source files or repository searches.

- **`/dr on`**: Run `curl -s -X POST http://127.0.0.1:8765/session/on -H "Content-Type: application/json" -d "{\"session_id\":\"claude-session\",\"label\":\"Claude\",\"state\":\"green\"}"`
- **`/dr yellow [msg]`**: Run `curl -s -X POST http://127.0.0.1:8765/state -H "Content-Type: application/json" -d "{\"session_id\":\"claude-session\",\"state\":\"yellow\",\"message\":\"$ARG\"}"`
- **`/dr green [msg]`**: Run `curl -s -X POST http://127.0.0.1:8765/state -H "Content-Type: application/json" -d "{\"session_id\":\"claude-session\",\"state\":\"green\",\"message\":\"$ARG\"}"`
- **`/dr red [msg]`**: Run `curl -s -X POST http://127.0.0.1:8765/state -H "Content-Type: application/json" -d "{\"session_id\":\"claude-session\",\"state\":\"red\",\"message\":\"$ARG\"}"`
- **`/dr off`**: Run `curl -s -X POST http://127.0.0.1:8765/session/off -H "Content-Type: application/json" -d "{\"session_id\":\"claude-session\"}"`

