use benchmark_llm_gui::presentation::components::top_bar::AppTab;
use benchmark_llm_gui::presentation::components::tour_overlay::TourState;
use benchmark_llm_gui::presentation::i18n::Language;

#[test]
fn test_tour_state_lifecycle_and_navigation() {
    let mut tour = TourState::default();
    assert!(!tour.is_active);
    assert_eq!(tour.current_step, 0);
    assert!(!tour.auto_play);

    tour.start();
    assert!(tour.is_active);
    assert_eq!(tour.current_step, 0);
    assert!(!tour.auto_play);

    // Next step
    tour.next(7);
    assert_eq!(tour.current_step, 1);

    // Prev step
    tour.prev();
    assert_eq!(tour.current_step, 0);
    tour.prev(); // At zero, remains zero
    assert_eq!(tour.current_step, 0);

    // Jump to step
    tour.jump_to(5);
    assert_eq!(tour.current_step, 5);

    // Jump clamp to max step
    tour.jump_to(99);
    assert_eq!(tour.current_step, 6);

    // Reaching end terminates tour
    tour.next(7);
    assert!(!tour.is_active);
}

#[test]
fn test_tour_autoplay_mode() {
    let mut tour = TourState::default();
    tour.start_autoplay();
    assert!(tour.is_active);
    assert!(tour.auto_play);
    assert_eq!(tour.timer, 0.0);

    tour.close();
    assert!(!tour.is_active);
    assert!(!tour.auto_play);
}

#[test]
fn test_tour_guide_and_gamified_progress() {
    let mut tour = TourState::default();
    tour.start();
    // Guide summary is closed by default to prevent masking the table content
    assert!(!tour.show_guide_summary);
    tour.show_guide_summary = true;
    assert!(tour.show_guide_summary);
    assert_eq!(tour.completed_count(), 0);

    // Auto detect trigger -> sets transition timer
    tour.jump_to(1);
    tour.on_auto_detect_clicked();
    assert!(tour.completed_steps[1]);
    assert!(tour.transition_timer.is_some());

    // Advancing through transition
    tour.update_transition(4.0, 7);
    assert!(tour.transition_timer.is_none());
    assert_eq!(tour.current_step, 2);
    assert_eq!(tour.completed_count(), 1);

    // Next step marks current as completed
    tour.next(7);
    assert!(tour.completed_steps[2]);
    assert_eq!(tour.current_step, 3);
    assert_eq!(tour.completed_count(), 2);
}

#[test]
fn test_tutorial_tab_localization() {
    let tab = AppTab::Tutorial;
    let fr_label = tab.label(Language::French);
    let en_label = tab.label(Language::English);

    assert!(fr_label.contains("Tutoriel") || fr_label.contains("Guide"));
    assert!(en_label.contains("Tutorial") || en_label.contains("Guide"));
}

#[test]
fn test_stable_hierarchical_sub_ranking() {
    use benchmark_llm_gui::domain::model::ModelBenchmark;
    use benchmark_llm_gui::presentation::components::filter_bar::FilterState;
    use benchmark_llm_gui::presentation::components::table_items::build_display_items;

    let models = vec![
        ModelBenchmark { id: "m1".into(), name: "GPT 6.1 Sol".into(), is_cloud: true, ..Default::default() },
        ModelBenchmark { id: "m2".into(), name: "GPT 6.1 Sol Pro".into(), is_cloud: true, ..Default::default() },
        ModelBenchmark { id: "m3".into(), name: "Claude 3.5 Sonnet".into(), is_cloud: true, ..Default::default() },
    ];
    let indices = vec![0, 1, 2];

    // 1. Grouped mode: GPT 6.1 is folded (#1), Claude Sonnet is #2
    let mut filters = FilterState::default();
    filters.group_by_root = true;
    let items_grouped = build_display_items(&models, &indices, &filters);
    assert_eq!(items_grouped.len(), 2);
    assert_eq!(items_grouped[0].rank_str, "🥇 1");
    assert_eq!(items_grouped[1].rank_str, "🥈 2");

    // 2. Ungrouped mode: GPT 6.1 variants are #1.1 and #1.2, Claude Sonnet STAYS #2!
    filters.group_by_root = false;
    let items_flat = build_display_items(&models, &indices, &filters);
    assert_eq!(items_flat.len(), 3);
    assert_eq!(items_flat[0].rank_str, "#1.1");
    assert_eq!(items_flat[1].rank_str, "#1.2");
    assert_eq!(items_flat[2].rank_str, "🥈 2"); // Claude Sonnet never jumps to #3!
}

#[test]
fn test_tour_end_to_end_walkthrough_flow() {
    use benchmark_llm_gui::domain::hardware::HardwarePlatform;
    use benchmark_llm_gui::presentation::filtering::{FilterState, ModelScope};

    let mut tour = TourState::default();
    let mut filters = FilterState::default();
    let total_steps = 7;

    tour.start();
    assert!(tour.is_active);
    assert_eq!(tour.current_step, 0);

    // Step 0: User filters scope to LocalOnly
    tour.check_and_update_progress(&filters, None, total_steps);
    filters.scope = ModelScope::LocalOnly;
    assert!(tour.check_and_update_progress(&filters, None, total_steps));
    assert!(tour.completed_steps[0]);
    tour.update_transition(4.0, total_steps);
    assert_eq!(tour.current_step, 1);

    // Step 1: User clicks Auto-Detect
    tour.on_auto_detect_clicked();
    assert!(tour.completed_steps[1]);
    tour.update_transition(4.0, total_steps);
    assert_eq!(tour.current_step, 2);

    // Step 2: User changes hardware filter
    tour.check_and_update_progress(&filters, None, total_steps);
    filters.hardware = HardwarePlatform::Gpu24Gb;
    assert!(tour.check_and_update_progress(&filters, None, total_steps));
    assert!(tour.completed_steps[2]);
    tour.update_transition(4.0, total_steps);
    assert_eq!(tour.current_step, 3);

    // Step 3: User toggles root model group folder
    tour.check_and_update_progress(&filters, None, total_steps);
    filters.group_by_root = !filters.group_by_root;
    assert!(tour.check_and_update_progress(&filters, None, total_steps));
    assert!(tour.completed_steps[3]);
    tour.update_transition(4.0, total_steps);
    assert_eq!(tour.current_step, 4);

    // Step 4: User clicks a model in table
    tour.check_and_update_progress(&filters, None, total_steps);
    assert!(tour.check_and_update_progress(&filters, Some("test/model-1"), total_steps));
    assert!(tour.completed_steps[4]);
    tour.update_transition(4.0, total_steps);
    assert_eq!(tour.current_step, 5);

    // Step 5: User clicks Recenter on Graph
    tour.on_recenter_clicked();
    assert!(tour.completed_steps[5]);
    tour.update_transition(4.0, total_steps);
    assert_eq!(tour.current_step, 6);

    // Step 6: User changes GPU in Simulator
    let mut sim_config = benchmark_llm_gui::domain::simulator::HardwareRigConfig::default();
    tour.check_and_update_sim_progress(&sim_config);
    sim_config.gpu = benchmark_llm_gui::domain::simulator::GpuTier::Rtx4090_24G;
    tour.check_and_update_sim_progress(&sim_config);
    assert!(tour.completed_steps[6]);
    tour.update_transition(4.0, total_steps);
    assert!(!tour.is_active, "Tour must be closed upon completing step 7");
    assert_eq!(tour.completed_count(), 7, "All 7 steps must be marked as completed");
}

#[test]
fn test_tour_step4_raw_sources_and_global_mode_compatibility() {
    use benchmark_llm_gui::presentation::filtering::{FilterState, TableColumnMode};

    let mut tour = TourState::default();
    let mut filters = FilterState::default();
    let total_steps = 7;

    // Start tour and jump directly to Step 4 (index 3)
    tour.start();
    tour.jump_to(3);
    assert_eq!(tour.current_step, 3);

    // Verify under RawBenchmarkSources mode
    filters.column_mode = TableColumnMode::RawBenchmarkSources;
    assert_eq!(filters.group_by_root, true);

    // Initial check
    tour.check_and_update_progress(&filters, None, total_steps);
    assert!(!tour.completed_steps[3]);

    // Toggling root grouping in raw sources mode triggers completion and transition
    filters.group_by_root = false;
    assert!(tour.check_and_update_progress(&filters, None, total_steps));
    assert!(tour.completed_steps[3]);
    assert!(tour.transition_timer.is_some());

    // Advance through observation pause
    tour.update_transition(4.0, total_steps);
    assert_eq!(tour.current_step, 4);
    assert_eq!(tour.transition_timer, None);
}
