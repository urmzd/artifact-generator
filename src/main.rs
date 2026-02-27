use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use artifact_generator::telemetry;
use artifact_generator::{spawn_file_watcher, spawn_render_thread, RenderMsg};
use tokio::sync::broadcast;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let guard = telemetry::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input.html> [--output output.pdf]", args[0]);
        std::process::exit(1);
    }

    let html_path = PathBuf::from(&args[1]);

    let mut pdf_path: Option<PathBuf> = None;
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--output" && i + 1 < args.len() {
            pdf_path = Some(PathBuf::from(&args[i + 1]));
            i += 2;
        } else {
            i += 1;
        }
    }

    let pdf_path = pdf_path.unwrap_or_else(|| html_path.with_extension("pdf"));

    info!(html = %html_path.display(), pdf = %pdf_path.display(), "watching");

    // Broadcast channel for file watcher -> main loop
    let (tx, mut rx) = broadcast::channel::<String>(16);
    spawn_file_watcher(tx, html_path.display().to_string(), Duration::from_millis(100));

    // mpsc channel for main loop -> render thread
    let (render_tx, render_rx) = mpsc::channel::<RenderMsg>();
    let render_handle = spawn_render_thread(render_rx, html_path.clone(), pdf_path.clone());

    let metrics = telemetry::Metrics::get();

    // Main loop: forward broadcast events to the render thread
    let forward = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(_) => {
                    if render_tx.send(RenderMsg::Trigger).is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!(lagged = n, "watcher lagged, rendering latest");
                    metrics
                        .broadcast_lag_count
                        .fetch_add(n, std::sync::atomic::Ordering::Relaxed);
                    if render_tx.send(RenderMsg::Trigger).is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    info!("shutting down");

    forward.abort();
    let _ = render_handle.join();

    guard.shutdown();

    Ok(())
}
