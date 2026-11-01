use eframe::egui;
use crate::domain::sources::{BenchmarkSourceInfo, SourceCategory};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_sources_view(ui: &mut egui::Ui, lang: Language) {
    let t = lang.t();
    let all = BenchmarkSourceInfo::all_sources();

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new(t.sources_title).color(ThemeTokens::TEXT_PRIMARY).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(format!("{} {}", all.len(), t.sources_count_suffix)).color(ThemeTokens::CYAN_EDGE).strong());
            });
        });

        ui.label(egui::RichText::new(t.sources_subtitle).color(ThemeTokens::TEXT_SECONDARY));
        ui.add_space(12.0);

        let categories = [
            SourceCategory::AcademicGeneral,
            SourceCategory::CodingDevelopment,
            SourceCategory::AgenticTools,
            SourceCategory::HardwareSpeed,
            SourceCategory::CommunityLibrary,
            SourceCategory::CloudProprietary,
        ];

        for cat in categories {
            let cat_sources: Vec<&BenchmarkSourceInfo> = all.iter().filter(|s| s.category == cat).collect();
            if cat_sources.is_empty() {
                continue;
            }

            ui.add_space(8.0);
            ui.label(egui::RichText::new(cat.label()).size(16.0).color(ThemeTokens::PRIMARY).strong());
            ui.separator();

            for s in cat_sources {
                egui::Frame::group(ui.style())
                    .fill(ThemeTokens::BG_CARD)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(s.name).size(14.0).strong().color(ThemeTokens::TEXT_PRIMARY));

                            if s.is_connected_live {
                                ui.label(egui::RichText::new(t.sources_live_badge).color(ThemeTokens::SUCCESS).size(11.0));
                            } else {
                                ui.label(egui::RichText::new(t.sources_doc_badge).color(ThemeTokens::TEXT_MUTED).size(11.0));
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(t.view_online).on_hover_text(s.url).clicked() {
                                    let _ = open::that(s.url);
                                }
                            });
                        });

                        ui.add_space(3.0);
                        ui.label(egui::RichText::new(s.description).color(ThemeTokens::TEXT_SECONDARY));

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(t.sources_metrics_label).color(ThemeTokens::TEXT_MUTED).size(12.0).strong());
                            ui.label(egui::RichText::new(s.metrics_measured).color(ThemeTokens::CYAN_EDGE).size(12.0));
                        });
                    });
                ui.add_space(4.0);
            }
        }
        ui.add_space(20.0);
    });
}
