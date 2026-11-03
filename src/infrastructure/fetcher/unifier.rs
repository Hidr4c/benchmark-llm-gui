use std::collections::HashMap;
use async_trait::async_trait;
use crate::domain::modality::ModelModality;
use crate::domain::specialty::DomainSpecialty;
use crate::domain::model::{BenchmarkScores, ModelBenchmark};
use crate::domain::traits::IModelFetcher;
use super::helpers::{
    estimate_gguf_size, estimate_scores, estimate_tps, extract_release_year, generate_ollama_tag, normalize,
    parse_model_identity,
};
use super::cloud_adapter::CloudModelAdapter;
use super::economic_adapter::EconomicBenchmarkAdapter;
use super::hf_adapter::{HfModelItem, HuggingFaceAdapter};
use super::leaderboard_adapter::{LeaderboardAdapter, LeaderboardRow};
use super::speed_adapter::{SpeedBenchmarkAdapter, SpeedRecord};
use super::vram_benchmark_adapter::{VramBenchmarkAdapter, VramBenchmarkRecord};
use crate::domain::economic::EconomicScores;

pub struct LiveModelFetcher {
    hf_adapter: HuggingFaceAdapter,
    lb_adapter: LeaderboardAdapter,
    speed_adapter: SpeedBenchmarkAdapter,
    vram_bench_adapter: VramBenchmarkAdapter,
    cloud_adapter: CloudModelAdapter,
    economic_adapter: EconomicBenchmarkAdapter,
}

impl Default for LiveModelFetcher {
    fn default() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) BenchmarkLLM/2.0")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("Failed to initialize HTTP client");

        Self {
            hf_adapter: HuggingFaceAdapter::new(client.clone()),
            lb_adapter: LeaderboardAdapter::new(client.clone()),
            speed_adapter: SpeedBenchmarkAdapter::new(client.clone()),
            vram_bench_adapter: VramBenchmarkAdapter::new(client.clone()),
            cloud_adapter: CloudModelAdapter::new(client.clone()),
            economic_adapter: EconomicBenchmarkAdapter::new(client),
        }
    }
}

impl LiveModelFetcher {
    async fn retry_fetch<F, Fut, T>(op: F, src_name: &str) -> Result<T, String>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        let mut delay = std::time::Duration::from_millis(600);
        for attempt in 1..=3 {
            match op().await {
                Ok(val) => return Ok(val),
                Err(e) if attempt == 3 => return Err(format!("{src_name} ({e})")),
                Err(_) => {
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
            }
        }
        Err(format!("{src_name} (Max retries)"))
    }
}

#[async_trait]
impl IModelFetcher for LiveModelFetcher {
    async fn fetch_and_unify(&self) -> Result<Vec<ModelBenchmark>, String> {
        let hf_fut = Self::retry_fetch(|| self.hf_adapter.fetch_models(), "HuggingFace Hub");
        let lb_fut = Self::retry_fetch(|| self.lb_adapter.fetch_scores(), "Open LLM Leaderboard");
        let speed_fut = Self::retry_fetch(|| self.speed_adapter.fetch_speeds(), "LLM-Speed");
        let vram_fut = Self::retry_fetch(|| self.vram_bench_adapter.fetch_benchmarks(), "VRAM Benchmarks");
        let cloud_fut = Self::retry_fetch(|| self.cloud_adapter.fetch_cloud_models(), "OpenRouter / LiteLLM");
        let econ_fut = self.economic_adapter.fetch_all();

        let (hf_res, lb_res, speed_res, vram_res, cloud_res, econ_map) =
            tokio::join!(hf_fut, lb_fut, speed_fut, vram_fut, cloud_fut, econ_fut);

        let hf_models = hf_res?;
        let lb_map = lb_res.unwrap_or_default();
        let speed_map = speed_res.unwrap_or_default();
        let vram_map = vram_res.unwrap_or_default();

        let mut unified = Self::unify_models(hf_models, &lb_map, &speed_map, &vram_map, &econ_map);
        if let Ok(mut cloud_models) = cloud_res {
            for m in &mut cloud_models {
                let norm = normalize(&m.name);
                if let Some(eb) = econ_map.get(&norm).or_else(|| econ_map.iter().find_map(|(k, v)| if norm.contains(k) { Some(v) } else { None })) {
                    m.economic = Some(eb.clone());
                    if let Some(cs) = eb.composite_score() {
                        m.scores.real_world = Some(cs);
                    }
                }
            }
            unified.extend(cloud_models);
        }
        Ok(unified)
    }
}

impl LiveModelFetcher {
    fn unify_models(
        hf_models: Vec<HfModelItem>,
        leaderboard: &HashMap<String, LeaderboardRow>,
        speeds: &HashMap<String, SpeedRecord>,
        vram_bench: &HashMap<String, VramBenchmarkRecord>,
        econ_bench: &HashMap<String, EconomicScores>,
    ) -> Vec<ModelBenchmark> {
        let mut results = Vec::new();

        for hf in hf_models {
            let (name, family, params_b) = parse_model_identity(&hf.id);
            let norm_id = normalize(&hf.id);
            let norm_name = normalize(&name);
            let modality = ModelModality::detect(&hf.id, &name, hf.pipeline_tag.as_deref(), &hf.tags);
            let is_embedding = norm_id.contains("gte") || norm_id.contains("embed") || norm_id.contains("bge");

            let lb_match = if is_embedding {
                None
            } else {
                leaderboard.get(&norm_id).or_else(|| leaderboard.get(&norm_name)).or_else(|| {
                    params_b.and_then(|p| {
                        let fam = family.to_lowercase();
                        leaderboard.iter().find_map(|(k, v)| {
                            let p_match = v.params_b.map_or(false, |lb_p| (lb_p as f32 - p).abs() < 1.0);
                            let same_gen = (norm_id.contains("qwen3") == k.contains("qwen3"))
                                && (norm_id.contains("qwen2") == k.contains("qwen2"));
                            if p_match && k.contains(&fam) && same_gen && !k.contains("gte") {
                                Some(v)
                            } else {
                                None
                            }
                        })
                    })
                })
            };

            let vram_match = vram_bench.get(&norm_id).or_else(|| vram_bench.get(&norm_name)).or_else(|| {
                params_b.and_then(|p| {
                    let fam = family.to_lowercase();
                    let p_tag = format!("{:.0}b", p);
                    vram_bench.iter().find_map(|(k, v)| {
                        if k.contains(&fam) {
                            if let Some(idx) = k.find(&p_tag) {
                                let is_preceded_by_digit = idx > 0 && k.as_bytes()[idx - 1].is_ascii_digit();
                                if !is_preceded_by_digit {
                                    return Some(v);
                                }
                            }
                        }
                        None
                    })
                })
            });

            let (mut scores, lb_params, lb_moe) = if let Some(lb) = lb_match {
                let cal_avg = lb.average.map(|v| ModelBenchmark::calibrate_score(v as f32, "open_llm"));
                (
                    BenchmarkScores {
                        open_llm: cal_avg,
                        average: cal_avg,
                        ifeval: lb.ifeval.map(|v| v as f32),
                        bbh: lb.bbh.map(|v| v as f32),
                        math_lvl5: lb.math.map(|v| ModelBenchmark::calibrate_score(v as f32, "math")),
                        gpqa: lb.gpqa.map(|v| ModelBenchmark::calibrate_score(v as f32, "gpqa")),
                        mmlu_pro: lb.mmlu_pro.map(|v| ModelBenchmark::calibrate_score(v as f32, "open_llm")),
                        musr: lb.musr.map(|v| v as f32),
                        coding: None,
                        agentic: None,
                        real_world: None,
                    },
                    lb.params_b.map(|p| p as f32),
                    lb.moe.unwrap_or(false),
                )
            } else {
                (estimate_scores(&name, params_b), None, false)
            };

            // Enrich with VRAM Benchmark multi-source data (Coding, Agentic, Quality)
            if let Some(vb) = vram_match {
                if let Some(c) = vb.coding { scores.coding = Some(c); }
                if let Some(a) = vb.agentic { scores.agentic = Some(a); }
                if scores.open_llm.is_none() { scores.open_llm = vb.quality; }
            }

            let fallback = estimate_scores(&name, params_b);
            if scores.coding.is_none() { scores.coding = fallback.coding; }
            if scores.agentic.is_none() { scores.agentic = fallback.agentic; }
            if scores.open_llm.is_none() { scores.open_llm = fallback.open_llm; }

            let econ_match = econ_bench.get(&norm_id)
                .or_else(|| econ_bench.get(&norm_name))
                .or_else(|| {
                    econ_bench.iter().find_map(|(k, v)| {
                        if norm_id.contains(k) || norm_name.contains(k) { Some(v) } else { None }
                    })
                });

            if let Some(eb) = econ_match {
                if let Some(cs) = eb.composite_score() {
                    scores.real_world = Some(cs);
                }
            }

            let final_params = params_b.or(lb_params);
            let size_gb = estimate_gguf_size(final_params, &name);
            let vram_rec_gb = ModelBenchmark::calculate_vram(size_gb);

            // Exact or parameter-specific speed matching with physical hardware capacity check
            let param_tag = final_params.map(|p| {
                if p >= 1.0 { format!("{:.0}b", p.round()) }
                else { format!("{:.1}b", p) }
            });

            let speed_match = speeds.get(&norm_id)
                .or_else(|| speeds.get(&norm_name))
                .or_else(|| {
                    if let Some(ref ptag) = param_tag {
                        speeds.iter().find_map(|(k, v)| {
                            if k.contains(&family.to_lowercase()) && k.contains(ptag) {
                                Some(v)
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    }
                });

            // Physical Gate: Reject benchmark records that violate GPU VRAM capacity or domestic physical limits
            let physically_valid_speed = speed_match.and_then(|s| {
                let is_24gb = s.accelerator.contains("4090") || s.accelerator.contains("24GB") || s.accelerator.contains("3090");
                if is_24gb && vram_rec_gb > 24.5 {
                    None
                } else if s.accelerator.contains("16GB") && vram_rec_gb > 16.5 {
                    None
                } else if s.accelerator.contains("8GB") && vram_rec_gb > 8.5 {
                    None
                } else if final_params.unwrap_or(0.0) >= 25.0 && s.tps > 75.0 {
                    // Outlier / server-grade cluster throughput impossible on single domestic GPU
                    None
                } else {
                    Some(s)
                }
            });

            let (tps, tps_hw) = match (physically_valid_speed, vram_match) {
                (Some(s), _) => (Some(s.tps), Some(s.accelerator.clone())),
                (None, Some(vb)) if vb.tok_s.is_some() && vram_rec_gb <= 24.5 && (vram_rec_gb < 16.0 || vb.tok_s.unwrap_or(0.0) <= 65.0) => {
                    (vb.tok_s, Some("GPU Unified".into()))
                }
                _ => estimate_tps(final_params),
            };
            let ollama_tag = generate_ollama_tag(&family, final_params);
            let hf_url = format!("https://huggingface.co/{}", hf.id);

            let is_moe = lb_moe
                || name.to_lowercase().contains("moe")
                || name.to_lowercase().contains("a3b")
                || hf.tags.iter().any(|t| t == "moe" || t == "mixture-of-experts");

            let release_year = extract_release_year(&hf.id, &name, hf.created_at.as_deref());
            let specialty = DomainSpecialty::detect_with_context(&hf.id, &name, hf.pipeline_tag.as_deref(), &hf.tags, "", Some(&scores), modality);

            results.push(ModelBenchmark {
                id: hf.id,
                name,
                family,
                param_count_b: final_params,
                size_gb,
                vram_rec_gb,
                tokens_per_sec: tps,
                tps_hardware: tps_hw,
                scores,
                downloads: hf.downloads,
                likes: hf.likes,
                ollama_tag,
                hf_url,
                is_moe,
                context_k: 32,
                is_cloud: false,
                provider: None,
                pricing_input_usd: None,
                pricing_output_usd: None,
                release_year,
                modality,
                specialty,
                economic: econ_match.cloned(),
            });
        }

        results
    }
}
