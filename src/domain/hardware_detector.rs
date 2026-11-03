use crate::domain::hardware::HardwarePlatform;

#[derive(Debug, Clone, PartialEq)]
pub struct DetectedHardware {
    pub cpu_name: String,
    pub total_ram_gb: f32,
    pub gpu_name: Option<String>,
    pub gpu_vram_gb: Option<f32>,
    pub os_name: String,
    pub matched_platform: HardwarePlatform,
}

impl DetectedHardware {
    pub fn summary(&self) -> String {
        let gpu_str = match (&self.gpu_name, self.gpu_vram_gb) {
            (Some(name), Some(vram)) => format!("{name} ({vram:.1} GB VRAM)"),
            (Some(name), None) => name.clone(),
            (None, _) => "No dedicated GPU (CPU/iGPU)".to_string(),
        };
        format!(
            "{} | {:.1} GB RAM | {}",
            self.cpu_name.trim(),
            self.total_ram_gb,
            gpu_str
        )
    }

    pub fn match_platform(vram_gb: Option<f32>, gpu_name: Option<&str>, total_ram_gb: f32) -> HardwarePlatform {
        let vram = vram_gb.unwrap_or(0.0);
        let name_lower = gpu_name.unwrap_or("").to_lowercase();

        if vram >= 28.0 || name_lower.contains("5090") {
            HardwarePlatform::Gpu32Gb
        } else if vram >= 20.0 || name_lower.contains("4090") {
            HardwarePlatform::Gpu24Gb
        } else if vram >= 14.0 || name_lower.contains("4080") {
            HardwarePlatform::Gpu16Gb
        } else if vram >= 10.0 || name_lower.contains("4070") || name_lower.contains("3080") {
            HardwarePlatform::Gpu12Gb
        } else if vram >= 7.0 || name_lower.contains("4060") || name_lower.contains("3060") {
            HardwarePlatform::Gpu8Gb
        } else if name_lower.contains("orin") || name_lower.contains("jetson") {
            HardwarePlatform::JetsonOrinNano
        } else if total_ram_gb >= 12.0 {
            HardwarePlatform::Gpu8Gb
        } else if total_ram_gb >= 6.0 {
            HardwarePlatform::MiniPcCpu
        } else if total_ram_gb >= 3.0 {
            HardwarePlatform::RaspberryPi5
        } else if total_ram_gb >= 1.5 {
            HardwarePlatform::RaspberryPi4
        } else {
            HardwarePlatform::MicroIoT
        }
    }
}

pub trait IHardwareDetector: Send + Sync {
    fn detect(&self) -> Result<DetectedHardware, String>;
}
