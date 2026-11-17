use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeHardwareFilter {
    All,
    DesktopGpu,
    AppleSilicon,
    CpuAndEdge,
    SmartphoneOffGrid,
}

impl RuntimeHardwareFilter {
    pub fn all() -> [Self; 5] {
        [
            Self::All,
            Self::DesktopGpu,
            Self::AppleSilicon,
            Self::CpuAndEdge,
            Self::SmartphoneOffGrid,
        ]
    }

    pub fn label(&self, is_fr: bool) -> &'static str {
        match (self, is_fr) {
            (Self::All, true) => "Tous les Matériels",
            (Self::All, false) => "All Hardware",
            (Self::DesktopGpu, true) => "PC GPU (CUDA / Vulkan / ROCm)",
            (Self::DesktopGpu, false) => "Desktop GPU (CUDA / Vulkan / ROCm)",
            (Self::AppleSilicon, true) => "Mac Apple Silicon (Metal / MLX)",
            (Self::AppleSilicon, false) => "Apple Silicon (Metal / MLX)",
            (Self::CpuAndEdge, true) => "CPU / Serveur / Raspberry Pi",
            (Self::CpuAndEdge, false) => "CPU / Server / Raspberry Pi",
            (Self::SmartphoneOffGrid, true) => "Smartphone Off-Grid (Android / iOS)",
            (Self::SmartphoneOffGrid, false) => "Smartphone Off-Grid (Android / iOS)",
        }
    }
}

/// Live local LLM execution engine / software profile discovered from GitHub + LLM-Speed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalRuntimeProfile {
    pub name: String,
    pub repo_slug: String,
    pub category: String,
    pub official_url: String,
    pub stars: u64,
    pub latest_version: String,
    pub last_updated: String,
    pub license: String,
    pub language: String,
    pub platforms: Vec<String>,
    pub backends: Vec<String>,
    pub supports_desktop_gpu: bool,
    pub supports_apple_silicon: bool,
    pub supports_cpu_edge: bool,
    pub supports_off_grid_mobile: bool,
    pub measured_avg_tps: Option<f32>,
    pub measured_runs: usize,
    pub sample_hardware: String,
    pub install_cmd: String,
    pub description: String,
}

impl LocalRuntimeProfile {
    pub fn matches_hardware(&self, hw: RuntimeHardwareFilter) -> bool {
        match hw {
            RuntimeHardwareFilter::All => true,
            RuntimeHardwareFilter::DesktopGpu => self.supports_desktop_gpu,
            RuntimeHardwareFilter::AppleSilicon => self.supports_apple_silicon,
            RuntimeHardwareFilter::CpuAndEdge => self.supports_cpu_edge,
            RuntimeHardwareFilter::SmartphoneOffGrid => self.supports_off_grid_mobile,
        }
    }

    /// Composite engineering note (0-100) from live GitHub stars, activity, hardware reach & measured tok/s.
    pub fn rating_score(&self) -> f32 {
        let star_score = ((self.stars.max(10) as f32).log10() / 5.3 * 45.0).clamp(10.0, 45.0);
        let reach_count = (self.supports_desktop_gpu as u8)
            + (self.supports_apple_silicon as u8)
            + (self.supports_cpu_edge as u8)
            + (self.supports_off_grid_mobile as u8);
        let reach_score = (reach_count as f32) * 6.5; // up to 26 pts
        let speed_bonus = self.measured_avg_tps.map(|t| (t / 6.0).clamp(5.0, 29.0)).unwrap_or(14.0);
        ((star_score + reach_score + speed_bonus).clamp(20.0, 99.0) * 10.0).round() / 10.0
    }
}
