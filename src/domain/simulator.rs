use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimTargetMode {
    WorkstationPc,
    HomeServerNas,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuTier {
    None,            // CPU Only (0 GB VRAM)
    Rtx4060_8G,      // 8 GB GDDR6 (272 GB/s, 115W)
    Rtx4070_12G,     // 12 GB GDDR6X (504 GB/s, 200W)
    Rtx4080_16G,     // 16 GB GDDR6X (717 GB/s, 320W)
    Rtx4090_24G,     // 24 GB GDDR6X (1008 GB/s, 450W)
    Rtx5090_32G,     // 32 GB GDDR7 (1792 GB/s, 600W)
    AppleM4_36G,     // 36 GB Unified (273 GB/s, 40W)
    AppleM4_128G,    // 128 GB Unified (546 GB/s, 60W)
}

impl GpuTier {
    pub fn all() -> [Self; 8] {
        [
            Self::None,
            Self::Rtx4060_8G,
            Self::Rtx4070_12G,
            Self::Rtx4080_16G,
            Self::Rtx4090_24G,
            Self::Rtx5090_32G,
            Self::AppleM4_36G,
            Self::AppleM4_128G,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "❌ Aucun GPU dédié (Inférence CPU/iGPU)",
            Self::Rtx4060_8G => "🟢 GPU 8 Go VRAM (GDDR6 | 272 Go/s)",
            Self::Rtx4070_12G => "🟢 GPU 12 Go VRAM (GDDR6X | 504 Go/s)",
            Self::Rtx4080_16G => "🟢 GPU 16 Go VRAM (GDDR6X | 717 Go/s)",
            Self::Rtx4090_24G => "🟣 GPU 24 Go VRAM (GDDR6X | 1008 Go/s)",
            Self::Rtx5090_32G => "👑 GPU 32 Go VRAM (GDDR7 | 1792 Go/s)",
            Self::AppleM4_36G => "🍏 Apple M4 Pro (36 Go Unifié | 273 Go/s)",
            Self::AppleM4_128G => "🍏 Apple M4 Max (128 Go Unifié | 546 Go/s)",
        }
    }

    pub fn vram_gb(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Rtx4060_8G => 8.0,
            Self::Rtx4070_12G => 12.0,
            Self::Rtx4080_16G => 16.0,
            Self::Rtx4090_24G => 24.0,
            Self::Rtx5090_32G => 32.0,
            Self::AppleM4_36G => 36.0,
            Self::AppleM4_128G => 128.0,
        }
    }

    pub fn bandwidth_gbps(&self) -> f32 {
        match self {
            Self::None => 45.0,
            Self::Rtx4060_8G => 272.0,
            Self::Rtx4070_12G => 504.0,
            Self::Rtx4080_16G => 717.0,
            Self::Rtx4090_24G => 1008.0,
            Self::Rtx5090_32G => 1792.0,
            Self::AppleM4_36G => 273.0,
            Self::AppleM4_128G => 546.0,
        }
    }

    pub fn tdp_watts(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Rtx4060_8G => 115.0,
            Self::Rtx4070_12G => 200.0,
            Self::Rtx4080_16G => 320.0,
            Self::Rtx4090_24G => 450.0,
            Self::Rtx5090_32G => 600.0,
            Self::AppleM4_36G => 40.0,
            Self::AppleM4_128G => 60.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareRigConfig {
    pub mode: SimTargetMode,
    pub gpu: GpuTier,
    pub ram_gb: f32,
    pub nas_web_ram_gb: f32,
}

impl Default for HardwareRigConfig {
    fn default() -> Self {
        Self {
            mode: SimTargetMode::WorkstationPc,
            gpu: GpuTier::Rtx5090_32G,
            ram_gb: 64.0,
            nas_web_ram_gb: 4.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RigSimulationResult {
    pub usable_vram_gb: f32,
    pub usable_ram_gb: f32,
    pub total_ai_capacity_gb: f32,
    pub effective_bandwidth_gbps: f32,
    pub max_pure_vram_params_b: f32,
    pub max_offloaded_params_b: f32,
    pub projected_speed_tps: f32,
    pub ram_headroom_gb: f32,
    pub stability_percent: f32,
    pub stability_verdict: &'static str,
    pub total_power_watts: f32,
    pub cost_per_100_tasks_local_eur: f32,
    pub cost_per_100_tasks_cloud_eur: f32,
    pub savings_percent: f32,
}

impl HardwareRigConfig {
    pub fn simulate(&self) -> RigSimulationResult {
        let (_raw_vram, usable_vram, reserved_ram, effective_bandwidth, base_watts) = match self.mode {
            SimTargetMode::WorkstationPc => {
                let vram = self.gpu.vram_gb();
                let usable_v = (vram - 0.6).max(0.0);
                let bw = if vram >= 8.0 { self.gpu.bandwidth_gbps() } else { 65.0 };
                let watts = 85.0 + self.gpu.tdp_watts();
                (vram, usable_v, 3.5, bw, watts)
            }
            SimTargetMode::HomeServerNas => {
                let reserved = 2.0 + self.nas_web_ram_gb;
                let bw = 38.0; // DDR4/DDR5 system memory in mini-PC / NAS
                let watts = 25.0; // Low-power 24/7 NAS
                (0.0, 0.0, reserved, bw, watts)
            }
        };

        let usable_ram = (self.ram_gb - reserved_ram).max(0.0);
        let total_capacity = usable_vram + usable_ram;

        // Model capacity in billions of parameters (Q4 quantization ~0.72 GB / 1B params)
        let max_pure_vram_b = (usable_vram / 0.72).floor().max(0.0);
        let max_offloaded_b = (total_capacity / 0.75).floor().max(0.5);

        // Projected throughput (reference 7B / 14B model)
        let ref_model_size_gb = if usable_vram >= 8.0 { 14.0 * 0.72 } else { 7.0 * 0.72 };
        let projected_speed_tps = ((effective_bandwidth / ref_model_size_gb) * 0.82).clamp(1.5, 300.0);

        // Stability & OOM Risk Analysis
        let ram_headroom = usable_ram - (ref_model_size_gb * 0.35);
        let (stability_percent, stability_verdict) = if usable_ram < 0.8 || ram_headroom < 0.2 {
            (25.0, "⚠️ Risque OOM Critique : La charge conjointe sature la RAM et risque de faire crasher le système !")
        } else if ram_headroom < 1.5 {
            (65.0, "⚡ Marge Restreinte : Inférence fonctionnelle, mais attention aux pics d'utilisation Web/Docker.")
        } else if usable_vram >= 8.0 {
            (98.0, "✅ Stabilité Maximale : Exécution 100% VRAM GPU isolée. Zéro impact sur les services de la machine.")
        } else {
            (88.0, "✅ Stable & Fluide : Marge RAM confortable pour l'OS et les requêtes concurrentes.")
        };

        // Economic calculation: 100 tasks of 200k tokens
        let duration_hours = (100.0 * 250.0 / projected_speed_tps) / 3600.0;
        let kwh = (base_watts * duration_hours) / 1000.0;
        let cost_local = kwh * 0.25;
        let cost_cloud = 0.60;
        let savings = ((cost_cloud - cost_local) / cost_cloud * 100.0).clamp(0.0, 99.9);

        RigSimulationResult {
            usable_vram_gb: usable_vram,
            usable_ram_gb: usable_ram,
            total_ai_capacity_gb: total_capacity,
            effective_bandwidth_gbps: effective_bandwidth,
            max_pure_vram_params_b: max_pure_vram_b,
            max_offloaded_params_b: max_offloaded_b,
            projected_speed_tps,
            ram_headroom_gb: ram_headroom.max(0.0),
            stability_percent,
            stability_verdict,
            total_power_watts: base_watts,
            cost_per_100_tasks_local_eur: cost_local,
            cost_per_100_tasks_cloud_eur: cost_cloud,
            savings_percent: savings,
        }
    }
}
