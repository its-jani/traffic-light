# Traffic Light Command (/traffic)

When `/traffic` is invoked, execute the corresponding action immediately in 1 step. **Do not search the repository, do not inspect source files, and do not run codebase searches.**

## Actions:

### 1. `/traffic on` (or `/traffic`)
Run this single command immediately in the background shell:
```bash
curl -s -X POST http://127.0.0.1:8765/session/on -H "Content-Type: application/json" -d "{\"session_id\":\"active-session\",\"label\":\"OpenCode\",\"state\":\"green\"}" || traffic-light
```
Output: `Traffic Light activated (Green) 🟢`

### 2. `/traffic off`
Run:
```bash
curl -s -X POST http://127.0.0.1:8765/session/off -H "Content-Type: application/json" -d "{\"session_id\":\"active-session\"}"
```
Output: `Traffic Light dismissed ⚪`

### 3. State Updates:
- `/traffic yellow [message]` -> `curl -s -X POST http://127.0.0.1:8765/state -H "Content-Type: application/json" -d "{\"session_id\":\"active-session\",\"state\":\"yellow\",\"message\":\"$ARG\"}"`
- `/traffic green [message]` -> `curl -s -X POST http://127.0.0.1:8765/state -H "Content-Type: application/json" -d "{\"session_id\":\"active-session\",\"state\":\"green\",\"message\":\"$ARG\"}"`
- `/traffic red [message]` -> `curl -s -X POST http://127.0.0.1:8765/state -H "Content-Type: application/json" -d "{\"session_id\":\"active-session\",\"state\":\"red\",\"message\":\"$ARG\"}"`

### Automatic Lifecycle Rule:
- When thinking, compiling, or executing tools in this session: send `state=yellow`.
- When an error occurs or waiting for user confirmation: send `state=red`.
- When task is completed and waiting for user prompt: send `state=green`.

