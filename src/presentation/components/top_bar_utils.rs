use eframe::egui;
use crate::domain::traits::SnapshotMeta;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;
use crate::presentation::updater_state::UpdaterState;

pub fn render_utility_controls_rtl(
    ui: &mut egui::Ui,
    language: &mut Language,
    zoom_factor: &mut f32,
    is_syncing: bool,
    snapshots: &[SnapshotMeta],
    selected_snapshot_id: Option<i64>,
    updater: &mut UpdaterState,
    on_sync: &mut bool,
    on_switch_snapshot: &mut Option<i64>,
    on_contact: &mut bool,
) {
    let t = language.t();

    // 1. Language Dropdown Selector (Strictly "FR" or "EN")
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
    render_zoom_combo(ui, zoom_factor, *language);

    ui.separator();

    // 2.5. Contact / Feedback Dialog Button
    if ui.button(egui::RichText::new(language.contact().btn_contact).size(12.0))
        .on_hover_text(language.contact().dialog_desc)
        .clicked()
    {
        *on_contact = true;
    }

    ui.separator();

    // 3. Application Update Button
    let egui_ctx = ui.ctx().clone();
    updater.render_top_bar_button(ui, &egui_ctx, *language);

    ui.separator();

    // 4. Refresh / Sync Button
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

    // 5. Snapshot time-travel dropdown
    render_snapshot_combo(ui, snapshots, selected_snapshot_id, on_switch_snapshot);
}

pub fn render_utility_controls_ltr(
    ui: &mut egui::Ui,
    language: &mut Language,
    zoom_factor: &mut f32,
    is_syncing: bool,
    snapshots: &[SnapshotMeta],
    selected_snapshot_id: Option<i64>,
    updater: &mut UpdaterState,
    on_sync: &mut bool,
    on_switch_snapshot: &mut Option<i64>,
    on_contact: &mut bool,
) {
    let t = language.t();

    render_snapshot_combo(ui, snapshots, selected_snapshot_id, on_switch_snapshot);

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

    let egui_ctx = ui.ctx().clone();
    updater.render_top_bar_button(ui, &egui_ctx, *language);

    if ui.button(egui::RichText::new(language.contact().btn_contact).size(12.0)).clicked() {
        *on_contact = true;
    }

    render_zoom_combo(ui, zoom_factor, *language);

    egui::ComboBox::from_id_salt("top_language_combobox_ltr")
        .selected_text(language.label())
        .width(55.0)
        .show_ui(ui, |ui| {
            for lang in Language::all() {
                if ui.selectable_label(*language == lang, lang.label()).clicked() {
                    *language = lang;
                }
            }
        });
}

pub fn render_zoom_combo(ui: &mut egui::Ui, zoom_factor: &mut f32, language: Language) {
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
}

pub fn render_snapshot_combo(
    ui: &mut egui::Ui,
    snapshots: &[SnapshotMeta],
    selected_snapshot_id: Option<i64>,
    on_switch_snapshot: &mut Option<i64>,
) {
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
}
