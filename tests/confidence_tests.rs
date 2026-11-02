use benchmark_llm_gui::domain::model::{BenchmarkScores, ModelBenchmark, SourceSelection};
use benchmark_llm_gui::domain::scoring::ScoreConfidence;
use benchmark_llm_gui::presentation::i18n::Language;

#[test]
fn test_score_confidence_badges_and_semantics() {
    // 1. High confidence
    let high = ScoreConfidence::High;
    assert_eq!(high.badge(), "[OK]");
    assert_eq!(high.color_rgb(), (46, 204, 113)); // Vibrant Green

    // 2. Medium confidence
    let med = ScoreConfidence::Medium;
    assert_eq!(med.badge(), "[~]");
    assert_eq!(med.color_rgb(), (241, 196, 15)); // Amber Gold

    // 3. Low confidence
    let low = ScoreConfidence::Low;
    assert_eq!(low.badge(), "[!]");
    assert_eq!(low.color_rgb(), (231, 76, 60)); // Vibrant Red

    // 4. i18n parity
    let fr = Language::French.confidence();
    let en = Language::English.confidence();
    assert!(fr.ok_label.contains("Élevé"));
    assert!(en.ok_label.contains("High"));
    assert!(fr.col_tooltip.contains("[OK] Vert"));
    assert!(en.col_tooltip.contains("[OK] Green"));
}

#[test]
fn test_scoring_engine_assigns_confidence_accurately() {
    let sources = SourceSelection::default();

    // High confidence: 3+ concordant scores with low variance
    let solid_model = ModelBenchmark {
        id: "solid/model".into(),
        name: "Solid Model".into(),
        scores: BenchmarkScores {
            average: Some(72.0),
            open_llm: Some(71.5),
            coding: Some(73.0),
            agentic: Some(72.5),
            mmlu_pro: Some(71.0),
            real_world: Some(72.0),
            ..Default::default()
        },
        ..Default::default()
    };
    let bd = solid_model.score_breakdown(&sources).expect("Breakdown should exist");
    assert_eq!(bd.confidence, ScoreConfidence::High, "Expected High confidence for 3+ concordant sources");
    assert!(bd.variance < 16.0);

    // Low confidence: high divergence across benchmarks (> 25 pts)
    let diverged_model = ModelBenchmark {
        id: "diverged/model".into(),
        name: "Diverged Model".into(),
        scores: BenchmarkScores {
            coding: Some(95.0),
            mmlu_pro: Some(40.0),
            ..Default::default()
        },
        ..Default::default()
    };
    let bd_div = diverged_model.score_breakdown(&sources).expect("Breakdown should exist");
    assert_eq!(bd_div.confidence, ScoreConfidence::Low, "Expected Low confidence for high divergence");
    assert!(bd_div.variance > 25.0);
}
