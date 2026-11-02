use benchmark_llm_gui::domain::updater::{is_version_newer, parse_semver, IUpdateChecker, ReleaseInfo};
use benchmark_llm_gui::presentation::i18n::Language;
use benchmark_llm_gui::presentation::updater_state::{UpdateStatus, UpdaterState};
use std::sync::Arc;

struct MockUpdateChecker {
    result: Result<ReleaseInfo, String>,
}

#[async_trait::async_trait]
impl IUpdateChecker for MockUpdateChecker {
    async fn check_latest_release(&self, _cur: &str) -> Result<ReleaseInfo, String> {
        self.result.clone()
    }
}

#[test]
fn test_semver_parsing_and_comparison() {
    assert_eq!(parse_semver("1.0.0"), Some((1, 0, 0)));
    assert_eq!(parse_semver("v1.2.3"), Some((1, 2, 3)));
    assert_eq!(parse_semver("V2.0.0-rc1"), Some((2, 0, 0)));
    assert_eq!(parse_semver("0.9"), Some((0, 9, 0)));

    // Newer candidates
    assert!(is_version_newer("1.0.0", "1.1.0"));
    assert!(is_version_newer("1.0.0", "v1.0.1"));
    assert!(is_version_newer("v1.0.0", "v2.0.0"));
    assert!(is_version_newer("1.9.9", "2.0.0"));

    // Identical or older
    assert!(!is_version_newer("1.0.0", "1.0.0"));
    assert!(!is_version_newer("v1.1.0", "1.1.0"));
    assert!(!is_version_newer("2.0.0", "1.9.9"));
    assert!(!is_version_newer("1.1.0", "1.0.9"));
}

#[tokio::test]
async fn test_updater_state_detects_available_update() {
    let mock_release = ReleaseInfo {
        tag_name: "v1.1.0".to_string(),
        version: "1.1.0".to_string(),
        html_url: "https://github.com/Hidr4c/benchmark-llm-gui/releases/tag/v1.1.0".to_string(),
        published_at: Some("2026-11-02".to_string()),
        body: Some("Exciting release notes".to_string()),
        download_url: Some("https://github.com/Hidr4c/benchmark-llm-gui/releases/download/v1.1.0/app.exe".to_string()),
        is_newer: true,
    };

    let checker = Arc::new(MockUpdateChecker {
        result: Ok(mock_release.clone()),
    });

    let mut state = UpdaterState::new(checker);
    state.current_version = "1.0.0";
    assert_eq!(state.status, UpdateStatus::Idle);

    // Simulate direct reception of update
    let ctx = eframe::egui::Context::default();
    state.trigger_check(ctx);
    assert_eq!(state.status, UpdateStatus::Checking);

    // Allow tokio task to deliver message
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    state.poll();

    match &state.status {
        UpdateStatus::Available(info) => {
            assert_eq!(info.tag_name, "v1.1.0");
            assert!(info.is_newer);
            assert!(state.show_modal);
        }
        other => panic!("Expected UpdateStatus::Available, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_updater_state_handles_up_to_date() {
    let mock_release = ReleaseInfo {
        tag_name: "v1.0.0".to_string(),
        version: "1.0.0".to_string(),
        html_url: "https://github.com/Hidr4c/benchmark-llm-gui".to_string(),
        published_at: None,
        body: None,
        download_url: None,
        is_newer: false,
    };

    let checker = Arc::new(MockUpdateChecker {
        result: Ok(mock_release),
    });

    let mut state = UpdaterState::new(checker);
    state.current_version = "1.0.0";

    let ctx = eframe::egui::Context::default();
    state.trigger_check(ctx);

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    state.poll();

    assert_eq!(state.status, UpdateStatus::UpToDate("v1.0.0".to_string()));
    assert!(!state.show_modal);
}

#[test]
fn test_update_i18n_translations() {
    for lang in Language::all() {
        let t = lang.t();
        assert!(!t.update.btn_check.is_empty());
        assert!(!t.update.btn_checking.is_empty());
        assert!(!t.update.btn_available.is_empty());
        assert!(!t.update.btn_up_to_date.is_empty());
        assert!(!t.update.modal_title.is_empty());
        assert!(!t.update.btn_download.is_empty());
    }
}

#[tokio::test]
async fn test_self_update_service_handles_empty_or_invalid_download() {
    use benchmark_llm_gui::infrastructure::updater::SelfUpdateService;
    let res = SelfUpdateService::download_update("http://127.0.0.1:9999/nonexistent.exe").await;
    assert!(res.is_err(), "Non-existent endpoint must return clean network Err");
}
