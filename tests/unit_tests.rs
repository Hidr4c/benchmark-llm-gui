use benchmark_llm_gui::domain::hardware::HardwarePlatform;
use benchmark_llm_gui::domain::model::{BenchmarkScores, ModelBenchmark};
use benchmark_llm_gui::infrastructure::fetcher::helpers::{estimate_gguf_size, parse_model_identity};

#[test]
fn test_parse_model_identity_edge_and_desktop() {
    // Ultra-light Edge models
    let (name, family, params) = parse_model_identity("HuggingFaceTB/SmolLM2-360M-Instruct-GGUF");
    assert_eq!(family, "SmolLM");
    assert_eq!(params, Some(0.36));
    assert!(name.contains("SmolLM2"));

    let (_, family, params) = parse_model_identity("Qwen/Qwen2.5-0.5B-Instruct-GGUF");
    assert_eq!(family, "Qwen");
    assert_eq!(params, Some(0.5));

    // Mid-range Desktop GPU models
    let (_, family, params) = parse_model_identity("bartowski/Meta-Llama-3.1-8B-Instruct-GGUF");
    assert_eq!(family, "Llama");
    assert_eq!(params, Some(8.0));

    // High-End RTX 5090 models
    let (_, family, params) = parse_model_identity("deepseek-ai/DeepSeek-R1-Distill-Qwen-32B-GGUF");
    assert_eq!(family, "DeepSeek");
    assert_eq!(params, Some(32.0));

    let (_, family, params) = parse_model_identity("unsloth/Qwen3.8-27B-GGUF");
    assert_eq!(family, "Qwen");
    assert_eq!(params, Some(27.0));
}

#[test]
fn test_embedded_size_and_efficiency_ratio() {
    // SmolLM2 360M
    let size_360m = estimate_gguf_size(Some(0.36), "smollm2-360m");
    assert!(size_360m <= 0.35, "Expected size <= 0.35 GB for 360M model, got {}", size_360m);

    let model_360m = ModelBenchmark {
        id: "HuggingFaceTB/SmolLM2-360M-Instruct-GGUF".into(),
        name: "SmolLM2 360M Instruct".into(),
        family: "SmolLM".into(),
        param_count_b: Some(0.36),
        size_gb: size_360m,
        vram_rec_gb: ModelBenchmark::calculate_vram(size_360m),
        tokens_per_sec: Some(280.0),
        tps_hardware: Some("Raspberry Pi 5".into()),
        scores: BenchmarkScores {
            average: Some(15.2),
            ..Default::default()
        },
        downloads: 500000,
        likes: 1200,
        ollama_tag: Some("smollm2:360m".into()),
        hf_url: "https://huggingface.co/test".into(),
        is_moe: false,
        context_k: 8,
        ..Default::default()
    };

    assert!(model_360m.is_embedded_friendly());
    assert_eq!(model_360m.embedded_recommendation(), "Micro-IoT / ESP32-P4 / 1 GB SBC (Ultra-Light)");

    // Efficiency ratio: 15.2 / 0.22  69 pts/GB!
    let efficiency = model_360m.efficiency_ratio();
    assert!(efficiency >= 35.0, "Expected high efficiency ratio for ultra-light model, got {}", efficiency);
}

#[test]
fn test_hardware_platform_memory_thresholds() {
    // Micro-IoT limit <= 0.8 GB
    let iot = HardwarePlatform::MicroIoT;
    assert!(iot.is_embedded());
    assert_eq!(iot.max_usable_memory_gb(), Some(0.8));

    // Raspberry Pi 4 limit <= 2.5 GB
    let pi4 = HardwarePlatform::RaspberryPi4;
    assert!(pi4.is_embedded());
    assert_eq!(pi4.max_usable_memory_gb(), Some(2.5));

    // RTX 5090 limit <= 32.5 GB
    let rtx5090 = HardwarePlatform::Gpu32Gb;
    assert!(!rtx5090.is_embedded());
    assert_eq!(rtx5090.max_usable_memory_gb(), Some(32.5));
}

#[test]
fn test_robust_median_and_outlier_rejection() {
    use benchmark_llm_gui::domain::model::SourceSelection;
    use benchmark_llm_gui::domain::scoring::ScoringEngine;

    // Model with 3 scores: 85 (Code), 84 (MMLU), and 20 (corrupted Agent score)
    let corrupted_scores = BenchmarkScores {
        coding: Some(85.0),
        mmlu_pro: Some(84.0),
        agentic: Some(20.0),
        ..Default::default()
    };
    let all_sel = SourceSelection { open_llm: true, coding: true, agentic: true, real_world: false };
    let breakdown = ScoringEngine::aggregate(&corrupted_scores, &all_sel, false, 1.0, 1.0).unwrap();

    // A simple average would yield 63.0. Our robust median yields 84.0, rejecting the outlier!
    assert!(breakdown.final_score >= 84.0, "Expected robust score >= 84.0, got {}", breakdown.final_score);
    assert_eq!(breakdown.variance, 65.0);
}

#[test]
fn test_source_coverage_and_quantization() {
    use benchmark_llm_gui::domain::model::SourceSelection;
    use benchmark_llm_gui::domain::scoring::{ScoreConfidence, ScoringEngine};

    // Small 8B model with verified coding: receives 100% full merit
    let verified_8b = BenchmarkScores {
        coding: Some(82.0),
        mmlu_pro: Some(80.0),
        agentic: Some(81.0),
        ..Default::default()
    };
    let sel = SourceSelection { open_llm: true, coding: true, agentic: true, real_world: false };
    let breakdown = ScoringEngine::aggregate(&verified_8b, &sel, false, 1.0, 1.0).unwrap();

    // No hardcoded parameter ceiling: gets full merit of verified practical capabilities
    assert_eq!(breakdown.confidence, ScoreConfidence::High);
    assert!(breakdown.final_score >= 80.0, "Verified 8B model score {} should achieve >= 80.0", breakdown.final_score);

    // Unverified model with isolated academic multiple-choice: confidence penalty applies
    let unverified_academic = BenchmarkScores {
        mmlu_pro: Some(85.0),
        ..Default::default()
    };
    let sel_acad = SourceSelection { open_llm: true, coding: false, agentic: false, real_world: false };
    let unverified_bd = ScoringEngine::aggregate(&unverified_academic, &sel_acad, false, 1.0, 1.0).unwrap();
    assert_eq!(unverified_bd.confidence, ScoreConfidence::Low);
    assert!(unverified_bd.final_score < 80.0, "Unverified academic-only model should receive coverage discount, got {}", unverified_bd.final_score);

    // Quantization degradation: Q4 vs FP16
    let (factor_q4, tag_q4) = ScoringEngine::extract_quant_factor("Bartowski/model-Q4_K_M.gguf");
    let (factor_fp16, tag_fp16) = ScoringEngine::extract_quant_factor("Bartowski/model-fp16.bin");
    assert_eq!(factor_q4, 0.93);
    assert!(tag_q4.contains("Q4_K_M"));
    assert_eq!(factor_fp16, 1.0);
    assert!(tag_fp16.contains("FP16"));
}

#[test]
fn test_temporal_size_inversion_check() {
    use benchmark_llm_gui::domain::consistency::ModelConsistencyValidator;

    // Model A: 2024, 8B, high score 80.0
    let model_a = ModelBenchmark {
        release_year: Some(2024),
        param_count_b: Some(8.0),
        ..Default::default()
    };
    // Model B: 2026, 35B, lower score 72.0
    let model_b = ModelBenchmark {
        release_year: Some(2026),
        param_count_b: Some(35.0),
        ..Default::default()
    };

    // Flagged as temporal anomaly: older & smaller model outscoring newer larger model by > 5 pts
    assert!(ModelConsistencyValidator::check_temporal_size_inversion(&model_a, 80.0, &model_b, 72.0));
    // Normal case: newer larger model has higher score -> no anomaly
    assert!(!ModelConsistencyValidator::check_temporal_size_inversion(&model_b, 85.0, &model_a, 70.0));
}

#[test]
fn test_model_modality_detection() {
    use benchmark_llm_gui::domain::modality::ModelModality;

    // Text LLMs
    assert_eq!(ModelModality::detect("Qwen/Qwen2.5-7B-Instruct-GGUF", "Qwen 2.5 7B", None, &[]), ModelModality::Text);
    assert_eq!(ModelModality::detect("meta-llama/Llama-3.3-70B", "Llama 3.3 70B", None, &[]), ModelModality::Text);

    // Multimodal / Vision
    assert_eq!(ModelModality::detect("Qwen/Qwen2.5-VL-7B-Instruct-GGUF", "Qwen 2.5 VL 7B", None, &[]), ModelModality::Multimodal);
    assert_eq!(ModelModality::detect("google/gemini-2.0-flash", "Gemini 2.0 Flash", None, &[]), ModelModality::Multimodal);
    assert_eq!(ModelModality::detect("mistralai/Pixtral-12B-2409", "Pixtral 12B", None, &[]), ModelModality::Multimodal);

    // Image Generation
    assert_eq!(ModelModality::detect("black-forest-labs/FLUX.1-dev", "FLUX.1 dev", None, &[]), ModelModality::ImageGen);
    assert_eq!(ModelModality::detect("stabilityai/stable-diffusion-xl-base-1.0", "SDXL Base", None, &[]), ModelModality::ImageGen);
    assert_eq!(ModelModality::detect("community/banana-image-gen-v1", "Banana Image Generator", None, &[]), ModelModality::ImageGen);

    // Video Generation
    assert_eq!(ModelModality::detect("kuaishou/kling-video-v1.5", "Kling Video v1.5", None, &[]), ModelModality::VideoGen);
    assert_eq!(ModelModality::detect("THUDM/CogVideoX-5b", "CogVideoX 5B", None, &[]), ModelModality::VideoGen);

    // Audio & Speech
    assert_eq!(ModelModality::detect("openai/whisper-large-v3", "Whisper Large v3", None, &[]), ModelModality::AudioSpeech);
}

#[test]
fn test_cloud_root_model_grouping() {
    use benchmark_llm_gui::presentation::filtering::FilterService;
    use benchmark_llm_gui::presentation::components::filter_bar::FilterState;

    let default_filters = FilterState::default();
    assert!(default_filters.group_by_root, "Grouping by root model must be activated by default");

    let m1 = ModelBenchmark {
        id: "openai/gpt-6.1-sol".into(),
        name: "GPT 6.1 Sol".into(),
        family: "OpenAI".into(),
        is_cloud: true,
        ..Default::default()
    };
    let m2 = ModelBenchmark {
        id: "openai/gpt-6.1-sol-pro".into(),
        name: "GPT 6.1 Sol Pro".into(),
        family: "OpenAI".into(),
        is_cloud: true,
        ..Default::default()
    };

    assert_eq!(m1.root_model_name(), "GPT 6.1 Sol");
    assert_eq!(m2.root_model_name(), "GPT 6.1 Sol");

    let models = vec![m1, m2];
    let groups = FilterService::group_into_roots(&models, &[0, 1]);

    assert_eq!(groups.len(), 1, "Expected both Sol and Sol Pro to group into 1 root family");
    assert_eq!(groups[0].root_name, "GPT 6.1 Sol");
    assert_eq!(groups[0].variant_indices.len(), 2);
}

#[test]
fn test_model_consistency_anomaly_detection() {
    use benchmark_llm_gui::domain::consistency::{ConsistencyIssue, ModelConsistencyValidator};

    // 1. Model with extreme divergence (SWE-Bench 76% vs MMLU-Pro 40%) -> should flag warning
    let diverged_model = ModelBenchmark {
        id: "anomaly/diverged-model".into(),
        name: "Diverged Model 35B".into(),
        scores: BenchmarkScores {
            mmlu_pro: Some(40.2),
            coding: Some(75.9),
            ..Default::default()
        },
        ..Default::default()
    };
    let issues = ModelConsistencyValidator::validate(&diverged_model);
    assert!(issues.contains(&ConsistencyIssue::ScoreDivergence));
    assert!(issues.contains(&ConsistencyIssue::HighVarianceAcrossSources));

    // 2. Realistic verified model (Qwen 3.6 35B: 85.2% MMLU-Pro & 75.9% SWE-Bench) -> clean!
    let coherent_model = ModelBenchmark {
        id: "qwen/qwen3.6-35b".into(),
        name: "Qwen 3.6 35B".into(),
        param_count_b: Some(35.0),
        size_gb: 19.5,
        release_year: Some(2026),
        tokens_per_sec: Some(138.0),
        tps_hardware: Some("RTX 5090 (NVFP4)".into()),
        scores: BenchmarkScores {
            mmlu_pro: Some(85.2),
            coding: Some(75.9),
            ..Default::default()
        },
        ..Default::default()
    };
    let issues_clean = ModelConsistencyValidator::validate(&coherent_model);
    assert!(issues_clean.is_empty(), "Verified flagship model should pass consistency check without warnings");
}

#[test]
fn test_economic_scores_composite_and_metrics() {
    use benchmark_llm_gui::domain::economic::EconomicScores;

    let econ = EconomicScores {
        rli_automation_rate: Some(12.5), // 12.5 / 25.0 * 100 = 50.0
        swe_lancer_score: Some(38.0),
        swe_lancer_dollars: Some(380000.0),
        swe_clar_impl: Some(60.0),
        swe_clar_clarif: Some(80.0), // 60 * 0.6 + 80 * 0.4 = 68.0
        os_world_success: Some(19.0), // 19 / 38 * 100 = 50.0
        web_arena_success: Some(65.0),
        mle_bench_score: Some(55.0),
        mle_bench_medals: Some("Low: 75% | Med: 50% | High: 40%".into()),
    };

    assert_eq!(econ.populated_count(), 6);
    let comp = econ.composite_score().unwrap();
    // (50.0 + 38.0 + 68.0 + 50.0 + 65.0 + 55.0) / 6 = 326.0 / 6 = 54.33 -> 54.3
    assert!((comp - 54.3).abs() < 0.2, "Expected composite ~54.3, got {}", comp);
}

#[test]
fn test_table_column_mode_toggle() {
    use benchmark_llm_gui::presentation::filtering::{ConfidenceFilter, FilterState, TableColumnMode};
    let mut state = FilterState::default();
    assert_eq!(state.column_mode, TableColumnMode::GlobalEvaluation);
    assert!(!state.show_advanced_columns);
    assert!(!state.show_raw_sources);
    assert_eq!(state.confidence_level, ConfidenceFilter::All);

    state.show_advanced_columns = true;
    state.show_raw_sources = true;
    state.confidence_level = ConfidenceFilter::HighOnly;
    assert!(state.show_advanced_columns);
    assert!(state.show_raw_sources);
    assert_eq!(state.confidence_level, ConfidenceFilter::HighOnly);
}
