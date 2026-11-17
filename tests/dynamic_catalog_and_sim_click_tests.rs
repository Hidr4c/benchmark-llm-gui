use benchmark_llm_gui::domain::hardware::HardwarePlatform;
use benchmark_llm_gui::domain::model::{BenchmarkScores, ModelBenchmark, SourceSelection};
use benchmark_llm_gui::domain::simulator::{GpuTier, HardwareRigConfig, SimTargetMode};
use benchmark_llm_gui::infrastructure::fetcher::awesome_discovery::parse_awesome_markdown;
use benchmark_llm_gui::presentation::components::simulator_table_view::render_simulator_compatible_table;
use benchmark_llm_gui::presentation::i18n::Language;
use eframe::egui;

#[test]
fn test_awesome_markdown_dynamic_discovery_and_noise_filtering() {
    let sample_md = r#"
## Inference engines
- [Ollama](https://ollama.com/) ![stars](https://img.shields.io/github/stars/ollama/ollama?style=flat) - Run local LLMs cleanly on GPU & CPU
- **[llama.cpp](https://github.com/ggml-org/llama.cpp)** `135k` — Fast C/C++ inference engine supporting CUDA, Metal & Vulkan
- [Awesome-LLM](https://github.com/Hannibal046/Awesome-LLM) - Meta link list should be excluded
## Tutorials
- [Beginner Guide](https://github.com/example/llm-tutorial) - Should be excluded by section filter
## Coding Agents
- **[OpenCode](https://github.com/anomalyco/opencode)** `227k` — Terminal-native coding agent harness
- [Kilo Code](https://github.com/Kilo-Org/kilocode) - AI coding extension for VS Code & JetBrains
"#;

    let runtimes = parse_awesome_markdown(sample_md, &["inference engines"], &["tutorials", "archived"]);
    assert_eq!(runtimes.len(), 2);
    assert_eq!(runtimes[0].slug, "ollama/ollama");
    assert_eq!(runtimes[1].slug, "ggml-org/llama.cpp");
    assert_eq!(runtimes[1].inline_stars, 135_000);

    let harnesses = parse_awesome_markdown(sample_md, &["coding agents"], &["tutorials"]);
    assert_eq!(harnesses.len(), 2);
    assert_eq!(harnesses[0].name, "OpenCode");
    assert_eq!(harnesses[0].inline_stars, 227_000);
    assert_eq!(harnesses[1].slug, "Kilo-Org/kilocode");
}

#[test]
fn test_zero_hardcoded_runtime_or_harness_catalogs_in_adapters() {
    let rt_src = include_str!("../src/infrastructure/fetcher/runtime_adapter.rs");
    let hn_src = include_str!("../src/infrastructure/fetcher/harness_adapter.rs");
    assert!(!rt_src.contains("RuntimeSourceSpec"), "Static RuntimeSourceSpec array must stay deleted");
    assert!(!rt_src.contains("nomic-ai/gpt4all"), "No hardcoded software repo slugs in runtime_adapter");
    assert!(!hn_src.contains("gh_specs"), "Static gh_specs harness list must stay deleted");
    assert!(!hn_src.contains("anthropics/claude-code"), "No hardcoded harness repo slugs in harness_adapter");
}

#[test]
fn test_simulator_table_row_click_reliably_selects_model_across_entire_row() {
    let ctx = egui::Context::default();
    let rig = HardwareRigConfig {
        mode: SimTargetMode::WorkstationPc,
        gpu: GpuTier::Rtx5090_32G,
        ram_gb: 64.0,
        nas_web_ram_gb: 0.0,
        platform: HardwarePlatform::Gpu32Gb,
        max_storage_gb: None,
    };
    let sim = rig.simulate();
    let models = vec![ModelBenchmark {
        id: "qwen/qwen3-14b-q4".into(),
        name: "Qwen3-14B".into(),
        family: "Qwen".into(),
        param_count_b: Some(14.0),
        size_gb: 9.0,
        vram_rec_gb: 11.0,
        scores: BenchmarkScores {
            open_llm: Some(82.0),
            average: Some(82.0),
            ..Default::default()
        },
        ..Default::default()
    }];

    let mut selected = None;

    // Settle layout across two frames before locating row coordinates
    let mut click_target: Option<egui::Pos2> = None;
    for _ in 0..2 {
        let mut raw1 = egui::RawInput::default();
        raw1.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1280.0, 720.0)));
        let out1 = ctx.run(raw1, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_simulator_compatible_table(
                    ui, &rig, &sim, &models, &SourceSelection::default(), &mut selected, Language::French,
                );
            });
        });
        for clipped in &out1.shapes {
            if let egui::epaint::Shape::Text(t) = &clipped.shape {
                if t.galley.text().contains("Qwen3-14B") {
                    click_target = Some(egui::pos2(t.pos.x + 120.0, t.pos.y + 6.0));
                }
            }
        }
    }
    let pos = click_target.expect("Qwen3-14B row must render in table");

    // Simulate single-frame pointer click (move + down + up) like UiDriver
    let mut raw_click = egui::RawInput::default();
    raw_click.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1280.0, 720.0)));
    raw_click.events = vec![
        egui::Event::PointerMoved(pos),
        egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::NONE },
        egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: false, modifiers: egui::Modifiers::NONE },
    ];
    let _ = ctx.run(raw_click, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_simulator_compatible_table(
                ui, &rig, &sim, &models, &SourceSelection::default(), &mut selected, Language::French,
            );
        });
    });

    assert_eq!(selected.as_deref(), Some("qwen/qwen3-14b-q4"), "Clicking row padding must reliably select the model for inspection");

    // Verify clicking another column on the exact same row (padding at +380px) also detects clicks
    selected = None;
    let pos2 = egui::pos2(pos.x + 260.0, pos.y);
    let mut raw_click2 = egui::RawInput::default();
    raw_click2.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1280.0, 720.0)));
    raw_click2.events = vec![
        egui::Event::PointerMoved(pos2),
        egui::Event::PointerButton { pos: pos2, button: egui::PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::NONE },
        egui::Event::PointerButton { pos: pos2, button: egui::PointerButton::Primary, pressed: false, modifiers: egui::Modifiers::NONE },
    ];
    let _ = ctx.run(raw_click2, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_simulator_compatible_table(
                ui, &rig, &sim, &models, &SourceSelection::default(), &mut selected, Language::French,
            );
        });
    });
    assert_eq!(selected.as_deref(), Some("qwen/qwen3-14b-q4"), "Clicking downstream metric columns must also select the model");
}

#[test]
#[ignore = "Live network verification of 100% dynamic catalog fetchers"]
fn test_live_dynamic_runtime_and_harness_sync_smoke() {
    use benchmark_llm_gui::infrastructure::fetcher::ecosystem_fetcher::LiveEcosystemFetcher;
    use benchmark_llm_gui::domain::traits::IEcosystemFetcher;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let fetcher = LiveEcosystemFetcher::default();
    let (h_res, r_res) = rt.block_on(async {
        tokio::join!(fetcher.fetch_harnesses(), fetcher.fetch_runtimes())
    });
    let harnesses = h_res.expect("fetch_harnesses failed");
    let runtimes = r_res.expect("fetch_runtimes failed");
    println!("LIVE Harnesses: {} profiles, {} runs", harnesses.profiles.len(), harnesses.runs.len());
    for p in harnesses.profiles.iter().take(10) {
        println!("  H: {} [best={:?}, runs={}, stars={}]", p.name, p.best_score_pct, p.runs_count, p.stars);
    }
    println!("LIVE Runtimes: {} profiles", runtimes.len());
    for p in runtimes.iter().take(15) {
        println!("  R: {} [score={:.1}, tps={:?}, stars={}, cat={}]", p.name, p.rating_score(), p.measured_avg_tps, p.stars, p.category);
    }
    assert!(harnesses.profiles.len() >= 40, "Expected 40+ dynamic harnesses");
    assert!(harnesses.runs.len() >= 100, "Expected 100+ live harness runs");
    assert!(runtimes.len() >= 20, "Expected 20+ dynamic local runtimes");
    assert!(runtimes.iter().any(|r| r.stars > 50_000), "Expected live GitHub stars");
}
