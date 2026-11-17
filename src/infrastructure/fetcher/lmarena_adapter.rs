use std::collections::HashMap;
use serde::Deserialize;
use crate::domain::modality::ModelModality;
use crate::domain::model::{BenchmarkScores, ModelBenchmark};
use crate::domain::specialty::DomainSpecialty;
use super::helpers::normalize;

const LMARENA_BASE: &str = "https://datasets-server.huggingface.co/rows?dataset=lmarena-ai/leaderboard-dataset&config=";

#[derive(Deserialize, Debug)]
struct ArenaResponse {
    #[serde(default)]
    rows: Vec<ArenaRowContainer>,
}

#[derive(Deserialize, Debug)]
struct ArenaRowContainer {
    row: ArenaRow,
}

#[derive(Deserialize, Debug)]
struct ArenaRow {
    model_name: Option<String>,
    organization: Option<String>,
    license: Option<String>,
    rating: Option<f64>,
    category: Option<String>,
    leaderboard_publish_date: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ArenaTrackScore {
    pub display_name: String,
    pub org: String,
    pub is_proprietary: bool,
    pub score_text: Option<f32>,
    pub score_image: Option<f32>,
    pub score_video: Option<f32>,
    pub score_vision: Option<f32>,
    pub score_webdev: Option<f32>,
    pub year: Option<u16>,
    pub dominant_modality: ModelModality,
}

pub struct LmArenaAdapter {
    client: reqwest::Client,
}

impl LmArenaAdapter {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    async fn fetch_config(&self, config: &str) -> Vec<ArenaRow> {
        let url = format!("{LMARENA_BASE}{config}&split=latest&offset=0&limit=100");
        let Ok(resp) = self.client.get(&url).timeout(std::time::Duration::from_secs(10)).send().await else {
            return Vec::new();
        };
        resp.json::<ArenaResponse>().await.map(|r| r.rows.into_iter().map(|c| c.row).collect()).unwrap_or_default()
    }

    fn elo_to_100(elo: f64) -> f32 {
        // Empirical ELO scale: ~1000 = 35 pts, ~1250 = 60 pts, ~1550 = 88 pts, top ~91.5
        let normalized = ((elo - 880.0) / 8.5).clamp(28.0, 91.5);
        ((normalized * 10.0).round() / 10.0) as f32
    }

    pub async fn fetch_all_tracks(&self) -> HashMap<String, ArenaTrackScore> {
        let (text_rows, img_rows, edit_rows, t2v_rows, i2v_rows, vis_rows, web_rows) = tokio::join!(
            self.fetch_config("text"),
            self.fetch_config("text_to_image"),
            self.fetch_config("image_edit"),
            self.fetch_config("text_to_video"),
            self.fetch_config("image_to_video"),
            self.fetch_config("vision"),
            self.fetch_config("webdev"),
        );

        let mut map: HashMap<String, ArenaTrackScore> = HashMap::new();
        let insert_track = |map: &mut HashMap<String, ArenaTrackScore>, rows: Vec<ArenaRow>, kind: &str, mod_hint: ModelModality| {
            for r in rows {
                let Some(cat) = r.category.as_deref() else { continue; };
                if cat != "overall" { continue; }
                let Some(name) = r.model_name else { continue; };
                let Some(rating) = r.rating else { continue; };
                let score = Self::elo_to_100(rating);
                let key = normalize(&name);
                if key.is_empty() { continue; }
                let year = r.leaderboard_publish_date.as_deref().and_then(|d| d.get(0..4)).and_then(|y| y.parse::<u16>().ok());
                let is_open_lic = r.license.as_deref().is_some_and(|l| {
                    let low = l.to_lowercase();
                    low.contains("mit") || low.contains("apache") || low.contains("open")
                        || low.contains("llama") || low.contains("gemma") || low.contains("qwen")
                        || low.contains("cc-by") || low.contains("bsd") || low.contains("weights")
                });
                let is_prop = !is_open_lic;
                let org = r.organization.unwrap_or_else(|| "Unknown".into());

                let entry = map.entry(key).or_insert_with(|| ArenaTrackScore {
                    display_name: name.clone(),
                    org: org.clone(),
                    is_proprietary: is_prop,
                    year,
                    dominant_modality: mod_hint,
                    ..Default::default()
                });
                if !is_prop { entry.is_proprietary = false; }
                match kind {
                    "text" => entry.score_text = Some(entry.score_text.map_or(score, |prev| prev.max(score))),
                    "image" => { entry.score_image = Some(entry.score_image.map_or(score, |prev| prev.max(score))); entry.dominant_modality = ModelModality::ImageGen; }
                    "video" => { entry.score_video = Some(entry.score_video.map_or(score, |prev| prev.max(score))); entry.dominant_modality = ModelModality::VideoGen; }
                    "vision" => entry.score_vision = Some(entry.score_vision.map_or(score, |prev| prev.max(score))),
                    "webdev" => entry.score_webdev = Some(entry.score_webdev.map_or(score, |prev| prev.max(score))),
                    _ => {}
                }
            }
        };

        insert_track(&mut map, text_rows, "text", ModelModality::Text);
        insert_track(&mut map, vis_rows, "vision", ModelModality::Multimodal);
        insert_track(&mut map, web_rows, "webdev", ModelModality::Text);
        insert_track(&mut map, img_rows, "image", ModelModality::ImageGen);
        insert_track(&mut map, edit_rows, "image", ModelModality::ImageGen);
        insert_track(&mut map, t2v_rows, "video", ModelModality::VideoGen);
        insert_track(&mut map, i2v_rows, "video", ModelModality::VideoGen);
        map
    }

    fn estimate_open_weights_gb(key: &str) -> (f32, f32) {
        // Parse explicit parameter size token like "27b", "397b", "14b" when present in LMArena slug
        let bytes = key.as_bytes();
        for i in 0..bytes.len() {
            if bytes[i] == b'b' && i > 0 && bytes[i - 1].is_ascii_digit() {
                let mut start = i;
                while start > 0 && bytes[start - 1].is_ascii_digit() { start -= 1; }
                if let Ok(b_val) = key[start..i].parse::<f32>() {
                    if (0.5..=2000.0).contains(&b_val) {
                        let size = (b_val * 0.58 * 10.0).round() / 10.0;
                        return (size, ModelBenchmark::calculate_vram(size));
                    }
                }
            }
        }
        if key.contains("mini") || key.contains("small") || key.contains("nano") || key.contains("lite") {
            (8.0, 9.8)
        } else if key.contains("flash") || key.contains("fast") || key.contains("air") {
            (16.0, 19.2)
        } else {
            // Unsized open frontier / MoE flagships typically require workstation/multi-GPU memory
            (48.0, 56.0)
        }
    }

    /// Converts standalone LMArena leaders (video/image/proprietary LLMs) into catalog entries.
    pub fn into_standalone_models(track_map: &HashMap<String, ArenaTrackScore>, existing_keys: &std::collections::HashSet<String>) -> Vec<ModelBenchmark> {
        let mut out = Vec::new();
        for (key, tr) in track_map {
            if existing_keys.contains(key) || existing_keys.iter().any(|k| k.len() >= 6 && (k.contains(key) || key.contains(k))) {
                continue;
            }
            let primary = tr.score_image.or(tr.score_video).or(tr.score_text).or(tr.score_vision).or(tr.score_webdev);
            let Some(sc) = primary else { continue; };
            let mod_detected = if tr.score_video.is_some() {
                ModelModality::VideoGen
            } else if tr.score_image.is_some() {
                ModelModality::ImageGen
            } else {
                ModelModality::detect(key, &tr.display_name, None, &[])
            };
            let specialty = DomainSpecialty::detect_with_context(key, &tr.display_name, None, &[], "", None, mod_detected);
            let (est_size, est_vram) = if tr.is_proprietary { (0.0, 0.0) } else { Self::estimate_open_weights_gb(key) };
            let coding_score = if matches!(mod_detected, ModelModality::Text | ModelModality::Multimodal) {
                tr.score_webdev.map(|w| ((w * 0.92).clamp(25.0, 86.0) * 10.0).round() / 10.0)
            } else {
                None
            };
            let provider_name = super::helpers::canonicalize_provider_slug(&tr.org);
            let family = super::helpers::extract_dynamic_family("", &tr.display_name);
            out.push(ModelBenchmark {
                id: format!("lmarena/{key}"),
                name: tr.display_name.clone(),
                family,
                size_gb: est_size,
                vram_rec_gb: est_vram,
                tokens_per_sec: None,
                tps_hardware: Some(if tr.is_proprietary { format!("API ({provider_name})") } else { "LMArena Verified".into() }),
                scores: BenchmarkScores {
                    open_llm: if matches!(mod_detected, ModelModality::Text | ModelModality::Multimodal) {
                        tr.score_text.or(tr.score_vision).or(Some(sc))
                    } else {
                        None
                    },
                    coding: coding_score,
                    image_bench: tr.score_image,
                    video_bench: tr.score_video,
                    average: Some(sc),
                    ..Default::default()
                },
                downloads: 2_500_000,
                likes: 8_000,
                hf_url: "https://huggingface.co/datasets/lmarena-ai/leaderboard-dataset".into(),
                is_moe: false,
                context_k: 128,
                is_cloud: tr.is_proprietary,
                provider: if tr.is_proprietary { Some(provider_name) } else { None },
                release_year: tr.year,
                modality: mod_detected,
                specialty,
                ..Default::default()
            });
        }
        out
    }
}
