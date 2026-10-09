//! Hook installation, uninstallation and diagnostics.
//!
//! Everything that writes files under `~/.claude`, `~/.config/opencode`,
//! `~/.opencode` or the stable per-user binary directory lives here. The
//! CLI entry point in [`crate::cli`] calls into these functions.

use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::{get_configured_port, is_daemon_alive, stop_daemon_if_running};

pub const MARKER_MD: &str = "<!-- generated-by: traffic-status -->";
pub const MARKER_JS: &str = "// generated-by: traffic-status";

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
