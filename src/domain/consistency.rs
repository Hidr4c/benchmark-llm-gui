use crate::domain::model::ModelBenchmark;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsistencyIssue {
    ScoreDivergence,
    ExtremeThroughputForHardware,
    UncalibratedHeavyModel,
    HighVarianceAcrossSources,
    TemporalSizeAnomaly,
}

pub struct ModelConsistencyValidator;

impl ModelConsistencyValidator {
    pub fn validate(model: &ModelBenchmark) -> Vec<ConsistencyIssue> {
        let mut issues = Vec::new();

        let mmlu = model.scores.mmlu_pro.or(model.scores.open_llm).unwrap_or(0.0);
        let coding = model.scores.coding.unwrap_or(0.0);
        let _agentic = model.scores.agentic.unwrap_or(0.0);
        let speed = model.tokens_per_sec.unwrap_or(0.0);
        let params = model.param_count_b.unwrap_or(0.0);
        let hw = model.tps_hardware.as_deref().unwrap_or("");

        // 1. Critical score divergence: extreme coding specialization vs abysmal knowledge
        if coding >= 75.0 && mmlu <= 42.0 && mmlu > 0.0 {
            issues.push(ConsistencyIssue::ScoreDivergence);
        }

        // 2. High variance rule: max - min > 35.0 pts across available benchmark sources
        let mut available_scores = Vec::new();
        if let Some(c) = model.scores.coding { available_scores.push(c); }
        if let Some(m) = model.scores.mmlu_pro.or(model.scores.open_llm) { available_scores.push(m); }
        if let Some(a) = model.scores.agentic { available_scores.push(a); }

        if available_scores.len() >= 2 {
            let min_s = available_scores.iter().cloned().fold(f32::INFINITY, f32::min);
            let max_s = available_scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            if max_s - min_s > 35.0 {
                issues.push(ConsistencyIssue::HighVarianceAcrossSources);
            }
        }

        // 3. Unrealistic speed on modest hardware (> 75 tok/s for >= 28B without NVFP4/flagship GPU)
        if params >= 28.0 && speed > 75.0 {
            let is_flagship_hw = hw.contains("5090") || hw.contains("H100") || hw.contains("A100") || hw.contains("Server") || hw.contains("NVFP4");
            if !is_flagship_hw {
                issues.push(ConsistencyIssue::ExtremeThroughputForHardware);
            }
        }

        // 4. Temporal & size anomaly: older small model reporting flagship score > 85.0
        let size = if model.size_gb > 0.0 { model.size_gb } else { model.param_count_b.unwrap_or(0.0) * 0.6 };
        let year = model.release_year.unwrap_or(2026);
        if year <= 2024 && size <= 4.0 && size > 0.0 && (mmlu > 85.0 || coding > 88.0) {
            issues.push(ConsistencyIssue::TemporalSizeAnomaly);
        }

        // 5. High memory footprint with sub-par intelligence
        if model.size_gb >= 35.0 && mmlu < 30.0 && mmlu > 0.0 {
            issues.push(ConsistencyIssue::UncalibratedHeavyModel);
        }

        issues
    }

    /// Systemic cross-comparison check between two models:
    /// IF (Year_A < Year_B) AND (Size_A < Size_B) THEN Score_A cannot exceed Score_B + 5.0
    pub fn check_temporal_size_inversion(
        model_a: &ModelBenchmark,
        score_a: f32,
        model_b: &ModelBenchmark,
        score_b: f32,
    ) -> bool {
        let year_a = model_a.release_year.unwrap_or(2025);
        let year_b = model_b.release_year.unwrap_or(2026);
        let size_a = if model_a.size_gb > 0.0 { model_a.size_gb } else { model_a.param_count_b.unwrap_or(0.0) * 0.6 };
        let size_b = if model_b.size_gb > 0.0 { model_b.size_gb } else { model_b.param_count_b.unwrap_or(0.0) * 0.6 };

        if year_a < year_b && size_a < size_b && score_a > score_b + 5.0 {
            return true;
        }
        false
    }
}
