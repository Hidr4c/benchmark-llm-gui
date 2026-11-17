use std::collections::{HashMap, HashSet};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct DiscoveredGitHubRepo {
    pub name: String,
    pub slug: String,
    pub section: String,
    pub url: String,
    pub description: String,
    pub inline_stars: u64,
    pub language: Option<String>,
    pub license: Option<String>,
    pub pushed_at: String,
    pub topics: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct EnrichedRepoMeta {
    pub stars: u64,
    pub pushed_at: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub language: Option<String>,
}

/// Parses live Awesome-list Markdown catalogs into unique GitHub repository entries without hardcoded names.
pub fn parse_awesome_markdown(
    markdown: &str,
    allowed_sections: &[&str],
    denied_sections: &[&str],
) -> Vec<DiscoveredGitHubRepo> {
    let mut section = String::new();
    let mut seen_slugs = HashSet::new();
    let mut out = Vec::new();

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            section = trimmed.trim_start_matches('#').trim().to_string();
            continue;
        }
        let sec_low = section.to_lowercase();
        if denied_sections.iter().any(|d| sec_low.contains(d)) {
            continue;
        }
        if !allowed_sections.is_empty() && !allowed_sections.iter().any(|a| sec_low.contains(a)) {
            continue;
        }
        if !(trimmed.starts_with('-') || trimmed.starts_with('*')) {
            continue;
        }

        let Some((name, primary_url)) = extract_first_md_link(trimmed) else { continue };
        let Some(slug) = extract_github_slug(trimmed) else { continue };
        let slug_low = slug.to_lowercase();
        if is_noise_slug(&slug_low, &name.to_lowercase()) || !seen_slugs.insert(slug_low) {
            continue;
        }

        let desc = extract_line_description(trimmed);
        let inline_stars = extract_inline_stars(trimmed);
        out.push(DiscoveredGitHubRepo {
            name,
            slug,
            section: section.clone(),
            url: primary_url,
            description: desc,
            inline_stars,
            language: None,
            license: None,
            pushed_at: String::new(),
            topics: Vec::new(),
        });
    }
    out
}

fn is_noise_slug(slug_low: &str, name_low: &str) -> bool {
    let tail = slug_low.split('/').last().unwrap_or(slug_low);
    tail.contains("awesome")
        || tail.contains("beginners")
        || tail.contains("tutorial")
        || tail.contains("cookbook")
        || name_low.contains("awesome ")
        || slug_low.contains("/issues")
        || slug_low.contains("/blob")
}

fn extract_first_md_link(line: &str) -> Option<(String, String)> {
    let mut rest = line;
    while let Some(open_b) = rest.find('[') {
        let after_b = &rest[open_b + 1..];
        let Some(close_b) = after_b.find("](") else { break };
        let label = after_b[..close_b].trim_matches(|c| c == '*' || c == '`' || c == ' ').to_string();
        let after_paren = &after_b[close_b + 2..];
        let Some(close_p) = after_paren.find(')') else { break };
        let url = after_paren[..close_p].trim().to_string();
        if !label.is_empty() && !label.starts_with('!') && !url.contains("img.shields.io") && url.starts_with("http") {
            return Some((label, url));
        }
        rest = &after_paren[close_p + 1..];
    }
    None
}

fn extract_github_slug(line: &str) -> Option<String> {
    for needle in ["img.shields.io/github/stars/", "https://github.com/"] {
        if let Some(pos) = line.find(needle) {
            let tail = &line[pos + needle.len()..];
            let clean = tail.split(|c: char| matches!(c, '?' | '#' | ')' | '"' | '\'' | ' ')).next().unwrap_or("");
            let mut parts = clean.split('/').filter(|p| !p.is_empty());
            if let (Some(owner), Some(repo)) = (parts.next(), parts.next()) {
                let repo_clean = repo.trim_end_matches(".git");
                if !owner.is_empty() && !repo_clean.is_empty() && !matches!(owner, "sponsors" | "orgs" | "topics") {
                    return Some(format!("{owner}/{repo_clean}"));
                }
            }
        }
    }
    None
}

fn extract_line_description(line: &str) -> String {
    let search_start = line.find("](").unwrap_or(0);
    let tail = &line[search_start..];
    for sep in [" — ", " - ", ": "] {
        if let Some(pos) = tail.find(sep) {
            let candidate = strip_md_images(&tail[pos + sep.len()..]);
            if candidate.len() >= 10 {
                return candidate;
            }
        }
    }
    String::new()
}

fn strip_md_images(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut skip_bracket = false;
    for ch in s.chars() {
        match ch {
            '[' | '`' | '<' => {}
            ')' if skip_bracket => skip_bracket = false,
            ']' => skip_bracket = true,
            _ if !skip_bracket => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_inline_stars(line: &str) -> u64 {
    let low = line.to_lowercase();
    for tok in low.split(|c: char| !c.is_ascii_alphanumeric() && c != '.') {
        if let Some(num) = tok.strip_suffix('k') {
            if let Ok(v) = num.parse::<f64>() {
                if v > 0.1 && v < 500.0 {
                    return (v * 1000.0) as u64;
                }
            }
        }
    }
    0
}

#[derive(Deserialize)]
struct UnghResp {
    repo: Option<UnghRepo>,
}

#[derive(Deserialize)]
struct UnghRepo {
    #[serde(default)]
    stars: u64,
    #[serde(default, rename = "pushedAt")]
    pushed_at: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
struct GhRepoJson {
    #[serde(default)]
    stargazers_count: u64,
    #[serde(default)]
    pushed_at: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    description: Option<String>,
    license: Option<GhLicense>,
}

#[derive(Deserialize)]
struct GhLicense {
    spdx_id: Option<String>,
}

/// Concurrently enriches GitHub repository metadata via ungh.cc (zero search rate-limit) with GitHub API fallback.
pub async fn enrich_repos_via_ungh(client: &reqwest::Client, slugs: &[String]) -> HashMap<String, EnrichedRepoMeta> {
    let mut set = tokio::task::JoinSet::new();
    for slug in slugs {
        let c = client.clone();
        let s = slug.clone();
        set.spawn(async move {
            let meta = fetch_single_repo_meta(&c, &s).await;
            (s.to_lowercase(), meta)
        });
    }
    let mut out = HashMap::new();
    while let Some(Ok((k, Some(m)))) = set.join_next().await {
        out.insert(k, m);
    }
    out
}

async fn fetch_single_repo_meta(client: &reqwest::Client, slug: &str) -> Option<EnrichedRepoMeta> {
    if let Ok(r) = client.get(format!("https://ungh.cc/repos/{slug}")).timeout(std::time::Duration::from_secs(6)).send().await {
        if r.status().is_success() {
            if let Ok(u) = r.json::<UnghResp>().await {
                if let Some(repo) = u.repo {
                    if repo.stars > 0 {
                        return Some(EnrichedRepoMeta {
                            stars: repo.stars,
                            pushed_at: repo.pushed_at,
                            description: repo.description,
                            license: None,
                            language: None,
                        });
                    }
                }
            }
        }
    }
    let resp = client.get(format!("https://api.github.com/repos/{slug}")).timeout(std::time::Duration::from_secs(6)).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let gh = resp.json::<GhRepoJson>().await.ok()?;
    Some(EnrichedRepoMeta {
        stars: gh.stargazers_count,
        pushed_at: gh.pushed_at,
        description: gh.description,
        license: gh.license.and_then(|l| l.spdx_id),
        language: gh.language,
    })
}

#[derive(Deserialize)]
struct GhSearchRoot {
    #[serde(default)]
    items: Vec<GhSearchItem>,
}

#[derive(Deserialize)]
struct GhSearchItem {
    name: String,
    full_name: String,
    html_url: String,
    homepage: Option<String>,
    #[serde(default)]
    stargazers_count: u64,
    #[serde(default)]
    pushed_at: String,
    language: Option<String>,
    description: Option<String>,
    license: Option<GhLicense>,
    #[serde(default)]
    topics: Vec<String>,
}

/// Optional live GitHub topic search (e.g. mobile/edge GGUF discovery) gracefully ignoring 403 rate limits.
pub async fn search_github_topic_repos(client: &reqwest::Client, query: &str, per_page: usize) -> Vec<DiscoveredGitHubRepo> {
    let url = format!("https://api.github.com/search/repositories?q={query}&sort=stars&order=desc&per_page={per_page}");
    let Ok(resp) = client.get(&url).timeout(std::time::Duration::from_secs(8)).send().await else {
        return Vec::new();
    };
    if !resp.status().is_success() {
        return Vec::new();
    }
    let Ok(root) = resp.json::<GhSearchRoot>().await else {
        return Vec::new();
    };
    root.items.into_iter().filter(|it| !is_noise_slug(&it.full_name.to_lowercase(), &it.name.to_lowercase())).map(|it| {
        DiscoveredGitHubRepo {
            name: it.name,
            slug: it.full_name,
            section: "GitHub Discovery".into(),
            url: it.homepage.filter(|h| !h.trim().is_empty()).unwrap_or(it.html_url),
            description: it.description.unwrap_or_default(),
            inline_stars: it.stargazers_count,
            language: it.language,
            license: it.license.and_then(|l| l.spdx_id),
            pushed_at: it.pushed_at,
            topics: it.topics,
        }
    }).collect()
}
