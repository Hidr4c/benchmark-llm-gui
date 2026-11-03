use benchmark_llm_gui::domain::model::{BenchmarkScores, ModelBenchmark};
use benchmark_llm_gui::domain::specialty::DomainSpecialty;
use benchmark_llm_gui::domain::traits::IBenchmarkRepository;
use benchmark_llm_gui::infrastructure::repository::sqlite_repo::SqliteRepository;
use benchmark_llm_gui::presentation::filtering::{FilterService, FilterState, SpecialtyFilter};

#[test]
fn test_domain_specialty_detection() {
    // 1. Carbon-X (DNA / Genomics)
    let spec_carbon = DomainSpecialty::detect(
        "bio/carbon-x-14b-dna",
        "Carbon-X 14B (DNA Foundation)",
        None,
        &["dna".into(), "genomics".into()],
        "Local foundation model for DNA sequence modeling",
    );
    assert_eq!(spec_carbon, DomainSpecialty::GenomicsBio);

    // 2. Fibo 2.0 (Visual Editing)
    let spec_fibo = DomainSpecialty::detect(
        "creative/fibo-2-visual-editor",
        "Fibo 2.0",
        Some("image-to-image"),
        &["inpainting".into()],
        "Natural language photo editing model",
    );
    assert_eq!(spec_fibo, DomainSpecialty::VisionCreative);

    // 3. Sky 001 (3D World Model)
    let spec_sky = DomainSpecialty::detect(
        "spatial/sky-001-world-cad",
        "Sky 001 (Physical World Model)",
        None,
        &["3d".into(), "cad".into()],
        "Generates physical 3D objects and drone parts",
    );
    assert_eq!(spec_sky, DomainSpecialty::WorldModel3D);

    // 4. Dessa Voice (Speech-to-Speech)
    let spec_dessa = DomainSpecialty::detect(
        "audio/dessa-voice-s2s",
        "Dessa Voice S2S",
        Some("audio-to-audio"),
        &["voice-conversion".into()],
        "Hyper-realistic human voice cloning",
    );
    assert_eq!(spec_dessa, DomainSpecialty::VoiceSpeech);

    // 5. Multi-Agent Swarm
    let spec_swarm = DomainSpecialty::detect(
        "agentic/swarm-2000-bioresearch",
        "CRISPR Multi-Agent Swarm (2000 Agents)",
        None,
        &["multi-agent".into(), "swarm".into()],
        "Autonomous multi-agent discovery team",
    );
    assert_eq!(spec_swarm, DomainSpecialty::MultiAgent);

    // 6. Coder
    let spec_coder = DomainSpecialty::detect(
        "qwen/qwen3-coder-30b",
        "Qwen3-Coder-30B",
        None,
        &["code".into()],
        "Deep software engineering model",
    );
    assert_eq!(spec_coder, DomainSpecialty::SoftwareDev);

    // 7. General LLM
    let spec_gen = DomainSpecialty::detect(
        "meta-llama/llama-3.1-8b",
        "Llama 3.1 8B Instruct",
        None,
        &[],
        "General purpose conversational assistant",
    );
    assert_eq!(spec_gen, DomainSpecialty::General);
}

#[test]
fn test_specialty_filtering_in_service() {
    let mut m1 = ModelBenchmark::default();
    m1.id = "m1".into();
    m1.name = "Carbon-X 14B".into();
    m1.specialty = DomainSpecialty::GenomicsBio;

    let mut m2 = ModelBenchmark::default();
    m2.id = "m2".into();
    m2.name = "Sky 001 3D".into();
    m2.specialty = DomainSpecialty::WorldModel3D;

    let mut m3 = ModelBenchmark::default();
    m3.id = "m3".into();
    m3.name = "Qwen 3.8 General".into();
    m3.specialty = DomainSpecialty::General;

    let models = vec![m1, m2, m3];

    // Filter by DNA / Genomics
    let mut filters_dna = FilterState::default();
    filters_dna.specialty = SpecialtyFilter::Specific(DomainSpecialty::GenomicsBio);
    let indices_dna = FilterService::compute_filtered_indices(&models, &filters_dna);
    assert_eq!(indices_dna.len(), 1);
    assert_eq!(models[indices_dna[0]].id, "m1");

    // Filter by 3D World Model
    let mut filters_3d = FilterState::default();
    filters_3d.specialty = SpecialtyFilter::Specific(DomainSpecialty::WorldModel3D);
    let indices_3d = FilterService::compute_filtered_indices(&models, &filters_3d);
    assert_eq!(indices_3d.len(), 1);
    assert_eq!(models[indices_3d[0]].id, "m2");

    // Filter All
    let filters_all = FilterState::default();
    let indices_all = FilterService::compute_filtered_indices(&models, &filters_all);
    assert_eq!(indices_all.len(), 3);
}

#[test]
fn test_sqlite_specialty_persistence_roundtrip() {
    let mut repo = SqliteRepository::open(":memory:").expect("in-memory DB init failed");

    let mut model_dna = ModelBenchmark::default();
    model_dna.id = "carbon-x".into();
    model_dna.name = "Carbon-X 14B".into();
    model_dna.specialty = DomainSpecialty::GenomicsBio;
    model_dna.scores = BenchmarkScores {
        average: Some(92.0),
        ..Default::default()
    };

    let mut model_voice = ModelBenchmark::default();
    model_voice.id = "dessa-voice".into();
    model_voice.name = "Dessa Voice S2S".into();
    model_voice.specialty = DomainSpecialty::VoiceSpeech;

    let models = vec![model_dna, model_voice];
    let snap_id = repo.save_snapshot(&models, "Test Specialty Snapshot").expect("save snapshot failed");

    let loaded = repo.load_snapshot_models(snap_id).expect("load snapshot failed");
    assert_eq!(loaded.len(), 2);

    let loaded_dna = loaded.iter().find(|m| m.id == "carbon-x").expect("dna model missing");
    assert_eq!(loaded_dna.specialty, DomainSpecialty::GenomicsBio);

    let loaded_voice = loaded.iter().find(|m| m.id == "dessa-voice").expect("voice model missing");
    assert_eq!(loaded_voice.specialty, DomainSpecialty::VoiceSpeech);
}

#[test]
fn test_badge_colors_and_labels_integrity() {
    for spec in DomainSpecialty::all() {
        assert!(!spec.label_fr().is_empty());
        assert!(!spec.label_en().is_empty());
        assert!(!spec.badge_text().is_empty());
        let (r, g, b) = spec.badge_color_rgb();
        assert!(r > 0 || g > 0 || b > 0);
        // Ensure no model names or noisy acronyms in badges
        assert!(!spec.badge_text().contains("Fibo"));
        assert!(!spec.badge_text().contains("S2S"));
        assert!(!spec.badge_text().contains('/'));
    }
    assert_eq!(DomainSpecialty::VisionCreative.badge_text(), "Vision");
    assert_eq!(DomainSpecialty::VoiceSpeech.badge_text(), "Voix");
    assert_eq!(DomainSpecialty::GenomicsBio.badge_text(), "ADN");
    assert_eq!(DomainSpecialty::WorldModel3D.badge_text(), "3D");
    assert_eq!(DomainSpecialty::SoftwareDev.badge_text(), "Code");
    assert_eq!(DomainSpecialty::MultiAgent.badge_text(), "Agents");
}

#[test]
fn test_sqlite_backward_compatibility_fallback_detection() {
    let mut repo = SqliteRepository::open(":memory:").expect("in-memory DB init failed");

    let mut model_coder = ModelBenchmark::default();
    model_coder.id = "qwen/qwen3-coder-30b".into();
    model_coder.name = "Qwen3-Coder-30B".into();
    model_coder.specialty = DomainSpecialty::General;

    let snap_id = repo.save_snapshot(&[model_coder], "Legacy Snapshot").expect("save snapshot failed");

    let loaded = repo.load_snapshot_models(snap_id).expect("load snapshot failed");
    assert_eq!(loaded[0].specialty, DomainSpecialty::SoftwareDev);
}

#[test]
fn test_score_and_modality_based_domain_detection() {
    use benchmark_llm_gui::domain::modality::ModelModality;

    // 1. Model without code keywords, but with dominant SWE-Bench score
    let scores_code = BenchmarkScores {
        coding: Some(84.5),
        open_llm: Some(52.0),
        ..Default::default()
    };
    let spec_code = DomainSpecialty::detect_with_context(
        "author/custom-eval-v1",
        "Custom-Eval-V1",
        None,
        &[],
        "Unlabeled community model checkpoint",
        Some(&scores_code),
        ModelModality::Text,
    );
    assert_eq!(spec_code, DomainSpecialty::SoftwareDev);

    // 2. Model without agent keywords, but with dominant BFCL tool score
    let scores_agent = BenchmarkScores {
        agentic: Some(82.0),
        open_llm: Some(48.0),
        ..Default::default()
    };
    let spec_agent = DomainSpecialty::detect_with_context(
        "author/pipeline-runner-v2",
        "Pipeline-Runner-V2",
        None,
        &[],
        "Unlabeled model",
        Some(&scores_agent),
        ModelModality::Text,
    );
    assert_eq!(spec_agent, DomainSpecialty::MultiAgent);

    // 3. Model without vision keywords, but with Multimodal modality
    let spec_vlm = DomainSpecialty::detect_with_context(
        "author/checkpoint-vlm",
        "Checkpoint-VLM",
        None,
        &[],
        "Unlabeled model",
        None,
        ModelModality::Multimodal,
    );
    assert_eq!(spec_vlm, DomainSpecialty::VisionCreative);

    // 4. Model without voice keywords, but with AudioSpeech modality
    let spec_audio = DomainSpecialty::detect_with_context(
        "author/checkpoint-tts",
        "Checkpoint-TTS",
        None,
        &[],
        "Unlabeled model",
        None,
        ModelModality::AudioSpeech,
    );
    assert_eq!(spec_audio, DomainSpecialty::VoiceSpeech);
}
