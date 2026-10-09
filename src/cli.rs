use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

use crate::installer::{
    get_home_dir, get_stable_bin_path, install_global, install_project, run_doctor,
    uninstall_global, uninstall_project,
};

pub fn handle_cli_args(args: &[String]) -> bool {
    if args.len() <= 1 {
        return false; // Launch GUI app
    }

    match args[1].as_str() {
        "install" => {
            let is_global = args.iter().any(|a| a == "--global" || a == "-g");
            let dry_run = args.iter().any(|a| a == "--dry-run");
            let force = args.iter().any(|a| a == "--force" || a == "-f");
            let project_idx = args.iter().position(|a| a == "--project" || a == "-p");

            if is_global {
                install_global(dry_run, force);
            } else if let Some(idx) = project_idx {
                let path = args.get(idx + 1).map(|s| s.as_str()).unwrap_or(".");
                install_project(Path::new(path), dry_run, force);
            } else if args.len() > 2 && !args[2].starts_with('-') {
                install_project(Path::new(&args[2]), dry_run, force);
            } else {
                install_project(Path::new("."), dry_run, force);
            }
            true
        }
        "uninstall" => {
            let is_global = args
                .iter()
                .any(|a| a == "--global" || a == "-g" || a == "--all");
            let dry_run = args.iter().any(|a| a == "--dry-run");
            let force = args.iter().any(|a| a == "--force" || a == "-f");
            let project_idx = args.iter().position(|a| a == "--project" || a == "-p");

            if is_global {
                uninstall_global(dry_run, force);
            } else if let Some(idx) = project_idx {
                let path = args.get(idx + 1).map(|s| s.as_str()).unwrap_or(".");
                uninstall_project(Path::new(path), dry_run, force);
            } else if args.len() > 2 && !args[2].starts_with('-') {
                uninstall_project(Path::new(&args[2]), dry_run, force);
            } else {
                uninstall_project(Path::new("."), dry_run, force);
            }
            true
        }
        "doctor" => {
            run_doctor();
            true
        }
        "on" => {
            let label = args.get(2).map(|s| s.as_str()).unwrap_or("CLI-Session");
            send_quick_command("on", label);
            true
        }
        "off" => {
            send_quick_command("off", "");
            true
        }
        "green" | "yellow" | "red" => {
            let state = &args[1];
            let msg = args.get(2).map(|s| s.as_str()).unwrap_or("");
            send_quick_command(state, msg);
            true
        }
        "clear" => {
            send_quick_command("clear", "");
            true
        }
        "--help" | "-h" | "help" => {
            print_help();
            true
        }
        "--version" | "-v" => {
            println!("traffic-status v{}", env!("CARGO_PKG_VERSION"));
            true
        }
        "run" | "gui" | "daemon" => {
            false // Handled by main.rs
        }
        _ => {
            eprintln!("Unknown command: '{}'", args[1]);
            print_help();
            true
        }
    }
}

fn print_help() {
    println!("🚦 Traffic Status v{}", env!("CARGO_PKG_VERSION"));
    println!("A lightweight floating desktop traffic light for AI coding agents.\n");
    println!("USAGE:");
    println!("  traffic-status                           Launch the floating traffic status UI");
    println!("  traffic-status install --global          Install globally for all current and future projects");
    println!("  traffic-status install --project [PATH]  Install commands into a specific project");
    println!("  traffic-status uninstall --global        Remove global hooks & stable binary");
    println!("  traffic-status uninstall --project [PATH] Remove hooks from a specific project");
    println!(
        "  traffic-status doctor                    Diagnose installation, hooks, and port status"
    );
    println!("  traffic-status on [LABEL]                Activate session");
    println!("  traffic-status off                       Dismiss session");
    println!("  traffic-status <green|yellow|red> [MSG]  Update status light");
    println!("  traffic-status clear                     Clear all active sessions");
    println!("  traffic-status --help                    Show this help message");
    println!("  traffic-status --version                 Print version");
    println!("\nFLAGS:");
    println!("  --dry-run                                Preview changes without modifying files");
    println!(
        "  --force, -f                              Overwrite files even if marker is missing"
    );
}

pub fn stop_daemon_if_running(dry_run: bool) {
    let port = get_configured_port();
    let addr = format!("127.0.0.1:{}", port);
    let mut stream = match TcpStream::connect_timeout(
        &addr
            .parse()
            .unwrap_or_else(|_| "127.0.0.1:8765".parse().unwrap()),
        Duration::from_millis(300),
    ) {
        Ok(s) => s,
        Err(_) => return,
    };

    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));

    let ping_req = format!(
        "GET /ping HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        port
    );
    if stream.write_all(ping_req.as_bytes()).is_err() {
        return;
    }

    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response);
    let resp_str = String::from_utf8_lossy(&response);

    // Verify it is our traffic-status app
    if !resp_str.contains("HTTP/1.1 200") || !resp_str.contains("traffic-status") {
        return;
    }

    if dry_run {
        println!("  [dry-run] Would stop running traffic-status daemon on port {port}");
        return;
    }

    // Send POST /shutdown
    let mut shutdown_stream = match TcpStream::connect_timeout(
        &addr
            .parse()
            .unwrap_or_else(|_| "127.0.0.1:8765".parse().unwrap()),
        Duration::from_millis(500),
    ) {
        Ok(s) => s,
        Err(_) => return,
    };

    let shutdown_req = format!(
        "POST /shutdown HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}",
        port
    );
    let _ = shutdown_stream.write_all(shutdown_req.as_bytes());
    let mut resp = Vec::new();
    let _ = shutdown_stream.read_to_end(&mut resp);

    // Wait up to 1.5s for process to exit
    for _ in 0..15 {
        std::thread::sleep(Duration::from_millis(100));
        if !is_daemon_alive(port) {
            println!("  🛑 Stopped running daemon on port {port}");
            return;
        }
    }
}

pub fn get_configured_port() -> u16 {
    std::env::var("TRAFFIC_STATUS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8765)
}

pub fn is_daemon_alive(port: u16) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    let mut stream = match TcpStream::connect_timeout(
        &addr
            .parse()
            .unwrap_or_else(|_| "127.0.0.1:8765".parse().unwrap()),
        Duration::from_millis(300),
    ) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));

    let req = format!(
        "GET /ping HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        port
    );
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }

    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response);
    let resp_str = String::from_utf8_lossy(&response);
    resp_str.contains("HTTP/1.1 200")
        && (resp_str.contains("traffic-status") || resp_str.contains("traffic-light"))
}

pub fn ensure_daemon_running() -> bool {
    let port = get_configured_port();
    if is_daemon_alive(port) {
        return true;
    }

    // Locate candidate executables
    let mut candidates = Vec::new();
    if let Ok(cur) = std::env::current_exe() {
        candidates.push(cur);
    }
    if let Some(stable) = get_stable_bin_path() {
        candidates.push(stable);
    }
    if let Some(home) = get_home_dir() {
        let bin_name = if cfg!(windows) {
            "traffic-status.exe"
        } else {
            "traffic-status"
        };
        candidates.push(home.join(".cargo").join("bin").join(bin_name));
    }

    for exe in candidates {
        if !exe.exists() {
            continue;
        }

        let mut cmd = std::process::Command::new(&exe);
        cmd.arg("daemon");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        if cmd.spawn().is_ok() {
            // Poll for daemon readiness
            for _ in 0..15 {
                std::thread::sleep(Duration::from_millis(100));
                if is_daemon_alive(port) {
                    return true;
                }
            }
        }
    }

    false
}

pub fn send_quick_command(action: &str, msg: &str) {
    let port = get_configured_port();
    let (endpoint, payload) = match action {
        "on" => {
            let label = if msg.is_empty() { "CLI-Session" } else { msg };
            (
                "/session/on",
                format!(
                    r#"{{"session_id":"cli-session","label":"{}","state":"green"}}"#,
                    label
                ),
            )
        }
        "off" => (
            "/session/off",
            r#"{"session_id":"cli-session"}"#.to_string(),
        ),
        "clear" => ("/clear", "{}".to_string()),
        "green" | "yellow" | "red" => (
            "/state",
            format!(
                r#"{{"session_id":"cli-session","state":"{}","message":"{}"}}"#,
                action, msg
            ),
        ),
        _ => return,
    };

    if !ensure_daemon_running() {
        eprintln!("⚠️ Unable to start or reach Traffic Status daemon on port {port}.");
        return;
    }

    let addr = format!("127.0.0.1:{}", port);
    let mut stream = match TcpStream::connect_timeout(
        &addr
            .parse()
            .unwrap_or_else(|_| "127.0.0.1:8765".parse().unwrap()),
        Duration::from_millis(1000),
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ Failed to connect to daemon: {e}");
            return;
        }
    };

    let _ = stream.set_read_timeout(Some(Duration::from_millis(1000)));
    let req = format!(
        "POST {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        endpoint,
        port,
        payload.len(),
        payload
    );

    if let Err(e) = stream.write_all(req.as_bytes()) {
        eprintln!("❌ Failed to send command: {e}");
        return;
    }

    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response);
    let resp_str = String::from_utf8_lossy(&response);

    if resp_str.contains("HTTP/1.1 200") {
        println!("✨ Traffic Status [{action}] updated successfully.");
    } else {
        eprintln!(
            "⚠️ Daemon returned non-200 response: {}",
            resp_str.lines().next().unwrap_or("")
        );
    }
}
