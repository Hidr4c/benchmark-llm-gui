use benchmark_llm_gui::domain::modality::ModelModality;
use benchmark_llm_gui::domain::model::{BenchmarkScores, ModelBenchmark};
use benchmark_llm_gui::infrastructure::fetcher::helpers::parse_model_identity;

#[test]
fn test_root_model_name_never_erases_media_or_unsized_models() {
    let cases = [
        ("lmarena/inkling2small", "Inkling 2 Small", "Thinky", ModelModality::VideoGen, None, "Inkling 2 Small"),
        ("ResembleAI/chatterbox", "chatterbox", "Chatterbox", ModelModality::AudioSpeech, None, "chatterbox"),
        ("unsloth/GLM-5.3-Flash-GGUF", "GLM-5.3-Flash", "GLM", ModelModality::Text, None, "GLM-5.3-Flash"),
        ("lmarena/mimov26pro", "MiMo-V2.6-Pro", "Xiaomi", ModelModality::Text, None, "MiMo-V2.6-Pro"),
        ("comfy-org/trellis2", "TRELLIS.2 (ComfyUI)", "Microsoft", ModelModality::ThreeDGen, None, "TRELLIS.2"),
        ("comfy-org/flux2dev", "FLUX.2 Dev (ComfyUI)", "Black Forest Labs", ModelModality::ImageGen, Some(12.0), "FLUX.2 Dev"),
    ];

    for (id, name, fam, modality, param_b, expected) in cases {
        let m = ModelBenchmark {
            id: id.into(),
            name: name.into(),
            family: fam.into(),
            param_count_b: param_b,
            modality,
            ..Default::default()
        };
        let root = m.root_model_name();
        assert_eq!(root, expected);
        assert!(!root.ends_with(" Model"), "Must never erase model identity into generic '{root}'");
    }
}

#[test]
fn test_parse_model_identity_recognizes_modern_families() {
    assert_eq!(parse_model_identity("unsloth/GLM-5.3-Flash-GGUF").1, "GLM");
    assert_eq!(parse_model_identity("xiaomi/MiMo-V2.6-Pro-GGUF").1, "MiMo");
    assert_eq!(parse_model_identity("moonshotai/Kimi-K2.5-Instruct").1, "Kimi");
    assert_eq!(parse_model_identity("MiniMax/MiniMax-H3-GGUF").1, "MiniMax");
    assert_eq!(parse_model_identity("tiel/Tiel-Coder-35B-A3B-GGUF").1, "Tiel");
    assert_eq!(parse_model_identity("ResembleAI/chatterbox").1, "Chatterbox");
}

#[test]
fn test_non_llm_media_scores_stay_clean_without_fake_llm_metrics() {
    let media_model = ModelBenchmark {
        id: "comfy-org/wan22".into(),
        name: "Wan 2.2 14B".into(),
        family: "Alibaba Wan".into(),
        modality: ModelModality::VideoGen,
        scores: BenchmarkScores { open_llm: Some(68.0), ..Default::default() },
        ..Default::default()
    };
    assert!(media_model.scores.coding.is_none());
    assert!(media_model.scores.agentic.is_none());
    assert!(media_model.scores.mmlu_pro.is_none());
    assert!(media_model.scores.real_world.is_none());
}

#[test]
fn test_modality_column_filtering_sorting_and_legacy_sanitize() {
    use benchmark_llm_gui::domain::model::{SortField, SortOrder};
    use benchmark_llm_gui::domain::table_preferences::{ColumnFilter, FilterOperator, TableColumnId, TablePreferences};
    use benchmark_llm_gui::presentation::filtering::{FilterService, FilterState};

    let mut prefs = TablePreferences::default();
    assert!(prefs.is_visible(TableColumnId::Modality));
    prefs.column_order.retain(|c| *c != TableColumnId::Modality);
    prefs.sanitize();
    let model_idx = prefs.column_order.iter().position(|c| *c == TableColumnId::Model).unwrap();
    assert_eq!(prefs.column_order[model_idx + 1], TableColumnId::Modality);

    let text_m = ModelBenchmark { id: "a".into(), name: "TextM".into(), modality: ModelModality::Text, scores: BenchmarkScores { open_llm: Some(70.0), ..Default::default() }, ..Default::default() };
    let video_m = ModelBenchmark { id: "b".into(), name: "VideoM".into(), modality: ModelModality::VideoGen, scores: BenchmarkScores { open_llm: Some(72.0), ..Default::default() }, ..Default::default() };

    prefs.column_filters.insert(TableColumnId::Modality, ColumnFilter::new(FilterOperator::Contains, "VIDEO"));
    assert!(prefs.matches_model(&video_m, "#1"));
    assert!(!prefs.matches_model(&text_m, "#2"));

    let mut filters = FilterState::default();
    filters.sort_field = SortField::Modality;
    filters.sort_order = SortOrder::Ascending;
    let sorted = FilterService::compute_filtered_indices(&[video_m, text_m], &filters);
    assert_eq!(sorted, vec![1, 0]); // "TEXT" < "VIDEO" ascending
}

#[test]
fn test_zero_hardcode_dynamic_family_detection_and_filter_extraction() {
    use benchmark_llm_gui::infrastructure::fetcher::helpers::{estimate_scores, extract_dynamic_family, extract_release_year, generate_ollama_tag};
    use benchmark_llm_gui::presentation::filtering::{FilterService, FilterState};

    assert_eq!(parse_model_identity("FutureLab/HyperNova-4.2-14B-Instruct-GGUF").1, "HyperNova");
    assert_eq!(parse_model_identity("acme/synthvox-2.1").1, "Synthvox");
    assert_eq!(parse_model_identity("huihui-ai/Huihui-Aether-12B-GGUF").1, "Aether");
    assert_eq!(parse_model_identity("bartowski/gguf-Q8_0-SmolLM2-135M").1, "SmolLM");
    assert_eq!(parse_model_identity("peculiar-ragdoll/Cyber-Tiel-Coder-35B-A3B-GGUF-MTP").1, "Tiel");
    assert_eq!(parse_model_identity("bottlecapai/ThinkingCap-Qwen3.8-27B-GGUF").1, "Qwen");
    assert_eq!(parse_model_identity("aion-labs/aion-rp-llama-3.1-8b").1, "Aion");
    assert_eq!(extract_dynamic_family("", "Muse-Glimmer-30B"), "Muse");
    assert_eq!(extract_dynamic_family("", "Z-Image-Turbo"), "Z-Image");
    assert_eq!(extract_dynamic_family("", "F5-TTS 1.0"), "F5-TTS");

    let est = estimate_scores("Spam-Coder-Opus5.5-27B", Some(27.0));
    assert!(est.coding.is_none() && est.agentic.is_none(), "Heuristic fallback must never fabricate coding/agentic scores");

    assert_eq!(extract_release_year("org/Nova-2603-7B", "Nova 2603", Some("2025-11-01")), Some(2026));
    assert_eq!(extract_release_year("org/OldModel-7B", "OldModel", Some("2023-06-15")), Some(2023));
    assert_eq!(generate_ollama_tag("HyperNova", Some(14.0)), Some("hypernova:14b".to_string()));
    assert_eq!(generate_ollama_tag("Other", Some(7.0)), None);

    let m1 = ModelBenchmark { id: "a".into(), name: "FLUX.2 Dev".into(), family: "FLUX".into(), scores: BenchmarkScores { open_llm: Some(70.0), ..Default::default() }, ..Default::default() };
    let m2 = ModelBenchmark { id: "b".into(), name: "Flux.1 Inpaint".into(), family: "Flux".into(), scores: BenchmarkScores { open_llm: Some(68.0), ..Default::default() }, ..Default::default() };
    let m3 = ModelBenchmark { id: "c".into(), name: "HyperNova 4.2 14B".into(), family: "HyperNova".into(), param_count_b: Some(14.0), modality: ModelModality::Text, scores: BenchmarkScores { open_llm: Some(75.0), ..Default::default() }, ..Default::default() };
    let m4 = ModelBenchmark { id: "d".into(), name: "MiscModel".into(), family: "Other".into(), ..Default::default() };

    assert_eq!(m3.root_model_name(), "HyperNova 4.2 14B");

    let catalog = vec![m1, m2, m3, m4];
    let fams = FilterService::extract_unique_families(&catalog);
    assert_eq!(fams, vec!["FLUX", "HyperNova", "Other"]);

    let mut filters = FilterState::default();
    filters.selected_family = "flux".to_string();
    let matched = FilterService::compute_filtered_indices(&catalog, &filters);
    assert_eq!(matched.len(), 2);
}

#[test]
fn test_video_image_audio_dedicated_benchmarks_no_open_llm_pollution() {
    use benchmark_llm_gui::domain::model::SourceSelection;
    use benchmark_llm_gui::domain::scoring::ScoreConfidence;
    use benchmark_llm_gui::domain::table_column::TableColumnId;
    use benchmark_llm_gui::domain::traits::IBenchmarkRepository;
    use benchmark_llm_gui::infrastructure::repository::sqlite_repo::SqliteRepository;

    let vid = ModelBenchmark {
        id: "Wan-AI/Wan2.2".into(),
        name: "Wan 2.2".into(),
        family: "Wan".into(),
        modality: ModelModality::VideoGen,
        scores: BenchmarkScores { video_bench: Some(78.5), average: Some(78.5), ..Default::default() },
        ..Default::default()
    };
    let img = ModelBenchmark {
        id: "black-forest-labs/FLUX.2".into(),
        name: "FLUX.2 Dev".into(),
        family: "FLUX".into(),
        modality: ModelModality::ImageGen,
        scores: BenchmarkScores { image_bench: Some(81.0), average: Some(81.0), ..Default::default() },
        ..Default::default()
    };
    let aud = ModelBenchmark {
        id: "Qwen/Qwen3-TTS".into(),
        name: "Qwen3-TTS".into(),
        family: "Qwen".into(),
        modality: ModelModality::AudioSpeech,
        scores: BenchmarkScores { audio_bench: Some(74.2), average: Some(74.2), ..Default::default() },
        ..Default::default()
    };

    assert_eq!(TableColumnId::OpenLlm.extract_value(&vid, "#1"), "-");
    assert_eq!(TableColumnId::VideoBench.extract_value(&vid, "#1"), "78.5");
    assert_eq!(TableColumnId::ImageBench.extract_value(&img, "#2"), "81.0");
    assert_eq!(TableColumnId::AudioBench.extract_value(&aud, "#3"), "74.2");

    let sel = SourceSelection::default();
    let bd = vid.score_breakdown(&sel).expect("Video model must compute score from video_bench");
    assert_eq!(bd.confidence, ScoreConfidence::Medium);
    assert!(bd.formula.contains("Median(78.5)"));

    let mut repo = SqliteRepository::open(":memory:").expect("in-memory db");
    let snap_id = repo.save_snapshot(&[vid, img, aud], "media-bench").expect("save");
    let loaded = repo.load_snapshot_models(snap_id).expect("load");
    assert!(loaded[0].scores.open_llm.is_none());
    assert_eq!(loaded[0].scores.video_bench, Some(78.5));
    assert_eq!(loaded[1].scores.image_bench, Some(81.0));
    assert_eq!(loaded[2].scores.audio_bench, Some(74.2));
}
