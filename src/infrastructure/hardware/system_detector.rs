use std::process::Command;
use crate::domain::hardware_detector::{DetectedHardware, IHardwareDetector};

#[derive(Default)]
pub struct SystemHardwareDetector;

impl SystemHardwareDetector {
    pub fn new() -> Self {
        Self
    }

    #[cfg(target_os = "windows")]
    fn detect_windows() -> Result<DetectedHardware, String> {
        // 1. Detect CPU via Registry
        let cpu_name = Self::query_registry_string(
            r"HKLM\HARDWARE\DESCRIPTION\System\CentralProcessor\0",
            "ProcessorNameString",
        ).unwrap_or_else(|| "Unknown Windows CPU".to_string());

        // 2. Detect GPU & VRAM via Registry (Class GUID for Display Adapters)
        let (gpu_name, gpu_vram_gb) = Self::detect_windows_gpu()
            .unwrap_or((Some("Generic Display Adapter".to_string()), None));

        // 3. Detect RAM via PowerShell CIM query
        let total_ram_gb = Self::query_windows_ram().unwrap_or(16.0);

        let matched_platform = DetectedHardware::match_platform(
            gpu_vram_gb,
            gpu_name.as_deref(),
            total_ram_gb,
        );

        Ok(DetectedHardware {
            cpu_name,
            total_ram_gb,
            gpu_name,
            gpu_vram_gb,
            os_name: "Windows".to_string(),
            matched_platform,
        })
    }

    #[cfg(target_os = "windows")]
    fn query_registry_string(key: &str, value_name: &str) -> Option<String> {
        let output = Command::new("reg")
            .args(["query", key, "/v", value_name])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains(value_name) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let val = parts[2..].join(" ");
                    return Some(val.trim().to_string());
                }
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    fn detect_windows_gpu() -> Option<(Option<String>, Option<f32>)> {
        // Enumerate through display adapter instances 0000, 0001, 0002
        for idx in 0..4 {
            let key = format!(r"HKLM\SYSTEM\CurrentControlSet\Control\Class\{{4d36e968-e325-11ce-bfc1-08002be10318}}\000{idx}");
            if let Some(desc) = Self::query_registry_string(&key, "DriverDesc") {
                // Ignore basic display adapters or mirroring drivers
                if desc.to_lowercase().contains("basic") {
                    continue;
                }

                // Query VRAM size (QWORD)
                let vram_gb = Self::query_registry_qw_vram(&key);
                return Some((Some(desc), vram_gb));
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    fn query_registry_qw_vram(key: &str) -> Option<f32> {
        let output = Command::new("reg")
            .args(["query", key, "/v", "HardwareInformation.qwMemorySize"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("HardwareInformation.qwMemorySize") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let hex_str = parts[2].trim_start_matches("0x");
                    if let Ok(bytes) = u64::from_str_radix(hex_str, 16) {
                        let gb = (bytes as f64 / (1024.0 * 1024.0 * 1024.0)) as f32;
                        return Some((gb * 10.0).round() / 10.0);
                    }
                }
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    fn query_windows_ram() -> Option<f32> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "[math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1GB, 1)",
            ])
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let cleaned = stdout.trim().replace(',', ".");
            if let Ok(ram) = cleaned.parse::<f32>() {
                return Some(ram);
            }
        }
        None
    }

    #[cfg(not(target_os = "windows"))]
    fn detect_fallback() -> Result<DetectedHardware, String> {
        Ok(DetectedHardware {
            cpu_name: "Generic 8-Core Processor".to_string(),
            total_ram_gb: 16.0,
            gpu_name: Some("Generic GPU (8 GB)".to_string()),
            gpu_vram_gb: Some(8.0),
            os_name: "Unix / POSIX".to_string(),
            matched_platform: HardwarePlatform::Rtx4060,
        })
    }
}

impl IHardwareDetector for SystemHardwareDetector {
    fn detect(&self) -> Result<DetectedHardware, String> {
        #[cfg(target_os = "windows")]
        {
            Self::detect_windows()
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self::detect_fallback()
        }
    }
}
