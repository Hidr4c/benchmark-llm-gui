use benchmark_llm_gui::domain::hardware::HardwarePlatform;
use benchmark_llm_gui::domain::harness::{
    compare_harnesses_by_model, normalize_base_model_key, HarnessCategory, HarnessProfile,
    HarnessRun, HarnessSnapshotData,
};
use benchmark_llm_gui::domain::modality::ModelModality;
use benchmark_llm_gui::domain::model::{BenchmarkScores, ModelBenchmark, SourceSelection};
use benchmark_llm_gui::domain::runtime::{LocalRuntimeProfile, RuntimeHardwareFilter};
use benchmark_llm_gui::domain::runtime_optimizer::{compute_optimal_runtime_plan, KvCacheType};
use benchmark_llm_gui::domain::simulator::{GpuTier, HardwareRigConfig, SimTargetMode};
use benchmark_llm_gui::domain::simulator_ranker::{
    rank_compatible_models, SimFitStatus, SimTableSort,
};
use benchmark_llm_gui::domain::specialty::DomainSpecialty;
use benchmark_llm_gui::domain::traits::IEcosystemRepository;
use benchmark_llm_gui::infrastructure::repository::sqlite_ecosystem_repo::SqliteEcosystemRepository;

fn make_local_model(id: &str, name: &str, param_b: f32, size_gb: f32, vram_gb: f32, score: f32) -> ModelBenchmark {
    ModelBenchmark {
        id: id.into(),
        name: name.into(),
        family: "Qwen".into(),
        param_count_b: Some(param_b),
        size_gb,
        vram_rec_gb: vram_gb,
        scores: BenchmarkScores {
            open_llm: Some(score),
            coding: Some(score * 0.8),
            agentic: Some(score * 0.7),
            average: Some(score),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn test_comfyui_and_3d_modality_detection() {
    // Arrange & Act: Detect latest ComfyUI 3D, Video, Image, and Audio models dynamically
    let trellis = ModelModality::detect("microsoft/TRELLIS.2", "TRELLIS.2 Image-to-3D", Some("image-to-3d"), &[]);
    let hunyuan = ModelModality::detect("tencent/Hunyuan3D-2.1", "Hunyuan3D 2.1", None, &[]);
    let wan2 = ModelModality::detect("Comfy-Org/Wan_2.2_repackaged", "Wan 2.2 14B Image to Video", Some("image-to-video"), &[]);
    let ltx2 = ModelModality::detect("Lightricks/LTX-2.5", "LTX-2.5 Video", None, &[]);
    let qwen_img = ModelModality::detect("Qwen/Qwen-Image-2.1", "Qwen Image 2.1", Some("text-to-image"), &[]);
    let z_img = ModelModality::detect("Tongyi-MAI/Z-Image-Turbo", "Z-Image-Turbo", None, &[]);
    let ace = ModelModality::detect("ACE-Step/ACE-Step-1.5XL-Turbo", "ACE-Step 1.5 Music", Some("text-to-audio"), &[]);

    // Assert: All ComfyUI modalities + SQLite roundtrip & specialty mapping behave consistently
    assert_eq!(trellis, ModelModality::ThreeDGen);
    assert_eq!(hunyuan, ModelModality::ThreeDGen);
    assert_eq!(wan2, ModelModality::VideoGen);
    assert_eq!(ltx2, ModelModality::VideoGen);
    assert_eq!(qwen_img, ModelModality::ImageGen);
    assert_eq!(z_img, ModelModality::ImageGen);
    assert_eq!(ace, ModelModality::AudioSpeech);
    assert_eq!(ModelModality::from_str_lenient(trellis.as_str()), ModelModality::ThreeDGen);
    assert_eq!(
        DomainSpecialty::detect_with_context("microsoft/TRELLIS.2", "TRELLIS.2", Some("image-to-3d"), &[], "Microsoft", None, ModelModality::ThreeDGen),
        DomainSpecialty::WorldModel3D
    );
}

#[test]
fn test_dev_agentic_source_preset() {
    // Arrange: Agentic Coding combines Pure Code (SWE-Bench) + CLI Tool Calling (BFCL)
    let dev = SourceSelection::dev_agentic();
    let model = make_local_model("qwen/qwen3-coder-32b", "Qwen 3 Coder 32B", 32.0, 19.0, 22.0, 80.0);

    // Assert: Flags & label differentiate Agentic Dev from standalone Code / Agent presets
    assert!(dev.is_dev_agentic());
    assert!(dev.coding && dev.agentic);
    assert!(!dev.open_llm && !dev.real_world);
    assert_eq!(dev.active_count(), 2);
    assert!(dev.label().contains("Code + Tools"));
    let score = model.score_for_sources(&dev).expect("Agentic Dev must produce composite score");
    assert!(score > 40.0);
}

#[test]
fn test_simulator_compatible_models_ranking_and_dedup() {
    // Arrange: 24 GB GPU workstation with 20 GB max storage cap
    let rig = HardwareRigConfig {
        mode: SimTargetMode::WorkstationPc,
        gpu: GpuTier::Rtx4090_24G,
        ram_gb: 64.0,
        nas_web_ram_gb: 0.0,
        platform: HardwarePlatform::Gpu24Gb,
        max_storage_gb: Some(20.0),
    };
    let sim = rig.simulate();
    let mut cloud = make_local_model("openai/gpt-5", "GPT-5 Cloud", 100.0, 0.0, 0.0, 99.0);
    cloud.is_cloud = true;

    let models = vec![
        cloud,
        make_local_model("qwen/qwen3-14b-q4", "Qwen 3 14B Q4", 14.0, 8.5, 11.0, 74.0),
        make_local_model("qwen/qwen3-14b-q8", "Qwen 3 14B Q8", 14.0, 15.0, 18.0, 76.0),
        make_local_model("qwen/qwen3-32b-q4", "Qwen 3 32B Q4", 32.0, 19.5, 22.8, 82.0), // Tight inside ~23.2GB usable
        make_local_model("qwen/qwen3-72b-q4", "Qwen 3 72B Q4", 72.0, 42.0, 48.0, 88.0), // Over storage & RAM/VRAM
    ];

    // Act: Deduplicate Qwen 3 14B variants and rank by Score
    let ranked = rank_compatible_models(&models, &rig, &sim, &SourceSelection::default(), SimTableSort::Score, true);

    // Assert: Cloud and 72B excluded; 32B Tight; one 14B root Kept as Comfortable
    assert!(ranked.iter().all(|e| !e.id.contains("72b") && !e.id.contains("gpt-5")));
    assert_eq!(ranked.len(), 2);
    assert_eq!(ranked[0].name, "Qwen 3 32B Q4");
    assert_eq!(ranked[0].fit_status, SimFitStatus::Tight);
    assert_eq!(ranked[1].fit_status, SimFitStatus::Comfortable);
}

#[test]
fn test_harness_same_model_comparison_and_normalization() {
    // Arrange: Two harnesses evaluated on identical base model (dated vs undated variant strings)
    let k1 = normalize_base_model_key("claude-3-7-sonnet-20250219");
    let k2 = normalize_base_model_key("Claude 3.7 Sonnet");
    assert_eq!(k1, k2);

    let runs = vec![
        HarnessRun {
            harness_name: "OpenHands".into(),
            harness_org: "All-Hands".into(),
            model_name: "Claude 3.7 Sonnet".into(),
            model_key: k1.clone(),
            suite: "SWE-bench Verified".into(),
            score_pct: 64.2,
            cost_usd: None,
            date: "2026-02".into(),
            source_url: "https://www.swebench.com".into(),
        },
        HarnessRun {
            harness_name: "Aider".into(),
            harness_org: "Aider-AI".into(),
            model_name: "claude-3-7-sonnet-20250219".into(),
            model_key: k2,
            suite: "SWE-bench Verified".into(),
            score_pct: 51.8,
            cost_usd: None,
            date: "2026-02".into(),
            source_url: "https://aider.chat".into(),
        },
        HarnessRun {
            harness_name: "SingleOnly".into(),
            harness_org: "Org".into(),
            model_name: "Isolated Model".into(),
            model_key: "isolatedmodel".into(),
            suite: "Terminal-Bench 2.0".into(),
            score_pct: 40.0,
            cost_usd: None,
            date: "2026-03".into(),
            source_url: "https://www.tbench.ai".into(),
        },
    ];

    // Act: Compare only base models benchmarked on >= 2 harnesses
    let groups = compare_harnesses_by_model(&runs, 2);

    // Assert: Single-harness model omitted; spread calculated accurately
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].entries.len(), 2);
    assert!((groups[0].best_score - 64.2).abs() < 0.01);
    assert!((groups[0].worst_score - 51.8).abs() < 0.01);
    assert!((groups[0].spread_pts - 12.4).abs() < 0.01);
}

#[test]
fn test_local_runtime_hardware_filtering_and_rating() {
    // Arrange: Smartphone Off-Grid vs Workstation runtimes
    let off_grid = LocalRuntimeProfile {
        name: "Off Grid AI (OGAM)".into(),
        repo_slug: "alichherawi/off-grid-mobile".into(),
        category: "Mobile Off-Grid".into(),
        official_url: "https://github.com/alichherawi/off-grid-mobile".into(),
        stars: 3400,
        latest_version: "v1.4".into(),
        last_updated: "2026-10-01".into(),
        license: "MIT".into(),
        language: "TypeScript / C++".into(),
        platforms: vec!["Android".into(), "iOS".into()],
        backends: vec!["llama.cpp".into()],
        supports_desktop_gpu: false,
        supports_apple_silicon: false,
        supports_cpu_edge: true,
        supports_off_grid_mobile: true,
        measured_avg_tps: Some(18.0),
        measured_runs: 5,
        sample_hardware: "Snapdragon 8 Gen 3".into(),
        install_cmd: "App Store / Play Store".into(),
        description: "Fully offline mobile inference".into(),
    };

    // Assert: Smartphone filter matches Off Grid; DesktopGPU rejects it
    assert!(off_grid.matches_hardware(RuntimeHardwareFilter::All));
    assert!(off_grid.matches_hardware(RuntimeHardwareFilter::SmartphoneOffGrid));
    assert!(off_grid.matches_hardware(RuntimeHardwareFilter::CpuAndEdge));
    assert!(!off_grid.matches_hardware(RuntimeHardwareFilter::DesktopGpu));
    assert!(!off_grid.matches_hardware(RuntimeHardwareFilter::AppleSilicon));
    assert!(off_grid.rating_score() > 30.0 && off_grid.rating_score() <= 99.0);
}

#[test]
fn test_sqlite_ecosystem_repository_roundtrip() {
    // Arrange: In-memory SQLite repository behind IEcosystemRepository IoC trait
    let mut repo: Box<dyn IEcosystemRepository> = Box::new(
        SqliteEcosystemRepository::open(":memory:").expect("Failed to open in-memory eco repo"),
    );

    let harness_data = HarnessSnapshotData {
        profiles: vec![HarnessProfile {
            name: "OpenCode".into(),
            repo_slug: "anomalyco/opencode".into(),
            org: "Anomaly".into(),
            category: HarnessCategory::CliAgent,
            url: "https://github.com/anomalyco/opencode".into(),
            license: "MIT".into(),
            stars: 42000,
            last_updated: "2026-11-01".into(),
            best_score_pct: Some(58.4),
            avg_score_pct: Some(52.1),
            runs_count: 3,
            description: "Terminal AI coding agent".into(),
        }],
        runs: Vec::new(),
    };
    let runtime_data = vec![LocalRuntimeProfile {
        name: "Ollama".into(),
        repo_slug: "ollama/ollama".into(),
        category: "Local Server".into(),
        official_url: "https://ollama.com".into(),
        stars: 160000,
        latest_version: "v0.9.0".into(),
        last_updated: "2026-11-10".into(),
        license: "MIT".into(),
        language: "Go / C++".into(),
        platforms: vec!["Windows".into(), "Linux".into(), "macOS".into()],
        backends: vec!["CUDA".into(), "Metal".into(), "ROCm".into()],
        supports_desktop_gpu: true,
        supports_apple_silicon: true,
        supports_cpu_edge: true,
        supports_off_grid_mobile: false,
        measured_avg_tps: Some(85.0),
        measured_runs: 120,
        sample_hardware: "RTX 4090".into(),
        install_cmd: "curl -fsSL https://ollama.com/install.sh | sh".into(),
        description: "Local LLM runner".into(),
    }];

    // Act
    repo.save_harness_snapshot(&harness_data).expect("Save harnesses failed");
    repo.save_runtimes_snapshot(&runtime_data).expect("Save runtimes failed");
    let loaded_h = repo.load_latest_harnesses().expect("Load harnesses failed");
    let loaded_r = repo.load_latest_runtimes().expect("Load runtimes failed");

    // Assert
    assert_eq!(loaded_h.profiles.len(), 1);
    assert_eq!(loaded_h.profiles[0].name, "OpenCode");
    assert_eq!(loaded_r.len(), 1);
    assert_eq!(loaded_r[0].name, "Ollama");
    assert_eq!(loaded_r[0].measured_runs, 120);
}

#[test]
fn test_runtime_optimizer_kv_cache_and_context_scaling() {
    // Arrange: Compare 24GB Workstation GPU vs 8GB Budget GPU (on 8B + 14B) vs CPU-only
    let mut m_8b = make_local_model("qwen/qwen3-8b-q4_k_m", "Qwen 3 8B", 8.0, 4.8, 6.2, 72.0);
    m_8b.context_k = 128;
    let mut m_14b = make_local_model("qwen/qwen3-14b-q4_k_m", "Qwen 3 14B", 14.0, 8.5, 10.5, 78.0);
    m_14b.context_k = 128;

    let mut rig_4090 = HardwareRigConfig::from_platform(HardwarePlatform::Gpu24Gb);
    rig_4090.gpu = GpuTier::Rtx4090_24G;
    let plan_4090 = compute_optimal_runtime_plan(&m_14b, &rig_4090, &rig_4090.simulate());

    let mut rig_4060 = HardwareRigConfig::from_platform(HardwarePlatform::Gpu8Gb);
    rig_4060.gpu = GpuTier::Rtx4060_8G;
    rig_4060.ram_gb = 32.0;
    let plan_4060_8b = compute_optimal_runtime_plan(&m_8b, &rig_4060, &rig_4060.simulate());
    let plan_4060_14b = compute_optimal_runtime_plan(&m_14b, &rig_4060, &rig_4060.simulate());

    let mut rig_cpu = HardwareRigConfig::from_platform(HardwarePlatform::MiniPcCpu);
    rig_cpu.gpu = GpuTier::None;
    rig_cpu.ram_gb = 16.0;
    let plan_cpu = compute_optimal_runtime_plan(&m_14b, &rig_cpu, &rig_cpu.simulate());

    // Assert: 24GB gets full GPU offload & >= 32k ctx; 8GB tight GPU picks Q8_0 KV to expand context; CPU uses -ngl 0
    assert_eq!(plan_4090.gpu_offload_pct, 100);
    assert!(plan_4090.recommended_ctx >= 32_768);
    assert!(plan_4090.context_options[2].max_safe_ctx >= plan_4090.context_options[1].max_safe_ctx);
    assert!(plan_4090.context_options[1].max_safe_ctx >= plan_4090.context_options[0].max_safe_ctx);
    assert_eq!(plan_4060_8b.kv_cache, KvCacheType::Q8_0);
    assert!(plan_4060_8b.recommended_ctx > plan_4060_8b.context_options[0].max_safe_ctx);
    assert_ne!(plan_4060_14b.kv_cache, KvCacheType::F16);
    assert_eq!(plan_cpu.gpu_offload_pct, 0);
    assert!(!plan_cpu.flash_attention);
    assert!(plan_cpu.llama_cmd.contains("-ngl 0"));
}

#[test]
fn test_compare_harnesses_tied_rows_deterministic_and_ui_tables_render() {
    use eframe::egui;
    use benchmark_llm_gui::presentation::components::harnesses_view::render_harnesses_view;
    use benchmark_llm_gui::presentation::components::runtimes_view::render_runtimes_view;
    use benchmark_llm_gui::presentation::i18n::Language;

    let runs = vec![
        HarnessRun {
            harness_name: "OpenCode".into(), harness_org: "OpenCode".into(),
            model_key: "glm52".into(), model_name: "GLM 5.2".into(),
            suite: "Terminal-Bench 2.0".into(), score_pct: 65.0, cost_usd: None,
            date: "2026".into(), source_url: "https://opencode.ai".into(),
        },
        HarnessRun {
            harness_name: "Cline".into(), harness_org: "Cline".into(),
            model_key: "glm52".into(), model_name: "GLM 5.2".into(),
            suite: "Terminal-Bench 2.0".into(), score_pct: 65.0, cost_usd: None,
            date: "2026".into(), source_url: "https://cline.bot".into(),
        },
        HarnessRun {
            harness_name: "Kilo Code".into(), harness_org: "Kilo".into(),
            model_key: "glm52".into(), model_name: "GLM 5.2".into(),
            suite: "KiloBench".into(), score_pct: 65.0, cost_usd: None,
            date: "2026".into(), source_url: "https://kilo.ai".into(),
        },
    ];

    let baseline: Vec<String> = compare_harnesses_by_model(&runs, 2)[0]
        .entries.iter().map(|e| e.harness_name.clone()).collect();
    for _ in 0..25 {
        let cur: Vec<String> = compare_harnesses_by_model(&runs, 2)[0]
            .entries.iter().map(|e| e.harness_name.clone()).collect();
        assert_eq!(cur, baseline, "Tied harness rows must stay deterministically ordered across frames");
    }

    let snap = HarnessSnapshotData {
        profiles: vec![
            HarnessProfile {
                name: "OpenCode".into(), repo_slug: "anomalyco/opencode".into(), org: "OpenCode".into(),
                category: HarnessCategory::CliAgent, url: "https://opencode.ai".into(),
                license: "MIT".into(), stars: 227000, last_updated: "2026-11-01".into(),
                best_score_pct: Some(65.0), avg_score_pct: Some(65.0), runs_count: 1,
                description: "Terminal AI harness".into(),
            },
            HarnessProfile {
                name: "Continue".into(), repo_slug: "continuedev/continue".into(), org: "Continue".into(),
                category: HarnessCategory::IdeExtension, url: "https://continue.dev".into(),
                license: "Apache-2.0".into(), stars: 36500, last_updated: "2026-11-01".into(),
                best_score_pct: None, avg_score_pct: None, runs_count: 0,
                description: "IDE coding extension".into(),
            },
        ],
        runs,
    };

    let runtimes = vec![LocalRuntimeProfile {
        name: "Ollama".into(), repo_slug: "ollama/ollama".into(), category: "CLI & Service".into(),
        official_url: "https://ollama.com".into(), stars: 187000, latest_version: "0.12".into(),
        last_updated: "2026-11-17".into(), license: "MIT".into(), language: "Go".into(),
        platforms: vec!["Windows".into(), "Linux".into()], backends: vec!["CUDA".into(), "Vulkan".into()],
        supports_desktop_gpu: true, supports_apple_silicon: true, supports_cpu_edge: true,
        supports_off_grid_mobile: false, measured_avg_tps: Some(113.3), measured_runs: 120,
        sample_hardware: "RTX 4090".into(), install_cmd: "ollama run qwen3.5:9b".into(),
        description: "Local runner".into(),
    }];

    let ctx = egui::Context::default();
    for subtab_idx in [0usize, 1usize] {
        ctx.memory_mut(|m| m.data.insert_temp(egui::Id::new("harness_subtab_idx"), subtab_idx));
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1280.0, 800.0)));
        let mut sync = false;
        let out = ctx.run(raw, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                if subtab_idx == 0 {
                    render_harnesses_view(ui, &snap, false, &mut sync, Language::French);
                } else {
                    render_harnesses_view(ui, &snap, false, &mut sync, Language::French);
                    ui.separator();
                    render_runtimes_view(ui, &runtimes, false, &mut sync, Language::French);
                }
            });
        });
        assert!(!out.shapes.is_empty());
    }
}
