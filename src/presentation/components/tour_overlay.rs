use eframe::egui;
use crate::presentation::components::top_bar::AppTab;
use crate::presentation::components::tour_feedback::{get_callout_label, render_transition_feedback};
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
    detected_hardware: Option<&crate::domain::hardware_detector::DetectedHardware>,
    all_models: &[crate::domain::model::ModelBenchmark],
    matching_count: usize,
    total_count: usize,
    lang: Language,
) {
    if !state.is_active {
        return;
    }

    ctx.request_repaint();

    render_quest_log(ctx, state, lang);

    let t = lang.t();
    let total_steps = 9;
    let dt = ctx.input(|i| i.stable_dt).min(0.1);

    state.update_transition(dt, total_steps);
    state.check_and_update_progress(filters, selected_model_id, total_steps);

    if state.auto_play && state.transition_timer.is_none() {
        state.timer += dt;
        if state.timer >= 7.5 {
            state.next(total_steps);
        }
    }

    let (step_tab, target_id, result_id, step_accent, title, desc, action_hint, target_label) = match state.current_step {
        0 => (
            AppTab::Table,
            egui::Id::new("tour_target_scope"),
            egui::Id::new("tour_result_table_rows"),
            ThemeTokens::CYAN_EDGE,
            t.tour_s1_title,
            t.tour_s1_desc,
            t.tour_s1_action,
            t.tour_s1_target,
        ),
        1 => (
            AppTab::Table,
            egui::Id::new("tour_target_detect"),
            egui::Id::new("tour_target_hardware"),
            ThemeTokens::CYAN_EDGE,
            t.tour_s2_title,
            t.tour_s2_desc,
            t.tour_s2_action,
            t.tour_s2_target,
        ),
        2 => (
            AppTab::Table,
            egui::Id::new("tour_target_hardware"),
            egui::Id::new("tour_result_table_rows"),
            ThemeTokens::GOLD_5090,
            t.tour_s3_title,
            t.tour_s3_desc,
            t.tour_s3_action,
            t.tour_s3_target,
        ),
        3 => (
            AppTab::Table,
            egui::Id::new("tour_target_adv_cols"),
            egui::Id::new("tour_result_table_rows"),
            ThemeTokens::CYAN_EDGE,
            t.tut_guide.tour_s3b_title,
            t.tut_guide.tour_s3b_desc,
            t.tut_guide.tour_s3b_action,
            t.tut_guide.tour_s3b_target,
        ),
        4 => (
            AppTab::Table,
            egui::Id::new("tour_target_grouped"),
            egui::Id::new("tour_result_grouped"),
            ThemeTokens::CYAN_EDGE,
            t.tour_s4_title,
            t.tour_s4_desc,
            t.tour_s4_action,
            t.tour_s4_target,
        ),
        5 => (
            AppTab::Table,
            egui::Id::new("tour_target_model_name"),
            egui::Id::new("tour_result_detail_panel"),
            ThemeTokens::GOLD_5090,
            t.tour_s5_title,
            t.tour_s5_desc,
            t.tour_s5_action,
            t.tour_s5_target,
        ),
        6 => (
            AppTab::Graph,
            egui::Id::new("tour_target_recenter_btn"),
            egui::Id::new("tour_result_graph_plot"),
            ThemeTokens::GOLD_5090,
            t.tour_s6_title,
            t.tour_s6_desc,
            t.tour_s6_action,
            t.tour_s6_target,
        ),
        7 => (
            AppTab::Comparator,
            egui::Id::new("tour_target_comp_header"),
            egui::Id::new("tour_result_comp_bars"),
            ThemeTokens::CYAN_EDGE,
            t.tour_s7_title,
            t.tour_s7_desc,
            t.tour_s7_action,
            t.tour_s7_target,
        ),
        _ => (
            AppTab::Simulator,
            egui::Id::new("tour_target_sim_gpu"),
            egui::Id::new("tour_result_sim_kpi"),
            ThemeTokens::CYAN_EDGE,
            t.tour_s8_title,
            t.tour_s8_desc,
            t.tour_s8_action,
            t.tour_s8_target,
        ),
    };

    let in_transition = state.transition_timer.is_some();
    let accent = if in_transition { ThemeTokens::SUCCESS } else { step_accent };

    if *active_tab != step_tab {
        *active_tab = step_tab;
    }

    let active_id = if in_transition { result_id } else { target_id };
    let spotlight_rect: Option<egui::Rect> = ctx.memory_mut(|m| m.data.get_temp(active_id))
        .or_else(|| ctx.memory_mut(|m| m.data.get_temp(target_id)));

    let callout = get_callout_label(state.current_step, in_transition, lang);
    TourSpotlight::draw_spotlight(ctx, spotlight_rect, accent, in_transition, callout);

    let screen = ctx.screen_rect();
    let card_w = 580.0f32.min(screen.width() - 40.0);
    let card_h = if in_transition { 220.0f32 } else { 185.0f32 };
    let card_pos = TourSpotlight::compute_card_position(screen, spotlight_rect, card_w, card_h);

    egui::Window::new("tour_guidance_floating_window")
        .title_bar(false)
        .movable(true)
        .resizable(false)
        .default_pos(card_pos)
        .order(egui::Order::Foreground)
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(14, 18, 28))
                .stroke(egui::Stroke::new(2.0, accent))
                .inner_margin(14.0)
                .rounding(10.0)
        )
        .show(ctx, |ui| {
            ui.set_width(card_w);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("[::]").color(ThemeTokens::TEXT_SECONDARY).size(13.0));
                    let step_badge = format!("{} {} / {} :", t.tour_step_prefix, state.current_step + 1, total_steps);
                    ui.label(egui::RichText::new(step_badge).strong().color(ThemeTokens::GOLD_5090).size(13.5));
                    ui.label(egui::RichText::new(title).strong().color(ThemeTokens::TEXT_PRIMARY).size(14.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let close_btn = egui::Button::new(
                            egui::RichText::new("X").size(13.0).strong().color(ThemeTokens::TEXT_SECONDARY),
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

                if state.transition_timer.is_some() {
                    render_transition_feedback(
                        ui, state, filters, selected_model_id, detected_hardware, all_models, matching_count, total_count, lang, total_steps,
                    );
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

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(egui::RichText::new(t.tour_validate_btn).strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                                        state.validate_current_step();
                                    }
                                });
                            });
                        });
                }

                ui.add_space(6.0);
                ui.label(egui::RichText::new(desc).color(ThemeTokens::TEXT_SECONDARY).size(12.0));

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let can_prev = state.current_step > 0;
                    if ui.add_enabled(can_prev, egui::Button::new(t.tour_prev)).clicked() {
                        state.prev();
                    }

                    let next_lbl = if state.current_step + 1 < total_steps { t.tour_next } else { "Terminer" };
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

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let guide_lbl = format!("{} ({}/{})", t.tour_guide_btn, state.completed_count(), total_steps);
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
}
