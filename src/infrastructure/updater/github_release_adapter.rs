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
        headers.insert(USER_AGENT, HeaderValue::from_static("benchmark-llm-gui-updater/1.0"));
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self { owner: owner.into(), repo: repo.into(), client }
    }

    fn matches_platform_asset(name: &str) -> bool {
        let n = name.to_lowercase();
        #[cfg(target_os = "windows")]
        { n.ends_with(".exe") || n.ends_with(".msi") || n.ends_with(".zip") }
        #[cfg(target_os = "linux")]
        { n.ends_with(".appimage") || n.ends_with(".deb") || n.ends_with(".tar.gz") || n.ends_with(".zip") }
        #[cfg(target_os = "macos")]
        { n.ends_with(".dmg") || n.ends_with(".tar.gz") || n.ends_with(".zip") }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        { n.ends_with(".tar.gz") || n.ends_with(".zip") }
    }

    fn find_platform_asset<'a>(&self, assets: &'a [GhAsset]) -> Option<&'a GhAsset> {
        assets.iter().find(|a| {
            let n = a.name.to_lowercase();
            #[cfg(target_os = "windows")]
            let os_ok = n.contains("windows") || (!n.contains("linux") && !n.contains("macos"));
            #[cfg(target_os = "linux")]
            let os_ok = n.contains("linux") || (!n.contains("windows") && !n.contains("macos"));
            #[cfg(target_os = "macos")]
            let os_ok = n.contains("macos") || n.contains("darwin") || (!n.contains("windows") && !n.contains("linux"));
            #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
            let os_ok = true;
            os_ok && Self::matches_platform_asset(&a.name)
        })
    }

    async fn check_via_web_redirect(&self, current_version: &str) -> Option<ReleaseInfo> {
        let web_latest = format!("https://github.com/{}/{}/releases/latest", self.owner, self.repo);
        let resp = self.client.get(&web_latest).send().await.ok()?;
        let final_url = resp.url().to_string();
        let tag_name = final_url.split("/releases/tag/").nth(1)?.split(['?', '#']).next()?.trim().to_string();
        if tag_name.is_empty() {
            return None;
        }

        let assets_url = format!(
            "https://github.com/{}/{}/releases/expanded_assets/{}",
            self.owner, self.repo, tag_name
        );
        let mut download_url = final_url.clone();
        if let Ok(a_resp) = self.client.get(&assets_url).send().await {
            if let Ok(html) = a_resp.text().await {
                let prefix = format!("/{}/{}/releases/download/{}/", self.owner, self.repo, tag_name);
                for seg in html.split(&prefix).skip(1) {
                    if let Some(filename) = seg.split(['"', '\'', '?', '#']).next() {
                        let candidate = format!("https://github.com{prefix}{filename}");
                        let low = filename.to_lowercase();
                        #[cfg(target_os = "windows")]
                        let platform_match = low.contains("windows") || low.ends_with(".exe");
                        #[cfg(target_os = "linux")]
                        let platform_match = low.contains("linux") || low.ends_with(".appimage");
                        #[cfg(target_os = "macos")]
                        let platform_match = low.contains("macos") || low.contains("darwin") || low.ends_with(".dmg");
                        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
                        let platform_match = true;
                        if platform_match && Self::matches_platform_asset(filename) {
                            download_url = candidate;
                            break;
                        }
                    }
                }
            }
        }

        Some(ReleaseInfo {
            version: tag_name.trim_start_matches(['v', 'V']).to_string(),
            is_newer: is_version_newer(current_version, &tag_name),
            tag_name,
            html_url: final_url,
            published_at: None,
            body: None,
            download_url: Some(download_url),
        })
    }
}

#[async_trait::async_trait]
impl IUpdateChecker for GitHubReleaseAdapter {
    async fn check_latest_release(&self, current_version: &str) -> Result<ReleaseInfo, String> {
        let release_url = format!("https://api.github.com/repos/{}/{}/releases/latest", self.owner, self.repo);

        if let Ok(resp) = self.client.get(&release_url).send().await {
            if resp.status().is_success() {
                if let Ok(rel) = resp.json::<GhRelease>().await {
                    let download_url = self
                        .find_platform_asset(&rel.assets)
                        .map(|a| a.browser_download_url.clone())
                        .unwrap_or_else(|| rel.html_url.clone());

                    let clean_version = rel.tag_name.trim_start_matches(['v', 'V']).to_string();
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
            }
        }

        // Unauthenticated fallback via GitHub web redirect + expanded_assets (avoids 403 API rate limit)
        if let Some(info) = self.check_via_web_redirect(current_version).await {
            return Ok(info);
        }

        // Last fallback: check repository tags if no formal release exists yet
        let tags_url = format!("https://api.github.com/repos/{}/{}/tags", self.owner, self.repo);
        if let Ok(tags_resp) = self.client.get(&tags_url).send().await {
            if tags_resp.status().is_success() {
                if let Ok(tags) = tags_resp.json::<Vec<GhTag>>().await {
                    if let Some(latest_tag) = tags.first() {
                        let clean_version = latest_tag.name.trim_start_matches(['v', 'V']).to_string();
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
            }
        }

        Err("No releases or tags found on the remote repository.".to_string())
    }
}
