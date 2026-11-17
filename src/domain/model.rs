use serde::{Deserialize, Serialize};
use crate::domain::economic::EconomicScores;
use crate::domain::modality::ModelModality;
use crate::domain::specialty::DomainSpecialty;
use crate::domain::scoring::{ScoreBreakdown, ScoringEngine};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortField {
    Intelligence,
    Efficiency,
    Speed,
    Size,
    Memory,
    Downloads,
    Name,
    Price,
    Year,
    RealWorld,
    Coding,
    Agentic,
    Confidence,
    Modality,
    ImageBench,
    VideoBench,
    AudioBench,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder { Ascending, Descending }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSelection {
    pub open_llm: bool,
    pub coding: bool,
    pub agentic: bool,
    pub real_world: bool,
}

impl Default for SourceSelection {
    fn default() -> Self {
        Self { open_llm: true, coding: true, agentic: true, real_world: true }
    }
}

impl SourceSelection {
    pub fn dev_agentic() -> Self { Self { open_llm: false, coding: true, agentic: true, real_world: false } }
    pub fn is_all(&self) -> bool { self.open_llm && self.coding && self.agentic && self.real_world }
    pub fn is_open_llm_only(&self) -> bool { self.open_llm && !self.coding && !self.agentic && !self.real_world }
    pub fn is_coding_only(&self) -> bool { !self.open_llm && self.coding && !self.agentic && !self.real_world }
    pub fn is_agentic_only(&self) -> bool { !self.open_llm && !self.coding && self.agentic && !self.real_world }
    pub fn is_dev_agentic(&self) -> bool { !self.open_llm && self.coding && self.agentic && !self.real_world }
    pub fn is_real_world_only(&self) -> bool { !self.open_llm && !self.coding && !self.agentic && self.real_world }
    pub fn active_count(&self) -> usize { (self.open_llm as usize) + (self.coding as usize) + (self.agentic as usize) + (self.real_world as usize) }
    pub fn label(&self) -> &'static str {
        if self.is_all() {
            "Cross-Referenced Pillars (All)"
        } else if self.is_dev_agentic() {
            "Agentic Coding (Code + Tools)"
        } else if self.is_open_llm_only() {
            "General & Academic (Open LLM)"
        } else if self.is_coding_only() {
            "Code & Patching (SWE-Bench)"
        } else if self.is_agentic_only() {
            "Agents & Tool-Calling (BFCL)"
        } else if self.is_real_world_only() {
            "Real-World & Economic (RLI, SWE-Lancer)"
        } else {
            "Custom Blend"
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkScores {
    pub open_llm: Option<f32>,
    pub ifeval: Option<f32>,
    pub bbh: Option<f32>,
    pub math_lvl5: Option<f32>,
    pub gpqa: Option<f32>,
    pub mmlu_pro: Option<f32>,
    pub musr: Option<f32>,
    pub coding: Option<f32>,
    pub agentic: Option<f32>,
    pub average: Option<f32>,
    pub real_world: Option<f32>,
    #[serde(default)]
    pub image_bench: Option<f32>,
    #[serde(default)]
    pub video_bench: Option<f32>,
    #[serde(default)]
    pub audio_bench: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelBenchmark {
    pub id: String,
    pub name: String,
    pub family: String,
    pub param_count_b: Option<f32>,
    pub size_gb: f32,
    pub vram_rec_gb: f32,
    pub tokens_per_sec: Option<f32>,
    pub tps_hardware: Option<String>,
    pub scores: BenchmarkScores,
    pub downloads: u64,
    pub likes: u64,
    pub ollama_tag: Option<String>,
    pub hf_url: String,
    pub is_moe: bool,
    pub context_k: u32,
    pub is_cloud: bool,
    pub provider: Option<String>,
    pub pricing_input_usd: Option<f32>,
    pub pricing_output_usd: Option<f32>,
    pub release_year: Option<u16>,
    pub modality: ModelModality,
    pub specialty: DomainSpecialty,
    pub economic: Option<EconomicScores>,
}

impl ModelBenchmark {
    pub fn license(&self) -> crate::domain::license::ModelLicense {
        crate::domain::license::ModelLicense::detect(&self.id, self.provider.as_deref(), self.is_cloud)
    }

    pub fn calculate_vram(size_gb: f32) -> f32 {
        if size_gb <= 0.4 { 0.8 } else if size_gb <= 1.2 { size_gb + 0.6 } else { (size_gb + 1.8).max(2.0) }
    }

    pub fn calibrate_score(raw: f32, domain: &str) -> f32 {
        let scaled = if raw < 50.0 && raw > 0.0 {
            match domain {
                "open_llm" => (raw / 55.0) * 100.0,
                "math" => (raw / 45.0) * 100.0,
                "gpqa" => (raw / 35.0) * 100.0,
                _ => raw,
            }
        } else {
            raw
        };
        (scaled.clamp(0.0, 100.0) * 10.0).round() / 10.0
    }

    pub fn intelligence(&self) -> Option<f32> {
        self.score_for_sources(&SourceSelection::default())
    }

    pub fn score_breakdown(&self, sel: &SourceSelection) -> Option<ScoreBreakdown> {
        let (quant_factor, _) = if self.is_cloud {
            (1.0, "Full Precision Cloud API")
        } else {
            ScoringEngine::extract_quant_factor(&self.id)
        };
        let time_factor = ScoringEngine::compute_time_decay(self.release_year);
        ScoringEngine::aggregate(&self.scores, sel, self.is_cloud, quant_factor, time_factor)
    }

    pub fn score_for_sources(&self, sel: &SourceSelection) -> Option<f32> {
        self.score_breakdown(sel).map(|b| b.final_score)
    }

    pub fn size_mb(&self) -> f32 { self.size_gb * 1024.0 }

    pub fn efficiency_ratio(&self) -> f32 {
        let score = self.intelligence().unwrap_or(15.0);
        (score / self.size_gb.max(0.2) * 10.0).round() / 10.0
    }

    pub fn efficiency_ratio_with_sources(&self, sel: &SourceSelection) -> f32 {
        let score = self.score_for_sources(sel).unwrap_or(15.0);
        (score / self.size_gb.max(0.2) * 10.0).round() / 10.0
    }

    pub fn is_embedded_friendly(&self) -> bool { self.size_gb <= 4.0 && !self.is_cloud }

    pub fn embedded_recommendation(&self) -> &'static str {
        if self.is_cloud {
            "Remote Cloud API (No local VRAM required)"
        } else if self.size_gb <= 0.45 {
            "Micro-IoT / ESP32-P4 / 1 GB SBC (Ultra-Light)"
        } else if self.size_gb <= 1.5 {
            "Raspberry Pi 4 (2-4 GB RAM) CPU llama.cpp"
        } else if self.size_gb <= 3.2 {
            "Raspberry Pi 5 (4-8 GB) / Orange Pi 5 NPU"
        } else if self.size_gb <= 5.5 {
            "Jetson Orin Nano / Mini-PC N100 (8 GB)"
        } else {
            "Domestic PC / Server Required (Not suited for light edge)"
        }
    }

    pub fn hardware_color_rgb(&self) -> (u8, u8, u8) {
        if self.is_cloud {
            (147, 112, 219) // Purple Cloud
        } else if self.size_gb <= 0.5 {
            (0, 188, 212)
        } else if self.size_gb <= 2.0 {
            (76, 175, 80)
        } else if self.vram_rec_gb <= 8.5 {
            (100, 200, 115)
        } else if self.vram_rec_gb <= 12.5 {
            (33, 150, 243)
        } else if self.vram_rec_gb <= 16.5 {
            (156, 39, 176)
        } else if self.vram_rec_gb <= 24.5 {
            (233, 30, 99)
        } else if self.vram_rec_gb <= 32.5 {
            (255, 152, 0)
        } else {
            (244, 67, 54)
        }
    }

    pub fn root_model_name(&self) -> String {
        crate::domain::model_naming::compute_root_model_name(self)
    }

    pub fn release_year_label(&self) -> String {
        self.release_year.map(|y| y.to_string()).unwrap_or_else(|| "-".into())
    }

    pub fn is_very_recent(&self) -> bool {
        self.release_year.is_some_and(|y| y >= 2025)
    }

    pub fn estimated_hardware_tdp_watts(&self) -> f32 {
        crate::domain::pricing::ModelPricing::estimated_hardware_tdp_watts(self.is_cloud, self.size_gb, self.vram_rec_gb)
    }

    pub fn estimated_kwh_per_1m(&self) -> f32 {
        crate::domain::pricing::ModelPricing::estimated_kwh_per_1m(self.is_cloud, self.size_gb, self.vram_rec_gb, self.tokens_per_sec)
    }

    pub fn estimated_electricity_cost_usd_per_1m(&self) -> f32 {
        crate::domain::pricing::ModelPricing::estimated_electricity_cost_usd_per_1m(self.is_cloud, self.size_gb, self.vram_rec_gb, self.tokens_per_sec)
    }

    pub fn estimated_cost_100_tasks(&self) -> f32 {
        crate::domain::pricing::ModelPricing::estimated_cost_100_tasks(
            self.is_cloud, self.pricing_input_usd, self.pricing_output_usd,
            self.size_gb, self.vram_rec_gb, self.tokens_per_sec,
        )
    }
}
