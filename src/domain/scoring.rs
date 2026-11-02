use serde::{Deserialize, Serialize};
use crate::domain::model::{BenchmarkScores, SourceSelection};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreConfidence {
    High,
    Medium,
    Low,
}

impl ScoreConfidence {
    pub fn badge(&self) -> &'static str {
        match self {
            ScoreConfidence::High => "[OK]",
            ScoreConfidence::Medium => "[~]",
            ScoreConfidence::Low => "[!]",
        }
    }

    pub fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            ScoreConfidence::High => (46, 204, 113),
            ScoreConfidence::Medium => (241, 196, 15),
            ScoreConfidence::Low => (231, 76, 60),
        }
    }

    pub fn label_fr(&self) -> &'static str {
        match self {
            ScoreConfidence::High => "Élevé (Haute Fiabilité)",
            ScoreConfidence::Medium => "Modéré (Fiabilité Moyenne)",
            ScoreConfidence::Low => "Faible (Prudence)",
        }
    }

    pub fn label_en(&self) -> &'static str {
        match self {
            ScoreConfidence::High => "High (High Reliability)",
            ScoreConfidence::Medium => "Medium (Moderate Reliability)",
            ScoreConfidence::Low => "Low (Caution)",
        }
    }

    pub fn tooltip_fr(&self) -> &'static str {
        match self {
            ScoreConfidence::High => "Indice de Confiance : [OK] ÉLEVÉ\n• 3+ benchmarks croisés concordants\n• Faible variance entre les sources (< 16 pts)\n• Résultat stable, robuste et hautement reproductible.",
            ScoreConfidence::Medium => "Indice de Confiance : [~] MODÉRÉ\n• Sources de benchmarks limitées ou variance modérée\n• Estimation indicative fiable mais à affiner avec de futurs tests.",
            ScoreConfidence::Low => "Indice de Confiance : [!] FAIBLE\n• Données parcellaires ou forte divergence entre les sources (> 25 pts)\n• Résultat à considérer avec précaution.",
        }
    }

    pub fn tooltip_en(&self) -> &'static str {
        match self {
            ScoreConfidence::High => "Confidence Index: [OK] HIGH\n• At least 3 agreeing cross-referenced benchmarks\n• Low variance between sources (< 16 pts)\n• Highly robust and reproducible result.",
            ScoreConfidence::Medium => "Confidence Index: [~] MODERATE\n• Limited benchmark sources or moderate variance\n• Reliable indicative estimation, subject to refinement.",
            ScoreConfidence::Low => "Confidence Index: [!] LOW\n• Sparse data or significant divergence between sources (> 25 pts)\n• Use with caution.",
        }
    }
}

pub struct ScoreBreakdown {
    pub final_score: f32,
    pub confidence: ScoreConfidence,
    pub variance: f32,
    pub formula: String,
    pub quant_factor: f32,
    pub time_factor: f32,
}

pub struct ScoringEngine;

impl ScoringEngine {
    pub const WEIGHT_CODE: f32 = 1.5;
    pub const WEIGHT_REAL_WORLD: f32 = 1.3;
    pub const WEIGHT_MMLU: f32 = 1.2;
    pub const WEIGHT_AGENT: f32 = 1.0;
    pub const WEIGHT_GENERAL: f32 = 0.8;

    pub fn extract_quant_factor(name_or_id: &str) -> (f32, &'static str) {
        let lower = name_or_id.to_lowercase();
        if lower.contains("fp16") || lower.contains("bf16") || lower.contains("float16") {
            (1.0, "FP16 (Base)")
        } else if lower.contains("q8_0") || lower.contains("q8") {
            (0.985, "Q8_0 (-1.5%)")
        } else if lower.contains("q6_k") || lower.contains("q6") {
            (0.975, "Q6_K (-2.5%)")
        } else if lower.contains("q5_k_m") || lower.contains("q5_k") || lower.contains("q5_0") {
            (0.96, "Q5_K_M (-4.0%)")
        } else if lower.contains("q4_k_m") || lower.contains("q4_0") || lower.contains("q4_1") || lower.contains("q4") {
            (0.93, "Q4_K_M (-7.0%)")
        } else if lower.contains("q3_k") || lower.contains("q3") || lower.contains("q2") {
            (0.84, "Q3/Q2 Low (-16%)")
        } else {
            (0.95, "Quantized Standard")
        }
    }

    pub fn compute_time_decay(year: Option<u16>) -> f32 {
        match year {
            Some(y) if y >= 2026 => 1.0,
            Some(2025) => 0.97,
            Some(2024) => 0.91,
            Some(2023) => 0.82,
            Some(_) => 0.70,
            None => 1.0,
        }
    }

    /// Evaluates benchmark coverage and reliability without imposing artificial parameter ceilings.
    /// Practical verified benchmarks (SWE-Bench, BFCL, RealWorld) receive 100% full merit (1.0).
    /// Unverified isolated academic QCMs receive a confidence penalty to account for data contamination.
    pub fn compute_source_coverage_factor(
        scores: &BenchmarkScores,
        is_cloud: bool,
    ) -> (f32, ScoreConfidence) {
        if is_cloud {
            return (1.0, ScoreConfidence::High);
        }

        let has_code = scores.coding.is_some();
        let has_agent = scores.agentic.is_some();
        let has_real_world = scores.real_world.is_some();
        let has_academic = scores.mmlu_pro.is_some() || scores.open_llm.is_some() || scores.average.is_some();

        let practical_sources = (has_code as usize) + (has_agent as usize) + (has_real_world as usize);

        if practical_sources >= 2 {
            (1.0, ScoreConfidence::High)
        } else if practical_sources == 1 {
            (1.0, ScoreConfidence::Medium)
        } else if has_academic {
            (0.88, ScoreConfidence::Low)
        } else {
            (0.80, ScoreConfidence::Low)
        }
    }

    pub fn aggregate(
        scores: &BenchmarkScores,
        sel: &SourceSelection,
        is_cloud: bool,
        quant_factor: f32,
        time_factor: f32,
    ) -> Option<ScoreBreakdown> {
        let mut items = Vec::new();

        if sel.coding {
            if let Some(c) = scores.coding {
                items.push((c, Self::WEIGHT_CODE, "Code"));
            }
        }
        if sel.open_llm {
            if let Some(m) = scores.mmlu_pro {
                items.push((m, Self::WEIGHT_MMLU, "MMLU"));
            } else if let Some(g) = scores.open_llm.or(scores.average) {
                items.push((g, Self::WEIGHT_GENERAL, "General"));
            }
        }
        if sel.agentic {
            if let Some(a) = scores.agentic {
                items.push((a, Self::WEIGHT_AGENT, "Agent"));
            }
        }
        if sel.real_world {
            if let Some(rw) = scores.real_world {
                items.push((rw, Self::WEIGHT_REAL_WORLD, "RealWorld"));
            }
        }

        if items.is_empty() {
            return None;
        }

        let raw_values: Vec<f32> = items.iter().map(|(val, _, _)| *val).collect();
        let min_val = raw_values.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_val = raw_values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let variance = (max_val - min_val).max(0.0);

        let (coverage_factor, coverage_confidence) = Self::compute_source_coverage_factor(scores, is_cloud);
        let weighted_median = Self::weighted_median(&items);

        // Apply quantization factor, generational time decay, and source coverage confidence
        let adjusted = weighted_median * quant_factor * time_factor * coverage_factor;
        let final_score = (adjusted.clamp(0.0, 100.0) * 10.0).round() / 10.0;

        let confidence = if coverage_confidence == ScoreConfidence::Low || variance > 25.0 {
            ScoreConfidence::Low
        } else if coverage_confidence == ScoreConfidence::High && items.len() >= 3 && variance < 16.0 {
            ScoreConfidence::High
        } else {
            ScoreConfidence::Medium
        };

        let formula = if (coverage_factor - 1.0).abs() > 0.001 {
            format!(
                "Median({:.1}) * Quant({:.2}) * Time({:.2}) * Cov({:.2}) = {:.1}",
                weighted_median, quant_factor, time_factor, coverage_factor, final_score
            )
        } else {
            format!(
                "Median({:.1}) * Quant({:.2}) * Time({:.2}) = {:.1}",
                weighted_median, quant_factor, time_factor, final_score
            )
        };

        Some(ScoreBreakdown {
            final_score,
            confidence,
            variance: (variance * 10.0).round() / 10.0,
            formula,
            quant_factor,
            time_factor,
        })
    }

    fn weighted_median(items: &[(f32, f32, &str)]) -> f32 {
        if items.is_empty() {
            return 0.0;
        }
        if items.len() == 1 {
            return items[0].0;
        }

        let mut sorted = items.to_vec();
        sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let total_weight: f32 = sorted.iter().map(|(_, w, _)| *w).sum();
        let half_weight = total_weight / 2.0;

        let mut cum_weight = 0.0;
        for (val, w, _) in &sorted {
            cum_weight += w;
            if cum_weight >= half_weight {
                return *val;
            }
        }

        sorted.last().map(|(v, _, _)| *v).unwrap_or(0.0)
    }
}
