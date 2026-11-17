use std::collections::HashMap;
use serde::Deserialize;
use crate::domain::harness::{
    normalize_base_model_key, HarnessCategory, HarnessProfile, HarnessRun, HarnessSnapshotData,
};
use super::awesome_discovery::{enrich_repos_via_ungh, parse_awesome_markdown, DiscoveredGitHubRepo};
use super::harness_extra::{extract_site_url, fetch_cline_opencode_runs, fetch_kilobench_runs};

const SWE_BENCH_JSON_URL: &str = "https://raw.githubusercontent.com/SWE-bench/swe-bench.github.io/master/data/leaderboards.json";
const TBENCH_URL: &str = "https://www.tbench.ai/leaderboard/terminal-bench/2.0";
const TBENCH_1_URL: &str = "https://www.tbench.ai/leaderboard/terminal-bench/1.0";
const AIDER_YML_URL: &str = "https://raw.githubusercontent.com/Aider-AI/aider/main/aider/website/_data/polyglot_leaderboard.yml";
const HARNESS_MD_FEEDS: &[(&str, &[&str])] = &[
    ("https://raw.githubusercontent.com/bradAGI/awesome-cli-coding-agents/main/README.md", &["open source", "closed source", "harnesses", "orchestration"]),
    ("https://raw.githubusercontent.com/rafska/awesome-local-llm/main/README.md", &["coding agents"]),
];

#[derive(Deserialize)]
struct SweRoot { #[serde(default)] leaderboards: Vec<SweTrack> }
#[derive(Deserialize)]
struct SweTrack { name: String, #[serde(default)] results: Vec<SweEntry> }
#[derive(Deserialize)]
struct SweEntry {
    name: Option<String>, agent: Option<String>, agent_org: Option<String>,
    resolved: Option<f32>, cost: Option<f32>, date: Option<String>,
    site: Option<serde_json::Value>, #[serde(default)] tags: Vec<String>,
}

pub struct HarnessAdapter {
    client: reqwest::Client,
}

impl HarnessAdapter {
    pub fn new(client: reqwest::Client) -> Self { Self { client } }

    /// Dynamically discovers AI coding harnesses across 6 live benchmarks + community catalogs (Zero Hardcoding).
    pub async fn fetch_all(&self) -> Result<HarnessSnapshotData, String> {
        let (swe_res, tb2_res, tb1_res, aider_res, cline_runs, kilo_runs, md_repos) = tokio::join!(
            self.fetch_swebench_runs(),
            self.fetch_terminal_bench_url(TBENCH_URL, "Terminal-Bench 2.0"),
            self.fetch_terminal_bench_url(TBENCH_1_URL, "Terminal-Bench 1.0"),
            self.fetch_aider_runs(),
            fetch_cline_opencode_runs(&self.client),
            fetch_kilobench_runs(&self.client),
            self.fetch_markdown_harnesses(),
        );

        let mut runs = Vec::new();
        runs.extend(swe_res.unwrap_or_default());
        runs.extend(tb2_res.unwrap_or_default());
        runs.extend(tb1_res.unwrap_or_default());
        runs.extend(aider_res.unwrap_or_default());
        runs.extend(cline_runs);
        runs.extend(kilo_runs);

        let run_keys: std::collections::HashSet<String> = runs.iter().map(|r| alnum_key(&r.harness_name)).collect();
        let mut candidates = Vec::new();
        let mut extra = Vec::new();
        for c in md_repos {
            let name_k = alnum_key(&c.name);
            let tail_k = alnum_key(c.slug.split('/').last().unwrap_or(&c.slug));
            if run_keys.contains(&name_k) || run_keys.contains(&tail_k) {
                candidates.push(c);
            } else {
                extra.push(c);
            }
        }
        extra.sort_by(|a, b| b.inline_stars.cmp(&a.inline_stars).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
        for c in extra.into_iter().take(45usize.saturating_sub(candidates.len()).max(30)) {
            candidates.push(c);
        }

        let slugs: Vec<String> = candidates.iter().map(|c| c.slug.clone()).collect();
        let meta_map = enrich_repos_via_ungh(&self.client, &slugs).await;

        let mut profiles_map: HashMap<String, HarnessProfile> = HashMap::new();
        let mut slug_lookup: HashMap<String, String> = HashMap::new();

        for c in candidates {
            let meta = meta_map.get(&c.slug.to_lowercase());
            let stars = meta.map(|m| m.stars).filter(|&s| s > 0).unwrap_or(c.inline_stars);
            let pushed = meta.map(|m| m.pushed_at.clone()).filter(|s| !s.is_empty()).unwrap_or_default();
            let desc = meta.and_then(|m| m.description.clone()).filter(|d| !d.is_empty()).unwrap_or(c.description.clone());
            let license = meta.and_then(|m| m.license.clone()).unwrap_or_else(|| "Open-Source".into());
            let org = c.slug.split('/').next().unwrap_or("Community").to_string();
            let blob = format!("{} {} {}", c.name, c.section, desc).to_lowercase();
            let category = classify_harness_category(&blob);

            let alnum = alnum_key(&c.name);
            let tail_alnum = alnum_key(c.slug.split('/').last().unwrap_or(&c.slug));
            slug_lookup.entry(alnum.clone()).or_insert_with(|| c.name.to_lowercase());
            slug_lookup.entry(tail_alnum).or_insert_with(|| c.name.to_lowercase());

            profiles_map.entry(c.name.to_lowercase()).or_insert_with(|| HarnessProfile {
                name: c.name.clone(),
                repo_slug: c.slug.clone(),
                org,
                category,
                url: if c.url.is_empty() { format!("https://github.com/{}", c.slug) } else { c.url },
                license,
                stars,
                last_updated: if pushed.is_empty() { "-".into() } else { pushed.chars().take(10).collect() },
                best_score_pct: None,
                avg_score_pct: None,
                runs_count: 0,
                description: if desc.is_empty() { format!("{} ({})", c.section, c.slug) } else { desc },
            });
        }

        let mut score_acc: HashMap<String, Vec<f32>> = HashMap::new();
        for r in &runs {
            let raw_k = r.harness_name.to_lowercase();
            let resolved_key = if profiles_map.contains_key(&raw_k) {
                raw_k
            } else if let Some(mapped) = slug_lookup.get(&alnum_key(&r.harness_name)) {
                mapped.clone()
            } else {
                raw_k
            };

            score_acc.entry(resolved_key.clone()).or_default().push(r.score_pct);
            profiles_map.entry(resolved_key).or_insert_with(|| HarnessProfile {
                name: r.harness_name.clone(),
                repo_slug: String::new(),
                org: r.harness_org.clone(),
                category: infer_run_category(&r.harness_name, &r.suite),
                url: r.source_url.clone(),
                license: "See Leaderboard".into(),
                stars: 0,
                last_updated: r.date.clone(),
                best_score_pct: None,
                avg_score_pct: None,
                runs_count: 0,
                description: format!("Observed in {} benchmark suite.", r.suite),
            });
        }

        let mut profiles: Vec<HarnessProfile> = profiles_map.into_values().map(|mut p| {
            let key = p.name.to_lowercase();
            if let Some(vals) = score_acc.get(&key) {
                p.runs_count = vals.len();
                let sum: f32 = vals.iter().sum();
                p.avg_score_pct = Some(((sum / vals.len() as f32) * 10.0).round() / 10.0);
                p.best_score_pct = vals.iter().cloned().reduce(f32::max).map(|v| (v * 10.0).round() / 10.0);
            }
            p
        }).collect();

        profiles.sort_by(|a, b| {
            b.best_score_pct.unwrap_or(-1.0).total_cmp(&a.best_score_pct.unwrap_or(-1.0))
                .then_with(|| b.stars.cmp(&a.stars))
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        Ok(HarnessSnapshotData { profiles, runs })
    }

    async fn fetch_markdown_harnesses(&self) -> Vec<DiscoveredGitHubRepo> {
        let mut set = tokio::task::JoinSet::new();
        for (idx, &(url, secs)) in HARNESS_MD_FEEDS.iter().enumerate() {
            let client = self.client.clone();
            set.spawn(async move {
                let txt = client.get(url).timeout(std::time::Duration::from_secs(10)).send().await.ok()?.text().await.ok()?;
                Some((idx, parse_awesome_markdown(&txt, secs, &["archived", "book", "course"])))
            });
        }
        let mut batches: Vec<Option<Vec<DiscoveredGitHubRepo>>> = vec![None; HARNESS_MD_FEEDS.len()];
        while let Some(Ok(Some((idx, items)))) = set.join_next().await {
            if idx < batches.len() { batches[idx] = Some(items); }
        }
        let mut out = Vec::new();
        let mut seen = HashMap::new();
        for items in batches.into_iter().flatten() {
            for item in items {
                let slug_k = item.slug.to_lowercase();
                seen.entry(slug_k).or_insert_with(|| out.push(item));
            }
        }
        out
    }

    async fn fetch_swebench_runs(&self) -> Result<Vec<HarnessRun>, String> {
        let resp = self.client.get(SWE_BENCH_JSON_URL).timeout(std::time::Duration::from_secs(12)).send().await.map_err(|e| e.to_string())?;
        let root: SweRoot = resp.json().await.map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for track in root.leaderboards {
            if !matches!(track.name.as_str(), "Verified" | "Lite" | "Multimodal" | "Multilingual") { continue; }
            for e in track.results {
                let Some(score) = e.resolved else { continue };
                let harness = e.agent.unwrap_or_else(|| e.name.clone().unwrap_or_else(|| "Unknown".into()));
                let model = e.tags.iter()
                    .find_map(|t| t.strip_prefix("Model:").map(|s| s.trim().to_string()))
                    .unwrap_or_else(|| e.name.unwrap_or_else(|| harness.clone()));
                out.push(HarnessRun {
                    harness_name: harness,
                    harness_org: e.agent_org.unwrap_or_else(|| "Community".into()),
                    model_key: normalize_base_model_key(&model),
                    model_name: model,
                    suite: format!("SWE-bench {}", track.name),
                    score_pct: score,
                    cost_usd: e.cost,
                    date: e.date.unwrap_or_default(),
                    source_url: extract_site_url(&e.site),
                });
            }
        }
        Ok(out)
    }

    async fn fetch_terminal_bench_url(&self, url: &str, suite_label: &str) -> Result<Vec<HarnessRun>, String> {
        let resp = self.client.get(url).timeout(std::time::Duration::from_secs(10)).send().await.map_err(|e| e.to_string())?;
        let raw = resp.text().await.map_err(|e| e.to_string())?.replace("\\\"", "\"");
        let mut out = Vec::new();
        for chunk in raw.split("\"agent_display\":{").skip(1) {
            let head = &chunk[..chunk.len().min(900)];
            let Some(ag) = extract_json_str(head, "\"label\":\"") else { continue };
            let model = head.find("\"model_display\":{").and_then(|idx| extract_json_str(&head[idx..], "\"label\":\"")).unwrap_or_else(|| "Unknown".into());
            let Some(acc) = extract_json_num(head, "\"accuracy\":") else { continue };
            let cost = extract_json_num(head, "\"total_cost_usd\":");
            out.push(HarnessRun {
                harness_name: ag, harness_org: "Terminal-Bench".into(), model_key: normalize_base_model_key(&model),
                model_name: model, suite: suite_label.into(), score_pct: acc, cost_usd: cost, date: "2026".into(), source_url: url.into(),
            });
        }
        Ok(out)
    }

    async fn fetch_aider_runs(&self) -> Result<Vec<HarnessRun>, String> {
        let resp = self.client.get(AIDER_YML_URL).timeout(std::time::Duration::from_secs(10)).send().await.map_err(|e| e.to_string())?;
        let text = resp.text().await.map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        let mut cur_model = String::new();
        for line in text.lines() {
            let tr = line.trim();
            if let Some(m) = tr.strip_prefix("model:") { cur_model = m.trim().trim_matches('"').to_string(); }
            else if let Some(p) = tr.strip_prefix("pass_rate_2:") {
                if !cur_model.is_empty() {
                    if let Ok(val) = p.trim().parse::<f32>() {
                        out.push(HarnessRun {
                            harness_name: "Aider".into(), harness_org: "Aider-AI".into(),
                            model_key: normalize_base_model_key(&cur_model), model_name: cur_model.clone(),
                            suite: "Aider Polyglot".into(), score_pct: val, cost_usd: None,
                            date: "2026".into(), source_url: "https://aider.chat/docs/leaderboards/".into(),
                        });
                    }
                    cur_model.clear();
                }
            }
        }
        Ok(out)
    }
}

fn alnum_key(s: &str) -> String {
    let low = s.to_lowercase();
    let trimmed = low.strip_suffix(" cli").unwrap_or(&low);
    trimmed.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

fn classify_harness_category(blob: &str) -> HarnessCategory {
    if blob.contains("vscode") || blob.contains("vs code") || blob.contains("jetbrains") || blob.contains("ide extension") || blob.contains("editor") {
        HarnessCategory::IdeExtension
    } else if blob.contains("research") || blob.contains("benchmark") || blob.contains("scaffold") || blob.contains("evaluation") {
        HarnessCategory::ResearchScaffold
    } else {
        HarnessCategory::CliAgent
    }
}

fn infer_run_category(harness_name: &str, suite: &str) -> HarnessCategory {
    let low = harness_name.to_lowercase();
    if low.contains("ide") || low.contains("vscode") { HarnessCategory::IdeExtension }
    else if suite.starts_with("Terminal-Bench") || suite.contains("Bench") { HarnessCategory::CliAgent }
    else { HarnessCategory::ResearchScaffold }
}

fn extract_json_str(haystack: &str, key: &str) -> Option<String> {
    let start = haystack.find(key)? + key.len();
    let end = haystack[start..].find('"')?;
    Some(haystack[start..start + end].to_string())
}

fn extract_json_num(haystack: &str, key: &str) -> Option<f32> {
    let start = haystack.find(key)? + key.len();
    let tail = &haystack[start..];
    let end = tail.find(|c: char| !c.is_ascii_digit() && c != '.').unwrap_or(tail.len());
    tail[..end].parse::<f32>().ok()
}
