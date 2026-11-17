use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::domain::model::{ModelBenchmark, SourceSelection};
use crate::domain::simulator::{HardwareRigConfig, RigSimulationResult, SimTargetMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimFitStatus {
    Comfortable,
    Tight,
    CpuOffload,
}

impl SimFitStatus {
    pub fn label(&self, is_fr: bool) -> &'static str {
        match (self, is_fr) {
            (Self::Comfortable, true) => "[OK] VRAM Confort",
            (Self::Comfortable, false) => "[OK] Pure VRAM",
            (Self::Tight, true) => "[~] Marge Serrée",
            (Self::Tight, false) => "[~] Tight Fit",
            (Self::CpuOffload, true) => "[!] Offload RAM",
            (Self::CpuOffload, false) => "[!] RAM Offload",
        }
    }

    pub fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Comfortable => (46, 204, 113),
            Self::Tight => (241, 196, 15),
            Self::CpuOffload => (255, 152, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimTableSort {
    Score,
    Coding,
    Agentic,
    Speed,
    Vram,
    Efficiency,
}

#[derive(Debug, Clone)]
pub struct SimulatedModelEntry {
    pub id: String,
    pub name: String,
    pub root_name: String,
    pub family: String,
    pub modality_tag: &'static str,
    pub score: f32,
    pub coding: Option<f32>,
    pub agentic: Option<f32>,
    pub size_gb: f32,
    pub vram_rec_gb: f32,
    pub memory_margin_gb: f32,
    pub est_tps: f32,
    pub efficiency: f32,
    pub fit_status: SimFitStatus,
}

/// Evaluates all local models compatible with the selected hardware configuration.
pub fn rank_compatible_models(
    all_models: &[ModelBenchmark],
    config: &HardwareRigConfig,
    sim: &RigSimulationResult,
    sources: &SourceSelection,
    sort_by: SimTableSort,
    deduplicate_roots: bool,
) -> Vec<SimulatedModelEntry> {
    let mut entries = Vec::new();
    let max_cap = sim.total_ai_capacity_gb;

    for m in all_models {
        if m.is_cloud || m.vram_rec_gb > max_cap {
            continue;
        }
        if let Some(max_s) = config.max_storage_gb {
            if m.size_gb > max_s {
                continue;
            }
        }

        let score = m.score_for_sources(sources).unwrap_or(0.0);
        let eff_gb = m.size_gb.max(0.3);
        let is_pure_vram = config.mode == SimTargetMode::WorkstationPc
            && !config.gpu.is_embedded()
            && m.vram_rec_gb <= sim.usable_vram_gb;

        let fit_status = if is_pure_vram && (sim.usable_vram_gb - m.vram_rec_gb) >= 1.5 {
            SimFitStatus::Comfortable
        } else if is_pure_vram || (config.gpu.is_embedded() && (max_cap - m.vram_rec_gb) >= 1.2) {
            SimFitStatus::Tight
        } else {
            SimFitStatus::CpuOffload
        };

        let bw_factor = if is_pure_vram {
            sim.effective_bandwidth_gbps
        } else if config.gpu.is_embedded() {
            sim.effective_bandwidth_gbps * 0.88
        } else {
            // Hybrid PCIe + DDR offload bottleneck
            (sim.effective_bandwidth_gbps * 0.14).clamp(18.0, 55.0)
        };
        let moe_bonus = if m.is_moe { 1.45 } else { 1.0 };
        let est_tps = ((bw_factor / eff_gb) * 0.78 * moe_bonus).clamp(1.0, 260.0);

        let root_name = if m.param_count_b.is_some() || m.is_cloud {
            m.root_model_name()
        } else {
            m.name.clone()
        };

        entries.push(SimulatedModelEntry {
            id: m.id.clone(),
            name: m.name.clone(),
            root_name,
            family: m.family.clone(),
            modality_tag: m.modality.tag_short(),
            score,
            coding: m.scores.coding,
            agentic: m.scores.agentic,
            size_gb: m.size_gb,
            vram_rec_gb: m.vram_rec_gb,
            memory_margin_gb: (max_cap - m.vram_rec_gb).max(0.0),
            est_tps,
            efficiency: m.efficiency_ratio_with_sources(sources),
            fit_status,
        });
    }

    entries.sort_by(|a, b| {
        let ord = match sort_by {
            SimTableSort::Score => b.score.total_cmp(&a.score),
            SimTableSort::Coding => b.coding.unwrap_or(0.0).total_cmp(&a.coding.unwrap_or(0.0)),
            SimTableSort::Agentic => b.agentic.unwrap_or(0.0).total_cmp(&a.agentic.unwrap_or(0.0)),
            SimTableSort::Speed => b.est_tps.total_cmp(&a.est_tps),
            SimTableSort::Vram => a.vram_rec_gb.total_cmp(&b.vram_rec_gb),
            SimTableSort::Efficiency => b.efficiency.total_cmp(&a.efficiency),
        };
        ord.then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.id.cmp(&b.id))
    });

    if deduplicate_roots {
        let mut seen = HashSet::new();
        entries.retain(|e| seen.insert(e.root_name.clone()));
    }
    entries
}

/// Sorts already-ranked compatible entries by visual table column index and direction.
pub fn sort_simulated_entries(
    entries: &mut [SimulatedModelEntry],
    sort_col: usize,
    ascending: bool,
    is_fr: bool,
) {
    entries.sort_by(|a, b| {
        let ord = match sort_col {
            1 => cmp_str(&a.name.to_lowercase(), &b.name.to_lowercase(), ascending),
            2 => cmp_str(a.modality_tag, b.modality_tag, ascending),
            4 => cmp_opt_f32(a.coding, b.coding, ascending),
            5 => cmp_opt_f32(a.agentic, b.agentic, ascending),
            6 => cmp_f32(a.vram_rec_gb, b.vram_rec_gb, ascending),
            7 => cmp_f32(a.est_tps, b.est_tps, ascending),
            8 => cmp_str(a.fit_status.label(is_fr), b.fit_status.label(is_fr), ascending),
            99 => cmp_f32(a.efficiency, b.efficiency, ascending),
            _ => cmp_f32(a.score, b.score, ascending),
        };
        ord.then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.id.cmp(&b.id))
    });
}

fn cmp_str(a: &str, b: &str, asc: bool) -> std::cmp::Ordering {
    let c = a.cmp(b);
    if asc { c } else { c.reverse() }
}

fn cmp_f32(a: f32, b: f32, asc: bool) -> std::cmp::Ordering {
    if asc { a.total_cmp(&b) } else { b.total_cmp(&a) }
}

fn cmp_opt_f32(a: Option<f32>, b: Option<f32>, asc: bool) -> std::cmp::Ordering {
    match (a, b) {
        (Some(x), Some(y)) => cmp_f32(x, y, asc),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}
