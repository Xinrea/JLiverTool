//! GitHub release update checker

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tracing::{debug, info, warn};

const GITHUB_API_URL: &str = "https://api.github.com/repos/Xinrea/JLiverTool/releases/latest";

pub mod install;

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
    pub digest: Option<String>,
}

/// GitHub release information
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub html_url: String,
    pub body: String,
    pub published_at: String,
    #[serde(default)]
    pub assets: Vec<ReleaseAsset>,
}

/// Update check result
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub release_notes: String,
    pub has_update: bool,
}

/// Check for updates from GitHub releases
pub async fn check_for_update(current_version: &str) -> Result<UpdateInfo> {
    info!("Checking for updates (current: v{})", current_version);

    let release = fetch_latest_release().await?;
    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let current = current_version.trim_start_matches('v');
    let has_update = compare_versions(current, &latest_version);

    Ok(UpdateInfo {
        current_version: current.to_string(),
        latest_version,
        release_url: release.html_url,
        release_notes: release.body,
        has_update,
    })
}

pub(super) async fn fetch_latest_release() -> Result<GitHubRelease> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("JLiverTool")
        .build()?;

    debug!("Fetching release info from GitHub API");
    let response = client.get(GITHUB_API_URL).send().await?;

    if !response.status().is_success() {
        warn!("Failed to fetch release info: HTTP {}", response.status());
        return Err(anyhow::anyhow!(
            "Failed to fetch release info: {}",
            response.status()
        ));
    }

    Ok(response.json().await?)
}

/// Compare two semantic versions, returns true if latest > current
fn compare_versions(current: &str, latest: &str) -> bool {
    match (
        semver::Version::parse(current),
        semver::Version::parse(latest),
    ) {
        (Ok(current), Ok(latest)) => latest > current,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        assert!(compare_versions("3.0.0", "3.0.1"));
        assert!(compare_versions("3.0.0", "3.1.0"));
        assert!(compare_versions("3.0.0", "4.0.0"));
        assert!(!compare_versions("3.0.0", "3.0.0"));
        assert!(!compare_versions("3.0.1", "3.0.0"));
        assert!(!compare_versions("3.1.0", "3.0.0"));
        assert!(compare_versions("2.9.9", "3.0.0"));
        assert!(compare_versions("3.1.1-beta.1", "3.1.1"));
        assert!(!compare_versions("3.1.1", "3.1.1-beta.1"));
        assert!(!compare_versions("invalid", "3.1.1"));
    }
}
