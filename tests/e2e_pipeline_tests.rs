use std::fs;
use std::sync::Arc;
use eframe::egui;
use benchmark_llm_gui::domain::hardware::HardwarePlatform;
use benchmark_llm_gui::domain::harness::{
    normalize_base_model_key, HarnessCategory, HarnessProfile, HarnessRun, HarnessSnapshotData,
};
use benchmark_llm_gui::domain::modality::ModelModality;
use benchmark_llm_gui::domain::model::{BenchmarkScores, ModelBenchmark, SourceSelection};
use benchmark_llm_gui::domain::runtime::LocalRuntimeProfile;
use benchmark_llm_gui::domain::runtime_optimizer::compute_optimal_runtime_plan;
use benchmark_llm_gui::domain::simulator::{GpuTier, HardwareRigConfig};
use benchmark_llm_gui::domain::simulator_ranker::{rank_compatible_models, SimTableSort};
use benchmark_llm_gui::domain::table_preferences::{TableColumnId, TablePreferences};
use benchmark_llm_gui::domain::traits::{IBenchmarkRepository, IEcosystemRepository};
use benchmark_llm_gui::infrastructure::repository::sqlite_ecosystem_repo::SqliteEcosystemRepository;
use benchmark_llm_gui::infrastructure::repository::sqlite_repo::SqliteRepository;
use benchmark_llm_gui::infrastructure::updater::GitHubReleaseAdapter;
use benchmark_llm_gui::presentation::components::filter_bar::{render_filter_bar, FilterState};
use benchmark_llm_gui::presentation::components::harnesses_view::render_harnesses_view;
use benchmark_llm_gui::presentation::components::runtimes_view::render_runtimes_view;
use benchmark_llm_gui::presentation::components::simulator_view::render_simulator_view;
use benchmark_llm_gui::presentation::components::sources_view::render_sources_view;
use benchmark_llm_gui::presentation::components::table_view::render_table;
use benchmark_llm_gui::presentation::components::top_bar::{render_top_header, AppTab};
use benchmark_llm_gui::presentation::filtering::{FilterService, ModalityFilter};
use benchmark_llm_gui::presentation::i18n::Language;
use benchmark_llm_gui::presentation::updater_state::UpdaterState;

fn build_e2e_catalog() -> (Vec<ModelBenchmark>, HarnessSnapshotData, Vec<LocalRuntimeProfile>) {
    let models = vec![
        ModelBenchmark {
            id: "qwen/qwen3-8b-q4_k_m-gguf".into(), name: "Qwen 3 8B Coder".into(), family: "Qwen".into(),
            param_count_b: Some(8.0), size_gb: 4.8, vram_rec_gb: 6.2, tokens_per_sec: Some(92.0), tps_hardware: Some("RTX 4070".into()),
            scores: BenchmarkScores { open_llm: Some(71.0), coding: Some(79.5), agentic: Some(74.0), real_world: Some(68.0), average: Some(73.1), ..Default::default() },
            context_k: 64, modality: ModelModality::Text, ollama_tag: Some("qwen3:8b".into()), ..Default::default()
        },
        ModelBenchmark {
            id: "microsoft/TRELLIS.2".into(), name: "TRELLIS.2 Image-to-3D".into(), family: "Microsoft".into(),
            size_gb: 4.2, vram_rec_gb: 6.0, tokens_per_sec: Some(25.0),
            scores: BenchmarkScores { real_world: Some(76.0), average: Some(76.0), ..Default::default() },
            modality: ModelModality::ThreeDGen, ..Default::default()
        },
        ModelBenchmark {
            id: "Comfy-Org/Wan_2.2_repackaged".into(), name: "Wan 2.2 14B Video".into(), family: "Wan".into(),
            size_gb: 9.5, vram_rec_gb: 11.8,
            scores: BenchmarkScores { real_world: Some(81.0), average: Some(81.0), ..Default::default() },
            modality: ModelModality::VideoGen, ..Default::default()
        },
        ModelBenchmark {
            id: "qwen/qwen4-max".into(), name: "Qwen4 Max".into(), family: "Qwen".into(), is_cloud: true,
            scores: BenchmarkScores { open_llm: Some(88.0), coding: Some(89.0), agentic: Some(88.5), real_world: Some(86.0), average: Some(88.5), ..Default::default() },
            context_k: 128, ..Default::default()
        },
    ];

    let harnesses = HarnessSnapshotData {
        profiles: vec![
            HarnessProfile {
                name: "Cline".into(), repo_slug: "cline/cline".into(), org: "Cline".into(), category: HarnessCategory::IdeExtension,
                url: "https://github.com/cline/cline".into(), license: "Apache-2.0".into(), stars: 71000, last_updated: "2026-11-10".into(),
                best_score_pct: Some(68.5), avg_score_pct: Some(60.2), runs_count: 4, description: "Autonomous IDE coding agent".into(),
            },
            HarnessProfile {
                name: "OpenCode".into(), repo_slug: "anomalyco/opencode".into(), org: "OpenCode".into(), category: HarnessCategory::CliAgent,
                url: "https://github.com/anomalyco/opencode".into(), license: "MIT".into(), stars: 227000, last_updated: "2026-11-12".into(),
                best_score_pct: Some(59.6), avg_score_pct: Some(52.4), runs_count: 4, description: "Open-source CLI coding harness".into(),
            },
        ],
        runs: vec![
            HarnessRun {
                harness_name: "Cline".into(), harness_org: "Cline".into(), model_name: "glm-5.2".into(),
                model_key: normalize_base_model_key("glm-5.2"), suite: "Terminal-Bench (cline-medium)".into(),
                score_pct: 68.5, cost_usd: None, date: "2026-06".into(), source_url: "https://github.com/cline/benchmark-results".into(),
            },
            HarnessRun {
                harness_name: "OpenCode".into(), harness_org: "OpenCode".into(), model_name: "GLM 5.2".into(),
                model_key: normalize_base_model_key("GLM 5.2"), suite: "Terminal-Bench (opencode-medium)".into(),
                score_pct: 59.6, cost_usd: None, date: "2026-06".into(), source_url: "https://github.com/cline/benchmark-results".into(),
            },
        ],
    };

    let runtimes = vec![LocalRuntimeProfile {
        name: "Ollama".into(), repo_slug: "ollama/ollama".into(), category: "Local Server".into(), official_url: "https://ollama.com".into(),
        stars: 160000, latest_version: "v0.9.0".into(), last_updated: "2026-11-12".into(), license: "MIT".into(), language: "Go / C++".into(),
        platforms: vec!["Windows".into(), "Linux".into(), "macOS".into()], backends: vec!["CUDA".into(), "Metal".into(), "ROCm".into()],
        supports_desktop_gpu: true, supports_apple_silicon: true, supports_cpu_edge: true, supports_off_grid_mobile: false,
        measured_avg_tps: Some(82.5), measured_runs: 48, sample_hardware: "RTX 4070 / M4".into(),
        install_cmd: "curl -fsSL https://ollama.com/install.sh | sh".into(), description: "Local LLM inference daemon".into(),
    }];

    (models, harnesses, runtimes)
}

#[test]
fn e2e_persistence_simulator_optimizer_and_headless_ui_pipeline() {
    // Stage 1: Persist models, harnesses and runtimes to a real temporary SQLite DB and reload
    let db_path = std::env::temp_dir().join(format!("e2e_bench_test_{}.db", std::process::id()));
    let db_str = db_path.to_string_lossy().to_string();
    let (models, harnesses, runtimes) = build_e2e_catalog();

    {
        let mut model_repo = SqliteRepository::open(&db_str).expect("Open temp SQLite models DB");
        model_repo.save_snapshot(&models, "E2E Snapshot").expect("Save models snapshot");
        let mut eco_repo = SqliteEcosystemRepository::open(&db_str).expect("Open temp SQLite eco DB");
        eco_repo.save_harness_snapshot(&harnesses).expect("Save harnesses");
        eco_repo.save_runtimes_snapshot(&runtimes).expect("Save runtimes");
    }

    let model_repo = SqliteRepository::open(&db_str).expect("Reopen SQLite models DB");
    let (_, loaded_models) = model_repo.load_latest_snapshot().expect("Load snapshot").expect("Missing snapshot");
    let eco_repo = SqliteEcosystemRepository::open(&db_str).expect("Reopen SQLite eco DB");
    let loaded_harnesses = eco_repo.load_latest_harnesses().expect("Load harnesses");
    let loaded_runtimes = eco_repo.load_latest_runtimes().expect("Load runtimes");
    drop(model_repo);
    drop(eco_repo);
    let _ = fs::remove_file(&db_path);

    assert_eq!(loaded_models.len(), 4);
    assert!(loaded_models.iter().any(|m| m.modality == ModelModality::ThreeDGen));
    assert_eq!(loaded_harnesses.runs.len(), 2);
    assert_eq!(loaded_runtimes.len(), 1);

    // Stage 2: Filter 3D & Agentic Dev + Simulate RTX 4070 12GB + Optimize Runtime Parameters
    let mut filters = FilterState::default();
    filters.modality = ModalityFilter::Specific(ModelModality::ThreeDGen);
    let three_d_indices = FilterService::compute_filtered_indices(&loaded_models, &filters);
    assert_eq!(three_d_indices.len(), 1);

    filters.modality = ModalityFilter::All;
    filters.sources = SourceSelection::dev_agentic();
    let all_indices = FilterService::compute_filtered_indices(&loaded_models, &filters);
    assert_eq!(all_indices.len(), 4);

    let mut rig = HardwareRigConfig::from_platform(HardwarePlatform::Gpu12Gb);
    rig.gpu = GpuTier::Rtx4070_12G;
    rig.ram_gb = 32.0;
    let sim = rig.simulate();
    let compatible = rank_compatible_models(&loaded_models, &rig, &sim, &filters.sources, SimTableSort::Score, true);
    assert_eq!(compatible.len(), 3); // Qwen3 8B + TRELLIS.2 + Wan 2.2 fit inside total VRAM+RAM budget; Qwen4 cloud excluded
    let plan = compute_optimal_runtime_plan(&loaded_models[0], &rig, &sim);
    assert_eq!(plan.gpu_offload_pct, 100);
    assert!(plan.recommended_ctx >= 16_384);
    assert!(plan.llama_cmd.contains("-ctk"));

    // Stage 3: Execute real headless egui frames across ALL tabs in both FR and EN
    let ctx = egui::Context::default();
    let mut zoom = 1.0;
    let mut selected_id = Some(loaded_models[0].id.clone());
    let mut contact_open = false;
    let mut updater = UpdaterState::new(Arc::new(GitHubReleaseAdapter::new("Hidr4c", "benchmark-llm-gui")));

    for lang in [Language::French, Language::English] {
        for tab in [AppTab::Table, AppTab::Simulator, AppTab::Harnesses, AppTab::Runtimes, AppTab::Sources] {
            let mut active_tab = tab;
            let mut cur_lang = lang;
            let mut trig_sync = false;
            let mut trig_snap = None;
            let mut trig_tour = false;
            let mut trig_auto = false;
            let mut trig_exp = false;
            let mut eco_sync = false;

            let mut raw_input = egui::RawInput::default();
            raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1280.0, 800.0)));
            let output = ctx.run(raw_input, |ctx| {
                egui::TopBottomPanel::top("e2e_top").show(ctx, |ui| {
                    render_top_header(
                        ui, &mut active_tab, &mut cur_lang, &mut zoom, false, &[], None, None,
                        &mut updater, &mut trig_sync, &mut trig_snap, &mut trig_tour, &mut contact_open,
                    );
                    let available_families = FilterService::extract_unique_families(&loaded_models);
                    render_filter_bar(ui, &mut filters, &available_families, loaded_models.len(), all_indices.len(), &mut trig_auto, &mut trig_exp, cur_lang);
                });
                egui::CentralPanel::default().show(ctx, |ui| match active_tab {
                    AppTab::Table => render_table(ui, &loaded_models, &all_indices, &mut filters, &mut selected_id, cur_lang),
                    AppTab::Simulator => render_simulator_view(ui, &mut rig, None, &loaded_models, &filters, &mut selected_id, cur_lang),
                    AppTab::Harnesses => render_harnesses_view(ui, &loaded_harnesses, false, &mut eco_sync, cur_lang),
                    AppTab::Runtimes => render_runtimes_view(ui, &loaded_runtimes, false, &mut eco_sync, cur_lang),
                    AppTab::Sources => render_sources_view(ui, cur_lang),
                    _ => {}
                });
            });
            assert!(!output.shapes.is_empty(), "Rendered UI shapes must not be empty for {:?}/{:?}", lang, tab);
        }
    }
}

#[test]
fn e2e_empty_states_search_and_hardware_switch_stability() {
    // Stage 1: Verify empty-state rendering never panics across Harnesses, Runtimes, Simulator & Table
    let ctx = egui::Context::default();
    let empty_models: Vec<ModelBenchmark> = Vec::new();
    let empty_harnesses = HarnessSnapshotData::default();
    let empty_runtimes: Vec<LocalRuntimeProfile> = Vec::new();
    let mut filters = FilterState::default();
    let mut selected_id = None;
    let mut rig = HardwareRigConfig::from_platform(HardwarePlatform::SmartphoneHigh);
    rig.gpu = GpuTier::SmartphoneHigh;
    let mut eco_sync = false;

    for subtab in [0usize, 1usize] {
        ctx.memory_mut(|m| {
            m.data.insert_temp(egui::Id::new("harness_subtab_idx"), subtab);
            m.data.insert_temp(egui::Id::new("harness_model_search"), "glm".to_string());
            m.data.insert_temp(egui::Id::new("runtime_hw_filter"), 4usize);
        });
        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1024.0, 720.0)));
        let _ = ctx.run(raw_input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_table(ui, &empty_models, &[], &mut filters, &mut selected_id, Language::French);
                render_simulator_view(ui, &mut rig, None, &empty_models, &filters, &mut selected_id, Language::English);
                render_harnesses_view(ui, &empty_harnesses, false, &mut eco_sync, Language::French);
                render_runtimes_view(ui, &empty_runtimes, false, &mut eco_sync, Language::English);
            });
        });
    }

    // Stage 2: Seed harnesses + simulate interactive model search "glm" in Harnesses comparator
    let (models, harnesses, runtimes) = build_e2e_catalog();
    ctx.memory_mut(|m| {
        m.data.insert_temp(egui::Id::new("harness_subtab_idx"), 0usize);
        m.data.insert_temp(egui::Id::new("harness_model_search"), "glm".to_string());
        m.data.insert_temp(egui::Id::new("runtime_hw_filter"), 1usize);
    });
    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 760.0)));
    let out = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_harnesses_view(ui, &harnesses, false, &mut eco_sync, Language::French);
            render_runtimes_view(ui, &runtimes, false, &mut eco_sync, Language::French);
            render_simulator_view(ui, &mut rig, None, &models, &filters, &mut selected_id, Language::French);
        });
    });
    assert!(!out.shapes.is_empty());
}

#[test]
fn e2e_table_right_click_and_dynamic_columns_no_panic() {
    let (models, _, _) = build_e2e_catalog();
    let all_indices: Vec<usize> = (0..models.len()).collect();
    let (mut filters, mut selected_id, ctx) = (FilterState::default(), None, egui::Context::default());
    let mut header_pos = egui::pos2(120.0, 20.0);
    for step in 0..7 {
        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1280.0, 800.0)));
        if step == 1 {
            raw_input.events.push(egui::Event::PointerMoved(header_pos));
            raw_input.events.push(egui::Event::PointerButton { pos: header_pos, button: egui::PointerButton::Secondary, pressed: true, modifiers: egui::Modifiers::NONE });
            raw_input.events.push(egui::Event::PointerButton { pos: header_pos, button: egui::PointerButton::Secondary, pressed: false, modifiers: egui::Modifiers::NONE });
        }
        match step {
            2 => filters.table_prefs.set_advanced_columns_visible(true),
            3 => filters.table_prefs.set_raw_sources_visible(true),
            4 => { filters.table_prefs.show_all_columns(); filters.column_chooser_open = true; }
            5 => { filters.table_prefs.hidden_columns = TableColumnId::all().iter().copied().collect(); filters.column_chooser_open = false; }
            6 => {
                let mut corrupted = TablePreferences::default();
                corrupted.column_order = vec![TableColumnId::Model, TableColumnId::Model];
                corrupted.sanitize();
                assert_eq!(corrupted.column_order.len(), TableColumnId::all().len());
                filters.table_prefs = corrupted;
            }
            _ => {}
        }
        let _ = ctx.run(raw_input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_table(ui, &models, &all_indices, &mut filters, &mut selected_id, Language::French);
                if step == 0 {
                    if let Some(r) = ctx.memory(|m| m.data.get_temp::<egui::Rect>(egui::Id::new("tour_target_grouped"))) {
                        header_pos = egui::pos2((r.left() - 25.0).max(8.0), r.center().y);
                    }
                }
            });
        });
    }

    // Verify mid-frame header->body mutation invariance directly via TableBuilder + render_table_row
    let frozen_cols: Vec<TableColumnId> = filters.table_prefs.column_order.iter().copied().filter(|c| filters.table_prefs.is_visible(*c)).collect();
    let items = benchmark_llm_gui::presentation::components::table_items::build_display_items(&models, &all_indices, &filters);
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut tbl = egui_extras::TableBuilder::new(ui);
            for _ in &frozen_cols { tbl = tbl.column(egui_extras::Column::initial(80.0)); }
            tbl.header(24.0, |_| { filters.table_prefs.show_all_columns(); }).body(|body| {
                body.rows(20.0, items.len(), |mut row| {
                    let r_idx = row.index();
                    let it = &items[r_idx];
                    benchmark_llm_gui::presentation::components::table_row::render_table_row(&mut row, &frozen_cols, it, &models[it.model_idx], r_idx == 0, false, &mut filters, &mut selected_id, Language::French);
                });
            });
        });
    });
}
