use std::collections::HashMap;
use super::helpers::normalize;

const TTS_ARENA_TSV_URL: &str = "https://huggingface.co/datasets/Pendrokar/TTS_Arena/raw/main/tts_arena_vote_summary.tsv";

pub struct TtsArenaAdapter {
    client: reqwest::Client,
}

impl TtsArenaAdapter {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// Computes live win-rate (%) per speech model from TTS Arena vote summary TSV.
    pub async fn fetch_tts_win_rates(&self) -> HashMap<String, (String, f32, u64)> {
        let mut stats: HashMap<String, (String, u64, u64)> = HashMap::new();
        let Ok(resp) = self.client.get(TTS_ARENA_TSV_URL).timeout(std::time::Duration::from_secs(10)).send().await else {
            return HashMap::new();
        };
        let Ok(body) = resp.text().await else {
            return HashMap::new();
        };

        for (idx, line) in body.lines().enumerate() {
            if idx == 0 { continue; }
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() < 4 { continue; }
            // Columns: spokentext \t rejected \t votes \t chosen \t lastvote
            let rejected_raw = cols[1].trim();
            let weight = cols[2].trim().parse::<u64>().unwrap_or(1).max(1);
            let rej_key = normalize(rejected_raw);
            if !rej_key.is_empty() {
                let entry = stats.entry(rej_key).or_insert_with(|| (rejected_raw.to_string(), 0, 0));
                entry.2 += weight;
            }
            for item_raw in cols[3].split(',') {
                let mut trimmed = item_raw.trim();
                // Strip leading total-vote prefix on first entry (e.g. "5  mrfakename/E2-F5-TTS")
                if let Some((num_part, rest)) = trimmed.split_once("  ") {
                    if num_part.trim().chars().all(|c| c.is_ascii_digit()) {
                        trimmed = rest.trim();
                    }
                }
                let ck = normalize(trimmed);
                if !ck.is_empty() {
                    let entry = stats.entry(ck).or_insert_with(|| (trimmed.to_string(), 0, 0));
                    entry.1 += weight;
                }
            }
        }

        let mut out = HashMap::new();
        for (key, (name, wins, losses)) in stats {
            let total = wins + losses;
            if total < 5 { continue; }
            // Scale multi-way TTS Arena win-rate smoothly onto 35..79 audio scale without saturating
            let win_ratio = wins as f32 / total as f32;
            let calibrated = ((35.0 + win_ratio * 44.0).clamp(35.0, 79.0) * 10.0).round() / 10.0;
            out.insert(key, (name, calibrated, total));
        }
        out
    }
}
