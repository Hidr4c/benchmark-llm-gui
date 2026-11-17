use std::collections::HashMap;
use serde_json::Value;
use crate::domain::modality::ModelModality;
use super::helpers::normalize;

const VBENCH_CONFIG_URL: &str = "https://vchitect-vbench-leaderboard.hf.space/config";
const OPEN_ASR_CONFIG_URL: &str = "https://hf-audio-open-asr-leaderboard.hf.space/config";

#[derive(Debug, Clone)]
pub struct MediaBenchRecord {
    pub display_name: String,
    pub modality: ModelModality,
    pub score: f32,
    pub source_url: String,
    pub suite: &'static str,
    pub release_year: Option<u16>,
}

pub struct MediaBenchAdapter {
    client: reqwest::Client,
}

impl MediaBenchAdapter {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// Fetches real public benchmarks for Video Generation (VBench) and Speech Audio (Open ASR).
    pub async fn fetch_all(&self) -> HashMap<String, MediaBenchRecord> {
        let (vbench, asr) = tokio::join!(self.fetch_vbench(), self.fetch_open_asr());
        let mut combined = vbench;
        for (k, v) in asr {
            combined.entry(k).or_insert(v);
        }
        combined
    }

    async fn fetch_vbench(&self) -> HashMap<String, MediaBenchRecord> {
        let mut out: HashMap<String, MediaBenchRecord> = HashMap::new();
        let Ok(resp) = self.client.get(VBENCH_CONFIG_URL).timeout(std::time::Duration::from_secs(12)).send().await else {
            return out;
        };
        let Ok(root) = resp.json::<Value>().await else { return out; };
        let Some(comps) = root.get("components").and_then(|v| v.as_array()) else { return out; };

        for c in comps {
            if c.get("type").and_then(|v| v.as_str()) != Some("dataframe") { continue; }
            let Some(val) = c.pointer("/props/value") else { continue; };
            let headers: Vec<String> = val.get("headers").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_lowercase())).collect())
                .unwrap_or_default();
            let Some(score_col) = headers.iter().position(|h| h == "total score" || h == "i2v score") else { continue; };
            let date_col = headers.iter().position(|h| h == "date");
            let Some(rows) = val.get("data").and_then(|v| v.as_array()) else { continue; };

            for row in rows {
                let Some(cells) = row.as_array() else { continue; };
                let Some(raw_name) = cells.first().and_then(|v| v.as_str()) else { continue; };
                let (name_raw, url) = parse_markdown_link(raw_name, "https://huggingface.co/spaces/Vchitect/VBench_Leaderboard");
                let name = strip_non_ascii_glyphs(&name_raw);
                let Some(raw_sc) = cells.get(score_col).and_then(extract_cell_f32) else { continue; };
                // Calibrate tightly-clustered VBench percentages (~65-91%) onto 30..83 comparability scale
                let score = (((raw_sc - 40.0) * 1.62).clamp(30.0, 83.0) * 10.0).round() / 10.0;
                let year = date_col
                    .and_then(|dc| cells.get(dc).and_then(|v| v.as_str()))
                    .and_then(|d| d.get(0..4))
                    .and_then(|y| y.parse::<u16>().ok());

                let key = normalize(&name);
                if key.len() < 3 { continue; }
                out.entry(key)
                    .and_modify(|prev| { if score > prev.score { prev.score = score; } })
                    .or_insert(MediaBenchRecord {
                        display_name: name,
                        modality: ModelModality::VideoGen,
                        score,
                        source_url: url,
                        suite: "VBench Official",
                        release_year: year,
                    });
            }
        }
        out
    }

    async fn fetch_open_asr(&self) -> HashMap<String, MediaBenchRecord> {
        let mut out: HashMap<String, MediaBenchRecord> = HashMap::new();
        let Ok(resp) = self.client.get(OPEN_ASR_CONFIG_URL).timeout(std::time::Duration::from_secs(12)).send().await else {
            return out;
        };
        let Ok(root) = resp.json::<Value>().await else { return out; };
        let Some(comps) = root.get("components").and_then(|v| v.as_array()) else { return out; };

        for c in comps {
            if c.get("type").and_then(|v| v.as_str()) != Some("dataframe") { continue; }
            let Some(val) = c.pointer("/props/value") else { continue; };
            let headers: Vec<String> = val.get("headers").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_lowercase())).collect())
                .unwrap_or_default();
            let Some(model_col) = headers.iter().position(|h| h == "model") else { continue; };
            let Some(wer_col) = headers.iter().position(|h| h.contains("average wer")) else { continue; };
            let Some(rows) = val.get("data").and_then(|v| v.as_array()) else { continue; };

            for row in rows {
                let Some(cells) = row.as_array() else { continue; };
                let Some(raw_model) = cells.get(model_col).and_then(|v| v.as_str()) else { continue; };
                let (name_raw, url) = parse_html_anchor(raw_model, "https://huggingface.co/spaces/hf-audio/open_asr_leaderboard");
                let name = strip_non_ascii_glyphs(&name_raw);
                let Some(wer) = cells.get(wer_col).and_then(extract_cell_f32) else { continue; };
                // Convert lower-is-better WER% onto calibrated 30..82 audio scale
                let score = ((84.0 - wer * 1.8).clamp(30.0, 82.0) * 10.0).round() / 10.0;
                let short_name = name.split('/').last().unwrap_or(&name).trim().to_string();
                if short_name.eq_ignore_ascii_case("multilingual") || short_name.is_empty() { continue; }
                let key = normalize(&short_name);
                if key.len() < 3 { continue; }
                out.entry(key)
                    .and_modify(|prev| { if score > prev.score { prev.score = score; } })
                    .or_insert(MediaBenchRecord {
                        display_name: short_name,
                        modality: ModelModality::AudioSpeech,
                        score,
                        source_url: url,
                        suite: "Open ASR Leaderboard",
                        release_year: Some(2026),
                    });
            }
        }
        out
    }
}

fn extract_cell_f32(v: &Value) -> Option<f32> {
    if let Some(f) = v.as_f64() {
        let val = f as f32;
        return val.is_finite().then_some(val);
    }
    v.as_str()?.trim().trim_end_matches('%').trim().parse::<f32>().ok().filter(|n| n.is_finite())
}

fn parse_markdown_link(raw: &str, fallback_url: &str) -> (String, String) {
    let trimmed = raw.trim();
    if let Some(stripped) = trimmed.strip_prefix('[') {
        if let Some((text, tail)) = stripped.split_once("](") {
            let url = tail.trim_end_matches(')').trim();
            return (text.trim().to_string(), if url.starts_with("http") { url.to_string() } else { fallback_url.to_string() });
        }
    }
    (trimmed.to_string(), fallback_url.to_string())
}

fn parse_html_anchor(raw: &str, fallback_url: &str) -> (String, String) {
    let url = raw
        .split("href=\"")
        .nth(1)
        .and_then(|t| t.split('"').next())
        .filter(|u| u.starts_with("http"))
        .unwrap_or(fallback_url)
        .to_string();
    let mut text = String::new();
    let mut in_tag = false;
    for ch in raw.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }
    (text.trim().to_string(), url)
}

fn strip_non_ascii_glyphs(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii() || c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.' | '/' | '(' | ')'))
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
