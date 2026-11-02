use crate::domain::hardware_detector::DetectedHardware;
use crate::domain::model::ModelBenchmark;
use crate::domain::simulator::HardwareRigConfig;

pub fn apply_detected_to_rig(rig: &mut HardwareRigConfig, hw: &DetectedHardware) {
    *rig = HardwareRigConfig::from_platform(hw.matched_platform);
    rig.ram_gb = hw.total_ram_gb;
}

pub fn apply_preset_fast_vs_deep(slots: &mut [Option<String>; 2], all: &[ModelBenchmark]) {
    slots[0] = all.iter().find(|m| !m.is_cloud && (m.size_gb >= 3.0 && m.size_gb <= 6.0 || m.name.contains("7B") || m.name.contains("8B"))).map(|m| m.id.clone());
    slots[1] = all.iter().find(|m| !m.is_cloud && (m.size_gb >= 14.0 && m.size_gb <= 24.0 || m.name.contains("27B") || m.name.contains("32B"))).map(|m| m.id.clone());
}

pub fn apply_preset_flagships(slots: &mut [Option<String>; 2], all: &[ModelBenchmark]) {
    slots[0] = all.iter().find(|m| !m.is_cloud && m.name.contains("27B")).map(|m| m.id.clone());
    slots[1] = all.iter().find(|m| !m.is_cloud && m.name.contains("32B")).map(|m| m.id.clone());
}

pub fn apply_preset_8g(slots: &mut [Option<String>; 2], all: &[ModelBenchmark]) {
    slots[0] = all.iter().find(|m| !m.is_cloud && m.name.contains("8B")).map(|m| m.id.clone());
    slots[1] = all.iter().find(|m| !m.is_cloud && m.name.contains("7B")).map(|m| m.id.clone());
}

pub fn apply_preset_local_vs_cloud(slots: &mut [Option<String>; 2], all: &[ModelBenchmark]) {
    slots[0] = all.iter().find(|m| !m.is_cloud && m.name.contains("27B")).map(|m| m.id.clone());
    slots[1] = all.iter().find(|m| m.is_cloud).map(|m| m.id.clone());
}
