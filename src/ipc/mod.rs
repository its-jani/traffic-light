//! IPC server: a thin async I/O shell around the pure request logic in
//! [`http`]. All parsing, validation and routing decisions live there so
//! they can be tested without opening a socket.

mod http;

use crossbeam_channel::Sender;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::types::IpcCommand;
use http::{decide, decide_raw, parse_head, HttpOutcome, MAX_REQUEST_SIZE};

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
                        eprintln!("[Traffic Status] Connection error: {e}");
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
    let Some(head) = parse_head(raw_text) else {
        send_response(
            socket,
            "400 Bad Request",
            "application/json",
            r#"{"error":"Invalid HTTP request"}"#,
        )
        .await?;
        return Ok(());
    };

    match decide(&head, raw_text) {
        HttpOutcome::Respond { status, body } => {
            send_response(socket, status, "application/json", &body).await?;
        }
        HttpOutcome::Dispatch {
            command,
            status,
            body,
        } => {
            let _ = tx.send(command);
            send_response(socket, status, "application/json", &body).await?;
        }
    }

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

    if let Some(command) = decide_raw(text) {
        let _ = tx.send(command);
    }

    let _ = socket.write_all(b"{\"status\":\"ok\"}\n").await;
    let _ = socket.flush().await;
    let _ = socket.shutdown().await;
    Ok(())
}
