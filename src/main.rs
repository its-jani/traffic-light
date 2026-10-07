pub mod types;
pub mod ipc;
pub mod app;
pub mod cli;

use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui;

use crate::app::TrafficLightApp;
use crate::cli::handle_cli_args;
use crate::ipc::start_ipc_server;
use crate::types::IpcCommand;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if handle_cli_args(&args) {
        return Ok(());
    }

    let (tx, rx): (Sender<IpcCommand>, Receiver<IpcCommand>) = unbounded();

    // Start background Tokio IPC server
    let default_port = 8765;
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
                if let Err(e) = start_ipc_server(default_port, server_tx).await {
                    eprintln!("[Traffic Light] IPC Server error: {e}");
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
            .with_title("Traffic Light"),
        ..Default::default()
    };

    eframe::run_native(
        "Traffic Light",
        native_options,
        Box::new(|cc| Ok(Box::new(TrafficLightApp::new(cc, rx)))),
    )
}
