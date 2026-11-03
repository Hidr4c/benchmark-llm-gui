use benchmark_llm_gui::domain::model::{BenchmarkScores, ModelBenchmark};
use benchmark_llm_gui::domain::table_preferences::{
    ColumnFilter, FilterOperator, TableColumnId, TablePreferences,
};
use benchmark_llm_gui::presentation::filtering::{FilterService, FilterState};

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
    assert!(!op_re.matches("Llama-3.3-70B", r"[invalid("));

    let op_not_re = FilterOperator::NotMatchRegex;
    assert!(op_not_re.matches("Llama-3.3-70B", r"^qwen.*"));
    assert!(!op_not_re.matches("Qwen2.5-7B", r"^qwen.*"));
}

#[test]
fn test_filter_operator_numeric_and_prefixes() {
    let le = FilterOperator::LessEqual;
    assert!(le.matches("6", "6"));
    assert!(le.matches("3", "6"));
    assert!(!le.matches("7", "6"));
    assert!(le.matches("14.5 Go", "16"));
    assert!(!le.matches("18.2 Go", "16"));

    let ge = FilterOperator::GreaterEqual;
    assert!(ge.matches("75.4", "70"));
    assert!(!ge.matches("68.2", "70"));

    // User typing leading symbols
    assert!(FilterOperator::Contains.matches("5", "<= 6"));
    assert!(!FilterOperator::Contains.matches("8", "<= 6"));
    assert!(FilterOperator::Contains.matches("82.0", ">= 80"));
    assert!(!FilterOperator::Contains.matches("74.0", ">= 80"));
    assert!(FilterOperator::Contains.matches("#3", "= 3"));
}

#[test]
fn test_top_n_rank_filtering_in_service() {
    let mut models = Vec::new();
    for i in 1..=10 {
        models.push(ModelBenchmark {
            id: format!("m{i}"),
            name: format!("Model-{i}"),
            scores: BenchmarkScores {
                open_llm: Some(100.0 - (i as f32) * 5.0),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    let mut filters = FilterState::default();
    // Default: all 10 models returned
    let indices = FilterService::compute_filtered_indices(&models, &filters);
    assert_eq!(indices.len(), 10);

    // Apply Rank filter: "6" (default operator is LessEqual <= 6)
    filters.table_prefs.column_filters.insert(
        TableColumnId::Rank,
        ColumnFilter::new(FilterOperator::LessEqual, "6"),
    );

    let top6_indices = FilterService::compute_filtered_indices(&models, &filters);
    assert_eq!(top6_indices.len(), 6);
    assert_eq!(top6_indices[0], 0); // Rank #1
    assert_eq!(top6_indices[5], 5); // Rank #6
}

#[test]
fn test_column_visibility_and_reset() {
    let mut prefs = TablePreferences::default();
    assert!(prefs.is_visible(TableColumnId::Model));
    assert!(prefs.is_visible(TableColumnId::CloudPrice));
    assert!(!prefs.is_visible(TableColumnId::Year));
    assert!(!prefs.is_visible(TableColumnId::MmluPro));
    assert!(!prefs.are_advanced_columns_visible());

    prefs.set_advanced_columns_visible(true);
    assert!(prefs.is_visible(TableColumnId::Year));
    assert!(prefs.are_advanced_columns_visible());

    prefs.toggle_visibility(TableColumnId::CloudPrice);
    assert!(!prefs.is_visible(TableColumnId::CloudPrice));

    prefs.show_all_columns();
    assert!(prefs.is_visible(TableColumnId::CloudPrice));
    assert!(prefs.is_visible(TableColumnId::MmluPro));

    prefs.reset_defaults();
    assert!(prefs.is_visible(TableColumnId::Model));
    assert!(!prefs.is_visible(TableColumnId::MmluPro));
}

#[test]
fn test_table_column_order_and_movement() {
    let mut prefs = TablePreferences::default();
    assert_eq!(prefs.column_order.len(), TableColumnId::all().len());
    assert_eq!(prefs.column_order[0], TableColumnId::Rank);
    assert_eq!(prefs.column_order[1], TableColumnId::Model);

    prefs.move_column(TableColumnId::Model, -1);
    assert_eq!(prefs.column_order[0], TableColumnId::Model);
    assert_eq!(prefs.column_order[1], TableColumnId::Rank);

    prefs.move_column(TableColumnId::Model, 2);
    assert_eq!(prefs.column_order[2], TableColumnId::Model);

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
        scores: BenchmarkScores { open_llm: Some(85.0), ..Default::default() },
        size_gb: 19.5,
        ..Default::default()
    };
    let model2 = ModelBenchmark {
        id: "m2".into(),
        name: "Llama-3.3-70B-Instruct".into(),
        scores: BenchmarkScores { open_llm: Some(88.0), ..Default::default() },
        size_gb: 42.0,
        ..Default::default()
    };

    assert!(prefs.matches_model(&model1, "#1"));
    assert!(prefs.matches_model(&model2, "#2"));

    prefs.column_filters.insert(
        TableColumnId::Model,
        ColumnFilter::new(FilterOperator::Contains, "coder"),
    );
    assert_eq!(prefs.active_filter_count(), 1);
    assert!(prefs.has_active_filter(TableColumnId::Model));
    assert!(prefs.matches_model(&model1, "#1"));
    assert!(!prefs.matches_model(&model2, "#2"));

    prefs.column_filters.insert(
        TableColumnId::Size,
        ColumnFilter::new(FilterOperator::MatchRegex, r"19\.\d"),
    );
    assert!(prefs.matches_model(&model1, "#1"));

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
    assert!(repo.load_table_preferences().expect("Load failed").is_none());

    let mut prefs = TablePreferences::default();
    prefs.move_column(TableColumnId::Score, -2);
    prefs.column_filters.insert(
        TableColumnId::Model,
        ColumnFilter::new(FilterOperator::MatchRegex, "^mistral"),
    );

    repo.save_table_preferences(&prefs).expect("Save failed");
    let loaded = repo.load_table_preferences().expect("Load failed").expect("Preferences not found");
    assert_eq!(loaded.column_order, prefs.column_order);
    assert_eq!(loaded.column_filters, prefs.column_filters);
}

#[test]
fn test_table_column_reorder_drag_drop() {
    let mut prefs = TablePreferences::default();
    assert_eq!(prefs.column_order[0], TableColumnId::Rank);
    assert_eq!(prefs.column_order[1], TableColumnId::Model);

    // Reorder Score to Model
    prefs.reorder_column(TableColumnId::Score, TableColumnId::Model);
    assert_eq!(prefs.column_order[1], TableColumnId::Score);
    assert_eq!(prefs.column_order[2], TableColumnId::Model);

    // Reorder Model to Rank
    prefs.reorder_column(TableColumnId::Model, TableColumnId::Rank);
    assert_eq!(prefs.column_order[0], TableColumnId::Model);
    assert_eq!(prefs.column_order[1], TableColumnId::Rank);
    assert_eq!(prefs.column_order[2], TableColumnId::Score);
}

#[test]
fn test_preferences_sanitize_inactive_filters() {
    let mut prefs = TablePreferences::default();
    prefs.column_filters.insert(
        TableColumnId::Rank,
        ColumnFilter {
            operator: FilterOperator::LessEqual,
            pattern: String::new(),
            active: false,
        },
    );
    prefs.column_filters.insert(
        TableColumnId::CloudPrice,
        ColumnFilter {
            operator: FilterOperator::LessEqual,
            pattern: String::new(),
            active: false,
        },
    );
    prefs.column_filters.insert(
        TableColumnId::Score,
        ColumnFilter {
            operator: FilterOperator::GreaterEqual,
            pattern: "70".into(),
            active: true,
        },
    );

    prefs.sanitize();

    assert_eq!(prefs.column_filters[&TableColumnId::Rank].operator, FilterOperator::Contains);
    assert_eq!(prefs.column_filters[&TableColumnId::CloudPrice].operator, FilterOperator::Contains);
    assert_eq!(prefs.column_filters[&TableColumnId::Score].operator, FilterOperator::GreaterEqual);
}
