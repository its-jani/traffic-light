# Traffic Light Controller (/traffic)

Floating desktop traffic light widget that **automatically** tracks your AI agent lifecycle in real time.

## User Commands
- `/traffic on [label]` — Attach / start traffic light widget for this session (Defaults to 🟢 Green)
- `/traffic off` — Dismiss traffic light for this session

## Automatic State Transitions
You do **not** need to change colors manually. The agent and hooks manage the traffic light automatically:
- 🟡 **Yellow (Pulse / Working)**: Automatically turns Yellow when AI is thinking, executing commands, or generating code.
- 🔴 **Red (Action Required)**: Automatically turns Red when AI needs user input, confirmation, or hits a blocking error.
- 🟢 **Green (Ready / Idle)**: Automatically turns Green when the task finishes and the AI is ready for your next prompt.

## Background API (For Hooks / Integrations)
```bash
# State updates sent automatically by lifecycle hooks:
curl -s -X POST http://127.0.0.1:8765/state \
  -H "Content-Type: application/json" \
  -d '{"session_id":"active-session","state":"yellow","message":"Generating code..."}'
```
