use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{Context, Result};
use headless_chrome::{Browser, LaunchOptions};
use tokio::fs;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

/// Watches a file for changes and broadcasts the content on each modification.
pub fn spawn_file_watcher(
    tx: broadcast::Sender<String>,
    file_path: String,
    interval: Duration,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut last_modified = None;
        let mut tick = tokio::time::interval(interval);
        loop {
            tick.tick().await;
            if let Ok(meta) = fs::metadata(&file_path).await {
                let modified = meta.modified().ok();
                if modified != last_modified {
                    last_modified = modified;
                    if let Ok(content) = fs::read_to_string(&file_path).await {
                        let _ = tx.send(content);
                    }
                }
            }
        }
    })
}

/// Renders HTML files to PDF using a persistent headless Chrome instance.
pub struct PdfRenderer {
    browser: Browser,
}

impl PdfRenderer {
    /// Launch a headless Chrome browser.
    pub fn new() -> Result<Self> {
        let options = LaunchOptions {
            headless: true,
            ..LaunchOptions::default()
        };
        let browser = Browser::new(options).context("failed to launch headless Chrome")?;
        Ok(Self { browser })
    }

    /// Navigate to `html_path` (as a file:// URL) and write a PDF to `pdf_path`.
    pub fn render(&self, html_path: &Path, pdf_path: &Path) -> Result<()> {
        let abs = std::fs::canonicalize(html_path)
            .with_context(|| format!("cannot resolve {}", html_path.display()))?;
        let url = format!("file://{}", abs.display());

        let tab = self
            .browser
            .new_tab()
            .context("failed to open new Chrome tab")?;

        tab.navigate_to(&url)
            .context("navigation failed")?
            .wait_until_navigated()
            .context("waiting for navigation failed")?;

        let pdf_data = tab
            .print_to_pdf(None)
            .context("print_to_pdf failed")?;

        std::fs::write(pdf_path, &pdf_data)
            .with_context(|| format!("failed to write {}", pdf_path.display()))?;

        tab.close(true).ok();

        Ok(())
    }
}

/// Messages sent to the render thread.
pub enum RenderMsg {
    /// A file change was detected; re-render.
    Trigger,
    /// Shut down the render thread.
    Shutdown,
}

/// Spawn a dedicated OS thread that owns a `PdfRenderer` and processes
/// render requests arriving on `rx`.
pub fn spawn_render_thread(
    rx: mpsc::Receiver<RenderMsg>,
    html_path: std::path::PathBuf,
    pdf_path: std::path::PathBuf,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let renderer = match PdfRenderer::new() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to start Chrome: {e:#}");
                return;
            }
        };

        let mut render_count: u64 = 0;

        for msg in rx {
            match msg {
                RenderMsg::Trigger => {
                    render_count += 1;
                    match renderer.render(&html_path, &pdf_path) {
                        Ok(()) => {
                            eprintln!("render #{render_count} → {}", pdf_path.display());
                        }
                        Err(e) => {
                            eprintln!("render #{render_count} failed: {e:#}");
                        }
                    }
                }
                RenderMsg::Shutdown => break,
            }
        }
    })
}
