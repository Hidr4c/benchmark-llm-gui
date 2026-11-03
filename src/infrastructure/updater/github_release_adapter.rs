use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use crate::domain::updater::{is_version_newer, IUpdateChecker, ReleaseInfo};

#[derive(Debug, Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    html_url: String,
    published_at: Option<String>,
    body: Option<String>,
    #[serde(default)]
    assets: Vec<GhAsset>,
}

#[derive(Debug, Deserialize)]
struct GhTag {
    name: String,
}

pub struct GitHubReleaseAdapter {
    owner: String,
    repo: String,
    client: reqwest::Client,
}

impl GitHubReleaseAdapter {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("benchmark-llm-gui-updater/1.0"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self {
            owner: owner.into(),
            repo: repo.into(),
            client,
        }
    }

    fn find_platform_asset<'a>(&self, assets: &'a [GhAsset]) -> Option<&'a GhAsset> {
        assets.iter().find(|a| {
            let n = a.name.to_lowercase();
            #[cfg(target_os = "windows")]
            { n.ends_with(".exe") || n.ends_with(".msi") || n.ends_with(".zip") }
            #[cfg(target_os = "linux")]
            { n.ends_with(".appimage") || n.ends_with(".deb") || n.ends_with(".tar.gz") || n.ends_with(".zip") }
            #[cfg(target_os = "macos")]
            { n.ends_with(".dmg") || n.ends_with(".zip") || n.ends_with(".tar.gz") }
            #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
            { n.ends_with(".tar.gz") || n.ends_with(".zip") }
        })
    }
}

#[async_trait::async_trait]
impl IUpdateChecker for GitHubReleaseAdapter {
    async fn check_latest_release(&self, current_version: &str) -> Result<ReleaseInfo, String> {
        let release_url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.owner, self.repo
        );

        let resp = self
            .client
            .get(&release_url)
            .send()
            .await
            .map_err(|e| format!("Network error querying GitHub releases: {e}"))?;

        if resp.status().is_success() {
            let rel: GhRelease = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse release metadata: {e}"))?;

            let download_url = self
                .find_platform_asset(&rel.assets)
                .map(|a| a.browser_download_url.clone())
                .unwrap_or_else(|| rel.html_url.clone());

            let clean_version = rel.tag_name.trim_start_matches('v').to_string();
            let is_newer = is_version_newer(current_version, &rel.tag_name);

            return Ok(ReleaseInfo {
                tag_name: rel.tag_name,
                version: clean_version,
                html_url: rel.html_url,
                published_at: rel.published_at,
                body: rel.body,
                download_url: Some(download_url),
                is_newer,
            });
        }

        // Fallback: check repository tags if no formal release was published yet
        let tags_url = format!(
            "https://api.github.com/repos/{}/{}/tags",
            self.owner, self.repo
        );
        let tags_resp = self
            .client
            .get(&tags_url)
            .send()
            .await
            .map_err(|e| format!("Network error querying tags: {e}"))?;

        if tags_resp.status().is_success() {
            let tags: Vec<GhTag> = tags_resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse tags: {e}"))?;

            if let Some(latest_tag) = tags.first() {
                let clean_version = latest_tag.name.trim_start_matches('v').to_string();
                let is_newer = is_version_newer(current_version, &latest_tag.name);
                let tag_url = format!(
                    "https://github.com/{}/{}/releases/tag/{}",
                    self.owner, self.repo, latest_tag.name
                );

                return Ok(ReleaseInfo {
                    tag_name: latest_tag.name.clone(),
                    version: clean_version,
                    html_url: tag_url.clone(),
                    published_at: None,
                    body: Some("Tag published on GitHub repository.".into()),
                    download_url: Some(tag_url),
                    is_newer,
                });
            }
        }

        Err("No releases or tags found on the remote repository.".to_string())
    }
}
