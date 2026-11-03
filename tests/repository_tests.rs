use benchmark_llm_gui::domain::model::{BenchmarkScores, ModelBenchmark};
use benchmark_llm_gui::domain::traits::IBenchmarkRepository;
use benchmark_llm_gui::infrastructure::repository::sqlite_repo::SqliteRepository;

fn create_sample_model(id: &str, name: &str, size_gb: f32, score: f32) -> ModelBenchmark {
    ModelBenchmark {
        id: id.into(),
        name: name.into(),
        family: "Test".into(),
        param_count_b: Some(size_gb / 0.58),
        size_gb,
        vram_rec_gb: ModelBenchmark::calculate_vram(size_gb),
        tokens_per_sec: Some(50.0),
        tps_hardware: Some("Test Device".into()),
        scores: BenchmarkScores {
            average: Some(score),
            ..Default::default()
        },
        downloads: 1000,
        likes: 50,
        ollama_tag: Some(format!("test:{}", name.to_lowercase())),
        hf_url: format!("https://huggingface.co/{id}"),
        is_moe: false,
        context_k: 16,
        ..Default::default()
    }
}

#[test]
fn test_repository_ioc_snapshot_lifecycle() {
    // Arrange: Create an isolated in-memory repository through the IBenchmarkRepository trait (IoC)
    let mut repo: Box<dyn IBenchmarkRepository> = Box::new(
        SqliteRepository::open(":memory:").expect("Failed to initialize in-memory SQLite"),
    );

    assert!(repo.list_snapshots().unwrap().is_empty());
    assert!(repo.load_latest_snapshot().unwrap().is_none());

    // Act 1: Save first snapshot (October 2026 data)
    let models_oct = vec![
        create_sample_model("vendor/edge-300m", "Edge-300M", 0.25, 14.0),
        create_sample_model("vendor/mid-8b", "Mid-8B", 4.8, 32.0),
    ];
    let snap1_id = repo.save_snapshot(&models_oct, "Sync Octobre 2026").expect("Failed to save snapshot 1");
    assert_eq!(snap1_id, 1);

    // Assert 1: Snapshots list contains 1 item
    let snapshots = repo.list_snapshots().expect("Failed to list snapshots");
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].model_count, 2);

    // Act 2: Save second snapshot (November 2026 data with new models)
    let models_nov = vec![
        create_sample_model("vendor/edge-300m", "Edge-300M", 0.25, 14.5), // Score improved
        create_sample_model("vendor/mid-8b", "Mid-8B", 4.8, 32.5),
        create_sample_model("vendor/flagship-32b", "Flagship-32B", 19.5, 46.0), // New 32B model
    ];
    let snap2_id = repo.save_snapshot(&models_nov, "Sync November 2026").expect("Failed to save snapshot 2");
    assert_eq!(snap2_id, 2);

    // Assert 2: Snapshots list now has 2 entries, ordered newest first
    let snapshots = repo.list_snapshots().unwrap();
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0].id, 2);
    assert_eq!(snapshots[0].model_count, 3);
    assert_eq!(snapshots[1].id, 1);
    assert_eq!(snapshots[1].model_count, 2);

    // Act 3: Historical Time Travel - Verify models from Snapshot 1 remain immutable
    let historical_models = repo.load_snapshot_models(snap1_id).expect("Failed to load historical snapshot 1");
    assert_eq!(historical_models.len(), 2);
    assert_eq!(historical_models[0].name, "Edge-300M");
    assert_eq!(historical_models[0].scores.average, Some(14.0)); // Raw original score preserved!
    assert_eq!(historical_models[0].intelligence(), Some(11.7)); // Calibrated score with source coverage confidence

    // Act 4: Verify latest snapshot
    let (latest_meta, latest_models) = repo.load_latest_snapshot().unwrap().unwrap();
    assert_eq!(latest_meta.id, 2);
    assert_eq!(latest_models.len(), 3);
    assert_eq!(latest_models[2].name, "Flagship-32B");
}
