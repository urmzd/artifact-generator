use std::convert::Infallible;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::Router;
use tokio::fs;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;

const DONE_SENTINEL: &str = "__ARTIFACT_DONE__";

const SHELL_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
  <style>
    * { margin: 0; box-sizing: border-box; }
    body { background: #1e1e1e; height: 100vh; display: flex;
           flex-direction: column; font-family: system-ui; }
    #bar  { background: #2d2d2d; color: #888; padding: 6px 12px;
            font-size: 12px; border-bottom: 1px solid #333; }
    iframe { flex: 1; border: none; background: white; }
  </style>
</head>
<body>
  <div id="bar">artifact-generator — waiting for content…</div>
  <iframe id="preview"></iframe>
  <script>
    const bar = document.getElementById('bar');
    const preview = document.getElementById('preview');
    const es = new EventSource('/events');
    let count = 0;
    es.onmessage = (e) => {
      const html = JSON.parse(e.data);
      const doc = preview.contentDocument;
      doc.open(); doc.write(html); doc.close();
      bar.textContent = `artifact-generator — update #${++count}`;
    };
    es.onerror = () => { bar.textContent = 'artifact-generator — connection lost'; };
    es.addEventListener('done', () => { bar.textContent = 'artifact-generator — done'; window.close(); });
  </script>
</body>
</html>"#;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
    file_path: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file-path> [--port <port>]", args[0]);
        std::process::exit(1);
    }

    let file_path = args[1].clone();

    let mut port: u16 = 3000;
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            port = args[i + 1].parse().unwrap_or(3000);
            i += 2;
        } else {
            i += 1;
        }
    }

    let (tx, _) = broadcast::channel::<String>(16);
    let tx_watcher = tx.clone();
    let watch_path = file_path.clone();

    tokio::spawn(async move {
        let mut last_modified = None;
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            if let Ok(meta) = fs::metadata(&watch_path).await {
                let modified = meta.modified().ok();
                if modified != last_modified {
                    last_modified = modified;
                    if let Ok(content) = fs::read_to_string(&watch_path).await {
                        let _ = tx_watcher.send(content);
                    }
                }
            }
        }
    });

    let state = AppState { tx, file_path };

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/events", get(sse_handler))
        .route("/content", get(content_handler))
        .route("/done", post(done_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    println!("Watching file, serving at http://localhost:{port}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root_handler() -> Html<&'static str> {
    Html(SHELL_HTML)
}

async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| {
        let content = msg.ok()?;
        if content == DONE_SENTINEL {
            Some(Ok(Event::default().event("done").data("done")))
        } else {
            let data = serde_json::to_string(&content).unwrap_or_default();
            Some(Ok(Event::default().data(data)))
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn done_handler(State(state): State<AppState>) -> StatusCode {
    let _ = state.tx.send(DONE_SENTINEL.to_string());
    StatusCode::OK
}

async fn content_handler(State(state): State<AppState>) -> impl IntoResponse {
    match fs::read_to_string(&state.file_path).await {
        Ok(content) => content,
        Err(_) => String::from("(file not found)"),
    }
}
