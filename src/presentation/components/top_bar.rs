use eframe::egui;
use crate::domain::traits::SnapshotMeta;
use crate::presentation::components::top_bar_utils::{render_utility_controls_ltr, render_utility_controls_rtl};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;
use crate::presentation::updater_state::UpdaterState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Table,
    Graph,
    Comparator,
    Simulator,
    Domains,
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
            Self::Domains => match lang { Language::French => "Domaines Métiers & Spécialités", Language::English => "Specialized Domains & Verticals" },
            Self::Sources => t.tab_sources,
            Self::Tutorial => t.tab_tutorial,
        }
    }

    pub fn short_label(&self, lang: Language) -> &'static str {
        match lang {
            Language::French => match self {
                Self::Table => "Classement",
                Self::Graph => "Graphique Pareto",
                Self::Comparator => "Comparateur VS",
                Self::Simulator => "Simulateur",
                Self::Domains => "Domaines",
                Self::Sources => "Sources",
                Self::Tutorial => "Guide IA",
            },
            Language::English => match self {
                Self::Table => "Rankings",
                Self::Graph => "Pareto Chart",
                Self::Comparator => "VS Comparator",
                Self::Simulator => "Simulator",
                Self::Domains => "Specialties",
                Self::Sources => "Sources",
                Self::Tutorial => "AI Guide",
            },
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
    _on_launch_tour: &mut bool,
    on_contact: &mut bool,
) {
    ui.add_space(3.0);

    // --- Row 1: Brand & Top Utilities (Responsive on Narrow Windows) ---
    let avail_w = ui.available_width();
    if avail_w < 780.0 {
        ui.horizontal(|ui| {
            ui.heading(
                egui::RichText::new("LLM Benchmark Explorer")
                    .size(16.5)
                    .strong()
                    .color(ThemeTokens::PRIMARY),
            );
        });
        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| {
            render_utility_controls_ltr(
                ui, language, zoom_factor, is_syncing, snapshots,
                selected_snapshot_id, updater, on_sync, on_switch_snapshot, on_contact,
            );
        });
    } else {
        ui.horizontal(|ui| {
            ui.heading(
                egui::RichText::new("LLM Benchmark Explorer")
                    .size(16.5)
                    .strong()
                    .color(ThemeTokens::PRIMARY),
            );

            // Right-Aligned Global Actions
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                render_utility_controls_rtl(
                    ui, language, zoom_factor, is_syncing, snapshots,
                    selected_snapshot_id, updater, on_sync, on_switch_snapshot, on_contact,
                );
            });
        });
    }

    ui.add_space(4.0);

    // --- Row 2: Real Binder Tabs Navigation (Onglets de classeur) ---
    egui::ScrollArea::horizontal()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);
                let views = [
                    AppTab::Table,
                    AppTab::Graph,
                    AppTab::Comparator,
                    AppTab::Simulator,
                    AppTab::Domains,
                    AppTab::Sources,
                    AppTab::Tutorial,
                ];

                for v in views {
                    let is_sel = *active_tab == v;
                    let (stroke, bg_fill, text_color, font_size) = if is_sel {
                        (
                            egui::Stroke::new(2.0, ThemeTokens::CYAN_EDGE),
                            egui::Color32::from_rgb(16, 68, 108),
                            egui::Color32::WHITE,
                            13.0,
                        )
                    } else {
                        (
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 75, 105)),
                            egui::Color32::from_rgb(30, 40, 58),
                            egui::Color32::from_rgb(205, 222, 245),
                            12.0,
                        )
                    };

                    let btn_text = egui::RichText::new(v.short_label(*language))
                        .size(font_size)
                        .strong()
                        .color(text_color);

                    let tab_btn = egui::Button::new(btn_text)
                        .fill(bg_fill)
                        .stroke(stroke)
                        .rounding(egui::Rounding {
                            nw: 6.0,
                            ne: 6.0,
                            sw: 0.0,
                            se: 0.0,
                        })
                        .min_size(egui::vec2(115.0, 32.0));

                    if ui.add(tab_btn)
                        .on_hover_text(v.label(*language))
                        .clicked()
                    {
                        *active_tab = v;
                    }
                }
            });
        });

    // Continuous baseline connecting the active binder tab to page content
    let rect = ui.available_rect_before_wrap();
    let line_y = ui.cursor().top() - 1.0;
    ui.painter().hline(rect.x_range(), line_y, egui::Stroke::new(2.0, ThemeTokens::CYAN_EDGE));
    ui.add_space(3.0);

    if let Some((msg, is_err)) = status_message {
        if *is_err {
            ui.add_space(2.0);
            ui.label(egui::RichText::new(msg).color(ThemeTokens::DANGER).size(12.0));
        }
    }
}
