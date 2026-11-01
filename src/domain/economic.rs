use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct EconomicScores {
    /// Remote Labor Index (RLI) - Automation Rate % on real Upwork/Fiverr freelance tasks
    pub rli_automation_rate: Option<f32>,
    /// SWE-Lancer - Software dev performance % on >1,400 Upwork tasks ($1M pool)
    pub swe_lancer_score: Option<f32>,
    /// SWE-Lancer - Real dollars earned ($) by resolving freelance tasks
    pub swe_lancer_dollars: Option<f32>,
    /// SWE-Clar - Implementation score % on $4,000+ greenfield Upwork jobs
    pub swe_clar_impl: Option<f32>,
    /// SWE-Clar - Requirement clarification score % with simulated clients
    pub swe_clar_clarif: Option<f32>,
    /// OSWorld - Task success rate % in real OS environments (Ubuntu, Windows, macOS)
    pub os_world_success: Option<f32>,
    /// WebArena - Autonomous web navigation and task completion %
    pub web_arena_success: Option<f32>,
    /// MLE-bench - Overall All score % on 75 real Kaggle ML engineering competitions
    pub mle_bench_score: Option<f32>,
    /// MLE-bench - Summary of medals achieved (Low, Medium, High)
    pub mle_bench_medals: Option<String>,
}

impl EconomicScores {
    /// Normalized composite economic score calibrated on a 0-100 base
    pub fn composite_score(&self) -> Option<f32> {
        let mut vals = Vec::new();

        // RLI: Frontier models currently achieve ~15-25% automation rate.
        // Calibrate so 25% RLI corresponds to 100 on standard scale.
        if let Some(rli) = self.rli_automation_rate {
            vals.push(((rli / 25.0) * 100.0).clamp(0.0, 100.0));
        }

        if let Some(swe) = self.swe_lancer_score {
            vals.push(swe.clamp(0.0, 100.0));
        }

        if let Some(impl_score) = self.swe_clar_impl {
            let combined = if let Some(clarif) = self.swe_clar_clarif {
                impl_score * 0.6 + clarif * 0.4
            } else {
                impl_score
            };
            vals.push(combined.clamp(0.0, 100.0));
        }

        // OSWorld: Frontier models achieve ~30-40% success rate on open OS tasks.
        if let Some(os) = self.os_world_success {
            vals.push(((os / 38.0) * 100.0).clamp(0.0, 100.0));
        }

        if let Some(web) = self.web_arena_success {
            vals.push(web.clamp(0.0, 100.0));
        }

        if let Some(mle) = self.mle_bench_score {
            vals.push(mle.clamp(0.0, 100.0));
        }

        if vals.is_empty() {
            None
        } else {
            let sum: f32 = vals.iter().sum();
            Some(((sum / vals.len() as f32) * 10.0).round() / 10.0)
        }
    }

    /// Count of populated real-world benchmarks
    pub fn populated_count(&self) -> usize {
        (self.rli_automation_rate.is_some() as usize)
            + (self.swe_lancer_score.is_some() as usize)
            + (self.swe_clar_impl.is_some() as usize)
            + (self.os_world_success.is_some() as usize)
            + (self.web_arena_success.is_some() as usize)
            + (self.mle_bench_score.is_some() as usize)
    }
}
