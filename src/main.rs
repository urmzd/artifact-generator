use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use artifact_generator::{spawn_file_watcher, spawn_render_thread, RenderMsg};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    eprintln!("Watching: {}", html_path.display());
    eprintln!("Output:   {}", pdf_path.display());

    // Broadcast channel for file watcher -> main loop
    let (tx, mut rx) = broadcast::channel::<String>(16);
    spawn_file_watcher(tx, html_path.display().to_string(), Duration::from_millis(100));

    // mpsc channel for main loop -> render thread
    let (render_tx, render_rx) = mpsc::channel::<RenderMsg>();
    let render_handle = spawn_render_thread(render_rx, html_path.clone(), pdf_path.clone());

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
                    eprintln!("watcher lagged by {n} messages, rendering latest");
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
    eprintln!("\nShutting down...");

    // Aborting the forward task drops render_tx, which closes the mpsc channel.
    // The render thread's `for msg in rx` loop then exits naturally.
    forward.abort();
    let _ = render_handle.join();

    Ok(())
}
