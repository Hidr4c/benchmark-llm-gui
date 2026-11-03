#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub version: String,
    pub html_url: String,
    pub published_at: Option<String>,
    pub body: Option<String>,
    pub download_url: Option<String>,
    pub is_newer: bool,
}

pub fn parse_semver(s: &str) -> Option<(u32, u32, u32)> {
    let clean = s.trim().trim_start_matches('v').trim_start_matches('V');
    let parts: Vec<&str> = clean.split('.').collect();
    if parts.len() >= 3 {
        let major = parts[0].parse::<u32>().ok()?;
        let minor = parts[1].parse::<u32>().ok()?;
        let patch = parts[2].split('-').next()?.parse::<u32>().ok()?;
        Some((major, minor, patch))
    } else if parts.len() == 2 {
        let major = parts[0].parse::<u32>().ok()?;
        let minor = parts[1].parse::<u32>().ok()?;
        Some((major, minor, 0))
    } else {
        None
    }
}

pub fn is_version_newer(current: &str, candidate: &str) -> bool {
    match (parse_semver(current), parse_semver(candidate)) {
        (Some((c_maj, c_min, c_pat)), Some((n_maj, n_min, n_pat))) => {
            (n_maj, n_min, n_pat) > (c_maj, c_min, c_pat)
        }
        _ => false,
    }
}

#[async_trait::async_trait]
pub trait IUpdateChecker: Send + Sync {
    async fn check_latest_release(&self, current_version: &str) -> Result<ReleaseInfo, String>;
}
