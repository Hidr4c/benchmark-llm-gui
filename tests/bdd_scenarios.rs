use benchmark_llm_gui::domain::hardware::HardwarePlatform;
use benchmark_llm_gui::domain::model::{BenchmarkScores, ModelBenchmark};
use benchmark_llm_gui::domain::traits::IBenchmarkRepository;
use benchmark_llm_gui::infrastructure::repository::sqlite_repo::SqliteRepository;

fn build_catalog() -> Vec<ModelBenchmark> {
    vec![
        ModelBenchmark {
            id: "smollm/smollm2-360m".into(),
            name: "SmolLM2 360M".into(),
            family: "SmolLM".into(),
            param_count_b: Some(0.36),
            size_gb: 0.22,
            vram_rec_gb: 0.8,
            tokens_per_sec: Some(250.0),
            tps_hardware: Some("Raspberry Pi 5".into()),
            scores: BenchmarkScores { average: Some(15.2), ..Default::default() },
            downloads: 800000,
            likes: 1200,
            ollama_tag: Some("smollm2:360m".into()),
            hf_url: "https://huggingface.co/test".into(),
            is_moe: false,
            context_k: 8,
            ..Default::default()
        },
        ModelBenchmark {
            id: "qwen/qwen2.5-0.5b".into(),
            name: "Qwen 2.5 0.5B".into(),
            family: "Qwen".into(),
            param_count_b: Some(0.49),
            size_gb: 0.35,
            vram_rec_gb: 1.0,
            tokens_per_sec: Some(280.0),
            tps_hardware: Some("Raspberry Pi 5".into()),
            scores: BenchmarkScores { average: Some(14.8), ..Default::default() },
            downloads: 1200000,
            likes: 1900,
            ollama_tag: Some("qwen2.5:0.5b".into()),
            hf_url: "https://huggingface.co/test".into(),
            is_moe: false,
            context_k: 32,
            ..Default::default()
        },
        ModelBenchmark {
            id: "qwen/qwen2.5-3b".into(),
            name: "Qwen 2.5 3B".into(),
            family: "Qwen".into(),
            param_count_b: Some(3.1),
            size_gb: 1.9,
            vram_rec_gb: 3.7,
            tokens_per_sec: Some(160.0),
            tps_hardware: Some("Raspberry Pi 5".into()),
            scores: BenchmarkScores { average: Some(27.1), ..Default::default() },
            downloads: 2500000,
            likes: 2100,
            ollama_tag: Some("qwen2.5:3b".into()),
            hf_url: "https://huggingface.co/test".into(),
            is_moe: false,
            context_k: 32,
            ..Default::default()
        },
        ModelBenchmark {
            id: "meta/llama-3.1-8b".into(),
            name: "Llama 3.1 8B".into(),
            family: "Llama".into(),
            param_count_b: Some(8.0),
            size_gb: 4.8,
            vram_rec_gb: 6.8,
            tokens_per_sec: Some(110.0),
            tps_hardware: Some("RTX 4060".into()),
            scores: BenchmarkScores { average: Some(32.8), ..Default::default() },
            downloads: 8000000,
            likes: 6000,
            ollama_tag: Some("llama3.1:8b".into()),
            hf_url: "https://huggingface.co/test".into(),
            is_moe: false,
            context_k: 128,
            ..Default::default()
        },
        ModelBenchmark {
            id: "deepseek/deepseek-r1-32b-fp16".into(),
            name: "DeepSeek R1 Distill 32B".into(),
            family: "DeepSeek".into(),
            param_count_b: Some(32.5),
            size_gb: 19.9,
            vram_rec_gb: 22.8,
            tokens_per_sec: Some(42.0),
            tps_hardware: Some("RTX 5090".into()),
            scores: BenchmarkScores {
                open_llm: Some(84.0),
                coding: Some(85.0),
                mmlu_pro: Some(84.0),
                agentic: Some(83.0),
                average: Some(84.0),
                ..Default::default()
            },
            release_year: Some(2026),
            downloads: 9000000,
            likes: 7000,
            ollama_tag: Some("deepseek-r1:32b".into()),
            hf_url: "https://huggingface.co/test".into(),
            is_moe: false,
            context_k: 64,
            ..Default::default()
        },
    ]
}

#[test]
fn bdd_scenario_embedded_iot_and_raspberry_pi() {
    // GIVEN a catalog of mixed LLMs (ranging from 0.36B to 32B)
    let catalog = build_catalog();

    // WHEN the user filters for the Raspberry Pi 4 hardware platform (RAM budget <= 2.5 GB)
    let max_ram = HardwarePlatform::RaspberryPi4.max_usable_memory_gb().unwrap();
    let mut pi_models: Vec<&ModelBenchmark> = catalog
        .iter()
        .filter(|m| m.size_gb <= max_ram)
        .collect();

    // THEN all returned models fit comfortably on a Raspberry Pi
    assert_eq!(pi_models.len(), 3);
    assert!(pi_models.iter().all(|m| m.is_embedded_friendly()));

    // AND WHEN sorted by Efficiency Ratio (Intelligence points per GB)
    pi_models.sort_by(|a, b| b.efficiency_ratio().partial_cmp(&a.efficiency_ratio()).unwrap());

    // THEN the #1 most efficient model is identified
    let top_efficient = pi_models[0];
    assert_eq!(top_efficient.name, "SmolLM2 360M");
    assert!(top_efficient.efficiency_ratio() >= 50.0);
}

#[test]
fn bdd_scenario_rtx_5090_flagship_discovery() {
    // GIVEN a catalog with various models
    let catalog = build_catalog();

    // WHEN an RTX 5090 enthusiast user selects the 32GB VRAM preset
    let max_vram = HardwarePlatform::Gpu32Gb.max_usable_memory_gb().unwrap();
    let mut desktop_models: Vec<&ModelBenchmark> = catalog
        .iter()
        .filter(|m| m.vram_rec_gb <= max_vram)
        .collect();

    // AND sorts by maximum intelligence score descending
    desktop_models.sort_by(|a, b| {
        let sa = a.intelligence().unwrap_or(0.0);
        let sb = b.intelligence().unwrap_or(0.0);
        sb.partial_cmp(&sa).unwrap()
    });

    // THEN DeepSeek R1 32B is ranked #1 with calibrated score 84.0 (0-100 baseline)
    let champion = desktop_models[0];
    assert_eq!(champion.name, "DeepSeek R1 Distill 32B");
    assert_eq!(champion.intelligence(), Some(84.0));
    assert!(champion.vram_rec_gb <= 32.0);
}

#[test]
fn bdd_scenario_time_travel_comparison() {
    // GIVEN an isolated SQLite repository
    let mut repo: Box<dyn IBenchmarkRepository> = Box::new(
        SqliteRepository::open(":memory:").expect("Failed to open test repository"),
    );

    let catalog_v1 = build_catalog();
    let snap1 = repo.save_snapshot(&catalog_v1, "Week 1 Snapshot").unwrap();

    // WHEN a new model appears in Week 2 (e.g. Qwen 3.8 27B)
    let mut catalog_v2 = catalog_v1;
    catalog_v2.push(ModelBenchmark {
        id: "qwen/qwen3.8-27b".into(),
        name: "Qwen 3.8 27B".into(),
        family: "Qwen".into(),
        param_count_b: Some(27.0),
        size_gb: 16.1,
        vram_rec_gb: 18.2,
        tokens_per_sec: Some(54.0),
        tps_hardware: Some("RTX 5090".into()),
        scores: BenchmarkScores { average: Some(44.8), ..Default::default() },
        downloads: 6000000,
        likes: 5400,
        ollama_tag: Some("qwen3.8:27b".into()),
        hf_url: "https://huggingface.co/test".into(),
        is_moe: false,
        context_k: 128,
        ..Default::default()
    });
    let snap2 = repo.save_snapshot(&catalog_v2, "Week 2 Snapshot").unwrap();

    // THEN the user can compare both snapshots accurately
    let v1_models = repo.load_snapshot_models(snap1).unwrap();
    let v2_models = repo.load_snapshot_models(snap2).unwrap();

    assert_eq!(v1_models.len(), 5);
    assert_eq!(v2_models.len(), 6);
    assert_eq!(v2_models.last().unwrap().name, "Qwen 3.8 27B");
}
