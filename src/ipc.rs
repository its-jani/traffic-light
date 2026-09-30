use std::collections::HashMap;
use crossbeam_channel::Sender;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::types::{IpcCommand, IpcPayload, LightState};

pub async fn start_ipc_server(port: u16, tx: Sender<IpcCommand>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    println!("[Traffic Light IPC] Listening for session updates on http://{}", addr);

    loop {
        match listener.accept().await {
            Ok((mut socket, _client_addr)) => {
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(&mut socket, tx_clone).await {
                        // ignore brief disconnect errors
                        let _ = e;
                    }
                });
            }
            Err(e) => {
                eprintln!("[Traffic Light IPC] Accept error: {e}");
            }
        }
    }
}

async fn handle_connection(
    socket: &mut TcpStream,
    tx: Sender<IpcCommand>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = vec![0u8; 8192];
    let n = socket.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }

    let raw_text = String::from_utf8_lossy(&buf[..n]);

    // Check if it's an HTTP request
    if raw_text.starts_with("GET ") || raw_text.starts_with("POST ") || raw_text.starts_with("PUT ") || raw_text.starts_with("DELETE ") || raw_text.starts_with("OPTIONS ") {
        handle_http_request(socket, &raw_text, tx).await?;
    } else {
        // Raw JSON or command line
        handle_raw_payload(socket, &raw_text, tx).await?;
    }

    Ok(())
}

async fn handle_http_request(
    socket: &mut TcpStream,
    raw_text: &str,
    tx: Sender<IpcCommand>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut lines = raw_text.lines();
    let first_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    if parts.len() < 2 {
        let resp = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        socket.write_all(resp.as_bytes()).await?;
        return Ok(());
    }

    let method = parts[0];
    let full_path = parts[1];

    if method == "OPTIONS" {
        let resp = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST, GET, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nConnection: close\r\n\r\n";
        socket.write_all(resp.as_bytes()).await?;
        return Ok(());
    }

    let (path, query_str) = match full_path.find('?') {
        Some(idx) => (&full_path[..idx], &full_path[idx + 1..]),
        None => (full_path, ""),
    };

    let query_params = parse_query_string(query_str);

    // Extract HTTP body if POST/PUT
    let body = match raw_text.find("\r\n\r\n") {
        Some(idx) => &raw_text[idx + 4..],
        None => match raw_text.find("\n\n") {
            Some(idx) => &raw_text[idx + 2..],
            None => "",
        },
    };

    let mut payload: IpcPayload = if !body.trim().is_empty() {
        serde_json::from_str(body.trim()).unwrap_or(IpcPayload {
            session_id: None,
            id: None,
            state: None,
            label: None,
            message: None,
            action: None,
        })
    } else {
        IpcPayload {
            session_id: None,
            id: None,
            state: None,
            label: None,
            message: None,
            action: None,
        }
    };

    // Overlay query parameters if provided
    if let Some(session_id) = query_params.get("session_id").or_else(|| query_params.get("id")) {
        payload.session_id = Some(session_id.to_string());
    }
    if let Some(state) = query_params.get("state") {
        payload.state = Some(state.to_string());
    }
    if let Some(label) = query_params.get("label") {
        payload.label = Some(label.to_string());
    }
    if let Some(msg) = query_params.get("message").or_else(|| query_params.get("msg")) {
        payload.message = Some(msg.to_string());
    }
    if let Some(action) = query_params.get("action") {
        payload.action = Some(action.to_string());
    }

    let session_id = payload.session_id.or(payload.id).unwrap_or_else(|| "default".to_string());
    let action = payload.action.unwrap_or_default().to_lowercase();

    let response_body = if path == "/health" || path == "/ping" {
        r#"{"status":"ok","app":"traffic-light"}"#.to_string()
    } else if path == "/session/on" || path == "/on" || action == "on" {
        let initial_state = payload.state.as_deref().and_then(LightState::parse_str);
        let _ = tx.send(IpcCommand::SessionOn {
            session_id: session_id.clone(),
            label: payload.label,
            initial_state,
        });
        format!(r#"{{"status":"ok","session_id":"{}","action":"on"}}"#, session_id)
    } else if path == "/session/off" || path == "/off" || action == "off" || action == "remove" {
        let _ = tx.send(IpcCommand::SessionOff {
            session_id: session_id.clone(),
        });
        format!(r#"{{"status":"ok","session_id":"{}","action":"off"}}"#, session_id)
    } else if path == "/clear" || action == "clear" {
        let _ = tx.send(IpcCommand::ClearAll);
        r#"{"status":"ok","action":"clear_all"}"#.to_string()
    } else {
        // Default to state update
        let state = payload.state.as_deref().and_then(LightState::parse_str).unwrap_or(LightState::Green);
        let _ = tx.send(IpcCommand::SetState {
            session_id: session_id.clone(),
            state,
            label: payload.label,
            message: payload.message,
        });
        format!(r#"{{"status":"ok","session_id":"{}","state":"{}"}}"#, session_id, state.display_name())
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response_body.len(),
        response_body
    );

    socket.write_all(response.as_bytes()).await?;
    Ok(())
}

async fn handle_raw_payload(
    socket: &mut TcpStream,
    raw_text: &str,
    tx: Sender<IpcCommand>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = raw_text.trim();
    if text.is_empty() {
        return Ok(());
    }

    if let Ok(payload) = serde_json::from_str::<IpcPayload>(text) {
        let session_id = payload.session_id.or(payload.id).unwrap_or_else(|| "default".to_string());
        let action = payload.action.unwrap_or_default().to_lowercase();

        if action == "off" || action == "remove" {
            let _ = tx.send(IpcCommand::SessionOff { session_id });
        } else if action == "on" {
            let initial_state = payload.state.as_deref().and_then(LightState::parse_str);
            let _ = tx.send(IpcCommand::SessionOn {
                session_id,
                label: payload.label,
                initial_state,
            });
        } else if action == "clear" {
            let _ = tx.send(IpcCommand::ClearAll);
        } else {
            let state = payload.state.as_deref().and_then(LightState::parse_str).unwrap_or(LightState::Green);
            let _ = tx.send(IpcCommand::SetState {
                session_id,
                state,
                label: payload.label,
                message: payload.message,
            });
        }
        let _ = socket.write_all(b"{\"status\":\"ok\"}\n").await;
    } else {
        // Fallback for simple single-word commands: "green", "yellow", "red", "off", "clear"
        let parts: Vec<&str> = text.split_whitespace().collect();
        if !parts.is_empty() {
            let cmd = parts[0].to_lowercase();
            let session_id = parts.get(1).copied().unwrap_or("default").to_string();

            if cmd == "clear" {
                let _ = tx.send(IpcCommand::ClearAll);
            } else if cmd == "off" {
                let _ = tx.send(IpcCommand::SessionOff { session_id });
            } else if let Some(state) = LightState::parse_str(&cmd) {
                let _ = tx.send(IpcCommand::SetState {
                    session_id,
                    state,
                    label: None,
                    message: None,
                });
            }
        }
        let _ = socket.write_all(b"{\"status\":\"ok\"}\n").await;
    }

    Ok(())
}

fn parse_query_string(qs: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in qs.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut kv = pair.splitn(2, '=');
        if let Some(key) = kv.next() {
            let val = kv.next().unwrap_or("");
            map.insert(
                url_decode(key),
                url_decode(val),
            );
        }
    }
    map
}

fn url_decode(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let mut hex = String::with_capacity(2);
            if let Some(&c1) = chars.peek() {
                if c1.is_ascii_hexdigit() {
                    hex.push(chars.next().unwrap());
                    if let Some(&c2) = chars.peek() {
                        if c2.is_ascii_hexdigit() {
                            hex.push(chars.next().unwrap());
                        }
                    }
                }
            }
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    bytes.push(byte);
                    continue;
                }
            }
            bytes.push(b'%');
            bytes.extend_from_slice(hex.as_bytes());
        } else if ch == '+' {
            bytes.push(b' ');
        } else {
            let mut buf = [0u8; 4];
            bytes.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
        }
    }
    String::from_utf8_lossy(&bytes).to_string()
}
