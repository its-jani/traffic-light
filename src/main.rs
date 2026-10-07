pub mod app;
pub mod cli;
pub mod ipc;
pub mod types;

use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui;

use crate::app::TrafficStatusApp;
use crate::cli::handle_cli_args;
use crate::ipc::start_ipc_server;
use crate::types::IpcCommand;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if handle_cli_args(&args) {
        return Ok(());
    }

    let port: u16 = std::env::var("TRAFFIC_STATUS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8765);

    // Test if already running before opening window
    if cli::is_daemon_alive(port) {
        println!("🚦 Traffic Status daemon is already running on port {port}.");
        return Ok(());
    }

    let (tx, rx): (Sender<IpcCommand>, Receiver<IpcCommand>) = unbounded();

    // Start background Tokio IPC server
    let server_tx = tx.clone();
    std::thread::Builder::new()
        .name("ipc-server".to_string())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime");
            rt.block_on(async move {
                if let Err(e) = start_ipc_server(port, server_tx).await {
                    eprintln!("[Traffic Status] IPC Server error on port {port}: {e}");
                }
            });
        })
        .expect("Failed to spawn IPC thread");

    // Eframe window options: Authentic vertical floating widget
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([88.0, 190.0])
            .with_min_inner_size([70.0, 140.0])
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_title("Traffic Status"),
        ..Default::default()
    };

    eframe::run_native(
        "Traffic Status",
        native_options,
        Box::new(|cc| Ok(Box::new(TrafficStatusApp::new(cc, rx)))),
    )
}
