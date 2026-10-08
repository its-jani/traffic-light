//! Pure HTTP request parsing, validation and routing for the IPC server.
//!
//! Everything here is synchronous and socket-free so it can be unit tested
//! directly. `mod.rs` owns the async accept/read/write shell.

use crate::types::{IpcCommand, IpcPayload, LightState};

pub const MAX_REQUEST_SIZE: usize = 65536; // 64 KB limit

/// The request line plus the headers we care about.
pub struct RequestHead<'a> {
    pub method: &'a str,
    /// Full request target, query string included.
    pub path: &'a str,
    pub host: Option<&'a str>,
    /// True when the request line declares HTTP/1.1.
    pub http11: bool,
    pub content_length: Option<usize>,
    pub has_json_content_type: bool,
}

/// Parse the request line and headers. Returns `None` for a request line
/// with fewer than two whitespace-separated parts.
pub fn parse_head(raw: &str) -> Option<RequestHead<'_>> {
    let prose = raw.lines().next().unwrap_or("");
    let mut parts = prose.split_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    let http11 = parts.next().is_some_and(|v| v.starts_with("HTTP/1.1"));

    let mut host = None;
    let mut content_length = None;
    let mut has_json_content_type = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("host:") && host.is_none() {
            host = Some(trimmed[5..].trim());
        }
        if lower.starts_with("content-length:") && content_length.is_none() {
            content_length = trimmed[15..].trim().parse::<usize>().ok();
        }
        if lower.starts_with("content-type:") && lower.contains("application/json") {
            has_json_content_type = true;
        }
    }

    Some(RequestHead {
        method,
        path,
        host,
        http11,
        content_length,
        has_json_content_type,
    })
}

/// What the I/O layer should do with a request.
pub enum HttpOutcome {
    /// Write this response and close.
    Respond { status: &'static str, body: String },
    /// Forward this command to the app, then write this response.
    Dispatch {
        command: IpcCommand,
        status: &'static str,
        body: String,
    },
}

fn respond(status: &'static str, body: &str) -> HttpOutcome {
    HttpOutcome::Respond {
        status,
        body: body.to_string(),
    }
}

/// Validate the request and produce the outcome. No sockets, no channels.
pub fn decide(head: &RequestHead<'_>, raw: &str) -> HttpOutcome {
    match head.host {
        Some(host) => {
            let host_without_port = host.split(':').next().unwrap_or("").trim();
            if host_without_port != "localhost"
                && host_without_port != "127.0.0.1"
                && host_without_port != "[::1]"
                && host_without_port != "::1"
            {
                return respond(
                    "403 Forbidden",
                    r#"{"error":"Forbidden: Host header must target loopback (localhost or 127.0.0.1)"}"#,
                );
            }
        }
        None => {
            // In HTTP/1.1, Host header is mandatory
            if head.http11 {
                return respond("400 Bad Request", r#"{"error":"Missing Host header"}"#);
            }
        }
    }

    if let Some(cl) = head.content_length {
        if cl > MAX_REQUEST_SIZE {
            return respond(
                "413 Payload Too Large",
                r#"{"error":"Payload too large (exceeds 64 KB)"}"#,
            );
        }
    }

    if head.method == "OPTIONS" {
        return respond(
            "405 Method Not Allowed",
            r#"{"error":"OPTIONS preflight is not permitted"}"#,
        );
    }

    let path = match head.path.find('?') {
        Some(idx) => &head.path[..idx],
        None => head.path,
    };

    // Health / Ping endpoint (GET only)
    if path == "/health" || path == "/ping" {
        if head.method != "GET" {
            return respond("405 Method Not Allowed", r#"{"error":"Use GET for /ping"}"#);
        }
        return respond(
            "200 OK",
            &format!(
                r#"{{"status":"ok","app":"traffic-status","version":"{}"}}"#,
                env!("CARGO_PKG_VERSION")
            ),
        );
    }

    // State mutating endpoints REQUIRE POST method
    let is_mutation_endpoint = path == "/state"
        || path == "/session/on"
        || path == "/on"
        || path == "/session/off"
        || path == "/off"
        || path == "/clear"
        || path == "/shutdown";

    if is_mutation_endpoint && head.method != "POST" {
        return respond(
            "405 Method Not Allowed",
            r#"{"error":"State-modifying requests must use POST with Content-Type: application/json"}"#,
        );
    }

    if !is_mutation_endpoint {
        return respond("404 Not Found", r#"{"error":"Not Found"}"#);
    }

    if !head.has_json_content_type {
        return respond(
            "415 Unsupported Media Type",
            r#"{"error":"Content-Type must be application/json"}"#,
        );
    }

    let body = match raw.find("\r\n\r\n") {
        Some(idx) => &raw[idx + 4..],
        None => match raw.find("\n\n") {
            Some(idx) => &raw[idx + 2..],
            None => "",
        },
    };

    let trimmed_body = body.trim();
    let payload: IpcPayload = if trimmed_body.is_empty() {
        IpcPayload {
            session_id: None,
            id: None,
            state: None,
            label: None,
            message: None,
            action: None,
        }
    } else {
        match serde_json::from_str::<IpcPayload>(trimmed_body) {
            Ok(p) => p,
            Err(e) => {
                return respond(
                    "400 Bad Request",
                    &format!(r#"{{"error":"Malformed JSON: {}"}}"#, e),
                );
            }
        }
    };

    dispatch(path, payload)
}

/// Map a validated `/state`-family request onto an IPC command + response.
fn dispatch(path: &str, payload: IpcPayload) -> HttpOutcome {
    let session_id = payload
        .session_id
        .clone()
        .or(payload.id.clone())
        .unwrap_or_else(|| "default".to_string());
    let action = payload.action.clone().unwrap_or_default().to_lowercase();

    if path == "/shutdown" || action == "shutdown" {
        HttpOutcome::Dispatch {
            command: IpcCommand::Shutdown,
            status: "200 OK",
            body: r#"{"status":"ok","action":"shutdown"}"#.to_string(),
        }
    } else if path == "/session/on" || path == "/on" || action == "on" {
        let initial_state = payload.state.as_deref().and_then(LightState::parse_str);
        let body = format!(
            r#"{{"status":"ok","session_id":"{}","action":"on"}}"#,
            session_id
        );
        HttpOutcome::Dispatch {
            command: IpcCommand::SessionOn {
                session_id,
                label: payload.label,
                initial_state,
            },
            status: "200 OK",
            body,
        }
    } else if path == "/session/off" || path == "/off" || action == "off" || action == "remove" {
        let body = format!(
            r#"{{"status":"ok","session_id":"{}","action":"off"}}"#,
            session_id
        );
        HttpOutcome::Dispatch {
            command: IpcCommand::SessionOff { session_id },
            status: "200 OK",
            body,
        }
    } else if path == "/clear" || action == "clear" {
        HttpOutcome::Dispatch {
            command: IpcCommand::ClearAll,
            status: "200 OK",
            body: r#"{"status":"ok","action":"clear_all"}"#.to_string(),
        }
    } else {
        // State update endpoint: POST /state
        let state = payload
            .state
            .as_deref()
            .and_then(LightState::parse_str)
            .unwrap_or(LightState::Green);
        let body = format!(
            r#"{{"status":"ok","session_id":"{}","state":"{}"}}"#,
            session_id,
            state.display_name()
        );
        HttpOutcome::Dispatch {
            command: IpcCommand::SetState {
                session_id,
                state,
                label: payload.label,
                message: payload.message,
            },
            status: "200 OK",
            body,
        }
    }
}

/// Interpret a non-HTTP (bare JSON or single-word) payload.
pub fn decide_raw(text: &str) -> Option<IpcCommand> {
    if let Ok(payload) = serde_json::from_str::<IpcPayload>(text) {
        let session_id = payload
            .session_id
            .clone()
            .or(payload.id.clone())
            .unwrap_or_else(|| "default".to_string());
        let action = payload.action.clone().unwrap_or_default().to_lowercase();

        if action == "off" || action == "remove" {
            Some(IpcCommand::SessionOff { session_id })
        } else if action == "on" {
            let initial_state = payload.state.as_deref().and_then(LightState::parse_str);
            Some(IpcCommand::SessionOn {
                session_id,
                label: payload.label,
                initial_state,
            })
        } else if action == "clear" {
            Some(IpcCommand::ClearAll)
        } else {
            let state = payload
                .state
                .as_deref()
                .and_then(LightState::parse_str)
                .unwrap_or(LightState::Green);
            Some(IpcCommand::SetState {
                session_id,
                state,
                label: payload.label,
                message: payload.message,
            })
        }
    } else {
        // Fallback for simple single-word commands: "green", "yellow", "red", "off", "clear"
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }
        let cmd = parts[0].to_lowercase();
        let session_id = parts.get(1).copied().unwrap_or("default").to_string();

        if cmd == "clear" {
            Some(IpcCommand::ClearAll)
        } else if cmd == "off" {
            Some(IpcCommand::SessionOff { session_id })
        } else {
            LightState::parse_str(&cmd).map(|state| IpcCommand::SetState {
                session_id,
                state,
                label: None,
                message: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn head(raw: &str) -> RequestHead<'_> {
        parse_head(raw).expect("request line should parse")
    }

    fn body_of(outcome: &HttpOutcome) -> &str {
        match outcome {
            HttpOutcome::Respond { body, .. } | HttpOutcome::Dispatch { body, .. } => body,
        }
    }

    fn status_of(outcome: &HttpOutcome) -> &'static str {
        match outcome {
            HttpOutcome::Respond { status, .. } | HttpOutcome::Dispatch { status, .. } => status,
        }
    }

    #[test]
    fn parse_head_reads_method_path_and_headers() {
        let h = head("POST /state?x=1 HTTP/1.1\r\nHost: 127.0.0.1:8765\r\nContent-Length: 12\r\nContent-Type: application/json\r\n\r\n{}");
        assert_eq!(h.method, "POST");
        assert_eq!(h.path, "/state?x=1");
        assert_eq!(h.host, Some("127.0.0.1:8765"));
        assert_eq!(h.content_length, Some(12));
        assert!(h.has_json_content_type);
        assert!(h.http11);
    }

    #[test]
    fn parse_head_rejects_junk() {
        assert!(parse_head("not-a-request").is_none());
        assert!(parse_head("").is_none());
    }

    #[test]
    fn health_is_get_only() {
        let raw = "GET /ping HTTP/1.1\r\nHost: localhost\r\n\r\n";
        assert_eq!(status_of(&decide(&head(raw), raw)), "200 OK");

        let raw =
            "POST /health HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n\r\n{}";
        assert_eq!(
            status_of(&decide(&head(raw), raw)),
            "405 Method Not Allowed"
        );
    }

    #[test]
    fn non_loopback_host_is_forbidden() {
        let raw = "POST /state HTTP/1.1\r\nHost: evil.example.com\r\nContent-Type: application/json\r\n\r\n{}";
        assert_eq!(status_of(&decide(&head(raw), raw)), "403 Forbidden");
    }

    #[test]
    fn http11_without_host_is_bad_request() {
        let raw = "POST /state HTTP/1.1\r\nContent-Type: application/json\r\n\r\n{}";
        assert_eq!(status_of(&decide(&head(raw), raw)), "400 Bad Request");
    }

    #[test]
    fn mutation_requires_post_and_json() {
        let raw = "GET /state HTTP/1.1\r\nHost: localhost\r\n\r\n";
        assert_eq!(
            status_of(&decide(&head(raw), raw)),
            "405 Method Not Allowed"
        );

        let raw = "POST /state HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n";
        assert_eq!(
            status_of(&decide(&head(raw), raw)),
            "415 Unsupported Media Type"
        );
    }

    #[test]
    fn unknown_path_is_404() {
        let raw = "GET /nope HTTP/1.1\r\nHost: localhost\r\n\r\n";
        assert_eq!(status_of(&decide(&head(raw), raw)), "404 Not Found");
    }

    #[test]
    fn malformed_json_is_bad_request() {
        let raw =
            "POST /state HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n\r\n{bad";
        assert_eq!(status_of(&decide(&head(raw), raw)), "400 Bad Request");
    }

    #[test]
    fn oversized_content_length_is_413() {
        let raw = "POST /state HTTP/1.1\r\nHost: localhost\r\nContent-Length: 999999\r\nContent-Type: application/json\r\n\r\n{}";
        assert_eq!(status_of(&decide(&head(raw), raw)), "413 Payload Too Large");
    }

    #[test]
    fn dispatch_routes_each_endpoint() {
        let raw = "POST /state HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n\r\n{\"session_id\":\"s1\",\"state\":\"red\"}";
        match decide(&head(raw), raw) {
            HttpOutcome::Dispatch { command, body, .. } => {
                assert!(matches!(
                    command,
                    IpcCommand::SetState {
                        state: LightState::Red,
                        ..
                    }
                ));
                assert!(body.contains("\"session_id\":\"s1\""));
            }
            _ => panic!("expected dispatch"),
        }

        let raw = "POST /shutdown HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n\r\n{}";
        assert!(matches!(
            decide(&head(raw), raw),
            HttpOutcome::Dispatch {
                command: IpcCommand::Shutdown,
                ..
            }
        ));
    }

    #[test]
    fn decide_raw_understands_json_and_bare_words() {
        assert!(matches!(
            decide_raw(r#"{"action":"clear"}"#),
            Some(IpcCommand::ClearAll)
        ));
        assert!(matches!(
            decide_raw("yellow session-9"),
            Some(IpcCommand::SetState {
                state: LightState::Yellow,
                ..
            })
        ));
        assert!(matches!(decide_raw("nonsense"), None));
    }

    #[test]
    fn decide_never_panics_on_garbage() {
        for raw in ["", " ", "GET", "GET ", "\r\n\r\n", "GET / HTTP/9.9\r\n\r\n"] {
            let outcome = match parse_head(raw) {
                Some(h) => Some(decide(&h, raw)),
                None => None,
            };
            if let Some(o) = outcome {
                let _ = body_of(&o);
            }
        }
    }
}
