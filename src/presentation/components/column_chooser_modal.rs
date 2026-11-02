use eframe::egui;
use crate::domain::table_preferences::{TableColumnId, TablePreferences};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_column_chooser_modal(
    ctx: &egui::Context,
    open: &mut bool,
    prefs: &mut TablePreferences,
    lang: Language,
) {
    if !*open {
        return;
    }

    let t = lang.t();
    let mut win_open = true;
    let mut should_close = false;

    egui::Window::new(egui::RichText::new("⚙ Sélecteur de Colonnes & Agencement").strong().color(ThemeTokens::CYAN_EDGE))
        .open(&mut win_open)
        .collapsible(false)
        .resizable(true)
        .default_size(egui::vec2(440.0, 480.0))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("Cochez les colonnes à afficher et utilisez les flèches pour modifier leur ordre d'apparition :").size(11.5).color(ThemeTokens::TEXT_SECONDARY));
            ui.add_space(8.0);

            let cols = prefs.column_order.clone();
            let total_cols = cols.len();
            let mut move_op: Option<(TableColumnId, isize)> = None;

            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgb(18, 24, 34))
                .inner_margin(8.0)
                .rounding(6.0)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
                        for (idx, col) in cols.into_iter().enumerate() {
                            ui.horizontal(|ui| {
                                let mut visible = prefs.is_visible(col);
                                if ui.checkbox(&mut visible, "").changed() {
                                    prefs.toggle_visibility(col);
                                }

                                let label = column_display_name(col, t);
                                let col_color = if visible { ThemeTokens::TEXT_PRIMARY } else { ThemeTokens::TEXT_MUTED };
                                ui.label(egui::RichText::new(label).color(col_color).strong().size(12.0));

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let down_enabled = idx < total_cols - 1;
                                    let up_enabled = idx > 0;

                                    if ui.add_enabled(down_enabled, egui::Button::new("⬇")).on_hover_text("Déplacer vers le bas (droite)").clicked() {
                                        move_op = Some((col, 1));
                                    }
                                    if ui.add_enabled(up_enabled, egui::Button::new("⬆")).on_hover_text("Déplacer vers le haut (gauche)").clicked() {
                                        move_op = Some((col, -1));
                                    }
                                });
                            });
                            ui.separator();
                        }
                    });
                });

            if let Some((col, delta)) = move_op {
                prefs.move_column(col, delta);
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new("↺ Rétablir tout").size(11.0)).on_hover_text("Rétablir l'ordre d'origine et réafficher toutes les colonnes").clicked() {
                    prefs.reset_order();
                    prefs.show_all_columns();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("Fermer").strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                        should_close = true;
                    }
                });
            });
        });

    if should_close || !win_open {
        *open = false;
    }
}

fn column_display_name(col: TableColumnId, t: &'static crate::presentation::i18n::Translations) -> &'static str {
    match col {
        TableColumnId::Rank => t.col_rank,
        TableColumnId::Model => t.col_model,
        TableColumnId::Year => t.col_year,
        TableColumnId::TargetHardware => t.col_hardware,
        TableColumnId::Size => t.col_size,
        TableColumnId::Efficiency => t.col_efficiency,
        TableColumnId::Score => t.col_score,
        TableColumnId::RealWorld => t.col_real_world,
        TableColumnId::Speed => t.col_speed,
        TableColumnId::Popularity => t.col_popularity,
        TableColumnId::MmluPro => t.col_mmlu_pro,
        TableColumnId::SweBench => t.col_swe_bench,
        TableColumnId::Agentic => t.col_agentic,
        TableColumnId::OpenLlm => t.col_open_llm,
        TableColumnId::CloudPrice => t.col_cloud_price,
    }
}
