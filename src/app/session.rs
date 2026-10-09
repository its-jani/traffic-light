//! Session bookkeeping, decoupled from egui.
//!
//! `SessionStore` owns the session map, tab order and active tab. It applies
//! IPC commands and returns the window-level [`AppEffect`]s the UI should
//! perform, so the state machine can be tested without a window.

use std::collections::HashMap;
use std::time::Instant;

use crate::types::{IpcCommand, LightState, SessionInfo};

/// Window/process side effects the UI must perform after a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEffect {
    Repaint,
    Unminimize,
    FocusWindow,
    CloseWindow,
    ExitProcess,
}

pub struct SessionStore {
    sessions: HashMap<String, SessionInfo>,
    order: Vec<String>,
    active: Option<String>,
}

impl SessionStore {
    /// Fresh store containing the "agent-1" placeholder session.
    pub fn with_default_session() -> Self {
        let id = "agent-1".to_string();
        let mut sessions = HashMap::new();
        sessions.insert(
            id.clone(),
            SessionInfo::new(
                id.clone(),
                Some("Session #1".to_string()),
                Some(LightState::Green),
            ),
        );
        Self {
            sessions,
            order: vec![id.clone()],
            active: Some(id),
        }
    }

    pub fn sessions(&self) -> &HashMap<String, SessionInfo> {
        &self.sessions
    }

    pub fn order(&self) -> &[String] {
        &self.order
    }

    pub fn active_id(&self) -> Option<&str> {
        self.active.as_deref()
    }

    pub fn active_session(&self) -> Option<&SessionInfo> {
        self.active.as_ref().and_then(|id| self.sessions.get(id))
    }

    pub fn select(&mut self, id: &str) {
        self.active = Some(id.to_string());
    }

    /// Fall back to the first tab when the active one is gone.
    pub fn ensure_active(&mut self) {
        if self.active.is_none() {
            self.active = self.order.first().cloned();
        }
    }

    pub fn has_active_animation(&self) -> bool {
        self.sessions
            .values()
            .any(|s| s.state == LightState::Yellow)
    }

    /// Drop the placeholder once a real session shows up.
    fn replace_placeholder(&mut self, session_id: &str) {
        if self.sessions.len() == 1
            && self.sessions.contains_key("agent-1")
            && session_id != "agent-1"
        {
            self.sessions.remove("agent-1");
            self.order.retain(|id| id != "agent-1");
        }
    }

    fn push_order_if_missing(&mut self, session_id: &str) {
        if !self.order.iter().any(|id| id == session_id) {
            self.order.push(session_id.to_string());
        }
    }

    pub fn apply(&mut self, cmd: IpcCommand) -> Vec<AppEffect> {
        match cmd {
            IpcCommand::SetState {
                session_id,
                state,
                label,
                message,
            } => {
                self.replace_placeholder(&session_id);

                if let Some(session) = self.sessions.get_mut(&session_id) {
                    session.state = state;
                    if let Some(lbl) = label {
                        session.label = lbl;
                    }
                    if message.is_some() {
                        session.message = message;
                    }
                    session.last_updated = Instant::now();
                } else {
                    let mut session = SessionInfo::new(session_id.clone(), label, Some(state));
                    session.message = message;
                    self.sessions.insert(session_id.clone(), session);
                    self.push_order_if_missing(&session_id);
                }
                self.active = Some(session_id);
                Vec::new()
            }
            IpcCommand::SessionOn {
                session_id,
                label,
                initial_state,
            } => {
                self.replace_placeholder(&session_id);

                if let Some(session) = self.sessions.get_mut(&session_id) {
                    if let Some(lbl) = label {
                        session.label = lbl;
                    }
                    if let Some(st) = initial_state {
                        session.state = st;
                    }
                    session.last_updated = Instant::now();
                } else {
                    let session = SessionInfo::new(session_id.clone(), label, initial_state);
                    self.sessions.insert(session_id.clone(), session);
                    self.push_order_if_missing(&session_id);
                }
                self.active = Some(session_id);
                vec![
                    AppEffect::Unminimize,
                    AppEffect::FocusWindow,
                    AppEffect::Repaint,
                ]
            }
            IpcCommand::SessionOff { session_id } => {
                self.sessions.remove(&session_id);
                self.order.retain(|id| id != &session_id);
                if self.active.as_deref() == Some(session_id.as_str()) {
                    self.active = self.order.last().cloned();
                }
                vec![AppEffect::Repaint]
            }
            IpcCommand::ClearAll => {
                self.sessions.clear();
                self.order.clear();
                self.active = None;
                vec![AppEffect::Repaint]
            }
            IpcCommand::Shutdown => vec![AppEffect::CloseWindow, AppEffect::ExitProcess],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> SessionStore {
        SessionStore::with_default_session()
    }

    #[test]
    fn starts_with_placeholder() {
        let s = store();
        assert_eq!(s.order(), ["agent-1"]);
        assert_eq!(s.active_id(), Some("agent-1"));
        assert!(s.sessions().contains_key("agent-1"));
    }

    #[test]
    fn first_real_session_replaces_placeholder() {
        let mut s = store();
        let effects = s.apply(IpcCommand::SetState {
            session_id: "session-1".into(),
            state: LightState::Red,
            label: None,
            message: Some("boom".into()),
        });
        assert!(effects.is_empty());
        assert!(!s.sessions().contains_key("agent-1"));
        assert_eq!(s.order(), ["session-1"]);
        assert_eq!(s.active_id(), Some("session-1"));
        assert_eq!(s.sessions()["session-1"].message.as_deref(), Some("boom"));
    }

    #[test]
    fn session_on_inserts_and_requests_focus() {
        let mut s = store();
        let effects = s.apply(IpcCommand::SessionOn {
            session_id: "session-2".into(),
            label: Some("Builder".into()),
            initial_state: Some(LightState::Yellow),
        });
        assert_eq!(
            effects,
            vec![
                AppEffect::Unminimize,
                AppEffect::FocusWindow,
                AppEffect::Repaint
            ]
        );
        assert_eq!(s.sessions()["session-2"].label, "Builder");
        assert!(s.has_active_animation());
    }

    #[test]
    fn session_off_removes_and_retargets_active() {
        let mut s = store();
        s.apply(IpcCommand::SessionOn {
            session_id: "a".into(),
            label: None,
            initial_state: None,
        });
        s.apply(IpcCommand::SessionOn {
            session_id: "b".into(),
            label: None,
            initial_state: None,
        });
        assert_eq!(s.active_id(), Some("b"));

        let effects = s.apply(IpcCommand::SessionOff {
            session_id: "b".into(),
        });
        assert_eq!(effects, vec![AppEffect::Repaint]);
        assert_eq!(s.active_id(), Some("a"));
        assert_eq!(s.order(), ["a"]);
    }

    #[test]
    fn removing_last_session_clears_active() {
        let mut s = store();
        s.apply(IpcCommand::SessionOff {
            session_id: "agent-1".into(),
        });
        assert_eq!(s.active_id(), None);
        assert!(s.order().is_empty());
        assert!(s.active_session().is_none());
    }

    #[test]
    fn clear_all_empties_everything() {
        let mut s = store();
        s.apply(IpcCommand::ClearAll);
        assert!(s.sessions().is_empty());
        assert!(s.order().is_empty());
        assert_eq!(s.active_id(), None);
        assert!(!s.has_active_animation());
    }

    #[test]
    fn ensuring_active_picks_first_tab() {
        let mut s = store();
        s.apply(IpcCommand::ClearAll);
        s.apply(IpcCommand::SetState {
            session_id: "x".into(),
            state: LightState::Green,
            label: None,
            message: None,
        });
        s.active = None;
        s.ensure_active();
        assert_eq!(s.active_id(), Some("x"));
    }

    #[test]
    fn shutdown_asks_ui_to_close_then_exit() {
        let mut s = store();
        assert_eq!(
            s.apply(IpcCommand::Shutdown),
            vec![AppEffect::CloseWindow, AppEffect::ExitProcess]
        );
    }
}
