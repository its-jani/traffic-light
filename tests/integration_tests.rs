use std::fs;
use tempfile::tempdir;
use traffic_status::cli::{
    has_our_marker, install_project, safe_remove_file, safe_write_file, uninstall_project,
    MARKER_MD,
};
use traffic_status::types::LightState;

#[test]
fn test_marker_detection() {
    assert!(has_our_marker(
        "<!-- generated-by: traffic-status -->\n# Test"
    ));
    assert!(has_our_marker(
        "// generated-by: traffic-status\nconsole.log(1);"
    ));
    assert!(has_our_marker("Traffic Light Slash Command (/traffic)"));
    assert!(has_our_marker("TrafficLightPlugin"));
    assert!(has_our_marker("TrafficStatusPlugin"));
    assert!(!has_our_marker(
        "This is an unrelated user configuration file."
    ));
}

#[test]
fn test_safe_write_protection_and_idempotency() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.md");

    // 1. Write an unrelated user file
    fs::write(&file_path, "# User Custom File without marker").unwrap();

    // 2. safe_write_file should fail without force
    let result = safe_write_file(
        &file_path,
        &format!("{}\n# New Content", MARKER_MD),
        false,
        false,
    );
    assert!(
        result.is_err(),
        "Should refuse to overwrite unmarked user file"
    );

    // 3. safe_write_file with force=true should succeed
    let result_forced = safe_write_file(
        &file_path,
        &format!("{}\n# New Content", MARKER_MD),
        false,
        true,
    );
    assert!(result_forced.is_ok(), "Forced write should succeed");
    assert_eq!(result_forced.unwrap(), true);

    // 4. Idempotency test: writing the same content again should return Ok(false)
    let result_idempotent = safe_write_file(
        &file_path,
        &format!("{}\n# New Content", MARKER_MD),
        false,
        false,
    );
    assert_eq!(
        result_idempotent.unwrap(),
        false,
        "Idempotent write should return false (no change)"
    );
}

#[test]
fn test_safe_remove_protection() {
    let dir = tempdir().unwrap();
    let user_file = dir.path().join("user_config.md");
    let marked_file = dir.path().join("our_hook.md");

    fs::write(&user_file, "# Important custom config").unwrap();
    fs::write(&marked_file, &format!("{}\n# Hook", MARKER_MD)).unwrap();

    // 1. safe_remove on user_file should fail without force
    let rem_user = safe_remove_file(&user_file, false, false);
    assert!(rem_user.is_err());
    assert!(user_file.exists());

    // 2. safe_remove on marked_file should succeed
    let rem_marked = safe_remove_file(&marked_file, false, false);
    assert_eq!(rem_marked.unwrap(), true);
    assert!(!marked_file.exists());
}

#[test]
fn test_full_project_install_and_uninstall_lifecycle() {
    let dir = tempdir().unwrap();
    let project_path = dir.path();

    // Setup sentinel files in the target directories to prove uninstall never deletes unrelated files
    let claude_sentinel = project_path
        .join(".claude")
        .join("commands")
        .join("custom_command.md");
    let opencode_sentinel = project_path
        .join(".opencode")
        .join("plugins")
        .join("custom_plugin.js");

    fs::create_dir_all(claude_sentinel.parent().unwrap()).unwrap();
    fs::create_dir_all(opencode_sentinel.parent().unwrap()).unwrap();
    fs::write(&claude_sentinel, "# My Custom Claude Command").unwrap();
    fs::write(&opencode_sentinel, "// My Custom OpenCode Plugin").unwrap();

    // 1. Install to project
    install_project(project_path, false, false);

    let claude_hook = project_path
        .join(".claude")
        .join("commands")
        .join("traffic.md");
    let opencode_cmd = project_path
        .join(".opencode")
        .join("commands")
        .join("traffic.md");
    let opencode_plugin = project_path
        .join(".opencode")
        .join("plugins")
        .join("traffic-status.js");

    assert!(claude_hook.exists(), "Claude hook should exist");
    assert!(opencode_cmd.exists(), "OpenCode command should exist");
    assert!(opencode_plugin.exists(), "OpenCode plugin should exist");

    // Check markers
    assert!(has_our_marker(&fs::read_to_string(&claude_hook).unwrap()));
    assert!(has_our_marker(&fs::read_to_string(&opencode_cmd).unwrap()));
    assert!(has_our_marker(
        &fs::read_to_string(&opencode_plugin).unwrap()
    ));

    // Check sentinel files are intact
    assert!(claude_sentinel.exists());
    assert!(opencode_sentinel.exists());

    // 2. Run install again (idempotent)
    install_project(project_path, false, false);
    assert!(claude_hook.exists());

    // 3. Uninstall from project
    uninstall_project(project_path, false, false);

    assert!(!claude_hook.exists(), "Claude hook should be removed");
    assert!(!opencode_cmd.exists(), "OpenCode command should be removed");
    assert!(
        !opencode_plugin.exists(),
        "OpenCode plugin should be removed"
    );

    // Sentinel files MUST remain intact
    assert!(
        claude_sentinel.exists(),
        "Unrelated user command must NOT be deleted"
    );
    assert!(
        opencode_sentinel.exists(),
        "Unrelated user plugin must NOT be deleted"
    );
}

#[test]
fn test_global_install_and_uninstall_lifecycle() {
    let dir = tempdir().unwrap();
    let temp_home = dir.path().to_path_buf();

    // Set temporary HOME / USERPROFILE for test isolation
    std::env::set_var("USERPROFILE", &temp_home);
    std::env::set_var("HOME", &temp_home);
    std::env::set_var("LOCALAPPDATA", temp_home.join("AppData").join("Local"));

    // Sentinel files in global directories
    let global_claude_sentinel = temp_home.join(".claude").join("commands").join("custom.md");
    let global_opencode_sentinel = temp_home
        .join(".config")
        .join("opencode")
        .join("plugins")
        .join("custom.js");

    fs::create_dir_all(global_claude_sentinel.parent().unwrap()).unwrap();
    fs::create_dir_all(global_opencode_sentinel.parent().unwrap()).unwrap();
    fs::write(&global_claude_sentinel, "# Unrelated Global Claude Command").unwrap();
    fs::write(
        &global_opencode_sentinel,
        "// Unrelated Global OpenCode Plugin",
    )
    .unwrap();

    // 1. Install globally
    traffic_status::cli::install_global(false, false);

    let global_claude = temp_home
        .join(".claude")
        .join("commands")
        .join("traffic.md");
    let global_opencode_cmd = temp_home
        .join(".config")
        .join("opencode")
        .join("commands")
        .join("traffic.md");
    let global_opencode_plugin = temp_home
        .join(".config")
        .join("opencode")
        .join("plugins")
        .join("traffic-status.js");

    assert!(global_claude.exists(), "Global Claude hook should exist");
    assert!(
        global_opencode_cmd.exists(),
        "Global OpenCode command should exist"
    );
    assert!(
        global_opencode_plugin.exists(),
        "Global OpenCode plugin should exist"
    );

    // Sentinel files untouched
    assert!(global_claude_sentinel.exists());
    assert!(global_opencode_sentinel.exists());

    // 2. Uninstall globally
    traffic_status::cli::uninstall_global(false, false);

    assert!(
        !global_claude.exists(),
        "Global Claude hook should be removed"
    );
    assert!(
        !global_opencode_cmd.exists(),
        "Global OpenCode command should be removed"
    );
    assert!(
        !global_opencode_plugin.exists(),
        "Global OpenCode plugin should be removed"
    );

    // Sentinel files MUST remain intact
    assert!(
        global_claude_sentinel.exists(),
        "Unrelated user global command must NOT be deleted"
    );
    assert!(
        global_opencode_sentinel.exists(),
        "Unrelated user global plugin must NOT be deleted"
    );
}

#[test]
fn test_light_state_parsing() {
    assert_eq!(LightState::parse_str("green"), Some(LightState::Green));
    assert_eq!(LightState::parse_str("ready"), Some(LightState::Green));
    assert_eq!(LightState::parse_str("done"), Some(LightState::Green));
    assert_eq!(LightState::parse_str("yellow"), Some(LightState::Yellow));
    assert_eq!(LightState::parse_str("working"), Some(LightState::Yellow));
    assert_eq!(LightState::parse_str("thinking"), Some(LightState::Yellow));
    assert_eq!(LightState::parse_str("red"), Some(LightState::Red));
    assert_eq!(LightState::parse_str("error"), Some(LightState::Red));
    assert_eq!(LightState::parse_str("needs_input"), Some(LightState::Red));
    assert_eq!(LightState::parse_str("off"), Some(LightState::Off));
    assert_eq!(LightState::parse_str("unknown_state"), None);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_server_http_security_and_contracts() {
    use crossbeam_channel::unbounded;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use traffic_status::ipc::start_ipc_server;
    use traffic_status::types::IpcCommand;

    let (tx, _rx) = unbounded::<IpcCommand>();
    let port = 18765; // Test port

    let server_handle = tokio::spawn(async move {
        let _ = start_ipc_server(port, tx).await;
    });

    // Wait for server to bind
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // 1. GET /ping -> 200 OK
    {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .unwrap();
        stream
            .write_all(b"GET /ping HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf);
        assert!(resp.starts_with("HTTP/1.1 200 OK"));
        assert!(resp.contains(r#""app":"traffic-status""#));
    }

    // 2. GET /state -> 405 Method Not Allowed (CSRF prevention)
    {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .unwrap();
        stream
            .write_all(
                b"GET /state?state=yellow HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
            )
            .await
            .unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf);
        assert!(resp.starts_with("HTTP/1.1 405 Method Not Allowed"));
    }

    // 3. POST /state without Content-Type -> 415 Unsupported Media Type
    {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .unwrap();
        stream
            .write_all(
                b"POST /state HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 18\r\nConnection: close\r\n\r\n{\"state\":\"yellow\"}",
            )
            .await
            .unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf);
        assert!(resp.starts_with("HTTP/1.1 415 Unsupported Media Type"));
    }

    // 4. POST /state with invalid JSON -> 400 Bad Request
    {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .unwrap();
        stream
            .write_all(
                b"POST /state HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: 15\r\nConnection: close\r\n\r\n{invalid json..",
            )
            .await
            .unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf);
        assert!(resp.starts_with("HTTP/1.1 400 Bad Request"));
    }

    // 5. POST /state with valid JSON -> 200 OK
    {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .unwrap();
        let body = r#"{"session_id":"test-sess","state":"yellow","message":"Running tests"}"#;
        let req = format!(
            "POST /state HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(req.as_bytes()).await.unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf);
        assert!(resp.starts_with("HTTP/1.1 200 OK"));
        assert!(resp.contains(r#""status":"ok""#));
    }

    server_handle.abort();
}
