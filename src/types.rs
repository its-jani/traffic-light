use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LightState {
    Green,
    Yellow,
    Red,
    Off,
}

impl LightState {
    pub fn parse_str(s: &str) -> Option<Self> {
        match s.to_lowercase().trim() {
            "green" | "idle" | "ready" | "done" | "success" | "ok" => Some(Self::Green),
            "yellow" | "working" | "thinking" | "generating" | "busy" | "running" => Some(Self::Yellow),
            "red" | "error" | "halted" | "needs_input" | "input" | "blocked" | "failed" => Some(Self::Red),
            "off" | "dim" => Some(Self::Off),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Green => "Ready / Done",
            Self::Yellow => "Working / Thinking",
            Self::Red => "Needs Input / Error",
            Self::Off => "Standby",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub label: String,
    pub state: LightState,
    pub message: Option<String>,
    pub last_updated: Instant,
}

impl SessionInfo {
    pub fn new(id: String, label: Option<String>, initial_state: Option<LightState>) -> Self {
        let display_label = label.unwrap_or_else(|| {
            if id.starts_with("session-") || id.len() <= 12 {
                id.clone()
            } else {
                format!("Agent {}", &id[..std::cmp::min(8, id.len())])
            }
        });
        Self {
            id,
            label: display_label,
            state: initial_state.unwrap_or(LightState::Green),
            message: None,
            last_updated: Instant::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcPayload {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub action: Option<String>, // "on", "off", "set", "clear", "ping"
}

#[derive(Debug, Clone)]
pub enum IpcCommand {
    SetState {
        session_id: String,
        state: LightState,
        label: Option<String>,
        message: Option<String>,
    },
    SessionOn {
        session_id: String,
        label: Option<String>,
        initial_state: Option<LightState>,
    },
    SessionOff {
        session_id: String,
    },
    ClearAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetOrientation {
    Horizontal,
    Vertical,
}
