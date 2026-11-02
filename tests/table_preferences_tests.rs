use benchmark_llm_gui::domain::model::{BenchmarkScores, ModelBenchmark};
use benchmark_llm_gui::domain::table_preferences::{
    ColumnFilter, FilterOperator, TableColumnId, TablePreferences,
};

#[test]
fn test_filter_operator_contains_and_exact() {
    let op_contains = FilterOperator::Contains;
    assert!(op_contains.matches("Qwen2.5-Coder-32B", "coder"));
    assert!(op_contains.matches("Qwen2.5-Coder-32B", "QWEN"));
    assert!(!op_contains.matches("Qwen2.5-Coder-32B", "llama"));

    let op_exact = FilterOperator::Exact;
    assert!(op_exact.matches("Qwen2.5-Coder-32B", "qwen2.5-coder-32b"));
    assert!(!op_exact.matches("Qwen2.5-Coder-32B", "qwen2.5"));
}

#[test]
fn test_filter_operator_not_contains() {
    let op_not = FilterOperator::NotContains;
    assert!(op_not.matches("Mistral Large 2", "llama"));
    assert!(!op_not.matches("Mistral Large 2", "mistral"));
}

#[test]
fn test_filter_operator_regex_and_not_regex() {
    let op_re = FilterOperator::MatchRegex;
    assert!(op_re.matches("Qwen2.5-Coder-32B-Instruct", r"^qwen.*coder.*"));
    assert!(!op_re.matches("Llama-3.3-70B", r"^qwen.*coder.*"));
    // Invalid regex should not panic and return false
    assert!(!op_re.matches("Llama-3.3-70B", r"[invalid("));

    let op_not_re = FilterOperator::NotMatchRegex;
    assert!(op_not_re.matches("Llama-3.3-70B", r"^qwen.*"));
    assert!(!op_not_re.matches("Qwen2.5-7B", r"^qwen.*"));
}

#[test]
fn test_table_column_order_and_movement() {
    let mut prefs = TablePreferences::default();
    assert_eq!(prefs.column_order.len(), TableColumnId::all().len());
    assert_eq!(prefs.column_order[0], TableColumnId::Rank);
    assert_eq!(prefs.column_order[1], TableColumnId::Model);

    // Move Model left by 1 (swap with Rank)
    prefs.move_column(TableColumnId::Model, -1);
    assert_eq!(prefs.column_order[0], TableColumnId::Model);
    assert_eq!(prefs.column_order[1], TableColumnId::Rank);

    // Move Model right by 2
    prefs.move_column(TableColumnId::Model, 2);
    assert_eq!(prefs.column_order[2], TableColumnId::Model);

    // Reset order
    prefs.reset_order();
    assert_eq!(prefs.column_order[0], TableColumnId::Rank);
    assert_eq!(prefs.column_order[1], TableColumnId::Model);
}

#[test]
fn test_preferences_model_filtering() {
    let mut prefs = TablePreferences::default();
    let model1 = ModelBenchmark {
        id: "m1".into(),
        name: "Qwen2.5-Coder-32B".into(),
        scores: BenchmarkScores {
            open_llm: Some(85.0),
            ..Default::default()
        },
        size_gb: 19.5,
        ..Default::default()
    };
    let model2 = ModelBenchmark {
        id: "m2".into(),
        name: "Llama-3.3-70B-Instruct".into(),
        scores: BenchmarkScores {
            open_llm: Some(88.0),
            ..Default::default()
        },
        size_gb: 42.0,
        ..Default::default()
    };

    // No filter: both match
    assert!(prefs.matches_model(&model1, "#1"));
    assert!(prefs.matches_model(&model2, "#2"));

    // Add filter: Name contains "coder"
    prefs.column_filters.insert(
        TableColumnId::Model,
        ColumnFilter::new(FilterOperator::Contains, "coder"),
    );
    assert_eq!(prefs.active_filter_count(), 1);
    assert!(prefs.has_active_filter(TableColumnId::Model));
    assert!(prefs.matches_model(&model1, "#1"));
    assert!(!prefs.matches_model(&model2, "#2"));

    // Add filter: Size regex match "19"
    prefs.column_filters.insert(
        TableColumnId::Size,
        ColumnFilter::new(FilterOperator::MatchRegex, r"19\.\d"),
    );
    assert!(prefs.matches_model(&model1, "#1"));

    // Clear filters
    prefs.reset_filters();
    assert_eq!(prefs.active_filter_count(), 0);
    assert!(prefs.matches_model(&model2, "#2"));
}

#[test]
fn test_preferences_serialization_roundtrip() {
    let mut prefs = TablePreferences::default();
    prefs.move_column(TableColumnId::Year, -1);
    prefs.column_filters.insert(
        TableColumnId::Model,
        ColumnFilter::new(FilterOperator::Exact, "Claude 3.5 Sonnet"),
    );

    let json = serde_json::to_string(&prefs).expect("Serialization failed");
    let deserialized: TablePreferences = serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(prefs.column_order, deserialized.column_order);
    assert_eq!(prefs.column_filters, deserialized.column_filters);
}

#[test]
fn test_sqlite_preferences_persistence() {
    use benchmark_llm_gui::domain::traits::IPreferencesRepository;
    use benchmark_llm_gui::infrastructure::preferences::SqlitePreferencesRepository;

    let repo = SqlitePreferencesRepository::open(":memory:").expect("Failed to open in-memory SQLite");
    // Initially empty
    assert!(repo.load_table_preferences().expect("Load failed").is_none());

    let mut prefs = TablePreferences::default();
    prefs.move_column(TableColumnId::Score, -2);
    prefs.column_filters.insert(
        TableColumnId::Model,
        ColumnFilter::new(FilterOperator::MatchRegex, "^mistral"),
    );

    // Save
    repo.save_table_preferences(&prefs).expect("Save failed");

    // Load back and verify parity
    let loaded = repo.load_table_preferences().expect("Load failed").expect("Preferences not found");
    assert_eq!(loaded.column_order, prefs.column_order);
    assert_eq!(loaded.column_filters, prefs.column_filters);
}
