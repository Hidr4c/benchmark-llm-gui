use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarnessCategory {
    CliAgent,
    IdeExtension,
    ResearchScaffold,
}

impl HarnessCategory {
    pub fn label(&self, is_fr: bool) -> &'static str {
        match (self, is_fr) {
            (Self::CliAgent, true) => "Agent Terminal / CLI",
            (Self::CliAgent, false) => "Terminal / CLI Agent",
            (Self::IdeExtension, true) => "Extension IDE (VS Code / JetBrains)",
            (Self::IdeExtension, false) => "IDE Extension (VS Code / JetBrains)",
            (Self::ResearchScaffold, true) => "Harnais Benchmark / Scaffold",
            (Self::ResearchScaffold, false) => "Benchmark Scaffold",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CliAgent => "cli_agent",
            Self::IdeExtension => "ide_extension",
            Self::ResearchScaffold => "research_scaffold",
        }
    }

    pub fn from_str_lenient(s: &str) -> Self {
        match s {
            "cli_agent" => Self::CliAgent,
            "ide_extension" => Self::IdeExtension,
            _ => Self::ResearchScaffold,
        }
    }
}

/// Individual benchmark observation for a (Harness, Model) pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessRun {
    pub harness_name: String,
    pub harness_org: String,
    pub model_name: String,
    pub model_key: String,
    pub suite: String,
    pub score_pct: f32,
    pub cost_usd: Option<f32>,
    pub date: String,
    pub source_url: String,
}

/// Dynamic profile for an AI coding harness combining GitHub & live benchmarks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessProfile {
    pub name: String,
    pub repo_slug: String,
    pub org: String,
    pub category: HarnessCategory,
    pub url: String,
    pub license: String,
    pub stars: u64,
    pub last_updated: String,
    pub best_score_pct: Option<f32>,
    pub avg_score_pct: Option<f32>,
    pub runs_count: usize,
    pub description: String,
}

/// Side-by-side comparison of different harnesses tested on the exact same base model.
#[derive(Debug, Clone)]
pub struct HarnessModelComparison {
    pub model_name: String,
    pub model_key: String,
    pub entries: Vec<HarnessRun>,
    pub best_score: f32,
    pub worst_score: f32,
    pub spread_pts: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HarnessSnapshotData {
    pub profiles: Vec<HarnessProfile>,
    pub runs: Vec<HarnessRun>,
}

/// Normalizes a base model identifier across SWE-bench, Terminal-Bench, Cline, Kilo & Aider labels.
pub fn normalize_base_model_key(raw: &str) -> String {
    let mut s = raw.trim().to_lowercase();
    if let Some((_, rest)) = s.split_once('/') {
        s = rest.to_string();
    }
    for noise in ["($$$$)", "($$$)", "($$)", "($)", "(thinking)", "(high)", "(medium)", "(low)", "(reasoning)", "-preview", "-instruct", ":free"] {
        s = s.replace(noise, "");
    }
    // Strip trailing release-date tokens (`-YYYYMMDD` or `-YYYY-MM-DD`) dynamically
    let cleaned_tokens: Vec<&str> = s
        .split('-')
        .filter(|tok| {
            let is_8_date = tok.len() == 8 && tok.chars().all(|c| c.is_ascii_digit()) && tok.starts_with("20");
            let is_year_tok = tok.len() == 4 && tok.chars().all(|c| c.is_ascii_digit()) && tok.starts_with("20");
            !is_8_date && !is_year_tok
        })
        .collect();
    let mut alnum: String = cleaned_tokens.join("").chars().filter(|c| c.is_alphanumeric()).collect();
    // Drop trailing 4-digit MMDD suffix left after removing `-YYYY` in `YYYY-MM-DD` dates
    if alnum.len() > 6 && alnum[alnum.len() - 4..].chars().all(|c| c.is_ascii_digit()) && s.contains("-20") {
        alnum.truncate(alnum.len() - 4);
    }
    for tier in ["opus", "sonnet", "haiku", "fable"] {
        if let Some(idx) = alnum.find(tier) {
            if let Some(first_digit) = alnum.find(|c: char| c.is_ascii_digit()) {
                let prefix = &alnum[..first_digit.min(idx)];
                let digits: String = alnum.chars().filter(|c| c.is_ascii_digit()).collect();
                if !prefix.is_empty() && !digits.is_empty() {
                    return format!("{prefix}{tier}{digits}");
                }
            }
        }
    }
    alnum
}

/// Groups runs by underlying base model and returns comparable multi-harness groups.
pub fn compare_harnesses_by_model(runs: &[HarnessRun], min_harnesses: usize) -> Vec<HarnessModelComparison> {
    let mut grouped: BTreeMap<String, Vec<HarnessRun>> = BTreeMap::new();
    for r in runs {
        if r.model_key.is_empty() || r.score_pct <= 0.0 {
            continue;
        }
        grouped.entry(r.model_key.clone()).or_default().push(r.clone());
    }

    let mut out = Vec::new();
    for (key, list) in grouped {
        // Keep highest score per (harness_name, suite) deterministically via BTreeMap
        let mut dedup: BTreeMap<(String, String), HarnessRun> = BTreeMap::new();
        for item in list {
            let k = (item.harness_name.to_lowercase(), item.suite.clone());
            dedup.entry(k)
                .and_modify(|prev| {
                    if item.score_pct > prev.score_pct {
                        *prev = item.clone();
                    }
                })
                .or_insert(item);
        }
        let mut entries: Vec<HarnessRun> = dedup.into_values().collect();
        let distinct_harnesses: std::collections::BTreeSet<String> = entries
            .iter()
            .map(|e| e.harness_name.to_lowercase())
            .collect();

        if distinct_harnesses.len() < min_harnesses {
            continue;
        }
        entries.sort_by(|a, b| {
            b.score_pct
                .total_cmp(&a.score_pct)
                .then_with(|| a.harness_name.to_lowercase().cmp(&b.harness_name.to_lowercase()))
                .then_with(|| a.suite.cmp(&b.suite))
        });
        let display = entries[0].model_name.clone();
        let best = entries.first().map(|e| e.score_pct).unwrap_or(0.0);
        let worst = entries.last().map(|e| e.score_pct).unwrap_or(0.0);
        out.push(HarnessModelComparison {
            model_name: display,
            model_key: key,
            entries,
            best_score: best,
            worst_score: worst,
            spread_pts: (best - worst).max(0.0),
        });
    }

    // Prioritize models with modern multi-suite CLI/IDE runs + highest distinct harness diversity
    out.sort_by(|a, b| {
        let modern_score = |c: &HarnessModelComparison| {
            let modern_hits = c.entries.iter().filter(|e| !e.suite.starts_with("SWE-bench")).count().min(4);
            modern_hits * 3 + c.entries.len()
        };
        modern_score(b)
            .cmp(&modern_score(a))
            .then_with(|| b.spread_pts.partial_cmp(&a.spread_pts).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| a.model_key.cmp(&b.model_key))
    });
    out
}
