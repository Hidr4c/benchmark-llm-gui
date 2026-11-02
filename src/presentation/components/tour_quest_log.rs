use eframe::egui;
use crate::presentation::components::tour_state::TourState;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_quest_log(
    ctx: &egui::Context,
    state: &mut TourState,
    lang: Language,
) {
    if !state.is_active || !state.show_guide_summary {
        return;
    }

    let t = lang.t();
    let screen = ctx.screen_rect();
    let panel_w = 265.0f32;
    let panel_pos = egui::pos2(screen.right() - panel_w - 16.0, screen.top() + 75.0);

    egui::Area::new(egui::Id::new("tour_quest_log_floating_panel"))
        .default_pos(panel_pos)
        .movable(true)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgba_unmultiplied(13, 18, 28, 245))
                .stroke(egui::Stroke::new(1.5, ThemeTokens::CYAN_EDGE))
                .inner_margin(12.0)
                .rounding(8.0)
                .shadow(egui::epaint::Shadow {
                    offset: egui::vec2(0.0, 6.0),
                    blur: 16.0,
                    spread: 2.0,
                    color: egui::Color32::from_black_alpha(180),
                })
                .show(ui, |ui| {
                    ui.set_width(panel_w);
                    ui.vertical(|ui| {
                        // Header: Title, counter, and clean close cross button
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(t.tour_guide_title)
                                    .strong()
                                    .color(ThemeTokens::GOLD_5090)
                                    .size(13.5),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let close_btn = egui::Button::new(
                                    egui::RichText::new("×")
                                        .size(15.0)
                                        .strong()
                                        .color(ThemeTokens::TEXT_SECONDARY),
                                )
                                .fill(egui::Color32::from_rgb(26, 32, 46))
                                .rounding(4.0);
                                if ui.add(close_btn).on_hover_text(t.tour_close).clicked() {
                                    state.show_guide_summary = false;
                                }

                                let completed = state.completed_count();
                                let badge_color = if completed == 7 { ThemeTokens::SUCCESS } else { ThemeTokens::CYAN_EDGE };
                                ui.label(
                                    egui::RichText::new(format!("{completed}/7"))
                                        .strong()
                                        .color(badge_color)
                                        .size(12.0),
                                );
                            });
                        });

                        ui.add_space(4.0);
                        let progress = state.completed_count() as f32 / 7.0;
                        ui.add(
                            egui::ProgressBar::new(progress)
                                .fill(ThemeTokens::CYAN_EDGE)
                                .animate(true),
                        );
                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // Listing of the 7 interactive steps
                        let step_targets = [
                            (0, t.tour_s1_target, t.tour_s1_action),
                            (1, t.tour_s2_target, t.tour_s2_action),
                            (2, t.tour_s3_target, t.tour_s3_action),
                            (3, t.tour_s4_target, t.tour_s4_action),
                            (4, t.tour_s5_target, t.tour_s5_action),
                            (5, t.tour_s6_target, t.tour_s6_action),
                            (6, t.tour_s7_target, t.tour_s7_action),
                        ];

                        for (idx, target_name, action_hint) in step_targets {
                            let is_current = state.current_step == idx;
                            let is_done = state.completed_steps[idx];

                            let (icon, icon_color) = if is_done {
                                ("✅", ThemeTokens::SUCCESS)
                            } else if is_current {
                                ("👉", ThemeTokens::GOLD_5090)
                            } else {
                                ("⬜", ThemeTokens::TEXT_MUTED)
                            };

                            let title_text = egui::RichText::new(format!("{} {}", idx + 1, target_name))
                                .size(12.0)
                                .color(if is_current {
                                    ThemeTokens::TEXT_PRIMARY
                                } else if is_done {
                                    ThemeTokens::SUCCESS
                                } else {
                                    ThemeTokens::TEXT_SECONDARY
                                });

                            let row_bg = if is_current {
                                egui::Color32::from_rgb(24, 34, 52)
                            } else {
                                egui::Color32::TRANSPARENT
                            };

                            let row_resp = egui::Frame::none()
                                .fill(row_bg)
                                .rounding(4.0)
                                .inner_margin(egui::vec2(4.0, 3.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(icon).color(icon_color).size(12.0));
                                        let mut label = title_text;
                                        if is_current {
                                            label = label.strong();
                                        }
                                        ui.label(label);
                                    });
                                })
                                .response;

                            let clickable_area = ui.interact(
                                row_resp.rect,
                                egui::Id::new(format!("guide_step_item_{idx}")),
                                egui::Sense::click(),
                            );

                            if clickable_area.on_hover_text(action_hint).clicked() {
                                state.jump_to(idx);
                            }
                        }
                    });
                });
        });
}
