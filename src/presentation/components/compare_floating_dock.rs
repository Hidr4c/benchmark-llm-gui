use eframe::egui;
use crate::domain::model::ModelBenchmark;
use crate::presentation::filtering::FilterState;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_floating_compare_dock(
    ctx: &egui::Context,
    filters: &mut FilterState,
    all_models: &[ModelBenchmark],
    lang: Language,
) {
    let dock = lang.dock();
    if filters.compared_model_ids.is_empty() {
        return;
    }

    egui::Area::new(egui::Id::new("floating_compare_action_dock"))
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -22.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(14, 20, 32))
                .stroke(egui::Stroke::new(1.5, ThemeTokens::CYAN_EDGE))
                .rounding(16.0)
                .shadow(egui::epaint::Shadow {
                    offset: egui::vec2(0.0, 6.0),
                    blur: 18.0,
                    spread: 2.0,
                    color: egui::Color32::from_black_alpha(220),
                })
                .inner_margin(egui::Margin::symmetric(14.0, 7.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Slot 1 (Cyan)
                        if let Some(id1) = filters.compared_model_ids.get(0) {
                            let name1 = all_models.iter().find(|m| &m.id == id1).map(|m| m.name.as_str()).unwrap_or(id1);
                            ui.label(
                                egui::RichText::new(format!("[#1] {}", name1))
                                    .color(ThemeTokens::CYAN_EDGE)
                                    .strong()
                                    .size(12.0),
                            );
                        }

                        ui.label(egui::RichText::new("VS").strong().color(ThemeTokens::TEXT_MUTED).size(11.0));

                        // Slot 2 (Gold)
                        if let Some(id2) = filters.compared_model_ids.get(1) {
                            let name2 = all_models.iter().find(|m| &m.id == id2).map(|m| m.name.as_str()).unwrap_or(id2);
                            ui.label(
                                egui::RichText::new(format!("[#2] {}", name2))
                                    .color(ThemeTokens::GOLD_5090)
                                    .strong()
                                    .size(12.0),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new(dock.prompt_pick_second)
                                    .italics()
                                    .color(ThemeTokens::TEXT_MUTED)
                                    .size(11.0),
                            );
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        let is_ready = filters.compared_model_ids.len() >= 2;
                        let btn_text = if is_ready { dock.btn_launch } else { dock.btn_waiting };
                        let btn = egui::Button::new(
                            egui::RichText::new(btn_text)
                                .strong()
                                .color(if is_ready { egui::Color32::WHITE } else { ThemeTokens::TEXT_MUTED })
                                .size(12.0),
                        )
                        .fill(if is_ready { ThemeTokens::PRIMARY } else { egui::Color32::from_rgb(26, 34, 48) })
                        .rounding(10.0);

                        if ui.add_enabled(is_ready, btn).clicked() {
                            ctx.memory_mut(|m| m.data.insert_temp(egui::Id::new("req_tab_switch_comparator"), true));
                        }

                        if ui.small_button(egui::RichText::new("X").color(ThemeTokens::TEXT_SECONDARY).strong()).clicked() {
                            filters.compared_model_ids.clear();
                        }
                    });
                });
        });
}
