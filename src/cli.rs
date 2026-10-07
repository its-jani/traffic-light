use std::fs;
use std::path::{Path, PathBuf};

pub fn handle_cli_args(args: &[String]) -> bool {
    if args.len() <= 1 {
        return false; // Launch GUI app
    }

    match args[1].as_str() {
        "install" => {
            let is_global = args.iter().any(|a| a == "--global" || a == "-g");
            let project_idx = args.iter().position(|a| a == "--project" || a == "-p");
            
            if is_global {
                install_global();
            } else if let Some(idx) = project_idx {
                let path = args.get(idx + 1).map(|s| s.as_str()).unwrap_or(".");
                install_project(Path::new(path));
            } else if args.len() > 2 {
                install_project(Path::new(&args[2]));
            } else {
                install_project(Path::new("."));
            }
            true
        }
        "uninstall" => {
            let is_global = args.iter().any(|a| a == "--global" || a == "-g" || a == "--all");
            let project_idx = args.iter().position(|a| a == "--project" || a == "-p");

            if is_global {
                uninstall_global();
            } else if let Some(idx) = project_idx {
                let path = args.get(idx + 1).map(|s| s.as_str()).unwrap_or(".");
                uninstall_project(Path::new(path));
            } else if args.len() > 2 {
                uninstall_project(Path::new(&args[2]));
            } else {
                uninstall_project(Path::new("."));
            }
            true
        }
        "on" | "off" | "state" | "green" | "yellow" | "red" => {
            let action = &args[1];
            let msg = args.get(2).map(|s| s.as_str()).unwrap_or("");
            send_quick_command(action, msg);
            true
        }
        "--help" | "-h" | "help" => {
            print_help();
            true
        }
        "--version" | "-v" => {
            println!("traffic-light v{}", env!("CARGO_PKG_VERSION"));
            true
        }
        "run" | "gui" => {
            false // Launch GUI app
        }
        _ => {
            eprintln!("Unknown command: '{}'", args[1]);
            print_help();
            true
        }
    }
}

fn print_help() {
    println!("🚦 Traffic Light v{}", env!("CARGO_PKG_VERSION"));
    println!("A lightweight floating desktop traffic light for AI coding agents.\n");
    println!("USAGE:");
    println!("  traffic-light                           Launch the floating traffic light UI");
    println!("  traffic-light install --project [PATH]  Install commands into a specific project");
    println!("  traffic-light install --global          Install globally for all current and future projects");
    println!("  traffic-light uninstall --project [PATH] Remove from a specific project");
    println!("  traffic-light uninstall --global        Completely remove all traces from this machine");
    println!("  traffic-light on [LABEL]                Activate session");
    println!("  traffic-light off                       Dismiss session");
    println!("  traffic-light <green|yellow|red> [MSG]  Update status light");
    println!("  traffic-light --help                    Show this help message");
}

fn install_project(target: &Path) {
    let target_dir = if target.is_relative() {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(target)
    } else {
        target.to_path_buf()
    };

    println!("🚦 Installing Traffic Light into project: {}", target_dir.display());

    let claude_cmd_content = include_str!("../.claude/commands/traffic.md");
    let opencode_cmd_content = include_str!("../.opencode/commands/traffic.md");
    let opencode_plugin_content = include_str!("../.opencode/plugins/traffic-light.js");

    // 1. Claude Code
    let claude_dir = target_dir.join(".claude").join("commands");
    if let Err(e) = fs::create_dir_all(&claude_dir) {
        eprintln!("  ❌ Failed to create {}: {e}", claude_dir.display());
    } else if let Err(e) = fs::write(claude_dir.join("traffic.md"), claude_cmd_content) {
        eprintln!("  ❌ Failed to write claude command: {e}");
    } else {
        println!("  ✅ Created .claude/commands/traffic.md");
    }

    // 2. OpenCode
    let opencode_cmd_dir = target_dir.join(".opencode").join("commands");
    let opencode_plugin_dir = target_dir.join(".opencode").join("plugins");

    let _ = fs::create_dir_all(&opencode_cmd_dir);
    let _ = fs::create_dir_all(&opencode_plugin_dir);

    if let Err(e) = fs::write(opencode_cmd_dir.join("traffic.md"), opencode_cmd_content) {
        eprintln!("  ❌ Failed to write opencode command: {e}");
    } else {
        println!("  ✅ Created .opencode/commands/traffic.md");
    }

    if let Err(e) = fs::write(opencode_plugin_dir.join("traffic-light.js"), opencode_plugin_content) {
        eprintln!("  ❌ Failed to write opencode plugin: {e}");
    } else {
        println!("  ✅ Created .opencode/plugins/traffic-light.js");
    }

    println!("\n✨ Project installation complete! (Scope: {})", target_dir.display());
}

fn install_global() {
    println!("🚦 Installing Traffic Light globally (All Projects)...");

    let home = match dirs_home() {
        Some(h) => h,
        None => {
            eprintln!("❌ Unable to determine home directory.");
            return;
        }
    };

    let claude_cmd_content = include_str!("../.claude/commands/traffic.md");
    let opencode_cmd_content = include_str!("../.opencode/commands/traffic.md");
    let opencode_plugin_content = include_str!("../.opencode/plugins/traffic-light.js");

    // 1. Global Claude Code (~/.claude/commands)
    let global_claude = home.join(".claude").join("commands");
    let _ = fs::create_dir_all(&global_claude);
    if let Ok(_) = fs::write(global_claude.join("traffic.md"), claude_cmd_content) {
        println!("  ✅ Global Claude Code command installed: {}", global_claude.join("traffic.md").display());
    }

    // 2. Global OpenCode (~/.config/opencode and ~/.opencode)
    let targets = [home.join(".config").join("opencode"), home.join(".opencode")];
    for base in &targets {
        let cmd_dir = base.join("commands");
        let plugin_dir = base.join("plugins");
        let _ = fs::create_dir_all(&cmd_dir);
        let _ = fs::create_dir_all(&plugin_dir);

        let _ = fs::write(cmd_dir.join("traffic.md"), opencode_cmd_content);
        let _ = fs::write(plugin_dir.join("traffic-light.js"), opencode_plugin_content);
        println!("  ✅ Global OpenCode configs installed in: {}", base.display());
    }

    println!("\n🎉 Global Installation Complete!");
    println!("Scope: Machine-Wide (Current & Future Projects)");
}

fn uninstall_project(target: &Path) {
    let target_dir = if target.is_relative() {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(target)
    } else {
        target.to_path_buf()
    };

    println!("🗑️  Removing Traffic Light from project: {}", target_dir.display());

    let claude_file = target_dir.join(".claude").join("commands").join("traffic.md");
    if claude_file.exists() {
        let _ = fs::remove_file(claude_file);
        println!("  🗑️  Removed .claude/commands/traffic.md");
    }

    let opencode_cmd = target_dir.join(".opencode").join("commands").join("traffic.md");
    if opencode_cmd.exists() {
        let _ = fs::remove_file(opencode_cmd);
        println!("  🗑️  Removed .opencode/commands/traffic.md");
    }

    let opencode_plugin = target_dir.join(".opencode").join("plugins").join("traffic-light.js");
    if opencode_plugin.exists() {
        let _ = fs::remove_file(opencode_plugin);
        println!("  🗑️  Removed .opencode/plugins/traffic-light.js");
    }

    println!("\n✅ Removed from project: {}", target_dir.display());
}

fn uninstall_global() {
    println!("🚨 Purging Traffic Light completely from laptop...");

    let home = match dirs_home() {
        Some(h) => h,
        None => {
            eprintln!("❌ Unable to determine home directory.");
            return;
        }
    };

    // Remove Claude global
    let global_claude = home.join(".claude").join("commands").join("traffic.md");
    if global_claude.exists() {
        let _ = fs::remove_file(global_claude);
        println!("  🗑️  Removed global Claude Code commands");
    }

    // Remove OpenCode globals
    let targets = [home.join(".config").join("opencode"), home.join(".opencode")];
    for base in &targets {
        let cmd = base.join("commands").join("traffic.md");
        let plugin = base.join("plugins").join("traffic-light.js");
        if cmd.exists() { let _ = fs::remove_file(cmd); }
        if plugin.exists() { let _ = fs::remove_file(plugin); }
        println!("  🗑️  Cleaned {}", base.display());
    }

    // Remove cargo binary if present
    let cargo_bin = home.join(".cargo").join("bin").join(if cfg!(windows) { "traffic-light.exe" } else { "traffic-light" });
    if cargo_bin.exists() {
        let _ = fs::remove_file(cargo_bin);
        println!("  🗑️  Removed binary from ~/.cargo/bin");
    }

    println!("\n✨ Traffic Light has been completely purged from your system.");
}

fn dirs_home() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("USERPROFILE") {
        return Some(PathBuf::from(p));
    }
    if let Ok(p) = std::env::var("HOME") {
        return Some(PathBuf::from(p));
    }
    None
}

fn send_quick_command(action: &str, msg: &str) {
    let url = match action {
        "on" => {
            let label = if msg.is_empty() { "CLI-Session" } else { msg };
            format!("http://127.0.0.1:8765/session/on?session_id=cli-session&label={label}&state=green")
        }
        "off" => "http://127.0.0.1:8765/session/off?session_id=cli-session".to_string(),
        "green" | "yellow" | "red" => {
            format!("http://127.0.0.1:8765/state?session_id=cli-session&state={action}&message={msg}")
        }
        _ => return,
    };

    println!("📡 Sending request: {url}");
    // Use std::net or tokio or curl fallback
    let _ = std::process::Command::new("curl.exe")
        .args(["-s", &url])
        .output();
}
