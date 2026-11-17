use std::collections::HashMap;
use eframe::egui;
use eframe::egui::accesskit::{Node, NodeId, Role};
use benchmark_llm_gui::domain::modality::ModelModality;
use benchmark_llm_gui::domain::model::{BenchmarkScores, ModelBenchmark};
use benchmark_llm_gui::domain::table_preferences::{TableColumnId, TablePreferences};
use benchmark_llm_gui::presentation::components::filter_bar::{render_filter_bar, FilterState};
use benchmark_llm_gui::presentation::filtering::FilterService;
use benchmark_llm_gui::presentation::components::table_view::render_table;
use benchmark_llm_gui::presentation::i18n::Language;

struct UiDriver {
    ctx: egui::Context,
    nodes: HashMap<NodeId, Node>,
    live_ids: Vec<NodeId>,
    root_id: Option<NodeId>,
    screen: egui::Vec2,
}

impl UiDriver {
    fn new(width: f32, height: f32) -> Self {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        Self { ctx, nodes: HashMap::new(), live_ids: Vec::new(), root_id: None, screen: egui::vec2(width, height) }
    }

    fn step<F: FnMut(&egui::Context)>(&mut self, events: Vec<egui::Event>, mut ui_fn: F) {
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), self.screen));
        raw.events = events;
        let out = self.ctx.run(raw, |ctx| ui_fn(ctx));
        if let Some(upd) = out.platform_output.accesskit_update {
            if let Some(tree) = upd.tree { self.root_id = Some(tree.root); }
            for (id, node) in upd.nodes {
                self.nodes.insert(id, node);
            }
            self.live_ids.clear();
            if let Some(root) = self.root_id {
                let mut stack = vec![root];
                while let Some(id) = stack.pop() {
                    self.live_ids.push(id);
                    if let Some(n) = self.nodes.get(&id) {
                        for &c in n.children() { stack.push(c); }
                    }
                }
            }
        }
    }

    fn find_center(&self, text: &str, role_filter: Option<Role>) -> Option<egui::Pos2> {
        self.live_ids.iter().rev().filter_map(|id| self.nodes.get(id)).find_map(|n| {
            if let Some(r) = role_filter {
                if n.role() != r { return None; }
            }
            let matches = n.name().is_some_and(|name| name.contains(text));
            if !matches { return None; }
            let b = n.bounds()?;
            if b.x1 <= b.x0 || b.y1 <= b.y0 { return None; }
            Some(egui::pos2(((b.x0 + b.x1) * 0.5) as f32, ((b.y0 + b.y1) * 0.5) as f32))
        })
    }

    fn escape_events() -> Vec<egui::Event> {
        vec![
            egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE },
            egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: false, repeat: false, modifiers: egui::Modifiers::NONE },
        ]
    }

    fn click_events(pos: egui::Pos2, button: egui::PointerButton) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton { pos, button, pressed: true, modifiers: egui::Modifiers::NONE },
            egui::Event::PointerButton { pos, button, pressed: false, modifiers: egui::Modifiers::NONE },
        ]
    }
}

fn sample_models() -> Vec<ModelBenchmark> {
    vec![
        ModelBenchmark {
            id: "qwen/qwen3-8b".into(), name: "Qwen 3 8B".into(), family: "Qwen".into(),
            param_count_b: Some(8.0), size_gb: 4.8, vram_rec_gb: 6.2, tokens_per_sec: Some(85.0),
            scores: BenchmarkScores { open_llm: Some(72.0), coding: Some(78.0), agentic: Some(71.0), real_world: Some(66.0), average: Some(71.8), mmlu_pro: Some(69.0), math_lvl5: Some(64.0), gpqa: Some(45.0), ifeval: Some(81.0), ..Default::default() },
            downloads: 250_000, likes: 1200, release_year: Some(2026), modality: ModelModality::Text, ..Default::default()
        },
        ModelBenchmark {
            id: "microsoft/TRELLIS.2".into(), name: "TRELLIS.2".into(), family: "Microsoft".into(),
            size_gb: 4.0, vram_rec_gb: 5.8, tokens_per_sec: Some(30.0),
            scores: BenchmarkScores { real_world: Some(76.0), average: Some(76.0), ..Default::default() },
            downloads: 90_000, likes: 800, release_year: Some(2026), modality: ModelModality::ThreeDGen, ..Default::default()
        },
    ]
}

fn render_frame(driver: &mut UiDriver, events: Vec<egui::Event>, models: &[ModelBenchmark], indices: &[usize], filters: &mut FilterState, sel: &mut Option<String>, lang: Language) {
    let total = models.len();
    let count = indices.len();
    let available_families = FilterService::extract_unique_families(models);
    driver.step(events, |ctx| {
        egui::TopBottomPanel::top("reg_top").show(ctx, |ui| {
            let (mut a, mut b) = (false, false);
            render_filter_bar(ui, filters, &available_families, total, count, &mut a, &mut b, lang);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            render_table(ui, models, indices, filters, sel, lang);
        });
    });
}

#[test]
fn test_real_right_click_add_columns_and_chooser_modal_flows() {
    let models = sample_models();
    let indices: Vec<usize> = (0..models.len()).collect();
    let mut filters = FilterState::default();
    let mut sel = None;
    let mut driver = UiDriver::new(1400.0, 900.0);

    // Frame 0: initial layout with default 7 columns
    render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, Language::French);
    assert!(!filters.table_prefs.are_advanced_columns_visible());
    assert!(!filters.table_prefs.are_raw_sources_visible());

    // Locate a real rendered header label ("Taille") and right-click it!
    let header_pos = driver.find_center("Taille", None).expect("Column header 'Taille' must exist");
    render_frame(&mut driver, UiDriver::click_events(header_pos, egui::PointerButton::Secondary), &models, &indices, &mut filters, &mut sel, Language::French);
    // Run one extra frame for egui popup layout to settle in AccessKit tree
    render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, Language::French);

    // Click REAL "Colonnes Avancees" checkbox inside the open right-click context menu!
    let adv_pos = driver.find_center("Colonnes Avancees", Some(Role::CheckBox)).expect("Context menu 'Colonnes Avancees' checkbox missing");
    render_frame(&mut driver, UiDriver::click_events(adv_pos, egui::PointerButton::Primary), &models, &indices, &mut filters, &mut sel, Language::French);
    assert!(filters.table_prefs.are_advanced_columns_visible(), "Real right-click -> 'Colonnes Avancees' must activate advanced columns without panic");
    // Render subsequent frames with new columns active
    render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, Language::French);

    // Close menu with Escape, then right-click "Score" -> click REAL "Sources Brutes" checkbox!
    render_frame(&mut driver, UiDriver::escape_events(), &models, &indices, &mut filters, &mut sel, Language::French);
    render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, Language::French);
    let score_pos = driver.find_center("Score", None).expect("Column header 'Score' must exist");
    render_frame(&mut driver, UiDriver::click_events(score_pos, egui::PointerButton::Secondary), &models, &indices, &mut filters, &mut sel, Language::French);
    render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, Language::French);
    let raw_pos = driver.find_center("Sources Brutes", Some(Role::CheckBox)).expect("Context menu 'Sources Brutes' checkbox missing");
    render_frame(&mut driver, UiDriver::click_events(raw_pos, egui::PointerButton::Primary), &models, &indices, &mut filters, &mut sel, Language::French);
    assert!(filters.table_prefs.are_raw_sources_visible(), "Real right-click -> 'Sources Brutes' must activate raw columns without panic");
    render_frame(&mut driver, UiDriver::escape_events(), &models, &indices, &mut filters, &mut sel, Language::French);
    render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, Language::French);

    // Right-click again -> click REAL "Choisir / Réorganiser Colonnes..." button to open modal!
    let model_pos = driver.find_center("Modèle", None).expect("Column header 'Modèle' must exist");
    render_frame(&mut driver, UiDriver::click_events(model_pos, egui::PointerButton::Secondary), &models, &indices, &mut filters, &mut sel, Language::French);
    render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, Language::French);
    let chooser_btn = driver.find_center("Sélecteur de colonnes", Some(Role::Button)).expect("Context menu column chooser button missing");
    render_frame(&mut driver, UiDriver::click_events(chooser_btn, egui::PointerButton::Primary), &models, &indices, &mut filters, &mut sel, Language::French);
    assert!(filters.column_chooser_open, "Column chooser modal must open from context menu");
    render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, Language::French);

    // Inside Column Chooser Modal: click "+ Avancé", "+ Sources Brutes", "Tout cocher", "Par défaut", "Fermer"!
    for btn_txt in ["+ Avancé", "+ Sources Brutes", "Tout cocher", "Par défaut"] {
        let p = driver.find_center(btn_txt, Some(Role::Button)).unwrap_or_else(|| panic!("Missing modal button: {btn_txt}"));
        render_frame(&mut driver, UiDriver::click_events(p, egui::PointerButton::Primary), &models, &indices, &mut filters, &mut sel, Language::French);
        render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, Language::French);
    }

    let close_btn = driver.find_center("Fermer", Some(Role::Button)).expect("'Fermer' button missing");
    render_frame(&mut driver, UiDriver::click_events(close_btn, egui::PointerButton::Primary), &models, &indices, &mut filters, &mut sel, Language::French);
    assert!(!filters.column_chooser_open);

    // Right-click "Score" again -> test sort ascending, move column left, and reset column order!
    for menu_txt in ["Trier par ordre croissant", "Déplacer à gauche", "Rétablir l'ordre d'origine"] {
        let sp = driver.find_center("Score", None).expect("Column header 'Score' must exist");
        render_frame(&mut driver, UiDriver::click_events(sp, egui::PointerButton::Secondary), &models, &indices, &mut filters, &mut sel, Language::French);
        render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, Language::French);
        let item_pos = driver.find_center(menu_txt, Some(Role::Button)).unwrap_or_else(|| panic!("Missing menu action: {menu_txt}"));
        render_frame(&mut driver, UiDriver::click_events(item_pos, egui::PointerButton::Primary), &models, &indices, &mut filters, &mut sel, Language::French);
        render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, Language::French);
    }
}

#[test]
fn test_column_reorder_drag_zero_cols_empty_rows_and_narrow_viewports() {
    let models = sample_models();
    let indices: Vec<usize> = (0..models.len()).collect();
    let mut filters = FilterState::default();
    let mut sel = None;

    for (w, h, lang) in [(640.0, 480.0, Language::French), (1920.0, 1080.0, Language::English)] {
        let mut driver = UiDriver::new(w, h);
        render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, lang);

        // Toggle every single column individually one by one while rendering live table
        for &col in TableColumnId::all() {
            filters.table_prefs.toggle_visibility(col);
            render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, lang);
        }
        // Force 0 visible columns + empty rows simultaneously -> must never panic
        filters.table_prefs.hidden_columns = TableColumnId::all().iter().copied().collect();
        render_frame(&mut driver, vec![], &models, &[], &mut filters, &mut sel, lang);
        render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, lang);

        // Drag header "Score" onto "Taille" to verify DnD reorder stability
        filters.table_prefs = TablePreferences::default();
        render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, lang);
        if let (Some(src), Some(dst)) = (driver.find_center("Score", None), driver.find_center("Taille", None).or_else(|| driver.find_center("Size", None))) {
            let drag = vec![
                egui::Event::PointerMoved(src),
                egui::Event::PointerButton { pos: src, button: egui::PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::NONE },
                egui::Event::PointerMoved(dst),
                egui::Event::PointerButton { pos: dst, button: egui::PointerButton::Primary, pressed: false, modifiers: egui::Modifiers::NONE },
            ];
            render_frame(&mut driver, drag, &models, &indices, &mut filters, &mut sel, lang);
            render_frame(&mut driver, vec![], &models, &indices, &mut filters, &mut sel, lang);
        }
    }
}
