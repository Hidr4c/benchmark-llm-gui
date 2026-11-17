use std::collections::BTreeMap;
use benchmark_llm_gui::domain::hardware::HardwarePlatform;
use benchmark_llm_gui::domain::harness::{HarnessCategory, HarnessProfile, HarnessRun, HarnessSnapshotData};
use benchmark_llm_gui::domain::model::{BenchmarkScores, ModelBenchmark, SourceSelection};
use benchmark_llm_gui::domain::runtime::LocalRuntimeProfile;
use benchmark_llm_gui::domain::simulator::{GpuTier, HardwareRigConfig, SimTargetMode};
use benchmark_llm_gui::domain::simulator_ranker::{
    rank_compatible_models, sort_simulated_entries, SimTableSort,
};
use benchmark_llm_gui::domain::table_filter::{ColumnFilter, FilterOperator};
use benchmark_llm_gui::presentation::components::graph_bounds::{
    format_log_price_tick, log_price_grid_marks, to_plot_x,
};
use benchmark_llm_gui::presentation::components::graph_types::{cloud_api_price_usd, GraphMetric, GraphViewState};
use benchmark_llm_gui::presentation::components::graph_view::render_graph_view;
use benchmark_llm_gui::presentation::components::harnesses_catalog::render_harness_catalog_tab;
use benchmark_llm_gui::presentation::components::runtimes_view::render_runtimes_view;
use benchmark_llm_gui::presentation::components::simulator_table_view::render_simulator_compatible_table;
use benchmark_llm_gui::presentation::components::simulator_view::render_simulator_view;
use benchmark_llm_gui::presentation::components::filter_bar::FilterState;
use benchmark_llm_gui::presentation::components::table_filter_header::{
    active_filter_count, matches_indexed_filters,
};
use benchmark_llm_gui::presentation::filtering::ModelScope;
use benchmark_llm_gui::presentation::i18n::Language;
use eframe::egui;

fn dummy_harness(name: &str, cat: HarnessCategory, best: Option<f32>, runs: usize, stars: u64) -> HarnessProfile {
    HarnessProfile {
        name: name.into(),
        repo_slug: format!("org/{}", name.to_lowercase()),
        org: "Org".into(),
        category: cat,
        url: "https://github.com/org/repo".into(),
        license: "MIT".into(),
        stars,
        last_updated: "2026-11-17".into(),
        best_score_pct: best,
        avg_score_pct: best,
        runs_count: runs,
        description: format!("{name} coding harness"),
    }
}

fn dummy_runtime(name: &str, cat: &str, tps: Option<f32>, stars: u64, off_grid: bool) -> LocalRuntimeProfile {
    LocalRuntimeProfile {
        name: name.into(),
        repo_slug: format!("org/{}", name.to_lowercase()),
        category: cat.into(),
        official_url: "https://github.com/org/rt".into(),
        stars,
        latest_version: "1.0".into(),
        last_updated: "2026-11-17".into(),
        license: "MIT".into(),
        language: "Rust".into(),
        platforms: vec!["Windows".into(), "Linux".into()],
        backends: if off_grid { vec!["Metal".into(), "CPU".into()] } else { vec!["CUDA".into(), "Vulkan".into()] },
        supports_desktop_gpu: true,
        supports_apple_silicon: true,
        supports_cpu_edge: true,
        supports_off_grid_mobile: off_grid,
        measured_avg_tps: tps,
        measured_runs: 50,
        sample_hardware: "RTX 4090".into(),
        install_cmd: "run model".into(),
        description: "Local engine".into(),
    }
}

fn dummy_model(id: &str, name: &str, vram: f32, score: f32, coding: Option<f32>) -> ModelBenchmark {
    ModelBenchmark {
        id: id.into(),
        name: name.into(),
        family: "Qwen".into(),
        param_count_b: Some(14.0),
        size_gb: vram * 0.85,
        vram_rec_gb: vram,
        scores: BenchmarkScores {
            open_llm: Some(score),
            coding,
            agentic: coding.map(|c| c * 0.9),
            average: Some(score),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn test_indexed_column_filters_operators_and_lazy_shortcircuit() {
    let mut filters = BTreeMap::new();
    filters.insert(1, ColumnFilter::new(FilterOperator::Contains, "Open"));
    filters.insert(3, ColumnFilter::new(FilterOperator::GreaterEqual, "60"));
    filters.insert(6, ColumnFilter::new(FilterOperator::LessEqual, "200000"));

    assert_eq!(active_filter_count(&filters), 3);
    assert!(matches_indexed_filters(&filters, |col| match col {
        1 => "OpenCode CLI".into(),
        3 => "68.5".into(),
        6 => "120000".into(),
        _ => String::new(),
    }));
    assert!(!matches_indexed_filters(&filters, |col| match col {
        1 => "OpenCode CLI".into(),
        3 => "52.0".into(),
        6 => "120000".into(),
        _ => String::new(),
    }));

    let empty_filters = BTreeMap::new();
    assert_eq!(active_filter_count(&empty_filters), 0);
    assert!(matches_indexed_filters(&empty_filters, |_| panic!("must short-circuit when no active filters")));
}

#[test]
fn test_sort_simulated_entries_keeps_missing_values_last_in_both_directions() {
    let rig = HardwareRigConfig {
        mode: SimTargetMode::WorkstationPc,
        gpu: GpuTier::Rtx4090_24G,
        ram_gb: 64.0,
        nas_web_ram_gb: 0.0,
        platform: HardwarePlatform::Gpu24Gb,
        max_storage_gb: None,
    };
    let sim = rig.simulate();
    let models = vec![
        dummy_model("m/alpha", "Alpha-7B", 6.0, 62.0, None),
        dummy_model("m/beta", "Beta-14B", 10.0, 75.0, Some(71.0)),
        dummy_model("m/gamma", "Gamma-32B", 18.0, 84.0, Some(55.0)),
    ];
    let mut entries = rank_compatible_models(&models, &rig, &sim, &SourceSelection::default(), SimTableSort::Score, false);

    sort_simulated_entries(&mut entries, 4, true, true);
    assert_eq!(entries[0].name, "Gamma-32B");
    assert_eq!(entries[1].name, "Beta-14B");
    assert_eq!(entries[2].name, "Alpha-7B");

    sort_simulated_entries(&mut entries, 1, false, true);
    assert_eq!(entries[0].name, "Gamma-32B");
    assert_eq!(entries[2].name, "Alpha-7B");
}

#[test]
fn test_cloud_pareto_excludes_unpriced_models_and_scales_logarithmically() {
    let mut unpriced = dummy_model("api/unpriced-media", "Unpriced-Media-X", 0.0, 92.0, None);
    unpriced.is_cloud = true;
    unpriced.pricing_input_usd = None;
    unpriced.pricing_output_usd = None;

    let mut priced = dummy_model("api/priced-llm", "Priced-LLM", 0.0, 81.0, Some(80.0));
    priced.is_cloud = true;
    priced.pricing_input_usd = Some(1.0);
    priced.pricing_output_usd = Some(5.0);

    assert_eq!(cloud_api_price_usd(&unpriced), None);
    assert_eq!(cloud_api_price_usd(&priced), Some(2.0)); // 0.75 * 1.0 + 0.25 * 5.0
    assert!(GraphMetric::PriceUsd.value_of(&unpriced, &FilterState::default(), false) < 0.0);

    assert!((to_plot_x(10.0, true) - 1.0).abs() < 1e-6);
    assert_eq!(format_log_price_tick(0.0), "$1.0");
    assert_eq!(format_log_price_tick(-1.0), "$0.10");
    let marks = log_price_grid_marks(egui_plot::GridInput { bounds: (-2.0, 2.0), base_step_size: 1.0 });
    assert!(marks.len() >= 10);

    // Render Cloud Pareto cleanly both with mixed unpriced/priced AND 100% unpriced fallback
    let ctx = egui::Context::default();
    let mut filters = FilterState::default();
    filters.scope = ModelScope::CloudOnly;
    let mut gstate = GraphViewState::default();
    let models = vec![unpriced.clone(), priced];
    let mut selected = None;

    for indices in [vec![0usize, 1usize], vec![0usize]] {
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1280.0, 800.0)));
        let out = ctx.run(raw, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_graph_view(ui, &mut gstate, &models, &indices, &filters, &mut selected, Language::French);
            });
        });
        assert!(!out.shapes.is_empty());
    }
}

#[test]
fn test_simulator_ram_reduction_updates_stability_and_embedded_view() {
    let rig_64 = HardwareRigConfig {
        mode: SimTargetMode::WorkstationPc,
        gpu: GpuTier::Rtx5090_32G,
        ram_gb: 64.0,
        nas_web_ram_gb: 0.0,
        platform: HardwarePlatform::Gpu32Gb,
        max_storage_gb: None,
    };
    let rig_11 = HardwareRigConfig { ram_gb: 11.5, ..rig_64.clone() };
    let sim_64 = rig_64.simulate();
    let sim_11 = rig_11.simulate();
    assert!(sim_11.total_ai_capacity_gb < sim_64.total_ai_capacity_gb);
    assert!(sim_11.stability_percent < sim_64.stability_percent);

    let ctx = egui::Context::default();
    let mut rig = rig_11;
    let filters = FilterState::default();
    let models = vec![
        dummy_model("qwen/a", "Qwen-14B", 10.0, 74.0, Some(69.0)),
        dummy_model("qwen/b", "Qwen-7B", 5.0, 61.0, Some(52.0)),
    ];
    let mut selected = None;
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1400.0, 900.0)));
    let out = ctx.run(raw, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                render_simulator_view(ui, &mut rig, None, &models, &filters, &mut selected, Language::French);
            });
        });
    });
    assert!(!out.shapes.is_empty());
}

#[test]
fn test_headless_secondary_tables_render_with_header_filters_and_sorts() {
    let ctx = egui::Context::default();
    let snap = HarnessSnapshotData {
        profiles: vec![
            dummy_harness("OpenCode", HarnessCategory::CliAgent, Some(68.4), 3, 42000),
            dummy_harness("Cline", HarnessCategory::IdeExtension, Some(61.2), 2, 58000),
            dummy_harness("Continue", HarnessCategory::IdeExtension, None, 0, 36500),
        ],
        runs: vec![HarnessRun {
            harness_name: "OpenCode".into(),
            harness_org: "OpenCode".into(),
            model_key: "qwen35".into(),
            suite: "SWE-bench".into(),
            model_name: "Qwen 3.5".into(),
            score_pct: 68.4,
            cost_usd: None,
            date: "2026-11".into(),
            source_url: "https://opencode.ai".into(),
        }],
    };
    let runtimes = vec![
        dummy_runtime("Ollama", "CLI & Service", Some(115.0), 187000, false),
        dummy_runtime("PocketPal", "Mobile", None, 9500, true),
    ];
    let rig = HardwareRigConfig {
        mode: SimTargetMode::WorkstationPc,
        gpu: GpuTier::Rtx4090_24G,
        ram_gb: 64.0,
        nas_web_ram_gb: 0.0,
        platform: HardwarePlatform::Gpu24Gb,
        max_storage_gb: None,
    };
    let sim = rig.simulate();
    let models = vec![
        dummy_model("qwen/a", "Qwen-14B", 10.0, 74.0, Some(69.0)),
        dummy_model("qwen/b", "Qwen-7B", 5.0, 61.0, Some(52.0)),
    ];

    for filter_pattern in ["Open", "zzz_non_matching_pattern"] {
        let mut h_filters = BTreeMap::new();
        h_filters.insert(1usize, ColumnFilter::new(FilterOperator::Contains, filter_pattern));
        let mut r_filters = BTreeMap::new();
        r_filters.insert(1usize, ColumnFilter::new(FilterOperator::Contains, if filter_pattern == "Open" { "Ollama" } else { filter_pattern }));
        let mut s_filters = BTreeMap::new();
        s_filters.insert(6usize, ColumnFilter::new(FilterOperator::LessEqual, if filter_pattern == "Open" { "12" } else { "0.1" }));

        ctx.memory_mut(|m| {
            m.data.insert_temp(egui::Id::new("harn_cat_col_filters"), h_filters);
            m.data.insert_temp(egui::Id::new("harn_cat_sort_col"), 6usize);
            m.data.insert_temp(egui::Id::new("harn_cat_sort_asc"), false);
            m.data.insert_temp(egui::Id::new("runtime_col_filters"), r_filters);
            m.data.insert_temp(egui::Id::new("runtime_sort_col_v2"), 3usize);
            m.data.insert_temp(egui::Id::new("sim_inline_col_filters"), s_filters);
            m.data.insert_temp(egui::Id::new("sim_inline_sort_col"), 6usize);
            m.data.insert_temp(egui::Id::new("sim_inline_sort_asc"), true);
        });

        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1400.0, 900.0)));
        let mut cat_idx = 0usize;
        let mut search = String::new();
        let mut sync = false;
        let mut selected = None;

        let out = ctx.run(raw, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_harness_catalog_tab(ui, &snap, &mut cat_idx, &mut search, true);
                ui.separator();
                render_runtimes_view(ui, &runtimes, false, &mut sync, Language::English);
                ui.separator();
                render_simulator_compatible_table(
                    ui, &rig, &sim, &models, &SourceSelection::default(), &mut selected, Language::French,
                );
            });
        });
        assert!(!out.shapes.is_empty());
    }
}
