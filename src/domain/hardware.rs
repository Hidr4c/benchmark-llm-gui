use serde::{Deserialize, Serialize};

/// Target Hardware Platforms, encompassing Embedded Edge Systems and Domestic GPUs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwarePlatform {
    All,
    // --- Embedded & Edge Systems ---
    MicroIoT,          // ESP32-P4, Micro-ARM (<= 1 GB RAM, footprint < 500 MB)
    RaspberryPi4,      // Raspberry Pi 4 (2 GB - 4 GB RAM, CPU-only llama.cpp)
    RaspberryPi5,      // Raspberry Pi 5 / Orange Pi 5 (RK3588 6 TOPS NPU, 4-8 GB)
    JetsonOrinNano,    // Nvidia Jetson Nano / Orin Nano (4-8 GB Unified Memory)
    MiniPcCpu,         // Mini PC (Intel N100, AMD Mini, 8-16 GB RAM)
    // --- Discrete Domestic GPUs ---
    Gpu8Gb,            // RTX 3060 Laptop, RTX 4060 (8 GB VRAM)
    Gpu12Gb,           // RTX 3060 12G, RTX 4070 Super (12 GB VRAM)
    Gpu16Gb,           // RTX 4080, RX 7800XT, Mac 18GB (16 GB VRAM)
    Gpu24Gb,           // RTX 3090, RTX 4090, Mac 32GB (24 GB VRAM)
    Gpu32Gb,           // RTX 5090 (32 GB VRAM)
    Workstation,       // Dual GPU, Mac Studio 64GB-128GB (48+ GB)
}

impl HardwarePlatform {
    pub const ALL: [HardwarePlatform; 12] = [
        HardwarePlatform::All,
        HardwarePlatform::MicroIoT,
        HardwarePlatform::RaspberryPi4,
        HardwarePlatform::RaspberryPi5,
        HardwarePlatform::JetsonOrinNano,
        HardwarePlatform::MiniPcCpu,
        HardwarePlatform::Gpu8Gb,
        HardwarePlatform::Gpu12Gb,
        HardwarePlatform::Gpu16Gb,
        HardwarePlatform::Gpu24Gb,
        HardwarePlatform::Gpu32Gb,
        HardwarePlatform::Workstation,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            HardwarePlatform::All => "All Hardware",
            HardwarePlatform::MicroIoT => "Micro-IoT (<= 1 GB RAM)",
            HardwarePlatform::RaspberryPi4 => "Raspberry Pi 4 (2-4 GB)",
            HardwarePlatform::RaspberryPi5 => "Raspberry Pi 5 / RK3588 (8 GB)",
            HardwarePlatform::JetsonOrinNano => "Jetson Orin Nano (Edge)",
            HardwarePlatform::MiniPcCpu => "Mini PC N100 / CPU (8-16 GB)",
            HardwarePlatform::Gpu8Gb => "8 GB (RTX 4060)",
            HardwarePlatform::Gpu12Gb => "12 GB (RTX 4070)",
            HardwarePlatform::Gpu16Gb => "16 GB (RTX 4080)",
            HardwarePlatform::Gpu24Gb => "24 GB (RTX 3090/4090)",
            HardwarePlatform::Gpu32Gb => "32 GB (RTX 5090)",
            HardwarePlatform::Workstation => "48+ GB (Workstation/Mac)",
        }
    }

    pub fn is_embedded(&self) -> bool {
        matches!(
            self,
            HardwarePlatform::MicroIoT
                | HardwarePlatform::RaspberryPi4
                | HardwarePlatform::RaspberryPi5
                | HardwarePlatform::JetsonOrinNano
                | HardwarePlatform::MiniPcCpu
        )
    }

    pub fn max_usable_memory_gb(&self) -> Option<f32> {
        match self {
            HardwarePlatform::All => None,
            HardwarePlatform::MicroIoT => Some(0.8),
            HardwarePlatform::RaspberryPi4 => Some(2.5),
            HardwarePlatform::RaspberryPi5 => Some(5.5),
            HardwarePlatform::JetsonOrinNano => Some(6.5),
            HardwarePlatform::MiniPcCpu => Some(12.0),
            HardwarePlatform::Gpu8Gb => Some(8.2),
            HardwarePlatform::Gpu12Gb => Some(12.3),
            HardwarePlatform::Gpu16Gb => Some(16.5),
            HardwarePlatform::Gpu24Gb => Some(24.5),
            HardwarePlatform::Gpu32Gb => Some(32.5),
            HardwarePlatform::Workstation => None,
        }
    }

    /// Checks if a model fits within this hardware platform's limits
    pub fn fits_model(&self, size_gb: f32, vram_rec_gb: f32) -> bool {
        match self.max_usable_memory_gb() {
            None => true,
            Some(max_mem) => {
                let req = if self.is_embedded() { size_gb } else { vram_rec_gb };
                req <= max_mem
            }
        }
    }

    #[allow(dead_code)]
    pub fn min_memory_gb(&self) -> f32 {
        match self {
            HardwarePlatform::All => 0.0,
            HardwarePlatform::MicroIoT => 0.0,
            HardwarePlatform::RaspberryPi4 => 0.4,
            HardwarePlatform::RaspberryPi5 => 1.8,
            HardwarePlatform::JetsonOrinNano => 2.0,
            HardwarePlatform::MiniPcCpu => 3.5,
            HardwarePlatform::Gpu8Gb => 5.0,
            HardwarePlatform::Gpu12Gb => 8.5,
            HardwarePlatform::Gpu16Gb => 12.5,
            HardwarePlatform::Gpu24Gb => 16.5,
            HardwarePlatform::Gpu32Gb => 24.5,
            HardwarePlatform::Workstation => 32.5,
        }
    }

    #[allow(dead_code)]
    pub fn badge_text(&self) -> &'static str {
        match self {
            HardwarePlatform::MicroIoT => "IoT / Micro",
            HardwarePlatform::RaspberryPi4 => "Raspberry Pi 4",
            HardwarePlatform::RaspberryPi5 => "Raspberry Pi 5",
            HardwarePlatform::JetsonOrinNano => "Jetson Edge",
            HardwarePlatform::MiniPcCpu => "Mini PC CPU",
            HardwarePlatform::Gpu8Gb => "RTX 4060",
            HardwarePlatform::Gpu12Gb => "RTX 4070",
            HardwarePlatform::Gpu16Gb => "RTX 4080",
            HardwarePlatform::Gpu24Gb => "RTX 4090",
            HardwarePlatform::Gpu32Gb => "RTX 5090",
            HardwarePlatform::Workstation => "Workstation",
            HardwarePlatform::All => "Universal",
        }
    }
}
