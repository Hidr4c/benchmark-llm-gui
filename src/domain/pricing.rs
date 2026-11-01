//! Domain model and calculation for hardware power consumption and operational inference costs.

/// Operational cost & electricity calculator for local and cloud models
pub struct ModelPricing;

impl ModelPricing {
    /// Reference electricity tariff in USD / EUR per kWh (EU/US blended domestic average)
    pub const ELECTRICITY_TARIFF_PER_KWH: f32 = 0.25;

    /// Estimate average hardware power draw (in Watts) during active LLM inference
    pub fn estimated_hardware_tdp_watts(is_cloud: bool, size_gb: f32, vram_rec_gb: f32) -> f32 {
        if is_cloud {
            return 0.0;
        }

        if size_gb <= 0.5 {
            10.0 // Micro-IoT / Tiny SBC
        } else if size_gb <= 1.5 {
            15.0 // Raspberry Pi 4 CPU
        } else if size_gb <= 3.2 {
            22.0 // Raspberry Pi 5 / RK3588 NPU
        } else if size_gb <= 5.5 {
            35.0 // Jetson Orin Nano / Mini-PC N100
        } else if vram_rec_gb <= 8.5 {
            115.0 // RTX 4060 class
        } else if vram_rec_gb <= 12.5 {
            200.0 // RTX 4070 class
        } else if vram_rec_gb <= 16.5 {
            280.0 // RTX 4080 class
        } else if vram_rec_gb <= 24.5 {
            380.0 // RTX 4090 / 3090 class
        } else if vram_rec_gb <= 32.5 {
            500.0 // RTX 5090 class
        } else {
            450.0 // Workstation / Multi-GPU / Offload
        }
    }

    /// Estimate effective token throughput if not explicitly provided
    pub fn fallback_throughput_tps(size_gb: f32) -> f32 {
        if size_gb <= 0.6 {
            120.0
        } else if size_gb <= 2.0 {
            75.0
        } else if size_gb <= 6.0 {
            45.0
        } else if size_gb <= 16.0 {
            32.0
        } else if size_gb <= 24.0 {
            24.0
        } else {
            16.0
        }
    }

    /// Estimate kWh consumed to generate / process 1,000,000 tokens
    pub fn estimated_kwh_per_1m(
        is_cloud: bool,
        size_gb: f32,
        vram_rec_gb: f32,
        tokens_per_sec: Option<f32>,
    ) -> f32 {
        if is_cloud {
            return 0.0;
        }

        let watts = Self::estimated_hardware_tdp_watts(is_cloud, size_gb, vram_rec_gb);
        let speed = tokens_per_sec
            .unwrap_or_else(|| Self::fallback_throughput_tps(size_gb))
            .max(1.0);

        let hours_for_1m = 1_000_000.0 / (speed * 3600.0);
        let kwh = (watts * hours_for_1m) / 1000.0;
        (kwh * 100.0).round() / 100.0
    }

    /// Estimate operational electricity cost in USD / EUR per 1,000,000 tokens
    pub fn estimated_electricity_cost_usd_per_1m(
        is_cloud: bool,
        size_gb: f32,
        vram_rec_gb: f32,
        tokens_per_sec: Option<f32>,
    ) -> f32 {
        if is_cloud {
            return 0.0;
        }

        let kwh = Self::estimated_kwh_per_1m(is_cloud, size_gb, vram_rec_gb, tokens_per_sec);
        let cost = kwh * Self::ELECTRICITY_TARIFF_PER_KWH;
        (cost * 1000.0).round() / 1000.0
    }

    /// Estimated cost for 100 typical coding/chat tasks (assuming ~250k total tokens)
    pub fn estimated_cost_100_tasks(
        is_cloud: bool,
        pricing_input_usd: Option<f32>,
        pricing_output_usd: Option<f32>,
        size_gb: f32,
        vram_rec_gb: f32,
        tokens_per_sec: Option<f32>,
    ) -> f32 {
        if is_cloud {
            let p_in = pricing_input_usd.unwrap_or(1.0);
            let p_out = pricing_output_usd.unwrap_or(p_in * 3.0);
            let cost = (p_in * 0.18) + (p_out * 0.07);
            (cost * 1000.0).round() / 1000.0
        } else {
            let cost_1m = Self::estimated_electricity_cost_usd_per_1m(
                is_cloud,
                size_gb,
                vram_rec_gb,
                tokens_per_sec,
            );
            ((cost_1m * 0.25).max(0.001) * 1000.0).round() / 1000.0
        }
    }
}
