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
    tour.next(8);
    assert_eq!(tour.current_step, 1);

    // Prev step
    tour.prev();
    assert_eq!(tour.current_step, 0);
    tour.prev(); // At zero, remains zero
    assert_eq!(tour.current_step, 0);

    // Jump to step
    tour.jump_to(5);
    assert_eq!(tour.current_step, 5);

    // Jump clamp to max step (9 steps -> index 8)
    tour.jump_to(99);
    assert_eq!(tour.current_step, 8);

    // Reaching end terminates tour
    tour.next(9);
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
    assert!(!tour.show_guide_summary);
    tour.show_guide_summary = true;
    assert!(tour.show_guide_summary);

    // Verify initial completed count
    assert_eq!(tour.completed_count(), 0);

    // Trigger action on step 1 (Auto-Detect)
    tour.current_step = 1;
    tour.on_auto_detect_clicked();
    assert!(tour.completed_steps[1]);
    assert_eq!(tour.completed_count(), 1);
    assert!(tour.transition_timer.is_some());
    assert_eq!(tour.transition_feedback.as_deref(), Some("Detection materielle reussie !"));
}

#[test]
fn test_tutorial_tab_localization() {
    let t_fr = Language::French.t();
    let t_en = Language::English.t();

    assert_eq!(t_fr.tab_tutorial, "Guide & Tutoriels");
    assert_eq!(t_en.tab_tutorial, "Guide & Tutorials");
    assert_eq!(t_fr.tab_table, "Benchmark & Top Modèles");
    assert_eq!(t_en.tab_table, "Benchmark & Leaderboard");
    assert_eq!(t_fr.comp.btn_back, "Retour au Classement (Benchmark)");
    assert_eq!(t_en.comp.btn_back, "Back to Model Leaderboard");

    assert!(!t_fr.tut_hero_title.is_empty());
    assert!(!t_en.tut_hero_title.is_empty());
    assert!(!t_fr.tour_s7_title.is_empty());
    assert!(!t_en.tour_s7_title.is_empty());
    assert!(!t_fr.tour_s8_title.is_empty());
    assert!(!t_en.tour_s8_title.is_empty());

    // Tutorial Guide Sections Verification
    assert_eq!(t_fr.tut_guide.table_guide_title, "Comprendre le Classement & les Colonnes");
    assert_eq!(t_en.tut_guide.table_guide_title, "Understanding Model Rankings & Columns");
    assert!(!t_fr.tut_guide.col_essential_desc.is_empty());
    assert!(!t_en.tut_guide.col_essential_desc.is_empty());
    assert!(!t_fr.tut_guide.col_adv_desc.is_empty());
    assert!(!t_en.tut_guide.col_adv_desc.is_empty());
    assert!(!t_fr.tut_guide.col_raw_desc.is_empty());
    assert!(!t_en.tut_guide.col_raw_desc.is_empty());
    assert_eq!(t_fr.tut_guide.filters_guide_title, "Maîtriser les Filtres & la Fiabilité");
    assert_eq!(t_en.tut_guide.filters_guide_title, "Mastering Filters & Reliability");
    assert!(!t_fr.tut_guide.filter_ram_desc.is_empty());
    assert!(!t_fr.tut_guide.filter_rel_desc.is_empty());

    // Tools Guide & Multi-Runner Verification
    let tools_fr = Language::French.tools();
    let tools_en = Language::English.tools();
    assert!(!tools_fr.hero_title.is_empty());
    assert!(!tools_en.hero_title.is_empty());
    assert_eq!(benchmark_llm_gui::presentation::i18n::TOOLS_LIST.len(), 7);
    for tool in &benchmark_llm_gui::presentation::i18n::TOOLS_LIST {
        assert!(!tool.name.is_empty());
        assert!(tool.url.starts_with("https://"));
        assert!(!tool.command.is_empty());
    }

    let runner_fr = Language::French.runner();
    let runner_en = Language::English.runner();
    assert_eq!(runner_fr.tab_lm_studio, "LM Studio");
    assert_eq!(runner_en.tab_lm_studio, "LM Studio");
    assert!(!runner_fr.hint_lms.is_empty());
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

    let mut filters = FilterState::default();
    filters.group_by_root = true;
    let items_grouped = build_display_items(&models, &indices, &filters);
    assert_eq!(items_grouped.len(), 2);
    assert_eq!(items_grouped[0].rank_str, "#1");
    assert_eq!(items_grouped[1].rank_str, "#2");

    filters.group_by_root = false;
    let items_flat = build_display_items(&models, &indices, &filters);
    assert_eq!(items_flat.len(), 3);
    assert_eq!(items_flat[0].rank_str, "#1.1");
    assert_eq!(items_flat[1].rank_str, "#1.2");
    assert_eq!(items_flat[2].rank_str, "#2");
}

#[test]
fn test_tour_end_to_end_walkthrough_flow() {
    use benchmark_llm_gui::presentation::filtering::{FilterState, ModelScope};
    use benchmark_llm_gui::domain::hardware::HardwarePlatform;

    let mut tour = TourState::default();
    let mut filters = FilterState::default();
    let total_steps = 9;

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

    // Step 3: User toggles Advanced Columns checkbox
    tour.check_and_update_progress(&filters, None, total_steps);
    filters.show_advanced_columns = true;
    assert!(tour.check_and_update_progress(&filters, None, total_steps));
    assert!(tour.completed_steps[3]);
    tour.update_transition(4.0, total_steps);
    assert_eq!(tour.current_step, 4);

    // Step 4: User toggles root model group folder
    tour.check_and_update_progress(&filters, None, total_steps);
    filters.group_by_root = !filters.group_by_root;
    assert!(tour.check_and_update_progress(&filters, None, total_steps));
    assert!(tour.completed_steps[4]);
    tour.update_transition(4.0, total_steps);
    assert_eq!(tour.current_step, 5);

    // Step 5: User clicks a model in table
    tour.check_and_update_progress(&filters, None, total_steps);
    assert!(tour.check_and_update_progress(&filters, Some("test/model-1"), total_steps));
    assert!(tour.completed_steps[5]);
    tour.update_transition(4.0, total_steps);
    assert_eq!(tour.current_step, 6);

    // Step 6: User clicks Recenter on Graph
    tour.on_recenter_clicked();
    assert!(tour.completed_steps[6]);
    tour.update_transition(4.0, total_steps);
    assert_eq!(tour.current_step, 7);

    // Step 7: User selects models in Comparator Face-à-Face
    tour.check_and_update_comp_progress(&[Some("qwen-7b".into()), None]);
    assert!(tour.completed_steps[7]);
    tour.update_transition(4.0, total_steps);
    assert_eq!(tour.current_step, 8);

    // Step 8: User changes GPU in Simulator
    let mut sim_config = benchmark_llm_gui::domain::simulator::HardwareRigConfig::default();
    tour.check_and_update_sim_progress(&sim_config);
    sim_config.gpu = benchmark_llm_gui::domain::simulator::GpuTier::Rtx4090_24G;
    tour.check_and_update_sim_progress(&sim_config);
    assert!(tour.completed_steps[8]);
    tour.update_transition(4.0, total_steps);
    assert!(!tour.is_active, "Tour must be closed upon completing step 9");
    assert_eq!(tour.completed_count(), 9, "All 9 steps must be marked as completed");
}

#[test]
fn test_tour_manual_validation_for_preselected_states() {
    let mut tour = TourState::default();
    tour.start();
    assert_eq!(tour.current_step, 0);

    // When option is already selected, user clicks manual validation button
    tour.validate_current_step();
    assert!(tour.completed_steps[0]);
    assert!(tour.transition_timer.is_some());
    tour.update_transition(4.0, 9);
    assert_eq!(tour.current_step, 1);
}

#[test]
fn test_tour_step4_raw_sources_and_global_mode_compatibility() {
    use benchmark_llm_gui::presentation::filtering::FilterState;

    let mut tour = TourState::default();
    let mut filters = FilterState::default();
    let total_steps = 9;

    tour.start();
    tour.jump_to(3);
    assert_eq!(tour.current_step, 3);

    tour.check_and_update_progress(&filters, None, total_steps);
    assert!(!tour.completed_steps[3]);

    filters.show_raw_sources = true;
    assert!(tour.check_and_update_progress(&filters, None, total_steps));
    assert!(tour.completed_steps[3]);
    assert!(tour.transition_timer.is_some());

    // When jumped to, tour is paused to allow reading; resume to advance transition
    assert!(tour.is_paused);
    tour.is_paused = false;
    tour.update_transition(4.0, total_steps);
    assert_eq!(tour.current_step, 4);
    assert_eq!(tour.transition_timer, None);
}
