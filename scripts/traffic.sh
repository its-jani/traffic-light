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

# Ensure background traffic-light process is running
_ensure_daemon() {
    if command -v curl >/dev/null 2>&1; then
        if curl -s -m 1 "${TRAFFIC_BASE_URL}/ping" >/dev/null 2>&1; then
            return 0
        fi
    fi

    # Try starting daemon in background
    if command -v traffic-light >/dev/null 2>&1; then
        nohup traffic-light >/dev/null 2>&1 &
        sleep 0.4
    elif [ -x "$HOME/.cargo/bin/traffic-light" ]; then
        nohup "$HOME/.cargo/bin/traffic-light" >/dev/null 2>&1 &
        sleep 0.4
    fi
}

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

traffic_on() {
    _ensure_daemon
    local sid="${1:-$TRAFFIC_SESSION_ID}"
    local label="${2:-Session ($$)}"
    _traffic_send "/session/on" "{\"session_id\":\"$sid\",\"label\":\"$label\",\"state\":\"green\"}"
    echo "[Traffic Light] Session '$sid' registered ($label)."
}

traffic_off() {
    local sid="${1:-$TRAFFIC_SESSION_ID}"
    _traffic_send "/session/off" "{\"session_id\":\"$sid\"}"
    echo "[Traffic Light] Session '$sid' dismissed."
}

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

traffic_yellow() {
    local msg="${1:-Working / Thinking...}"
    traffic_state "yellow" "$msg"
}

traffic_green() {
    local msg="${1:-Ready for next prompt}"
    traffic_state "green" "$msg"
}

traffic_red() {
    local msg="${1:-Needs user input / Halted}"
    traffic_state "red" "$msg"
}

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
    local script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
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
        "install-project")
            bash "$script_dir/install-project.sh" "$2"
            ;;
        "install-global")
            bash "$script_dir/install-global.sh"
            ;;
        "uninstall-project")
            bash "$script_dir/uninstall-project.sh" "$2"
            ;;
        "uninstall-global")
            bash "$script_dir/uninstall-global.sh"
            ;;
        "wrap")
            shift
            traffic_wrap "$@"
            ;;
        "clear")
            _traffic_send "/clear" "{}"
            ;;
        *)
            echo "🚦 Traffic Light CLI Controller"
            echo "Session Controls:"
            echo "  traffic on                        - Register / show traffic light"
            echo "  traffic off                       - Dismiss current session"
            echo "  traffic green|yellow|red [msg]    - Set current state"
            echo "  traffic wrap <command...>         - Wrap a command execution"
            echo ""
            echo "Management Commands:"
            echo "  traffic install-project [path]    - Install traffic light for specific project"
            echo "  traffic install-global            - Install traffic light globally for all projects"
            echo "  traffic uninstall-project [path]  - Remove traffic light from specific project"
            echo "  traffic uninstall-global          - Completely remove traffic light from laptop"
            ;;
    esac
}

# Support slash prefixes: /traffic and /dr
alias "/traffic"=traffic
alias "dr"=traffic
alias "/dr"=traffic
