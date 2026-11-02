use benchmark_llm_gui::domain::comparator::{CompMetricX, CompMetricY, HardwareComparator, ModelRole};
use benchmark_llm_gui::domain::model::{BenchmarkScores, ModelBenchmark, SourceSelection};
use benchmark_llm_gui::domain::simulator::{GpuTier, HardwareRigConfig, SimTargetMode};

fn make_test_model(id: &str, name: &str, vram_gb: f32, score: f32) -> ModelBenchmark {
    ModelBenchmark {
        id: id.to_string(),
        name: name.to_string(),
        family: "Test".to_string(),
        param_count_b: Some(vram_gb * 1.3),
        size_gb: vram_gb * 0.9,
        vram_rec_gb: vram_gb,
        tokens_per_sec: Some(60.0),
        tps_hardware: Some("RTX 4090".to_string()),
        scores: BenchmarkScores {
            average: Some(score),
            coding: Some(score),
            agentic: Some(score),
            open_llm: Some(score),
            ..Default::default()
        },
        downloads: 1000,
        likes: 100,
        ollama_tag: Some(name.to_lowercase()),
        hf_url: "https://huggingface.co".to_string(),
        is_moe: false,
        context_k: 8192,
        is_cloud: false,
        provider: None,
        pricing_input_usd: None,
        pricing_output_usd: None,
        release_year: Some(2026),
        modality: Default::default(),
        economic: None,
    }
}

#[test]
fn test_comparator_metrics_within_vram() {
    let rig = HardwareRigConfig {
        mode: SimTargetMode::WorkstationPc,
        gpu: GpuTier::Rtx5090_32G,
        ram_gb: 64.0,
        nas_web_ram_gb: 4.0,
    };
    let sources = SourceSelection::default();

    let model = make_test_model("qwen/qwen3.8-27b", "Qwen 3.8 27B", 16.0, 78.0);
    let metrics = HardwareComparator::compute_metrics(&model, &rig, &sources);

    assert!(metrics.fits_pure_vram, "16 GB model should fit in 32 GB VRAM");
    assert!(metrics.vram_headroom_gb > 14.0, "Expected >14 GB headroom");
    assert!(metrics.estimated_tps > 50.0, "RTX 5090 bandwidth should yield high TPS");
    assert_eq!(metrics.role, ModelRole::HeavyReasoner);
    assert!(!metrics.is_cloud);
    assert!(metrics.sec_per_200_words < 5.0);
    assert!(metrics.kwh_per_100k_tokens > 0.0);
    assert!(metrics.cost_per_100k_tokens_eur > 0.0);
}

#[test]
fn test_comparator_metrics_exceeding_vram() {
    let rig = HardwareRigConfig {
        mode: SimTargetMode::WorkstationPc,
        gpu: GpuTier::Rtx4060_8G,
        ram_gb: 16.0,
        nas_web_ram_gb: 4.0,
    };
    let sources = SourceSelection::default();

    let heavy_model = make_test_model("deepseek/r1-32b", "DeepSeek R1 32B", 20.0, 82.0);
    let metrics = HardwareComparator::compute_metrics(&heavy_model, &rig, &sources);

    assert!(!metrics.fits_pure_vram, "20 GB model cannot fit in 8 GB VRAM");
    assert_eq!(metrics.role, ModelRole::OffloadWarning);
    assert!(metrics.estimated_tps < 20.0, "Offloading to DDR RAM must throttle TPS heavily");
    assert!(metrics.sec_per_200_words > 10.0);
}

#[test]
fn test_comparator_cloud_model_zero_vram() {
    let rig = HardwareRigConfig {
        mode: SimTargetMode::WorkstationPc,
        gpu: GpuTier::Rtx5090_32G,
        ram_gb: 64.0,
        nas_web_ram_gb: 4.0,
    };
    let sources = SourceSelection::default();
    let mut cloud_model = make_test_model("google/gemini-pro", "Gemini 4 Carbon", 0.0, 92.0);
    cloud_model.is_cloud = true;
    cloud_model.pricing_input_usd = Some(2.50);
    cloud_model.pricing_output_usd = Some(10.00);

    let metrics = HardwareComparator::compute_metrics(&cloud_model, &rig, &sources);
    assert!(metrics.is_cloud);
    assert_eq!(metrics.vram_usage_gb, 0.0, "Cloud models do not use local VRAM");
    assert_eq!(metrics.power_watts, 0.0, "Cloud models do not use local GPU power");
    assert_eq!(metrics.role, ModelRole::CloudHosted);
    assert!(metrics.cost_per_100k_tokens_eur > 0.0, "API pricing must be computed");
}

#[test]
fn test_comparator_metric_axes_extraction() {
    let model = make_test_model("qwen/qwen3.8-27b", "Qwen 3.8 27B", 16.0, 78.0);
    let rig = HardwareRigConfig {
        mode: SimTargetMode::WorkstationPc,
        gpu: GpuTier::Rtx5090_32G,
        ram_gb: 64.0,
        nas_web_ram_gb: 4.0,
    };
    let sources = SourceSelection::default();
    let metrics = HardwareComparator::compute_metrics(&model, &rig, &sources);

    for x in CompMetricX::ALL {
        assert!(!x.label().is_empty());
        let val = x.extract(&metrics);
        assert!(val >= 0.0, "Metric X {} returned invalid value", x.label());
    }

    for y in CompMetricY::ALL {
        assert!(!y.label().is_empty());
        let val = y.extract(&metrics, &model);
        assert!(val >= 0.0, "Metric Y {} returned invalid value", y.label());
    }
}

#[test]
fn test_search_and_zoom_bounds() {
    let zoom_levels: [f32; 6] = [0.90, 1.0, 1.15, 1.25, 1.40, 1.50];
    for &z in &zoom_levels {
        assert!(z >= 0.85 && z <= 1.60);
        let p = (z * 100.0).round() as i32;
        assert!(p >= 90 && p <= 150);
    }

    let m1 = make_test_model("qwen/3.8-9b", "Qwen3.8-9B-Distill", 5.2, 54.9);
    let m2 = make_test_model("qwen/2.5-7b", "Qwen2.5-7B", 4.1, 66.4);
    let catalog = vec![m1, m2];

    let query = "qwen3.8".to_lowercase();
    let matches: Vec<_> = catalog.iter().filter(|m| m.name.to_lowercase().contains(&query)).collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].name, "Qwen3.8-9B-Distill");
}

#[test]
fn test_graph_data_bounds_computation() {
    use benchmark_llm_gui::presentation::components::graph_bounds::compute_data_bounds;
    use benchmark_llm_gui::presentation::components::graph_types::PlottedPoint;

    // Empty list returns None
    assert!(compute_data_bounds(&[]).is_none());

    // Single point
    let p1 = PlottedPoint {
        index: 0, x: 12.0, y: 75.0, name: "M1".into(), family: "F1".into(),
        vram_gb: 12.0, size_gb: 10.0, score: 75.0, speed: Some(60.0),
        speed_hw: None, year: Some(2025), modality: Default::default(),
    };
    let bounds1 = compute_data_bounds(&[p1]).expect("Expected bounds");
    assert_eq!(bounds1.0, [12.0, 75.0]);
    assert_eq!(bounds1.1, [12.0, 75.0]);

    // Multi points (including small price numbers)
    let p2 = PlottedPoint {
        index: 1, x: 0.25, y: 88.0, name: "Cloud1".into(), family: "F2".into(),
        vram_gb: 0.0, size_gb: 0.0, score: 88.0, speed: Some(120.0),
        speed_hw: None, year: Some(2026), modality: Default::default(),
    };
    let p3 = PlottedPoint {
        index: 2, x: 1.50, y: 92.5, name: "Cloud2".into(), family: "F3".into(),
        vram_gb: 0.0, size_gb: 0.0, score: 92.5, speed: Some(90.0),
        speed_hw: None, year: Some(2026), modality: Default::default(),
    };
    let bounds_price = compute_data_bounds(&[p2, p3]).expect("Expected price bounds");
    assert_eq!(bounds_price.0, [0.25, 88.0]);
    assert_eq!(bounds_price.1, [1.50, 92.5]);
}

#[test]
fn test_graph_view_state_and_node_labels() {
    use benchmark_llm_gui::presentation::components::graph_types::GraphViewState;
    let state = GraphViewState::default();
    assert!(!state.show_node_labels);
    assert!(state.show_pareto_curve);
    assert_eq!(state.vram_budget_gb, 48.0);
}
