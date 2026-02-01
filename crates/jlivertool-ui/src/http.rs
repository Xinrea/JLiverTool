//! HTTP client adapter for GPUI using isahc with image caching

use bytes::Bytes;
use futures::future::BoxFuture;
use gpui::http_client::{AsyncBody, HttpClient, Request, Response, Url};
use http_body_util::BodyExt;
use isahc::prelude::*;
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

    fn insert(&mut self, url: String, data: Bytes, status: u16, headers: Vec<(String, String)>, ttl: Duration) {
        // Clean up expired entries if cache is getting full
        if self.entries.len() >= self.max_size {
            let now = Instant::now();
            self.entries.retain(|_, entry| entry.expires_at > now);
        }

        // If still full, remove oldest entries
        if self.entries.len() >= self.max_size {
            // Remove ~25% of entries (oldest first by expiration)
            let mut entries_vec: Vec<_> = self.entries.iter()
                .map(|(k, v)| (k.clone(), v.expires_at))
                .collect();
            entries_vec.sort_by_key(|(_, exp)| *exp);

            let to_remove = self.max_size / 4;
            for (key, _) in entries_vec.into_iter().take(to_remove) {
                self.entries.remove(&key);
            }
        }

        self.entries.insert(url, CacheEntry {
            data,
            status,
            headers,
            expires_at: Instant::now() + ttl,
        });
    }
}

/// HTTP client implementation using isahc with image caching
pub struct IsahcHttpClient {
    client: isahc::HttpClient,
    user_agent: gpui::http_client::http::HeaderValue,
    cache: Arc<RwLock<ImageCache>>,
    cache_ttl: Duration,
}

impl IsahcHttpClient {
    pub fn new() -> anyhow::Result<Arc<Self>> {
        let client = isahc::HttpClient::builder()
            .timeout(std::time::Duration::from_secs(30))
            .redirect_policy(isahc::config::RedirectPolicy::Limit(10))
            .build()?;

        Ok(Arc::new(Self {
            client,
            user_agent: gpui::http_client::http::HeaderValue::from_static("JLiverTool/0.1"),
            cache: Arc::new(RwLock::new(ImageCache::new(500))), // Cache up to 500 images
            cache_ttl: Duration::from_secs(30 * 60),  // 30 minutes TTL
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

impl HttpClient for IsahcHttpClient {
    fn type_name(&self) -> &'static str {
        "IsahcHttpClient"
    }

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

            // Convert AsyncBody to bytes using BodyExt
            let body_data = body.collect().await?.to_bytes();

            // Build isahc request
            let mut isahc_req = isahc::Request::builder()
                .method(parts.method.as_str())
                .uri(parts.uri.to_string());

            for (name, value) in parts.headers.iter() {
                isahc_req = isahc_req.header(name.as_str(), value.to_str().unwrap_or(""));
            }

            let isahc_req = isahc_req.body(body_data.to_vec())?;

            // Send request
            let mut isahc_resp = client.send_async(isahc_req).await?;

            // Read response body
            let status = isahc_resp.status();
            let headers = isahc_resp.headers().clone();
            let body_bytes = isahc_resp.bytes().await?;
            let body_bytes = Bytes::from(body_bytes);

            // Cache successful image responses
            if let Some((cache_lock, ttl, cache_url)) = cache {
                if status.is_success() {
                    let headers_vec: Vec<(String, String)> = headers.iter()
                        .filter_map(|(name, value)| {
                            value.to_str().ok().map(|v| (name.to_string(), v.to_string()))
                        })
                        .collect();

                    cache_lock.write().insert(
                        cache_url,
                        body_bytes.clone(),
                        status.as_u16(),
                        headers_vec,
                        ttl,
                    );
                }
            }

            // Build gpui response
            let mut builder = Response::builder().status(status.as_u16());

            for (name, value) in headers.iter() {
                if let Ok(value_str) = value.to_str() {
                    builder = builder.header(name.as_str(), value_str);
                }
            }

            let response = builder.body(AsyncBody::from_bytes(body_bytes))?;

            Ok(response)
        })
    }
}
