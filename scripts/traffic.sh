#!/usr/bin/env bash
# ==============================================================================
# Traffic Light CLI Integration Script
# Compatible with Bash / Zsh / POSIX shells
# Supports OpenCode, Claude Code, Gemini CLI, Aider, and custom agent workflows
# ==============================================================================

TRAFFIC_LIGHT_HOST="${TRAFFIC_LIGHT_HOST:-127.0.0.1}"
TRAFFIC_LIGHT_PORT="${TRAFFIC_LIGHT_PORT:-8765}"
TRAFFIC_BASE_URL="http://${TRAFFIC_LIGHT_HOST}:${TRAFFIC_LIGHT_PORT}"

# Default session ID for current shell instance
if [ -z "$TRAFFIC_SESSION_ID" ]; then
    export TRAFFIC_SESSION_ID="session-$$-$(date +%s | tail -c 5)"
fi

# Send HTTP/JSON payload or fallback to raw TCP
_traffic_send() {
    local endpoint="$1"
    local json_payload="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -s -m 1 -X POST "${TRAFFIC_BASE_URL}${endpoint}" \
            -H "Content-Type: application/json" \
            -d "$json_payload" >/dev/null 2>&1 || true
    elif command -v nc >/dev/null 2>&1; then
        echo "$json_payload" | nc -w 1 "$TRAFFIC_LIGHT_HOST" "$TRAFFIC_LIGHT_PORT" >/dev/null 2>&1 || true
    fi
}

# --- Core Commands ---

# /traffic on [session_id] [label]
traffic_on() {
    local sid="${1:-$TRAFFIC_SESSION_ID}"
    local label="${2:-Terminal Agent ($$)}"
    _traffic_send "/session/on" "{\"session_id\":\"$sid\",\"label\":\"$label\",\"state\":\"green\"}"
    echo "[Traffic Light] Session '$sid' registered ($label)."
}

# /traffic off [session_id]
traffic_off() {
    local sid="${1:-$TRAFFIC_SESSION_ID}"
    _traffic_send "/session/off" "{\"session_id\":\"$sid\"}"
    echo "[Traffic Light] Session '$sid' dismissed."
}

# /traffic state <state> [message] [session_id]
# state can be: green, yellow, red, off
traffic_state() {
    local state="$1"
    local message="$2"
    local sid="${3:-$TRAFFIC_SESSION_ID}"

    if [ -z "$state" ]; then
        echo "Usage: traffic_state <green|yellow|red> [message] [session_id]"
        return 1
    fi

    if [ -n "$message" ]; then
        _traffic_send "/state" "{\"session_id\":\"$sid\",\"state\":\"$state\",\"message\":\"$message\"}"
    else
        _traffic_send "/state" "{\"session_id\":\"$sid\",\"state\":\"$state\"}"
    fi
}

# --- Quick Convenience State Functions ---

# Yellow: Working / Thinking / Generating
traffic_yellow() {
    local msg="${1:-Working / Thinking...}"
    traffic_state "yellow" "$msg"
}

# Green: Task Done / Ready / Idle
traffic_green() {
    local msg="${1:-Ready for next prompt}"
    traffic_state "green" "$msg"
}

# Red: Error / Needs user input / Halted
traffic_red() {
    local msg="${1:-Needs user input / Halted}"
    traffic_state "red" "$msg"
}

# --- Agent Command Wrapper ---
# Automatically turns light Yellow during command execution,
# Green on success (exit code 0), and Red on error / non-zero exit code.
# Usage: traffic_wrap npm test
traffic_wrap() {
    traffic_yellow "Running: $*"
    "$@"
    local exit_code=$?
    if [ $exit_code -eq 0 ]; then
        traffic_green "Completed successfully"
    else
        traffic_red "Failed (exit code $exit_code)"
    fi
    return $exit_code
}

# --- Slash Command Interceptor ---
traffic() {
    case "$1" in
        "on")
            traffic_on "$2" "$3"
            ;;
        "off")
            traffic_off "$2"
            ;;
        "yellow"|"working"|"thinking")
            traffic_yellow "$2"
            ;;
        "green"|"idle"|"done"|"ready")
            traffic_green "$2"
            ;;
        "red"|"error"|"halt"|"input")
            traffic_red "$2"
            ;;
        "wrap")
            shift
            traffic_wrap "$@"
            ;;
        "clear")
            _traffic_send "/clear" "{}"
            ;;
        *)
            echo "Traffic Light CLI Controller"
            echo "Usage: traffic <on|off|green|yellow|red|wrap|clear> [args...]"
            echo ""
            echo "Examples:"
            echo "  traffic on [session_id] [label]   - Register new traffic light"
            echo "  traffic yellow 'Generating...'    - Set working/thinking state"
            echo "  traffic green 'Done'              - Set ready/done state"
            echo "  traffic red 'Needs confirmation'  - Set input-required/error state"
            echo "  traffic wrap <command...>         - Wrap a command execution"
            echo "  traffic off                       - Dismiss current session"
            ;;
    esac
}

# Support slash prefix: /traffic
alias "/traffic"=traffic
