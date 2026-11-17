use serde::{Deserialize, Serialize};
use crate::domain::model::ModelBenchmark;
use crate::domain::simulator::{HardwareRigConfig, RigSimulationResult, SimTargetMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KvCacheType {
    F16,
    Q8_0,
    Q4_0,
}

impl KvCacheType {
    pub fn cli_flag(&self) -> &'static str {
        match self {
            Self::F16 => "f16",
            Self::Q8_0 => "q8_0",
            Self::Q4_0 => "q4_0",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::F16 => "FP16 (100% précision)",
            Self::Q8_0 => "Q8_0 (-47% VRAM KV)",
            Self::Q4_0 => "Q4_0 (-72% VRAM KV)",
        }
    }

    pub fn memory_factor(&self) -> f32 {
        match self {
            Self::F16 => 1.0,
            Self::Q8_0 => 0.53,
            Self::Q4_0 => 0.28,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KvContextOption {
    pub kv_type: KvCacheType,
    pub max_safe_ctx: u32,
    pub kv_vram_gb: f32,
}

#[derive(Debug, Clone)]
pub struct OptimalRuntimePlan {
    pub model_name: String,
    pub recommended_quant: &'static str,
    pub weight_vram_gb: f32,
    pub kv_cache: KvCacheType,
    pub recommended_ctx: u32,
    pub kv_vram_gb: f32,
    pub compute_overhead_gb: f32,
    pub total_vram_gb: f32,
    pub gpu_offload_pct: u8,
    pub recommended_ngl: u16,
    pub flash_attention: bool,
    pub batch_size: u16,
    pub context_options: Vec<KvContextOption>,
    pub llama_cmd: String,
    pub ollama_cmd: String,
}

/// Computes optimal inference settings (`Quant`, `KV Cache`, `n_ctx`, `-ngl`, `-fa`, `-b`)
/// for a given model on the simulated hardware configuration.
pub fn compute_optimal_runtime_plan(
    model: &ModelBenchmark,
    config: &HardwareRigConfig,
    sim: &RigSimulationResult,
) -> OptimalRuntimePlan {
    let params_b = model.param_count_b.unwrap_or_else(|| (model.size_gb / 0.58).max(0.5)).max(0.2);
    let native_ctx = (model.context_k.max(4) as u32).saturating_mul(1024).min(262_144);
    let is_pure_cpu = sim.usable_vram_gb <= 0.1;
    let total_budget = if is_pure_cpu {
        sim.total_ai_capacity_gb
    } else if config.mode == SimTargetMode::WorkstationPc && !config.gpu.is_embedded() {
        sim.usable_vram_gb
    } else {
        sim.total_ai_capacity_gb
    };

    let compute_overhead_gb = if is_pure_cpu {
        0.4
    } else if total_budget >= 20.0 {
        1.2
    } else if total_budget >= 8.0 {
        0.8
    } else {
        0.45
    };

    // Baseline KV footprint per 1k tokens in FP16 for GQA transformers
    let fp16_kv_per_1k = (0.032 + params_b * 0.011).clamp(0.035, 0.48);

    // Choose highest quality quantization that leaves reasonable KV budget (prefer >= 16k Q8_0)
    let quant_tiers: [(&str, f32); 5] = [
        ("Q8_0", 1.05),
        ("Q6_K", 0.82),
        ("Q5_K_M", 0.72),
        ("Q4_K_M", 0.58),
        ("IQ4_XS", 0.48),
    ];
    let min_kv_want = (fp16_kv_per_1k * 16.0 * KvCacheType::Q8_0.memory_factor()).min(4.0);
    let mut chosen_quant = "Q4_K_M";
    let mut weight_vram_gb = (params_b * 0.58).max(0.25);

    if !model.is_cloud && model.size_gb > 0.05 && model.param_count_b.is_none() {
        weight_vram_gb = model.size_gb;
    } else {
        for (q_name, mult) in quant_tiers {
            let est_w = (params_b * mult).max(0.25);
            if est_w + compute_overhead_gb + min_kv_want <= total_budget {
                chosen_quant = q_name;
                weight_vram_gb = est_w;
                break;
            }
        }
        // If even IQ4_XS + desired 16k headroom exceeds pure budget, check raw weight fit before offload
        if weight_vram_gb + compute_overhead_gb > total_budget {
            let iq4 = (params_b * 0.48).max(0.25);
            if iq4 + compute_overhead_gb <= total_budget {
                chosen_quant = "IQ4_XS";
                weight_vram_gb = iq4;
            } else {
                chosen_quant = if total_budget >= 6.0 { "Q4_K_M" } else { "IQ4_XS" };
                weight_vram_gb = if chosen_quant == "Q4_K_M" { params_b * 0.58 } else { params_b * 0.48 };
            }
        }
    }

    let kv_budget = (sim.total_ai_capacity_gb - weight_vram_gb - compute_overhead_gb).max(0.2);
    let vram_kv_budget = (total_budget - weight_vram_gb - compute_overhead_gb).max(0.2);
    let effective_kv_budget = if weight_vram_gb + compute_overhead_gb <= total_budget {
        vram_kv_budget
    } else {
        kv_budget
    };

    let mut context_options = Vec::with_capacity(3);
    for kv_type in [KvCacheType::F16, KvCacheType::Q8_0, KvCacheType::Q4_0] {
        let per_1k = fp16_kv_per_1k * kv_type.memory_factor();
        let max_ctx = snap_context_tier(effective_kv_budget / per_1k, native_ctx);
        let used_kv = ((max_ctx as f32 / 1000.0) * per_1k * 100.0).round() / 100.0;
        context_options.push(KvContextOption {
            kv_type,
            max_safe_ctx: max_ctx,
            kv_vram_gb: used_kv,
        });
    }

    // Balance precision & context length: keep F16 only when weights stay in VRAM and >= 32k (or full native)
    let f16_ctx = context_options[0].max_safe_ctx;
    let q8_ctx = context_options[1].max_safe_ctx;
    let q4_ctx = context_options[2].max_safe_ctx;
    let spills_to_ram = !is_pure_cpu && (weight_vram_gb + compute_overhead_gb > total_budget);

    let selected_idx = if spills_to_ram {
        if q8_ctx >= 16_384 || q8_ctx == q4_ctx { 1 } else { 2 }
    } else if f16_ctx >= 32_768 || f16_ctx >= native_ctx || (f16_ctx == q8_ctx && f16_ctx == q4_ctx) {
        0
    } else if q8_ctx >= 16_384 || (q8_ctx > f16_ctx && q8_ctx == q4_ctx) {
        1
    } else if q4_ctx > q8_ctx {
        2
    } else {
        1
    };

    let chosen_kv = &context_options[selected_idx];
    let total_vram_gb = ((weight_vram_gb + chosen_kv.kv_vram_gb + compute_overhead_gb) * 10.0).round() / 10.0;

    let (gpu_offload_pct, recommended_ngl) = if is_pure_cpu {
        (0, 0)
    } else if total_vram_gb <= total_budget {
        (100, 99)
    } else {
        let pct = ((total_budget - compute_overhead_gb).max(0.0) / weight_vram_gb * 100.0).clamp(5.0, 95.0);
        let est_layers = if params_b >= 60.0 { 80.0 } else if params_b >= 24.0 { 64.0 } else if params_b >= 10.0 { 40.0 } else { 32.0 };
        let ngl = ((pct / 100.0) * est_layers).round() as u16;
        (pct.round() as u8, ngl.max(1))
    };

    let flash_attention = !is_pure_cpu && (chosen_kv.kv_type != KvCacheType::F16 || chosen_kv.max_safe_ctx >= 8192);
    let batch_size = if is_pure_cpu {
        512
    } else if total_budget - weight_vram_gb >= 4.0 {
        2048
    } else if total_budget - weight_vram_gb >= 1.5 {
        1024
    } else {
        512
    };

    let fa_flag = if flash_attention { "on" } else { "off" };
    let hf_ref = if model.id.contains('/') { &model.id } else { "org/model-gguf" };
    let llama_cmd = format!(
        "llama-server -hf {} -c {} -ngl {} -ctk {} -ctv {} -fa {} -b {}",
        hf_ref,
        chosen_kv.max_safe_ctx,
        recommended_ngl,
        chosen_kv.kv_type.cli_flag(),
        chosen_kv.kv_type.cli_flag(),
        fa_flag,
        batch_size
    );

    let fallback_tag = format!(
        "{}:latest",
        model.family.trim().to_lowercase().replace(' ', "-")
    );
    let o_tag = model.ollama_tag.as_deref().unwrap_or(&fallback_tag);
    let ollama_cmd = if flash_attention {
        format!(
            "OLLAMA_FLASH_ATTENTION=1 OLLAMA_KV_CACHE_TYPE={} ollama run {} # /set parameter num_ctx {}",
            chosen_kv.kv_type.cli_flag(),
            o_tag,
            chosen_kv.max_safe_ctx
        )
    } else {
        format!("ollama run {} # /set parameter num_ctx {}", o_tag, chosen_kv.max_safe_ctx)
    };

    OptimalRuntimePlan {
        model_name: model.name.clone(),
        recommended_quant: chosen_quant,
        weight_vram_gb: (weight_vram_gb * 10.0).round() / 10.0,
        kv_cache: chosen_kv.kv_type,
        recommended_ctx: chosen_kv.max_safe_ctx,
        kv_vram_gb: chosen_kv.kv_vram_gb,
        compute_overhead_gb,
        total_vram_gb,
        gpu_offload_pct,
        recommended_ngl,
        flash_attention,
        batch_size,
        context_options,
        llama_cmd,
        ollama_cmd,
    }
}

fn snap_context_tier(raw_k_tokens: f32, native_ctx: u32) -> u32 {
    let raw_tokens = (raw_k_tokens * 1024.0).floor().max(2048.0) as u32;
    let capped = raw_tokens.min(native_ctx);
    let tiers = [131_072, 98_304, 65_536, 49_152, 32_768, 24_576, 16_384, 12_288, 8_192, 6_144, 4_096, 2_048];
    for t in tiers {
        if capped >= t {
            return t;
        }
    }
    2_048
}
