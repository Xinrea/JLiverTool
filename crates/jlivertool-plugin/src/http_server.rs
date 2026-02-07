//! HTTP server for serving plugin files
//!
//! Serves plugin files from the plugins directory and the jliver-api.js script.
//! Automatically injects the jliver-api.js script into HTML files.

use anyhow::Result;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Response, StatusCode},
    routing::get,
    Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

/// The jliver-api.js script content (embedded at compile time)
const JLIVER_API_JS: &str = include_str!("jliver-api.js");

/// Shared state for the HTTP server
#[derive(Clone)]
struct ServerState {
    plugins_dir: PathBuf,
}

/// HTTP server for serving plugin files
pub struct PluginHttpServer {
    port: u16,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl PluginHttpServer {
    /// Create a new HTTP server (not yet started)
    pub fn new() -> Self {
        Self {
            port: 0,
            shutdown_tx: None,
        }
    }

    /// Start the HTTP server
    /// If port is 0, a random available port will be used
    pub async fn start(&mut self, plugins_dir: PathBuf) -> Result<u16> {
        self.start_on_port(plugins_dir, 0).await
    }

    /// Start the HTTP server on a specific port
    /// If port is 0, a random available port will be used
    pub async fn start_on_port(&mut self, plugins_dir: PathBuf, port: u16) -> Result<u16> {
        // Ensure plugins directory exists
        if !plugins_dir.exists() {
            std::fs::create_dir_all(&plugins_dir)?;
        }

        // Bind to localhost with specified port
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        let addr = listener.local_addr()?;
        self.port = addr.port();

        log::info!("Plugin HTTP server starting on {}", addr);
        log::info!("Serving plugins from: {:?}", plugins_dir);

        // Create shared state
        let state = Arc::new(ServerState { plugins_dir });

        // Configure CORS to allow WebSocket connections from plugins
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        // Build router
        let app = Router::new()
            .route("/jliver-api.js", get(serve_api_script))
            .route("/{plugin_id}/{*path}", get(serve_plugin_file))
            .layer(cors)
            .with_state(state);

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        self.shutdown_tx = Some(shutdown_tx);

        // Spawn server task
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .ok();
        });

        Ok(self.port)
    }

    /// Stop the HTTP server
    pub async fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }

    /// Get the port the server is listening on
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Default for PluginHttpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Serve the jliver-api.js script
async fn serve_api_script() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(JLIVER_API_JS))
        .unwrap()
}

/// Serve a plugin file
async fn serve_plugin_file(
    State(state): State<Arc<ServerState>>,
    Path((plugin_id, path)): Path<(String, String)>,
) -> Response<Body> {
    log::debug!("Serving plugin file: plugin_id={}, path={}", plugin_id, path);

    // Construct the file path
    let file_path = state.plugins_dir.join(&plugin_id).join(&path);
    log::debug!("Resolved file path: {:?}", file_path);

    // Check if file exists first
    if !file_path.exists() {
        log::warn!("Plugin file not found: {:?}", file_path);
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("File not found"))
            .unwrap();
    }

    // Security: Ensure the resolved path is within the plugins directory
    let canonical_plugins_dir = match state.plugins_dir.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to canonicalize plugins dir: {}", e);
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Failed to resolve plugins directory"))
                .unwrap();
        }
    };

    let canonical_file_path = match file_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to canonicalize file path {:?}: {}", file_path, e);
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("File not found"))
                .unwrap();
        }
    };

    if !canonical_file_path.starts_with(&canonical_plugins_dir) {
        log::warn!(
            "Path traversal attempt blocked: {} -> {:?}",
            path,
            canonical_file_path
        );
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Access denied"))
            .unwrap();
    }

    // Read the file
    let content = match tokio::fs::read(&canonical_file_path).await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to read file {:?}: {}", canonical_file_path, e);
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("File not found"))
                .unwrap();
        }
    };

    // Guess the MIME type
    let mime_type = mime_guess::from_path(&canonical_file_path)
        .first_or_octet_stream()
        .to_string();

    // Check if this is an HTML file and inject the script
    let is_html = mime_type.starts_with("text/html")
        || path.ends_with(".html")
        || path.ends_with(".htm");

    let final_content = if is_html {
        // Convert to string and inject script after <head> tag
        match String::from_utf8(content) {
            Ok(html) => {
                let injected = inject_script_into_html(&html);
                injected.into_bytes()
            }
            Err(e) => {
                // Not valid UTF-8, return as-is
                log::warn!("HTML file is not valid UTF-8: {}", e);
                e.into_bytes()
            }
        }
    } else {
        content
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(final_content))
        .unwrap()
}

/// Inject the jliver-api.js script into an HTML document
fn inject_script_into_html(html: &str) -> String {
    // Create inline script tag with the full script content
    let script_tag = format!("<script>\n{}\n</script>", JLIVER_API_JS);

    // Try to inject after <head> tag
    if let Some(pos) = html.to_lowercase().find("<head>") {
        let insert_pos = pos + 6; // Length of "<head>"
        let mut result = String::with_capacity(html.len() + script_tag.len() + 1);
        result.push_str(&html[..insert_pos]);
        result.push('\n');
        result.push_str(&script_tag);
        result.push_str(&html[insert_pos..]);
        return result;
    }

    // Try to inject after <head ...> with attributes
    let lower = html.to_lowercase();
    if let Some(head_start) = lower.find("<head") {
        if let Some(head_end) = lower[head_start..].find('>') {
            let insert_pos = head_start + head_end + 1;
            let mut result = String::with_capacity(html.len() + script_tag.len() + 1);
            result.push_str(&html[..insert_pos]);
            result.push('\n');
            result.push_str(&script_tag);
            result.push_str(&html[insert_pos..]);
            return result;
        }
    }

    // Fallback: inject at the beginning of the document
    log::warn!("Could not find <head> tag, injecting at beginning");
    format!("{}\n{}", script_tag, html)
}
