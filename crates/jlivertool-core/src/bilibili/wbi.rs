//! WBI signature algorithm for Bilibili API
//!
//! Reference: https://github.com/SocialSisterYi/bilibili-API-collect/blob/master/docs/misc/sign/wbi.md

use anyhow::Result;
use md5::{Digest, Md5};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Mixin key encoding table
const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

/// WBI keys cache
#[derive(Debug)]
struct WbiKeysCache {
    mixin_key: String,
    last_update: Instant,
}

/// WBI signer for Bilibili API requests
#[derive(Debug, Clone)]
pub struct WbiSigner {
    cache: Arc<RwLock<Option<WbiKeysCache>>>,
    cache_ttl: Duration,
}

impl Default for WbiSigner {
    fn default() -> Self {
        Self::new()
    }
}

impl WbiSigner {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(None)),
            cache_ttl: Duration::from_secs(3600), // 1 hour
        }
    }

    /// Get mixin key from img_key and sub_key
    fn get_mixin_key(img_key: &str, sub_key: &str) -> String {
        let raw_wbi_key = format!("{}{}", img_key, sub_key);
        let raw_wbi_key_bytes = raw_wbi_key.as_bytes();

        MIXIN_KEY_ENC_TAB
            .iter()
            .take(32)
            .filter_map(|&i| raw_wbi_key_bytes.get(i).map(|&b| b as char))
            .collect()
    }

    /// Update WBI keys from nav API response
    pub fn update_keys(&self, img_url: &str, sub_url: &str) {
        // Extract key from URL: https://i0.hdslb.com/bfs/wbi/{key}.png
        let img_key = img_url
            .rsplit('/')
            .next()
            .and_then(|s| s.strip_suffix(".png"))
            .unwrap_or("")
            .to_string();

        let sub_key = sub_url
            .rsplit('/')
            .next()
            .and_then(|s| s.strip_suffix(".png"))
            .unwrap_or("")
            .to_string();

        let mixin_key = Self::get_mixin_key(&img_key, &sub_key);

        let mut cache = self.cache.write();
        *cache = Some(WbiKeysCache {
            mixin_key,
            last_update: Instant::now(),
        });
    }

    /// Check if cache is valid
    pub fn is_cache_valid(&self) -> bool {
        let cache = self.cache.read();
        cache
            .as_ref()
            .map(|c| c.last_update.elapsed() < self.cache_ttl)
            .unwrap_or(false)
    }

    /// Sign parameters with WBI
    pub fn sign(&self, params: &mut HashMap<String, String>) -> Result<()> {
        let cache = self.cache.read();
        let cache = cache
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("WBI keys not initialized"))?;

        // Add wts (timestamp)
        let wts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
            .to_string();
        params.insert("wts".to_string(), wts);

        // Sort parameters by key
        let mut sorted_params: Vec<_> = params.iter().collect();
        sorted_params.sort_by_key(|(k, _)| *k);

        // Build query string (filter special characters)
        let query: String = sorted_params
            .iter()
            .map(|(k, v)| {
                let clean_v: String = v
                    .chars()
                    .filter(|c| !matches!(c, '!' | '\'' | '(' | ')' | '*'))
                    .collect();
                format!("{}={}", k, clean_v)
            })
            .collect::<Vec<_>>()
            .join("&");

        // Calculate MD5
        let to_sign = format!("{}{}", query, cache.mixin_key);
        let mut hasher = Md5::new();
        hasher.update(to_sign.as_bytes());
        let w_rid = format!("{:x}", hasher.finalize());

        params.insert("w_rid".to_string(), w_rid);

        Ok(())
    }

    /// Get signed query string
    pub fn get_signed_query(&self, params: &mut HashMap<String, String>) -> Result<String> {
        self.sign(params)?;

        let mut sorted_params: Vec<_> = params.iter().collect();
        sorted_params.sort_by_key(|(k, _)| *k);

        let query = sorted_params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        Ok(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixin_key() {
        let img_key = "7cd084941338484aae1ad9425b84077c";
        let sub_key = "4932caff0ff746eab6f01bf08b70ac45";
        let mixin_key = WbiSigner::get_mixin_key(img_key, sub_key);
        assert_eq!(mixin_key, "ea1db124af3c7062474693fa704f4ff8");
    }
}
