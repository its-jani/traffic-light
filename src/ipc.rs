use crossbeam_channel::Sender;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::types::{IpcCommand, IpcPayload, LightState};

const MAX_REQUEST_SIZE: usize = 65536; // 64 KB limit

pub async fn start_ipc_server(
    port: u16,
    tx: Sender<IpcCommand>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    println!("[Traffic Status] IPC Server listening on http://{}", addr);

    loop {
        match listener.accept().await {
            Ok((mut socket, _client_addr)) => {
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(&mut socket, tx_clone).await {
                        let _ = e;
                    }
                });
            }
            Err(e) => {
                eprintln!("[Traffic Status] Accept error: {e}");
            }
        }
    }
}

async fn handle_connection(
    socket: &mut TcpStream,
    tx: Sender<IpcCommand>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = vec![0u8; MAX_REQUEST_SIZE];
    let n = socket.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }

    if n >= MAX_REQUEST_SIZE {
        let resp = "HTTP/1.1 413 Payload Too Large\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"error\":\"Payload too large\"}";
        socket.write_all(resp.as_bytes()).await?;
        let _ = socket.flush().await;
        let _ = socket.shutdown().await;
        return Ok(());
    }

    let raw_text = String::from_utf8_lossy(&buf[..n]);

    if raw_text.starts_with("GET ")
        || raw_text.starts_with("POST ")
        || raw_text.starts_with("PUT ")
        || raw_text.starts_with("DELETE ")
        || raw_text.starts_with("OPTIONS ")
    {
        handle_http_request(socket, &raw_text, tx).await?;
    } else {
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
        send_response(
            socket,
            "400 Bad Request",
            "application/json",
            r#"{"error":"Invalid HTTP request"}"#,
        )
        .await?;
        return Ok(());
    }

    let method = parts[0];
    let full_path = parts[1];

    if method == "OPTIONS" {
        let resp = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Methods: POST, GET, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nConnection: close\r\n\r\n";
        socket.write_all(resp.as_bytes()).await?;
        let _ = socket.flush().await;
        let _ = socket.shutdown().await;
        return Ok(());
    }

    let (path, _query_str) = match full_path.find('?') {
        Some(idx) => (&full_path[..idx], &full_path[idx + 1..]),
        None => (full_path, ""),
    };

    // Health / Ping endpoint (GET only)
    if path == "/health" || path == "/ping" {
        if method != "GET" {
            send_response(
                socket,
                "405 Method Not Allowed",
                "application/json",
                r#"{"error":"Use GET for /ping"}"#,
            )
            .await?;
            return Ok(());
        }
        let body = format!(
            r#"{{"status":"ok","app":"traffic-status","version":"{}"}}"#,
            env!("CARGO_PKG_VERSION")
        );
        send_response(socket, "200 OK", "application/json", &body).await?;
        return Ok(());
    }

    // State mutating endpoints REQUIRE POST method
    let is_mutation_endpoint = path == "/state"
        || path == "/session/on"
        || path == "/on"
        || path == "/session/off"
        || path == "/off"
        || path == "/clear";

    if is_mutation_endpoint && method != "POST" {
        send_response(
            socket,
            "405 Method Not Allowed",
            "application/json",
            r#"{"error":"State-modifying requests must use POST with Content-Type: application/json"}"#,
        )
        .await?;
        return Ok(());
    }

    if !is_mutation_endpoint {
        send_response(
            socket,
            "404 Not Found",
            "application/json",
            r#"{"error":"Not Found"}"#,
        )
        .await?;
        return Ok(());
    }

    // Verify Content-Type header on POST
    let has_json_header = lines.clone().any(|line| {
        let lower = line.to_ascii_lowercase();
        lower.starts_with("content-type:") && lower.contains("application/json")
    });

    if !has_json_header {
        send_response(
            socket,
            "415 Unsupported Media Type",
            "application/json",
            r#"{"error":"Content-Type must be application/json"}"#,
        )
        .await?;
        return Ok(());
    }

    // Extract HTTP body
    let body = match raw_text.find("\r\n\r\n") {
        Some(idx) => &raw_text[idx + 4..],
        None => match raw_text.find("\n\n") {
            Some(idx) => &raw_text[idx + 2..],
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
                let err_msg = format!(r#"{{"error":"Malformed JSON: {}"}}"#, e);
                send_response(socket, "400 Bad Request", "application/json", &err_msg).await?;
                return Ok(());
            }
        }
    };

    let session_id = payload
        .session_id
        .or(payload.id)
        .unwrap_or_else(|| "default".to_string());
    let action = payload.action.unwrap_or_default().to_lowercase();

    let response_body = if path == "/session/on" || path == "/on" || action == "on" {
        let initial_state = payload.state.as_deref().and_then(LightState::parse_str);
        let _ = tx.send(IpcCommand::SessionOn {
            session_id: session_id.clone(),
            label: payload.label,
            initial_state,
        });
        format!(
            r#"{{"status":"ok","session_id":"{}","action":"on"}}"#,
            session_id
        )
    } else if path == "/session/off" || path == "/off" || action == "off" || action == "remove" {
        let _ = tx.send(IpcCommand::SessionOff {
            session_id: session_id.clone(),
        });
        format!(
            r#"{{"status":"ok","session_id":"{}","action":"off"}}"#,
            session_id
        )
    } else if path == "/clear" || action == "clear" {
        let _ = tx.send(IpcCommand::ClearAll);
        r#"{"status":"ok","action":"clear_all"}"#.to_string()
    } else {
        // State update endpoint: POST /state
        let state = payload
            .state
            .as_deref()
            .and_then(LightState::parse_str)
            .unwrap_or(LightState::Green);
        let _ = tx.send(IpcCommand::SetState {
            session_id: session_id.clone(),
            state,
            label: payload.label,
            message: payload.message,
        });
        format!(
            r#"{{"status":"ok","session_id":"{}","state":"{}"}}"#,
            session_id,
            state.display_name()
        )
    };

    send_response(socket, "200 OK", "application/json", &response_body).await?;
    Ok(())
}

async fn send_response(
    socket: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );
    socket.write_all(response.as_bytes()).await?;
    let _ = socket.flush().await;
    let _ = socket.shutdown().await;
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
        let session_id = payload
            .session_id
            .or(payload.id)
            .unwrap_or_else(|| "default".to_string());
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
            let state = payload
                .state
                .as_deref()
                .and_then(LightState::parse_str)
                .unwrap_or(LightState::Green);
            let _ = tx.send(IpcCommand::SetState {
                session_id,
                state,
                label: payload.label,
                message: payload.message,
            });
        }
        let _ = socket.write_all(b"{\"status\":\"ok\"}\n").await;
        let _ = socket.flush().await;
        let _ = socket.shutdown().await;
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
        let _ = socket.flush().await;
        let _ = socket.shutdown().await;
    }

    Ok(())
}
