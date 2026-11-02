use eframe::egui;
use crate::domain::traits::SnapshotMeta;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;
use crate::presentation::updater_state::UpdaterState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Table,
    Graph,
    Comparator,
    Simulator,
    Sources,
    Tutorial,
}

impl AppTab {
    pub fn label(&self, lang: Language) -> &'static str {
        let t = lang.t();
        match self {
            Self::Table => t.tab_table,
            Self::Graph => t.tab_graph,
            Self::Comparator => t.tab_comparator,
            Self::Simulator => t.tab_simulator,
            Self::Sources => t.tab_sources,
            Self::Tutorial => t.tab_tutorial,
        }
    }
}

pub fn render_top_header(
    ui: &mut egui::Ui,
    active_tab: &mut AppTab,
    language: &mut Language,
    zoom_factor: &mut f32,
    is_syncing: bool,
    snapshots: &[SnapshotMeta],
    selected_snapshot_id: Option<i64>,
    status_message: Option<&(String, bool)>,
    updater: &mut UpdaterState,
    on_sync: &mut bool,
    on_switch_snapshot: &mut Option<i64>,
    on_launch_tour: &mut bool,
    on_contact: &mut bool,
) {
    let t = language.t();
    ui.add_space(4.0);

    // --- Single Header Line: Brand, Navigation View Dropdown & Utility Tools ---
    ui.horizontal(|ui| {
        ui.heading(
            egui::RichText::new("LLM Benchmark Explorer")
                .size(17.0)
                .strong()
                .color(ThemeTokens::PRIMARY),
        );

        ui.separator();

        // 1. View Navigation Dropdown List (Single clear ComboBox instead of multiple flat buttons)
        egui::ComboBox::from_id_salt("main_view_nav_dropdown")
            .selected_text(
                egui::RichText::new(active_tab.label(*language))
                    .strong()
                    .color(ThemeTokens::CYAN_EDGE)
            )
            .width(200.0)
            .show_ui(ui, |ui| {
                let views = [
                    AppTab::Table,
                    AppTab::Graph,
                    AppTab::Comparator,
                    AppTab::Simulator,
                    AppTab::Sources,
                    AppTab::Tutorial,
                ];
                for v in views {
                    let is_sel = *active_tab == v;
                    let text = egui::RichText::new(v.label(*language)).strong();
                    if ui.selectable_label(is_sel, text).clicked() {
                        *active_tab = v;
                    }
                }
            });



        // 2. Right-Aligned Global Actions
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // 1. Language Dropdown Selector (Strictly "FR" or "EN") - Placed FIRST in right_to_left so it appears at the EXTREME RIGHT
            egui::ComboBox::from_id_salt("top_language_combobox")
                .selected_text(language.label())
                .width(55.0)
                .show_ui(ui, |ui| {
                    for lang in Language::all() {
                        if ui.selectable_label(*language == lang, lang.label()).clicked() {
                            *language = lang;
                        }
                    }
                });

            ui.separator();

            // 2. Zoom / Text Scale Selector
            let zoom_percent = (*zoom_factor * 100.0).round() as i32;
            let zoom_prefix = language.dock().zoom_label;
            let zoom_label = format!("{zoom_prefix} {zoom_percent}%");
            egui::ComboBox::from_id_salt("top_zoom_combobox")
                .selected_text(egui::RichText::new(zoom_label).strong())
                .width(96.0)
                .show_ui(ui, |ui| {
                    let zoom_levels: [f32; 6] = [0.90, 1.0, 1.15, 1.25, 1.40, 1.50];
                    for &z in &zoom_levels {
                        let p = (z * 100.0).round() as i32;
                        let text = format!("{p}%");
                        if ui.selectable_label((*zoom_factor - z).abs() < 0.01, text).clicked() {
                            *zoom_factor = z;
                            ui.ctx().set_zoom_factor(z);
                        }
                    }
                });

            ui.separator();

            // 2.5. Contact / Feedback Dialog Button
            if ui.button(egui::RichText::new(language.contact().btn_contact).size(12.0))
                .on_hover_text(language.contact().dialog_desc)
                .clicked()
            {
                *on_contact = true;
            }

            ui.separator();

            // 3. Interactive Guided Demo / Walkthrough Button
            if ui.button(egui::RichText::new(t.btn_tour_demo).color(ThemeTokens::GOLD_5090).strong())
                .on_hover_text(t.tooltip_tour_demo)
                .clicked()
            {
                *on_launch_tour = true;
            }

            ui.separator();

            // 3. Application Update Button / Status Badge
            let egui_ctx = ui.ctx().clone();
            updater.render_top_bar_button(ui, &egui_ctx, *language);

            ui.separator();

            // 3. Refresh / Sync Button (clearly styled as a button)
            let sync_text = if is_syncing {
                egui::RichText::new(t.sync_downloading).color(ThemeTokens::PRIMARY)
            } else {
                egui::RichText::new(t.sync_refresh).color(ThemeTokens::SUCCESS).strong()
            };
            if ui.add_enabled(!is_syncing, egui::Button::new(sync_text))
                .on_hover_text(t.sync_tooltip)
                .clicked()
            {
                *on_sync = true;
            }

            // 4. Snapshot time-travel dropdown
            let snap_title = if let Some(id) = selected_snapshot_id {
                snapshots
                    .iter()
                    .find(|s| s.id == id)
                    .map(|s| s.created_at.clone())
                    .unwrap_or_else(|| "Snapshot".into())
            } else if let Some(first) = snapshots.first() {
                format!("{} (Latest)", first.created_at)
            } else {
                "Empty".into()
            };
            egui::ComboBox::from_id_salt("snap_history_combo_top")
                .selected_text(format!("Snapshot: {snap_title}"))
                .width(150.0)
                .show_ui(ui, |ui| {
                    for snap in snapshots {
                        let is_sel = selected_snapshot_id == Some(snap.id);
                        let lbl = format!("{} ({} mod.)", snap.created_at, snap.model_count);
                        if ui.selectable_label(is_sel, lbl).clicked() {
                            *on_switch_snapshot = Some(snap.id);
                        }
                    }
                });
        });
    });

    if let Some((msg, is_err)) = status_message {
        if *is_err {
            ui.add_space(2.0);
            ui.label(egui::RichText::new(msg).color(ThemeTokens::DANGER).size(12.0));
        }
    }
}
