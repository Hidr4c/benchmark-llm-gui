use serde::{Deserialize, Serialize};

/// Target Hardware Platforms, encompassing Mobile Smartphones, Embedded Edge Systems and Discrete GPUs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwarePlatform {
    All,
    // --- Smartphones & Mobile Devices (On-Device Mobile AI) ---
    SmartphoneMid,     // 4 - 6 GB RAM (iPhone base, Android standard) - Max ~2.2 GB usable
    SmartphoneHigh,    // 8 - 12 GB RAM (iPhone 15/16 Pro, Galaxy S24, Apple Intel.) - Max ~5.5 GB
    SmartphonePro,     // 16+ GB RAM (OnePlus 12/13, Ultra, Gaming) - Max ~9.0 GB
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
    pub const ALL: [HardwarePlatform; 15] = [
        HardwarePlatform::All,
        HardwarePlatform::SmartphoneMid,
        HardwarePlatform::SmartphoneHigh,
        HardwarePlatform::SmartphonePro,
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
            HardwarePlatform::All => "Tous Materiels",
            HardwarePlatform::SmartphoneMid => "Smartphone 4-6 Go (iPhone / Android)",
            HardwarePlatform::SmartphoneHigh => "Smartphone 8-12 Go (iPhone Pro, S24)",
            HardwarePlatform::SmartphonePro => "Smartphone Pro 16 Go+ (Ultra / ROG)",
            HardwarePlatform::Gpu8Gb => "GPU 8 Go VRAM",
            HardwarePlatform::Gpu12Gb => "GPU 12 Go VRAM",
            HardwarePlatform::Gpu16Gb => "GPU 16 Go VRAM",
            HardwarePlatform::Gpu24Gb => "GPU 24 Go VRAM",
            HardwarePlatform::Gpu32Gb => "GPU 32 Go VRAM",
            HardwarePlatform::Workstation => "Workstation & Mac (48+ Go)",
            HardwarePlatform::MicroIoT => "Micro-IoT (<= 1 Go RAM)",
            HardwarePlatform::RaspberryPi4 => "Raspberry Pi 4 (2-4 Go)",
            HardwarePlatform::RaspberryPi5 => "Raspberry Pi 5 / RK3588 (8 Go)",
            HardwarePlatform::JetsonOrinNano => "Jetson Orin Nano (Edge)",
            HardwarePlatform::MiniPcCpu => "Mini PC N100 / CPU (8-16 Go)",
        }
    }

    pub fn is_embedded(&self) -> bool {
        matches!(
            self,
            HardwarePlatform::SmartphoneMid
                | HardwarePlatform::SmartphoneHigh
                | HardwarePlatform::SmartphonePro
                | HardwarePlatform::MicroIoT
                | HardwarePlatform::RaspberryPi4
                | HardwarePlatform::RaspberryPi5
                | HardwarePlatform::JetsonOrinNano
                | HardwarePlatform::MiniPcCpu
        )
    }

    pub fn max_usable_memory_gb(&self) -> Option<f32> {
        match self {
            HardwarePlatform::All => None,
            HardwarePlatform::SmartphoneMid => Some(2.2),
            HardwarePlatform::SmartphoneHigh => Some(5.5),
            HardwarePlatform::SmartphonePro => Some(9.0),
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
            HardwarePlatform::SmartphoneMid => 0.0,
            HardwarePlatform::SmartphoneHigh => 1.5,
            HardwarePlatform::SmartphonePro => 4.5,
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
            HardwarePlatform::SmartphoneMid => "Mobile 4-6 Go",
            HardwarePlatform::SmartphoneHigh => "Mobile 8-12 Go",
            HardwarePlatform::SmartphonePro => "Mobile 16 Go+",
            HardwarePlatform::MicroIoT => "IoT / Micro",
            HardwarePlatform::RaspberryPi4 => "Raspberry Pi 4",
            HardwarePlatform::RaspberryPi5 => "Raspberry Pi 5",
            HardwarePlatform::JetsonOrinNano => "Jetson Edge",
            HardwarePlatform::MiniPcCpu => "Mini PC CPU",
            HardwarePlatform::Gpu8Gb => "GPU 8 Go",
            HardwarePlatform::Gpu12Gb => "GPU 12 Go",
            HardwarePlatform::Gpu16Gb => "GPU 16 Go",
            HardwarePlatform::Gpu24Gb => "GPU 24 Go",
            HardwarePlatform::Gpu32Gb => "GPU 32 Go",
            HardwarePlatform::Workstation => "Station 48G+",
            HardwarePlatform::All => "Tous Materiels",
        }
    }
}
