//! HTTP client adapter for GPUI with image caching.
//!
//! Image bytes are fetched with reqwest on a private runtime, because GPUI
//! polls this future on its own executor. libcurl (isahc) is not used: on
//! Windows its multi interface never wakes up during threaded DNS, and
//! Schannel revocation checks can stall the handshake, so every image request
//! hits the timeout. macOS and Linux are unaffected. The Bilibili API client
//! already reaches the network through reqwest.

use anyhow::Context;
use bytes::Bytes;
use futures::future::BoxFuture;
use gpui::http_client::{AsyncBody, HttpClient, Request, Response, Url};
use http_body_util::BodyExt;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Type alias for cache get result
type CacheResult = (Bytes, u16, Vec<(String, String)>);

/// Cache entry with data and expiration time
struct CacheEntry {
    data: Bytes,
    status: u16,
    headers: Vec<(String, String)>,
    expires_at: Instant,
}

/// Image cache with TTL
struct ImageCache {
    entries: HashMap<String, CacheEntry>,
    max_size: usize,
}

impl ImageCache {
    fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
        }
    }

    fn get(&self, url: &str) -> Option<CacheResult> {
        if let Some(entry) = self.entries.get(url) {
            if entry.expires_at > Instant::now() {
                return Some((entry.data.clone(), entry.status, entry.headers.clone()));
            }
        }
        None
    }

    fn insert(
        &mut self,
        url: String,
        data: Bytes,
        status: u16,
        headers: Vec<(String, String)>,
        ttl: Duration,
    ) {
        // Clean up expired entries if cache is getting full
        if self.entries.len() >= self.max_size {
            let now = Instant::now();
            self.entries.retain(|_, entry| entry.expires_at > now);
        }

        // If still full, remove oldest entries
        if self.entries.len() >= self.max_size {
            // Remove ~25% of entries (oldest first by expiration)
            let mut entries_vec: Vec<_> = self
                .entries
                .iter()
                .map(|(k, v)| (k.clone(), v.expires_at))
                .collect();
            entries_vec.sort_by_key(|(_, exp)| *exp);

            let to_remove = self.max_size / 4;
            for (key, _) in entries_vec.into_iter().take(to_remove) {
                self.entries.remove(&key);
            }
        }

        self.entries.insert(
            url,
            CacheEntry {
                data,
                status,
                headers,
                expires_at: Instant::now() + ttl,
            },
        );
    }
}

/// HTTP client used by GPUI to load remote images.
pub struct ImageHttpClient {
    // Dropped before `runtime` so in-flight connections close first.
    client: reqwest::Client,
    runtime: tokio::runtime::Runtime,
    user_agent: gpui::http_client::http::HeaderValue,
    cache: Arc<RwLock<ImageCache>>,
    cache_ttl: Duration,
}

impl ImageHttpClient {
    pub fn new() -> anyhow::Result<Arc<Self>> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("jliver-image-http")
            .enable_all()
            .build()
            .context("failed to start image HTTP runtime")?;

        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent("JLiverTool/0.1")
            .build()
            .context("failed to build image HTTP client")?;

        Ok(Arc::new(Self {
            client,
            runtime,
            user_agent: gpui::http_client::http::HeaderValue::from_static("JLiverTool/0.1"),
            cache: Arc::new(RwLock::new(ImageCache::new(500))), // Cache up to 500 images
            cache_ttl: Duration::from_secs(30 * 60),            // 30 minutes TTL
        }))
    }

    /// Check if URL is likely an image request
    fn is_image_url(url: &str) -> bool {
        let lower = url.to_lowercase();
        lower.contains("hdslb.com")
            || lower.contains(".png")
            || lower.contains(".jpg")
            || lower.contains(".jpeg")
            || lower.contains(".gif")
            || lower.contains(".webp")
            || lower.contains("/face/")
            || lower.contains("/garb/")
    }
}

impl HttpClient for ImageHttpClient {
    fn user_agent(&self) -> Option<&gpui::http_client::http::HeaderValue> {
        Some(&self.user_agent)
    }

    fn proxy(&self) -> Option<&Url> {
        None
    }

    fn send(
        &self,
        req: Request<AsyncBody>,
    ) -> BoxFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        let client = self.client.clone();
        let handle = self.runtime.handle().clone();
        let url = req.uri().to_string();
        let is_image = Self::is_image_url(&url);

        // Check cache for image requests
        if is_image {
            if let Some((data, status, headers)) = self.cache.read().get(&url) {
                return Box::pin(async move {
                    let mut builder = Response::builder().status(status);
                    for (name, value) in headers {
                        builder = builder.header(name, value);
                    }
                    let response = builder.body(AsyncBody::from_bytes(data))?;
                    Ok(response)
                });
            }
        }

        let cache = if is_image {
            Some((self.cache.clone(), self.cache_ttl, url.clone()))
        } else {
            None
        };

        Box::pin(async move {
            let (parts, body) = req.into_parts();
            let body_data = body.collect().await?.to_bytes();

            let fetched = handle
                .spawn(async move {
                    execute_request(
                        client,
                        parts.method,
                        parts.uri.to_string(),
                        parts.headers,
                        body_data,
                    )
                    .await
                })
                .await
                .context("image request task failed")?
                .with_context(|| format!("image request failed: {url}"))?;

            let (status, headers, body_bytes) = fetched;

            // Cache successful image responses
            if let Some((cache_lock, ttl, cache_url)) = cache {
                if (200..300).contains(&status) {
                    cache_lock.write().insert(
                        cache_url,
                        body_bytes.clone(),
                        status,
                        headers.clone(),
                        ttl,
                    );
                }
            }

            let mut builder = Response::builder().status(status);
            for (name, value) in headers {
                builder = builder.header(name, value);
            }
            let response = builder.body(AsyncBody::from_bytes(body_bytes))?;
            Ok(response)
        })
    }
}

async fn execute_request(
    client: reqwest::Client,
    method: gpui::http_client::http::Method,
    url: String,
    headers: gpui::http_client::http::HeaderMap,
    body: Bytes,
) -> anyhow::Result<(u16, Vec<(String, String)>, Bytes)> {
    let method = reqwest::Method::from_bytes(method.as_str().as_bytes())
        .with_context(|| format!("invalid method for {url}"))?;
    let mut builder = client.request(method, &url);
    for (name, value) in headers.iter() {
        builder = builder.header(name.as_str(), value.as_bytes());
    }
    if !body.is_empty() {
        builder = builder.body(body.to_vec());
    }

    let response = builder
        .send()
        .await
        .with_context(|| format!("image request failed: {url}"))?;
    let status = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|v| (name.to_string(), v.to_string()))
        })
        .collect();
    let body_bytes = response
        .bytes()
        .await
        .with_context(|| format!("failed to read image body: {url}"))?;

    Ok((status, headers, body_bytes))
}

#[cfg(test)]
mod tests {
    use super::ImageHttpClient;
    use futures::executor::block_on;
    use gpui::http_client::{AsyncBody, HttpClient, Request};
    use http_body_util::BodyExt;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::Duration;

    fn listen() -> (TcpListener, String) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        (listener, format!("http://127.0.0.1:{port}/avatar.png"))
    }

    fn serve_once(listener: TcpListener, body: &'static [u8]) {
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut buf = Vec::new();
            let mut byte = [0u8; 1];
            loop {
                if stream.read(&mut byte).unwrap_or(0) == 0 || buf.len() > 8192 {
                    break;
                }
                buf.push(byte[0]);
                if buf.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(header.as_bytes()).unwrap();
            stream.write_all(body).unwrap();
        });
    }

    fn get(url: &str) -> Request<AsyncBody> {
        Request::builder()
            .method("GET")
            .uri(url)
            .body(AsyncBody::from_bytes(bytes::Bytes::new()))
            .unwrap()
    }

    #[test]
    fn loads_image_and_serves_the_second_request_from_cache() {
        let (listener, url) = listen();
        serve_once(listener, b"png-bytes");
        let client = ImageHttpClient::new().unwrap();

        let response = block_on(client.send(get(&url))).unwrap();
        assert_eq!(response.status().as_u16(), 200);
        let bytes = block_on(response.into_body().collect()).unwrap().to_bytes();
        assert_eq!(bytes.as_ref(), b"png-bytes");

        // The listener accepted only one connection. A cache miss would fail here.
        let response = block_on(client.send(get(&url))).unwrap();
        assert_eq!(response.status().as_u16(), 200);
        let bytes = block_on(response.into_body().collect()).unwrap().to_bytes();
        assert_eq!(bytes.as_ref(), b"png-bytes");
    }
}
