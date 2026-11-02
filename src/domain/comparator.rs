use crate::domain::model::{ModelBenchmark, SourceSelection};
use crate::domain::simulator::HardwareRigConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelRole {
    FastRouter,
    HeavyReasoner,
    BalancedGeneral,
    OffloadWarning,
    CloudHosted,
}

impl ModelRole {
    pub fn badge_text(&self, is_fr: bool) -> (&'static str, &'static str) {
        match self {
            Self::FastRouter => {
                if is_fr {
                    ("[Rapide] Léger & Instantané", "Réponse ultra-rapide (<1s) : idéal pour tester ou vérifier vite")
                } else {
                    ("[Fast] Light & Instant", "Ultra-fast response (<1s): ideal for quick checks and tests")
                }
            }
            Self::HeavyReasoner => {
                if is_fr {
                    ("[Précis] Raisonneur & Exact", "Compréhension profonde : réponse juste du 1er coup sans reformuler")
                } else {
                    ("[Deep] Deep & Accurate", "Deep comprehension: right answer on the 1st try without re-prompting")
                }
            }
            Self::BalancedGeneral => {
                if is_fr {
                    ("[Equilibré] Polyvalent", "Bon compromis entre vitesse d'affichage et intelligence")
                } else {
                    ("[Balanced] Generalist", "Solid compromise between response speed and reasoning")
                }
            }
            Self::OffloadWarning => {
                if is_fr {
                    ("[Avertissement] Dépasse la VRAM", "Dépasse la mémoire GPU : tourne sur la RAM système (très ralenti)")
                } else {
                    ("[Warning] Exceeds VRAM", "Exceeds GPU VRAM: runs on system RAM (heavily throttled)")
                }
            }
            Self::CloudHosted => {
                if is_fr {
                    ("[Cloud] API Distante", "Modèle hébergé : 0 Go de VRAM locale, facturation API par jeton")
                } else {
                    ("[Cloud] Remote API", "Hosted model: 0 GB local VRAM required, billed per API token")
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompMetricX {
    #[default]
    SpeedTps,
    ResponseTimeSec,
    VramUsageGb,
    PowerWatts,
    ElectricCostEur,
}

impl CompMetricX {
    pub const ALL: [Self; 5] = [
        Self::SpeedTps,
        Self::ResponseTimeSec,
        Self::VramUsageGb,
        Self::PowerWatts,
        Self::ElectricCostEur,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::SpeedTps => "Vitesse Réelle (tok/s)",
            Self::ResponseTimeSec => "Temps de Réponse 200 Mots (sec)",
            Self::VramUsageGb => "Empreinte VRAM (Go)",
            Self::PowerWatts => "Puissance Electrique GPU (Watts)",
            Self::ElectricCostEur => "Coût (€/100k tokens)",
        }
    }

    pub fn extract(&self, m: &ModelHardwareMetrics) -> f64 {
        match self {
            Self::SpeedTps => m.estimated_tps as f64,
            Self::ResponseTimeSec => m.sec_per_200_words as f64,
            Self::VramUsageGb => m.vram_usage_gb as f64,
            Self::PowerWatts => m.power_watts as f64,
            Self::ElectricCostEur => m.cost_per_100k_tokens_eur as f64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompMetricY {
    #[default]
    Intelligence,
    CodingSweBench,
    AgenticScore,
    MathReasoning,
    EfficiencyRatio,
}

impl CompMetricY {
    pub const ALL: [Self; 5] = [
        Self::Intelligence,
        Self::CodingSweBench,
        Self::AgenticScore,
        Self::MathReasoning,
        Self::EfficiencyRatio,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Intelligence => "Score d'Intelligence Global (0-100)",
            Self::CodingSweBench => "Code & SWE-Bench (0-100)",
            Self::AgenticScore => "Agents & Outillage (0-100)",
            Self::MathReasoning => "Raisonnement & Math GPQA (0-100)",
            Self::EfficiencyRatio => "Rapport Efficience (Score / Go VRAM)",
        }
    }

    pub fn extract(&self, m: &ModelHardwareMetrics, model: &ModelBenchmark) -> f64 {
        match self {
            Self::Intelligence => m.intelligence_score as f64,
            Self::CodingSweBench => model.scores.coding.unwrap_or(m.intelligence_score) as f64,
            Self::AgenticScore => model.scores.agentic.unwrap_or(m.intelligence_score) as f64,
            Self::MathReasoning => model.scores.math_lvl5.or(model.scores.gpqa).unwrap_or(m.intelligence_score) as f64,
            Self::EfficiencyRatio => m.lightness_ratio as f64,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelHardwareMetrics {
    pub model_id: String,
    pub model_name: String,
    pub intelligence_score: f32,
    pub estimated_tps: f32,
    pub sec_per_200_words: f32,
    pub vram_usage_gb: f32,
    pub vram_headroom_gb: f32,
    pub fits_pure_vram: bool,
    pub power_watts: f32,
    pub kwh_per_100k_tokens: f32,
    pub cost_per_100k_tokens_eur: f32,
    pub lightness_ratio: f32,
    pub role: ModelRole,
    pub is_cloud: bool,
}

pub struct HardwareComparator;

impl HardwareComparator {
    pub fn compute_metrics(
        model: &ModelBenchmark,
        rig: &HardwareRigConfig,
        sources: &SourceSelection,
    ) -> ModelHardwareMetrics {
        let intelligence = model
            .score_for_sources(sources)
            .unwrap_or_else(|| model.scores.average.unwrap_or(50.0));

        if model.is_cloud {
            let api_tps = model.tokens_per_sec.unwrap_or(95.0);
            let in_usd = model.pricing_input_usd.unwrap_or(3.0);
            let out_usd = model.pricing_output_usd.unwrap_or(10.0);
            let cost_100k_eur = ((in_usd * 0.03 + out_usd * 0.07) * 0.92) as f32;

            return ModelHardwareMetrics {
                model_id: model.id.clone(),
                model_name: model.name.clone(),
                intelligence_score: intelligence,
                estimated_tps: api_tps,
                sec_per_200_words: (250.0 / api_tps.max(1.0)).clamp(0.1, 30.0),
                vram_usage_gb: 0.0,
                vram_headroom_gb: rig.gpu.vram_gb(),
                fits_pure_vram: true,
                power_watts: 0.0,
                kwh_per_100k_tokens: 0.0,
                cost_per_100k_tokens_eur: cost_100k_eur,
                lightness_ratio: intelligence,
                role: ModelRole::CloudHosted,
                is_cloud: true,
            };
        }

        let size_gb = if model.vram_rec_gb > 0.0 {
            model.vram_rec_gb
        } else if model.size_gb > 0.0 {
            model.size_gb
        } else {
            4.0
        };

        let vram_avail = if rig.gpu.tdp_watts() <= 25.0 {
            (rig.gpu.vram_gb() - 0.2).max(0.0)
        } else {
            (rig.gpu.vram_gb() - 0.6).max(0.0)
        };
        let fits_pure_vram = size_gb <= vram_avail;
        let vram_headroom = vram_avail - size_gb;

        let estimated_tps: f32 = if fits_pure_vram {
            let bw = rig.gpu.bandwidth_gbps();
            ((bw / size_gb.max(0.5)) * 0.82_f32).clamp(1.5_f32, 350.0_f32)
        } else {
            let bw: f32 = 25.0_f32.min(rig.gpu.bandwidth_gbps() * 0.4);
            ((bw / size_gb.max(0.5)) * 0.60_f32).clamp(0.2_f32, 20.0_f32)
        };

        let sec_per_200_words = (250.0_f32 / estimated_tps.max(0.1_f32)).clamp(0.1_f32, 120.0_f32);

        let base_watts = if rig.gpu.tdp_watts() <= 25.0 { 5.0 } else { 85.0 };
        let gpu_power = rig.gpu.tdp_watts() * if fits_pure_vram { 0.85 } else { 0.40 };
        let power_watts = base_watts + gpu_power;

        let duration_hours = (100_000.0 / estimated_tps.max(0.5)) / 3600.0;
        let kwh_per_100k = (power_watts * duration_hours) / 1000.0;
        let cost_per_100k = kwh_per_100k * 0.25;

        let lightness_ratio = (intelligence / size_gb.max(0.5)).round();

        let role = if !fits_pure_vram {
            ModelRole::OffloadWarning
        } else if estimated_tps >= 55.0 && size_gb <= 9.0 {
            ModelRole::FastRouter
        } else if intelligence >= 70.0 {
            ModelRole::HeavyReasoner
        } else {
            ModelRole::BalancedGeneral
        };

        ModelHardwareMetrics {
            model_id: model.id.clone(),
            model_name: model.name.clone(),
            intelligence_score: intelligence,
            estimated_tps,
            sec_per_200_words,
            vram_usage_gb: size_gb,
            vram_headroom_gb: vram_headroom,
            fits_pure_vram,
            power_watts,
            kwh_per_100k_tokens: kwh_per_100k,
            cost_per_100k_tokens_eur: cost_per_100k,
            lightness_ratio,
            role,
            is_cloud: false,
        }
    }
}
