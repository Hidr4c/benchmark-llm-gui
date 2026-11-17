use std::collections::HashMap;
use serde::Deserialize;
use crate::domain::modality::ModelModality;
use crate::domain::model::{BenchmarkScores, ModelBenchmark};
use crate::domain::specialty::DomainSpecialty;
use super::helpers::{extract_dynamic_family, normalize};

const COMFY_INDEX_URL: &str = "https://raw.githubusercontent.com/Comfy-Org/workflow_templates/main/templates/index.json";

#[derive(Deserialize, Debug)]
struct ComfyCategory {
    #[serde(rename = "type", default)]
    cat_type: String,
    #[serde(default)]
    templates: Vec<ComfyTemplate>,
}

#[derive(Deserialize, Debug)]
struct ComfyTemplate {
    name: String,
    #[serde(default)]
    models: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    usage: u64,
    #[serde(default)]
    date: String,
    #[serde(rename = "openSource", default)]
    open_source: bool,
}

pub struct ComfyUiAdapter {
    client: reqwest::Client,
}

impl ComfyUiAdapter {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn fetch_comfy_models(&self) -> Result<Vec<ModelBenchmark>, String> {
        let resp = self.client
            .get(COMFY_INDEX_URL)
            .timeout(std::time::Duration::from_secs(12))
            .send()
            .await
            .map_err(|e| format!("ComfyUI index request failed: {e}"))?;

        let categories: Vec<ComfyCategory> = resp.json().await
            .map_err(|e| format!("ComfyUI index parse failed: {e}"))?;

        // (modality, size_gb, usage, open_count, closed_count, year, tags, tpl_slug)
        let mut agg: HashMap<String, (ModelModality, f32, u64, u32, u32, u16, Vec<String>, String)> = HashMap::new();
        for cat in categories {
            let base_mod = match cat.cat_type.as_str() {
                "3d" => ModelModality::ThreeDGen,
                "video" => ModelModality::VideoGen,
                "audio" => ModelModality::AudioSpeech,
                "image" => ModelModality::ImageGen,
                "llm" => ModelModality::Text,
                _ => continue,
            };

            for tpl in cat.templates {
                let has_api_tag = tpl.name.starts_with("api_")
                    || tpl.tags.iter().any(|t| t.eq_ignore_ascii_case("API"));
                let is_open_tpl = tpl.open_source && !has_api_tag;
                let is_mega_bundle = tpl.models.len() > 2;

                for m_raw in &tpl.models {
                    let clean = m_raw.trim();
                    if clean.is_empty() || clean.eq_ignore_ascii_case("None") {
                        continue;
                    }
                    let key = normalize(clean);
                    if key.len() < 2 { continue; }
                    let tpl_open = is_open_tpl;
                    let size_gb = if tpl_open && !is_mega_bundle && tpl.size > 0 {
                        ((tpl.size as f64 / 1_073_741_824.0) * 10.0).round() as f32 / 10.0
                    } else {
                        0.0
                    };
                    let year = tpl.date.get(0..4).and_then(|s| s.parse::<u16>().ok()).unwrap_or(2026);
                    let detected_mod = ModelModality::detect(&key, clean, None, &tpl.tags);
                    let final_mod = if detected_mod == ModelModality::Text && base_mod != ModelModality::Text {
                        base_mod
                    } else {
                        detected_mod
                    };

                    agg.entry(clean.to_string())
                        .and_modify(|e| {
                            if tpl_open && size_gb > 0.0 && (e.1 == 0.0 || size_gb < e.1) { e.1 = size_gb; }
                            e.2 += tpl.usage;
                            if tpl_open { e.3 += 1; } else { e.4 += 1; }
                            if year > e.5 { e.5 = year; }
                        })
                        .or_insert((
                            final_mod, size_gb, tpl.usage,
                            u32::from(tpl_open), u32::from(!tpl_open),
                            year, tpl.tags.clone(), tpl.name.clone(),
                        ));
                }
            }
        }

        let max_usage = agg.values().map(|v| v.2).max().unwrap_or(1).max(1) as f32;
        let mut results = Vec::new();
        for (name, (modality, size_gb, usage, open_count, _closed_count, year, tags, tpl_slug)) in agg {
            let is_cloud = open_count == 0;
            let eff_size = if is_cloud { 0.0 } else if size_gb <= 0.1 { 6.5 } else { size_gb };
            let vram = if is_cloud { 0.0 } else { ModelBenchmark::calculate_vram(eff_size) };
            // Calibrated adoption proxy (42..72) in open_llm ONLY — never fake multi-pillar RealWorld
            let usage_ratio = ((usage.max(1) as f32).ln() / max_usage.ln().max(1.0)).clamp(0.0, 1.0);
            let adoption_score = ((42.0 + usage_ratio * 30.0) * 10.0).round() / 10.0;
            let family = extract_dynamic_family("", &name);
            let specialty = DomainSpecialty::detect_with_context(&tpl_slug, &name, None, &tags, "", None, modality);

            results.push(ModelBenchmark {
                id: format!("comfy-org/{}", normalize(&name)),
                name: format!("{name} (ComfyUI)"),
                family: family.clone(),
                size_gb: eff_size,
                vram_rec_gb: vram,
                tps_hardware: Some(if is_cloud { "ComfyUI Partner API".into() } else { "ComfyUI Workflow".into() }),
                scores: BenchmarkScores {
                    average: Some(adoption_score),
                    ..Default::default()
                },
                downloads: usage * 1500,
                likes: usage,
                hf_url: format!("https://github.com/Comfy-Org/workflow_templates/blob/main/templates/{tpl_slug}.json"),
                context_k: 32,
                is_cloud,
                provider: if is_cloud { Some(family) } else { None },
                release_year: Some(year),
                modality,
                specialty,
                ..Default::default()
            });
        }

        Ok(results)
    }
}
