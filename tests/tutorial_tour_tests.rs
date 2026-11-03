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
fn test_tutorial_tab_localization() {
    let tab = AppTab::Tutorial;
    let fr_label = tab.label(Language::French);
    let en_label = tab.label(Language::English);

    assert!(fr_label.contains("Tutoriel") || fr_label.contains("Guide"));
    assert!(en_label.contains("Tutorial") || en_label.contains("Guide"));
}
