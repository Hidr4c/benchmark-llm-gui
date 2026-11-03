use benchmark_llm_gui::domain::hardware::HardwarePlatform;
use benchmark_llm_gui::domain::hardware_detector::DetectedHardware;
use benchmark_llm_gui::domain::license::{LicenseCategory, ModelLicense};
use benchmark_llm_gui::domain::model::ModelBenchmark;
use benchmark_llm_gui::domain::simulator::{GpuTier, HardwareRigConfig, SimTargetMode};
use benchmark_llm_gui::presentation::filtering::{FilterService, FilterState};

#[test]
fn test_license_detection_and_commercial_viability() {
    let qwen_lic = ModelLicense::detect("Qwen/Qwen2.5-Coder-32B-Instruct-GGUF", None, false);
    assert_eq!(qwen_lic, ModelLicense::Apache2);
    assert!(qwen_lic.is_permissive_commercial());
    assert_eq!(qwen_lic.category(), LicenseCategory::PermissiveCommercial);

    let deepseek_lic = ModelLicense::detect("deepseek-ai/DeepSeek-R1-Distill-Qwen-32B-GGUF", None, false);
    assert_eq!(deepseek_lic, ModelLicense::Mit);
    assert!(deepseek_lic.is_permissive_commercial());

    let llama_lic = ModelLicense::detect("meta-llama/Llama-3.3-70B-Instruct-GGUF", None, false);
    assert_eq!(llama_lic, ModelLicense::LlamaCommunity);
    assert!(!llama_lic.is_permissive_commercial());
    assert_eq!(llama_lic.category(), LicenseCategory::CommunityCommercial);

    let cloud_lic = ModelLicense::detect("openai/gpt-4o", Some("OpenAI"), true);
    assert_eq!(cloud_lic, ModelLicense::ProprietaryCloud);
    assert_eq!(cloud_lic.category(), LicenseCategory::CloudProprietary);
}

#[test]
fn test_hardware_platform_matcher() {
    let plat_5090 = DetectedHardware::match_platform(Some(32.0), Some("NVIDIA GeForce RTX 5090"), 64.0);
    assert_eq!(plat_5090, HardwarePlatform::Gpu32Gb);

    let plat_4090 = DetectedHardware::match_platform(Some(24.0), Some("NVIDIA GeForce RTX 4090"), 32.0);
    assert_eq!(plat_4090, HardwarePlatform::Gpu24Gb);

    let plat_4060 = DetectedHardware::match_platform(Some(8.0), Some("NVIDIA GeForce RTX 4060 Laptop"), 16.0);
    assert_eq!(plat_4060, HardwarePlatform::Gpu8Gb);

    let plat_nas = DetectedHardware::match_platform(None, None, 8.0);
    assert_eq!(plat_nas, HardwarePlatform::MiniPcCpu);

    let plat_iot = DetectedHardware::match_platform(None, None, 0.5);
    assert_eq!(plat_iot, HardwarePlatform::MicroIoT);
}

#[test]
fn test_dedicated_workstation_simulation() {
    let rig = HardwareRigConfig {
        mode: SimTargetMode::WorkstationPc,
        gpu: GpuTier::Rtx5090_32G,
        ram_gb: 64.0,
        nas_web_ram_gb: 0.0,
    };

    let sim = rig.simulate();
    assert!(sim.usable_vram_gb >= 31.0);
    assert!(sim.effective_bandwidth_gbps >= 1700.0);
    assert!(sim.max_pure_vram_params_b >= 40.0);
    assert!(sim.projected_speed_tps > 80.0);
    assert!(sim.stability_percent >= 90.0);
    assert!(sim.savings_percent > 85.0);
}

#[test]
fn test_multi_service_nas_web_scenario_stability() {
    let nas_rig = HardwareRigConfig {
        mode: SimTargetMode::HomeServerNas,
        gpu: GpuTier::None,
        ram_gb: 8.0,
        nas_web_ram_gb: 6.5,
    };

    let sim = nas_rig.simulate();
    assert_eq!(sim.usable_vram_gb, 0.0);
    // Reserved RAM is 2.0 (OS) + 6.5 (Web/Docker) = 8.5 GB > 8.0 GB RAM!
    assert!(sim.ram_headroom_gb < 1.0);
    assert!(sim.stability_percent < 50.0);
    assert!(sim.stability_verdict.contains("Risque OOM"));
}

#[test]
fn test_license_category_filtering() {
    let models = vec![
        ModelBenchmark {
            id: "Qwen/Qwen2.5-Coder-7B-Instruct-GGUF".into(),
            name: "Qwen 2.5 Coder 7B".into(),
            is_cloud: false,
            ..Default::default()
        },
        ModelBenchmark {
            id: "meta-llama/Llama-3.2-3B-Instruct-GGUF".into(),
            name: "Llama 3.2 3B".into(),
            is_cloud: false,
            ..Default::default()
        },
        ModelBenchmark {
            id: "openai/gpt-4o".into(),
            name: "GPT-4o".into(),
            is_cloud: true,
            ..Default::default()
        },
    ];

    let mut filters = FilterState::default();
    filters.license = LicenseCategory::PermissiveCommercial;
    let filtered = FilterService::compute_filtered_indices(&models, &filters);
    assert_eq!(filtered.len(), 1);
    assert_eq!(models[filtered[0]].name, "Qwen 2.5 Coder 7B");
}
