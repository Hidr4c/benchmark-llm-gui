use eframe::egui;
use crate::presentation::components::top_bar::AppTab;
use crate::presentation::components::tour_quest_log::render_quest_log;
use crate::presentation::components::tour_spotlight::TourSpotlight;
pub use crate::presentation::components::tour_state::TourState;
use crate::presentation::filtering::FilterState;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_tour_overlay(
    ctx: &egui::Context,
    state: &mut TourState,
    active_tab: &mut AppTab,
    filters: &FilterState,
    selected_model_id: Option<&str>,
    lang: Language,
) {
    if !state.is_active {
        return;
    }

    ctx.request_repaint();

    // Render interactive Guide Summary side panel
    render_quest_log(ctx, state, lang);

    let t = lang.t();
    let total_steps = 7;
    let dt = ctx.input(|i| i.stable_dt).min(0.1);

    // Update action observation timer before auto-advancing
    state.update_transition(dt, total_steps);

    // Gamified Auto-Advance: Detect when user interacts with the targeted widget
    state.check_and_update_progress(filters, selected_model_id, total_steps);

    // Advance autoplay timer
    if state.auto_play && state.transition_timer.is_none() {
        state.timer += dt;
        if state.timer >= 7.5 {
            state.next(total_steps);
        }
    }

    let (step_tab, target_id, step_accent, title, desc, action_hint, target_label) = match state.current_step {
        0 => (
            AppTab::Table,
            egui::Id::new("tour_target_scope"),
            ThemeTokens::CYAN_EDGE,
            t.tour_s1_title,
            t.tour_s1_desc,
            t.tour_s1_action,
            t.tour_s1_target,
        ),
        1 => (
            AppTab::Table,
            egui::Id::new("tour_target_detect"),
            ThemeTokens::CYAN_EDGE,
            t.tour_s2_title,
            t.tour_s2_desc,
            t.tour_s2_action,
            t.tour_s2_target,
        ),
        2 => (
            AppTab::Table,
            egui::Id::new("tour_target_hardware"),
            ThemeTokens::GOLD_5090,
            t.tour_s3_title,
            t.tour_s3_desc,
            t.tour_s3_action,
            t.tour_s3_target,
        ),
        3 => (
            AppTab::Table,
            egui::Id::new("tour_target_grouped"),
            ThemeTokens::CYAN_EDGE,
            t.tour_s4_title,
            t.tour_s4_desc,
            t.tour_s4_action,
            t.tour_s4_target,
        ),
        4 => (
            AppTab::Table,
            egui::Id::new("tour_target_table_row"),
            ThemeTokens::GOLD_5090,
            t.tour_s5_title,
            t.tour_s5_desc,
            t.tour_s5_action,
            t.tour_s5_target,
        ),
        5 => (
            AppTab::Graph,
            egui::Id::new("tour_target_graph_area"),
            ThemeTokens::GOLD_5090,
            t.tour_s6_title,
            t.tour_s6_desc,
            t.tour_s6_action,
            t.tour_s6_target,
        ),
        _ => (
            AppTab::Simulator,
            egui::Id::new("tour_target_sim_area"),
            ThemeTokens::CYAN_EDGE,
            t.tour_s7_title,
            t.tour_s7_desc,
            t.tour_s7_action,
            t.tour_s7_target,
        ),
    };

    let accent = if state.transition_timer.is_some() {
        ThemeTokens::SUCCESS
    } else {
        step_accent
    };

    if *active_tab != step_tab {
        *active_tab = step_tab;
    }

    let target_rect: Option<egui::Rect> = ctx.memory_mut(|m| m.data.get_temp(target_id));
    TourSpotlight::draw_spotlight(ctx, target_rect, accent);

    let screen = ctx.screen_rect();
    let card_w = 640.0f32.min(screen.width() - 48.0);
    let card_h = 190.0f32;
    let card_pos = TourSpotlight::compute_card_position(screen, target_rect, card_w, card_h, state.show_guide_summary);

    egui::Area::new(egui::Id::new("tour_interactive_guidance_card"))
        .fixed_pos(card_pos)
        .pivot(egui::Align2::LEFT_TOP)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgb(14, 18, 28))
                .stroke(egui::Stroke::new(2.0, accent))
                .inner_margin(16.0)
                .rounding(10.0)
                .shadow(egui::epaint::Shadow {
                    offset: egui::vec2(0.0, 8.0),
                    blur: 20.0,
                    spread: 2.0,
                    color: egui::Color32::from_black_alpha(200),
                })
                .show(ui, |ui| {
                    ui.set_width(card_w);
                    ui.vertical(|ui| {
                        // Header: Step counter, localized title, target badge, clean close cross
                        ui.horizontal(|ui| {
                            let step_badge = format!("🎬 {} {} / {} :", t.tour_step_prefix, state.current_step + 1, total_steps);
                            ui.label(egui::RichText::new(step_badge).strong().color(ThemeTokens::GOLD_5090).size(13.5));
                            ui.label(egui::RichText::new(title).strong().color(ThemeTokens::TEXT_PRIMARY).size(14.0));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let close_btn = egui::Button::new(
                                    egui::RichText::new("×")
                                        .size(16.0)
                                        .strong()
                                        .color(ThemeTokens::TEXT_SECONDARY),
                                )
                                .fill(egui::Color32::from_rgb(26, 32, 46))
                                .rounding(4.0);
                                if ui.add(close_btn).on_hover_text(t.tour_close).clicked() {
                                    state.close();
                                }
                                ui.label(egui::RichText::new(format!("[ {target_label} ]")).strong().color(accent).size(12.0));
                            });
                        });

                        ui.add_space(6.0);

                        // Action Directive Banner OR Observation Confirmation Banner
                        if let Some(remaining) = state.transition_timer {
                            let feedback = state.transition_feedback.clone().unwrap_or_else(|| t.tour_step_validated.to_string());
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(18, 42, 28))
                                .stroke(egui::Stroke::new(1.5, ThemeTokens::SUCCESS))
                                .inner_margin(8.0)
                                .rounding(6.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("✓").strong().color(ThemeTokens::SUCCESS).size(13.5));
                                        ui.label(egui::RichText::new(feedback).color(ThemeTokens::SUCCESS).strong().size(12.5));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button(egui::RichText::new(t.tour_skip_transition).size(11.0).color(ThemeTokens::CYAN_EDGE)).clicked() {
                                                state.next(total_steps);
                                            }
                                            ui.label(egui::RichText::new(format!("({:.1}s)", remaining)).color(ThemeTokens::TEXT_SECONDARY).size(11.0));
                                        });
                                    });
                                });
                        } else {
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(20, 29, 46))
                                .stroke(egui::Stroke::new(1.0, accent))
                                .inner_margin(8.0)
                                .rounding(6.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(t.tour_action_label).strong().color(accent).size(12.5));
                                        ui.label(egui::RichText::new(action_hint).color(ThemeTokens::TEXT_PRIMARY).strong().size(12.5));
                                    });
                                });
                        }

                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(desc).color(ThemeTokens::TEXT_SECONDARY).size(12.0));

                        ui.add_space(8.0);
                        // Bottom Navigation Toolbar
                        ui.horizontal(|ui| {
                            let can_prev = state.current_step > 0;
                            if ui.add_enabled(can_prev, egui::Button::new(t.tour_prev)).clicked() {
                                state.prev();
                            }

                            let next_lbl = if state.current_step + 1 < total_steps { t.tour_next } else { "🏁 Terminer" };
                            if ui.button(egui::RichText::new(next_lbl).strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                                state.next(total_steps);
                            }

                            ui.separator();

                            let auto_btn = if state.auto_play {
                                egui::RichText::new(format!("{} ({:.0}s)", t.tour_pause, 7.5 - state.timer)).color(ThemeTokens::GOLD_5090)
                            } else {
                                egui::RichText::new(t.tour_autoplay).color(ThemeTokens::SUCCESS)
                            };
                            if ui.button(auto_btn).clicked() {
                                state.auto_play = !state.auto_play;
                                state.timer = 0.0;
                            }

                            // Interactive Guide Summary toggle and step progress indicator
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let guide_lbl = format!("{} ({}/7)", t.tour_guide_btn, state.completed_count());
                                let guide_btn = egui::Button::new(
                                    egui::RichText::new(guide_lbl)
                                        .color(if state.show_guide_summary { ThemeTokens::GOLD_5090 } else { ThemeTokens::CYAN_EDGE })
                                        .strong()
                                        .size(12.0),
                                );
                                if ui.add(guide_btn).clicked() {
                                    state.show_guide_summary = !state.show_guide_summary;
                                }

                                ui.label(
                                    egui::RichText::new(format!("{} {} / {}", t.tour_step_prefix, state.current_step + 1, total_steps))
                                        .color(ThemeTokens::TEXT_SECONDARY)
                                        .size(12.0),
                                );
                            });
                        });
                    });
                });
        });
}
