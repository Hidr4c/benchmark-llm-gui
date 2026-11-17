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

#[tokio::test]
async fn test_startup_background_check_and_version_button_labels() {
    use eframe::egui;

    let newer = ReleaseInfo {
        tag_name: "v1.3.0".to_string(),
        version: "1.3.0".to_string(),
        html_url: "https://github.com/Hidr4c/benchmark-llm-gui/releases/tag/v1.3.0".to_string(),
        published_at: Some("2026-11-18".to_string()),
        body: Some("Release 1.3.0 notes".to_string()),
        download_url: Some("https://github.com/Hidr4c/benchmark-llm-gui/releases/download/v1.3.0/benchmark-llm-gui-windows-x64.zip".to_string()),
        is_newer: true,
    };

    let checker = Arc::new(MockUpdateChecker { result: Ok(newer) });
    let mut state = UpdaterState::new(checker);
    state.current_version = "1.2.0";

    let ctx = egui::Context::default();
    ctx.enable_accesskit();

    let collect_labels = |out: &egui::FullOutput| {
        out.platform_output.accesskit_update.as_ref().map(|u| {
            u.nodes.iter().filter_map(|(_, n)| n.name()).collect::<Vec<_>>().join(" | ")
        }).unwrap_or_default()
    };

    // Before check completes: idle badge shows current version ("v1.2.0") cleanly
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 200.0)));
    let out_idle = ctx.run(raw.clone(), |ctx| {
        egui::TopBottomPanel::top("t").show(ctx, |ui| {
            state.render_top_bar_button(ui, ctx, Language::French);
        });
    });
    let idle_txt = collect_labels(&out_idle);
    assert!(idle_txt.contains("v1.2.0"), "Got idle text: {idle_txt}");
    assert!(!idle_txt.contains("MàJ"));

    // Startup check runs once silently without popping modal uninvited
    state.ensure_startup_check(ctx.clone());
    state.ensure_startup_check(ctx.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    state.poll();

    assert!(!state.show_modal, "Silent startup/refresh check should surface top-bar button without auto-popping modal");
    assert!(matches!(state.status, UpdateStatus::Available(_)));

    // Both FR & EN top-bar buttons display actionable update target version
    let out_fr = ctx.run(raw.clone(), |ctx| {
        egui::TopBottomPanel::top("t").show(ctx, |ui| {
            state.render_top_bar_button(ui, ctx, Language::French);
        });
    });
    let fr_txt = collect_labels(&out_fr);
    assert!(fr_txt.contains("Mettre à jour vers v1.3.0"), "Got FR text: {fr_txt}");

    let out_en = ctx.run(raw, |ctx| {
        egui::TopBottomPanel::top("t").show(ctx, |ui| {
            state.render_top_bar_button(ui, ctx, Language::English);
        });
    });
    let en_txt = collect_labels(&out_en);
    assert!(en_txt.contains("Update version to v1.3.0"), "Got EN text: {en_txt}");
}

#[cfg(target_os = "windows")]
#[tokio::test]
async fn test_download_update_extracts_windows_zip_archive_with_valid_pe_header() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use benchmark_llm_gui::infrastructure::updater::SelfUpdateService;

    // Build real ZIP containing benchmark-llm-gui.exe (starting with MZ header) in temp file via .NET ZipFile
    let temp_dir = std::env::temp_dir();
    let stage_dir = temp_dir.join(format!("mock_stage_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&stage_dir);
    std::fs::create_dir_all(&stage_dir).unwrap();
    let src_exe = stage_dir.join("benchmark-llm-gui.exe");
    let mock_zip = temp_dir.join(format!("mock_pkg_{}.zip", std::process::id()));
    let _ = std::fs::remove_file(&mock_zip);
    std::fs::write(&src_exe, b"MZ\x90\x00mock_windows_pe_payload").unwrap();
    let script = format!(
        "Add-Type -AssemblyName System.IO.Compression.FileSystem; [System.IO.Compression.ZipFile]::CreateFromDirectory('{}', '{}')",
        stage_dir.display().to_string().replace('\'', "''"),
        mock_zip.display().to_string().replace('\'', "''")
    );
    let st = std::process::Command::new("powershell").args(["-NoProfile", "-Command", &script]).status().unwrap();
    assert!(st.success());
    let zip_bytes = std::fs::read(&mock_zip).unwrap();
    let _ = std::fs::remove_dir_all(&stage_dir);
    let _ = std::fs::remove_file(&mock_zip);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server_handle = std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let hdr = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                zip_bytes.len()
            );
            let _ = stream.write_all(hdr.as_bytes());
            let _ = stream.write_all(&zip_bytes);
        }
    });

    let url = format!("http://{addr}/benchmark-llm-gui-windows-x64.zip");
    let extracted_path = SelfUpdateService::download_update(&url).await.expect("ZIP download & extract must succeed");
    let data = std::fs::read(&extracted_path).unwrap();
    assert_eq!(data, b"MZ\x90\x00mock_windows_pe_payload");
    let _ = std::fs::remove_file(&extracted_path);
    let _ = server_handle.join();
}
