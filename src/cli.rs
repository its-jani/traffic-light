use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const MARKER_MD: &str = "<!-- generated-by: traffic-status -->";
pub const MARKER_JS: &str = "// generated-by: traffic-status";

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

pub fn has_our_marker(content: &str) -> bool {
    content.contains("generated-by: traffic-status")
        || content.contains("Traffic Light Slash Command (/traffic)")
        || content.contains("Traffic Light Command (/traffic)")
        || content.contains("TrafficLightPlugin")
        || content.contains("TrafficStatusPlugin")
        || content.contains("generated-by: traffic-light")
}

pub fn get_home_dir() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("USERPROFILE") {
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    if let Ok(p) = std::env::var("HOME") {
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    dirs::home_dir()
}

pub fn get_stable_bin_dir() -> Option<PathBuf> {
    if cfg!(windows) {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            if !local_app_data.is_empty() {
                return Some(
                    PathBuf::from(local_app_data)
                        .join("traffic-status")
                        .join("bin"),
                );
            }
        }
        dirs::data_local_dir().map(|d| d.join("traffic-status").join("bin"))
    } else {
        if let Some(home) = get_home_dir() {
            return Some(
                home.join(".local")
                    .join("share")
                    .join("traffic-status")
                    .join("bin"),
            );
        }
        dirs::data_dir().map(|d| d.join("traffic-status").join("bin"))
    }
}

pub fn get_stable_bin_path() -> Option<PathBuf> {
    let bin_name = if cfg!(windows) {
        "traffic-status.exe"
    } else {
        "traffic-status"
    };
    get_stable_bin_dir().map(|d| d.join(bin_name))
}

fn copy_self_to_stable_location(dry_run: bool) {
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };

    let target_path = match get_stable_bin_path() {
        Some(p) => p,
        None => return,
    };

    // If already running from stable path, skip
    if let (Ok(c), Ok(t)) = (current_exe.canonicalize(), target_path.canonicalize()) {
        if c == t {
            println!(
                "  ℹ️ Already running from stable location: {}",
                target_path.display()
            );
            return;
        }
    }

    let target_dir = match target_path.parent() {
        Some(d) => d,
        None => return,
    };

    if dry_run {
        println!(
            "  [dry-run] Would copy binary to: {}",
            target_path.display()
        );
        return;
    }

    if let Err(e) = fs::create_dir_all(target_dir) {
        eprintln!(
            "  ⚠️ Could not create stable binary directory {}: {e}",
            target_dir.display()
        );
        return;
    }

    match fs::copy(&current_exe, &target_path) {
        Ok(_) => {
            println!(
                "  ✅ Copied native binary to stable path: {}",
                target_path.display()
            );
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&target_path, fs::Permissions::from_mode(0o755));
            }
        }
        Err(e) => {
            eprintln!(
                "  ⚠️ Could not copy binary to {}: {e}",
                target_path.display()
            );
        }
    }
}

pub fn safe_write_file(
    path: &Path,
    content: &str,
    dry_run: bool,
    force: bool,
) -> Result<bool, String> {
    if path.exists() {
        match fs::read_to_string(path) {
            Ok(existing) => {
                if !force && !has_our_marker(&existing) {
                    return Err(format!(
                        "File '{}' exists and was not created by traffic-status. Use --force to overwrite.",
                        path.display()
                    ));
                }
                if existing == content {
                    // Already up to date
                    return Ok(false);
                }
            }
            Err(e) => {
                if !force {
                    return Err(format!(
                        "Could not read existing file '{}': {e}",
                        path.display()
                    ));
                }
            }
        }
    }

    if dry_run {
        println!("  [dry-run] Would write: {}", path.display());
        return Ok(true);
    }

    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!(
                "Failed to create directory '{}': {e}",
                parent.display()
            ));
        }
    }

    if let Err(e) = fs::write(path, content) {
        return Err(format!("Failed to write '{}': {e}", path.display()));
    }

    Ok(true)
}

pub fn safe_remove_file(path: &Path, dry_run: bool, force: bool) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }

    if !force {
        match fs::read_to_string(path) {
            Ok(content) => {
                if !has_our_marker(&content) {
                    return Err(format!(
                        "Skipping '{}': file does not contain traffic-status marker.",
                        path.display()
                    ));
                }
            }
            Err(e) => {
                return Err(format!("Could not read file '{}': {e}", path.display()));
            }
        }
    }

    if dry_run {
        println!("  [dry-run] Would remove: {}", path.display());
        return Ok(true);
    }

    if let Err(e) = fs::remove_file(path) {
        return Err(format!("Failed to remove '{}': {e}", path.display()));
    }

    Ok(true)
}

pub fn safe_remove_empty_dir(dir: &Path, dry_run: bool) {
    if !dir.exists() || !dir.is_dir() {
        return;
    }

    if let Ok(mut entries) = fs::read_dir(dir) {
        if entries.next().is_none() {
            if dry_run {
                println!(
                    "  [dry-run] Would remove empty directory: {}",
                    dir.display()
                );
            } else {
                let _ = fs::remove_dir(dir);
            }
        }
    }
}

pub fn install_project(target: &Path, dry_run: bool, force: bool) {
    let target_dir = if target.is_relative() {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(target)
    } else {
        target.to_path_buf()
    };

    if dry_run {
        println!(
            "🚦 [dry-run] Installing Traffic Status into project: {}",
            target_dir.display()
        );
    } else {
        println!(
            "🚦 Installing Traffic Status into project: {}",
            target_dir.display()
        );
    }

    let claude_cmd_content = include_str!("../.claude/commands/traffic.md");
    let opencode_cmd_content = include_str!("../.opencode/commands/traffic.md");
    let opencode_plugin_content = include_str!("../.opencode/plugins/traffic-status.js");

    // 1. Claude Code
    let claude_file = target_dir
        .join(".claude")
        .join("commands")
        .join("traffic.md");
    match safe_write_file(&claude_file, claude_cmd_content, dry_run, force) {
        Ok(true) => {
            if !dry_run {
                println!("  ✅ Installed .claude/commands/traffic.md");
            }
        }
        Ok(false) => {
            if !dry_run {
                println!("  ℹ️ .claude/commands/traffic.md is already up to date");
            }
        }
        Err(e) => eprintln!("  ❌ {e}"),
    }

    // 2. OpenCode
    let opencode_cmd = target_dir
        .join(".opencode")
        .join("commands")
        .join("traffic.md");
    let opencode_plugin = target_dir
        .join(".opencode")
        .join("plugins")
        .join("traffic-status.js");

    match safe_write_file(&opencode_cmd, opencode_cmd_content, dry_run, force) {
        Ok(true) => {
            if !dry_run {
                println!("  ✅ Installed .opencode/commands/traffic.md");
            }
        }
        Ok(false) => {
            if !dry_run {
                println!("  ℹ️ .opencode/commands/traffic.md is already up to date");
            }
        }
        Err(e) => eprintln!("  ❌ {e}"),
    }

    match safe_write_file(&opencode_plugin, opencode_plugin_content, dry_run, force) {
        Ok(true) => {
            if !dry_run {
                println!("  ✅ Installed .opencode/plugins/traffic-status.js");
            }
        }
        Ok(false) => {
            if !dry_run {
                println!("  ℹ️ .opencode/plugins/traffic-status.js is already up to date");
            }
        }
        Err(e) => eprintln!("  ❌ {e}"),
    }

    // Clean legacy project files if present
    let old_plugin = target_dir
        .join(".opencode")
        .join("plugins")
        .join("traffic-light.js");
    let _ = safe_remove_file(&old_plugin, dry_run, false);

    if !dry_run {
        println!(
            "\n✨ Project installation complete! (Scope: {})",
            target_dir.display()
        );
    }
}

pub fn install_global(dry_run: bool, force: bool) {
    if dry_run {
        println!("🚦 [dry-run] Installing Traffic Status globally (All Projects)...");
    } else {
        println!("🚦 Installing Traffic Status globally (All Projects)...");
    }

    let home = match get_home_dir() {
        Some(h) => h,
        None => {
            eprintln!("❌ Unable to determine home directory.");
            return;
        }
    };

    // Copy binary to stable per-user location
    copy_self_to_stable_location(dry_run);

    let claude_cmd_content = include_str!("../.claude/commands/traffic.md");
    let opencode_cmd_content = include_str!("../.opencode/commands/traffic.md");
    let opencode_plugin_content = include_str!("../.opencode/plugins/traffic-status.js");

    // 1. Global Claude Code (~/.claude/commands/traffic.md)
    let global_claude = home.join(".claude").join("commands").join("traffic.md");
    match safe_write_file(&global_claude, claude_cmd_content, dry_run, force) {
        Ok(true) => {
            if !dry_run {
                println!(
                    "  ✅ Global Claude Code command installed: {}",
                    global_claude.display()
                );
            }
        }
        Ok(false) => {
            if !dry_run {
                println!("  ℹ️ Global Claude Code command is already up to date");
            }
        }
        Err(e) => eprintln!("  ❌ {e}"),
    }

    // 2. Global OpenCode (~/.config/opencode and ~/.opencode)
    let targets = [
        home.join(".config").join("opencode"),
        home.join(".opencode"),
    ];
    for base in &targets {
        let cmd_file = base.join("commands").join("traffic.md");
        let plugin_file = base.join("plugins").join("traffic-status.js");

        match safe_write_file(&cmd_file, opencode_cmd_content, dry_run, force) {
            Ok(true) => {
                if !dry_run {
                    println!(
                        "  ✅ Global OpenCode command installed: {}",
                        cmd_file.display()
                    );
                }
            }
            Ok(false) => {
                if !dry_run {
                    println!(
                        "  ℹ️ Global OpenCode command is up to date: {}",
                        cmd_file.display()
                    );
                }
            }
            Err(e) => eprintln!("  ❌ {e}"),
        }

        match safe_write_file(&plugin_file, opencode_plugin_content, dry_run, force) {
            Ok(true) => {
                if !dry_run {
                    println!(
                        "  ✅ Global OpenCode plugin installed: {}",
                        plugin_file.display()
                    );
                }
            }
            Ok(false) => {
                if !dry_run {
                    println!(
                        "  ℹ️ Global OpenCode plugin is up to date: {}",
                        plugin_file.display()
                    );
                }
            }
            Err(e) => eprintln!("  ❌ {e}"),
        }

        // Clean legacy files in global opencode
        let old_plugin = base.join("plugins").join("traffic-light.js");
        let _ = safe_remove_file(&old_plugin, dry_run, false);
    }

    // Clean legacy binary if present and marked
    clean_legacy_installations(dry_run);

    if !dry_run {
        println!("\n🎉 Global Installation Complete!");
        println!("Scope: Machine-Wide (Current & Future Projects)");
    }
}

pub fn uninstall_project(target: &Path, dry_run: bool, force: bool) {
    let target_dir = if target.is_relative() {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(target)
    } else {
        target.to_path_buf()
    };

    if dry_run {
        println!(
            "🗑️  [dry-run] Removing Traffic Status from project: {}",
            target_dir.display()
        );
    } else {
        println!(
            "🗑️  Removing Traffic Status from project: {}",
            target_dir.display()
        );
    }

    // Stop daemon if running
    stop_daemon_if_running(dry_run);

    let claude_file = target_dir
        .join(".claude")
        .join("commands")
        .join("traffic.md");
    if claude_file.exists() {
        match safe_remove_file(&claude_file, dry_run, force) {
            Ok(true) => {
                if !dry_run {
                    println!("  🗑️  Removed .claude/commands/traffic.md");
                }
                safe_remove_empty_dir(&target_dir.join(".claude").join("commands"), dry_run);
                safe_remove_empty_dir(&target_dir.join(".claude"), dry_run);
            }
            Ok(false) => {}
            Err(e) => eprintln!("  ⚠️ {e}"),
        }
    }

    let opencode_cmd = target_dir
        .join(".opencode")
        .join("commands")
        .join("traffic.md");
    if opencode_cmd.exists() {
        match safe_remove_file(&opencode_cmd, dry_run, force) {
            Ok(true) => {
                if !dry_run {
                    println!("  🗑️  Removed .opencode/commands/traffic.md");
                }
                safe_remove_empty_dir(&target_dir.join(".opencode").join("commands"), dry_run);
            }
            Ok(false) => {}
            Err(e) => eprintln!("  ⚠️ {e}"),
        }
    }

    let opencode_plugin = target_dir
        .join(".opencode")
        .join("plugins")
        .join("traffic-status.js");
    if opencode_plugin.exists() {
        match safe_remove_file(&opencode_plugin, dry_run, force) {
            Ok(true) => {
                if !dry_run {
                    println!("  🗑️  Removed .opencode/plugins/traffic-status.js");
                }
                safe_remove_empty_dir(&target_dir.join(".opencode").join("plugins"), dry_run);
            }
            Ok(false) => {}
            Err(e) => eprintln!("  ⚠️ {e}"),
        }
    }

    // Remove legacy plugin if any
    let old_plugin = target_dir
        .join(".opencode")
        .join("plugins")
        .join("traffic-light.js");
    if old_plugin.exists() {
        let _ = safe_remove_file(&old_plugin, dry_run, force);
    }

    safe_remove_empty_dir(&target_dir.join(".opencode"), dry_run);

    if !dry_run {
        println!("\n✅ Removed from project: {}", target_dir.display());
    }
}

pub fn uninstall_global(dry_run: bool, force: bool) {
    if dry_run {
        println!("🗑️  [dry-run] Removing Traffic Status globally...");
    } else {
        println!("🗑️  Removing Traffic Status globally...");
    }

    let home = match get_home_dir() {
        Some(h) => h,
        None => {
            eprintln!("❌ Unable to determine home directory.");
            return;
        }
    };

    // Stop daemon if running
    stop_daemon_if_running(dry_run);

    // 1. Remove Claude global command
    let global_claude = home.join(".claude").join("commands").join("traffic.md");
    if global_claude.exists() {
        match safe_remove_file(&global_claude, dry_run, force) {
            Ok(true) => {
                if !dry_run {
                    println!("  🗑️  Removed global Claude Code command");
                }
                safe_remove_empty_dir(&home.join(".claude").join("commands"), dry_run);
            }
            Ok(false) => {}
            Err(e) => eprintln!("  ⚠️ {e}"),
        }
    }

    // 2. Remove OpenCode global configs (individual files only!)
    let targets = [
        home.join(".config").join("opencode"),
        home.join(".opencode"),
    ];
    for base in &targets {
        let cmd = base.join("commands").join("traffic.md");
        let plugin = base.join("plugins").join("traffic-status.js");
        let old_plugin = base.join("plugins").join("traffic-light.js");

        if cmd.exists() {
            let _ = safe_remove_file(&cmd, dry_run, force);
            safe_remove_empty_dir(&base.join("commands"), dry_run);
        }
        if plugin.exists() {
            let _ = safe_remove_file(&plugin, dry_run, force);
            safe_remove_empty_dir(&base.join("plugins"), dry_run);
        }
        if old_plugin.exists() {
            let _ = safe_remove_file(&old_plugin, dry_run, force);
            safe_remove_empty_dir(&base.join("plugins"), dry_run);
        }
        safe_remove_empty_dir(base, dry_run);
        if !dry_run {
            println!(
                "  🗑️  Cleaned Traffic Status entries from {}",
                base.display()
            );
        }
    }

    // 3. Remove stable binary
    if let Some(stable_bin) = get_stable_bin_path() {
        if stable_bin.exists() {
            if dry_run {
                println!("  [dry-run] Would remove binary: {}", stable_bin.display());
            } else {
                let _ = fs::remove_file(&stable_bin);
                println!("  🗑️  Removed binary from {}", stable_bin.display());
                if let Some(parent) = stable_bin.parent() {
                    safe_remove_empty_dir(parent, dry_run);
                    if let Some(grandparent) = parent.parent() {
                        safe_remove_empty_dir(grandparent, dry_run);
                    }
                }
            }
        }
    }

    // 4. Legacy cleanups
    clean_legacy_installations(dry_run);

    if !dry_run {
        println!("\n✨ Traffic Status global uninstallation complete.");
    }
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

fn clean_legacy_installations(dry_run: bool) {
    if let Some(home) = get_home_dir() {
        // Old cargo binary
        let old_cargo_bin = home.join(".cargo").join("bin").join(if cfg!(windows) {
            "traffic-light.exe"
        } else {
            "traffic-light"
        });
        if old_cargo_bin.exists() {
            if dry_run {
                println!(
                    "  [dry-run] Would remove legacy binary: {}",
                    old_cargo_bin.display()
                );
            } else {
                let _ = fs::remove_file(&old_cargo_bin);
                println!(
                    "  🗑️  Removed legacy binary from {}",
                    old_cargo_bin.display()
                );
            }
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

pub fn run_doctor() {
    let mut has_errors = false;
    println!("🩺 Traffic Status Diagnostics");
    println!("=============================");

    // 1. Version & Executable
    println!(
        "• Version:     traffic-status v{}",
        env!("CARGO_PKG_VERSION")
    );
    if let Ok(exe) = std::env::current_exe() {
        println!("• Binary Path: {}", exe.display());
    }

    // 2. Stable per-user binary
    if let Some(stable) = get_stable_bin_path() {
        if stable.exists() {
            println!("• Stable Binary: Installed ({})", stable.display());
        } else {
            println!("• Stable Binary: Not installed ({})", stable.display());
        }
    }

    // 3. PATH check
    let bin_name = if cfg!(windows) {
        "traffic-status.exe"
    } else {
        "traffic-status"
    };
    let on_path = is_on_path(bin_name);
    if on_path {
        println!("• PATH Status:   'traffic-status' found on PATH");
    } else {
        println!("• PATH Status:   'traffic-status' NOT found on PATH (hooks will use stable per-user binary)");
    }

    // 4. Daemon & Port
    let port = get_configured_port();
    println!("• Config Port:   {port} (TRAFFIC_STATUS_PORT)");
    if is_daemon_alive(port) {
        println!("• Daemon:        Running and responding on http://127.0.0.1:{port}");
    } else {
        // Test if port is available
        match std::net::TcpListener::bind(format!("127.0.0.1:{port}")) {
            Ok(_) => println!("• Daemon:        Not running (port {port} is free)"),
            Err(e) => {
                println!("• Daemon:        Port {port} is occupied by another process: {e}");
                has_errors = true;
            }
        }
    }

    // 5. Hooks Check
    if let Some(home) = get_home_dir() {
        let global_claude = home.join(".claude").join("commands").join("traffic.md");
        if global_claude.exists() {
            if let Ok(content) = fs::read_to_string(&global_claude) {
                if has_our_marker(&content) {
                    println!(
                        "• Global Claude: Installed & Verified ({})",
                        global_claude.display()
                    );
                } else {
                    println!(
                        "• Global Claude: Present but missing marker ({})",
                        global_claude.display()
                    );
                }
            }
        } else {
            println!("• Global Claude: Not installed");
        }

        let opencode_targets = [
            home.join(".config").join("opencode"),
            home.join(".opencode"),
        ];
        let mut opencode_installed = false;
        for base in &opencode_targets {
            let cmd = base.join("commands").join("traffic.md");
            let plugin = base.join("plugins").join("traffic-status.js");
            if cmd.exists() || plugin.exists() {
                opencode_installed = true;
                println!("• Global OpenCode: Found in {}", base.display());
            }
        }
        if !opencode_installed {
            println!("• Global OpenCode: Not installed");
        }
    }

    // 6. OS & Session Info
    println!(
        "• OS / Arch:     {} / {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    #[cfg(unix)]
    {
        if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
            println!("• Session Type:  {session_type}");
            if session_type.to_lowercase() == "wayland" {
                println!("  ⚠️ Wayland detected: compositor may limit 'always-on-top' behavior. Run with WINIT_UNIX_BACKEND=x11 if needed.");
            }
        }
    }

    if has_errors {
        std::process::exit(1);
    }
}

fn is_on_path(bin_name: &str) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            if dir.join(bin_name).exists() {
                return true;
            }
        }
    }
    false
}
