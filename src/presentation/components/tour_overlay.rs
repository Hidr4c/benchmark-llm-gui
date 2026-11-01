use eframe::egui;
use crate::presentation::components::top_bar::AppTab;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub struct TourState {
    pub is_active: bool,
    pub current_step: usize,
    pub auto_play: bool,
    pub timer: f32,
}

impl Default for TourState {
    fn default() -> Self {
        Self {
            is_active: false,
            current_step: 0,
            auto_play: false,
            timer: 0.0,
        }
    }
}

impl TourState {
    pub fn start(&mut self) {
        self.is_active = true;
        self.current_step = 0;
        self.auto_play = false;
        self.timer = 0.0;
    }

    pub fn start_autoplay(&mut self) {
        self.is_active = true;
        self.current_step = 0;
        self.auto_play = true;
        self.timer = 0.0;
    }

    pub fn close(&mut self) {
        self.is_active = false;
        self.auto_play = false;
        self.timer = 0.0;
    }

    pub fn jump_to(&mut self, step: usize) {
        self.is_active = true;
        self.current_step = step.min(6);
        self.auto_play = false;
        self.timer = 0.0;
    }

    pub fn next(&mut self, total_steps: usize) {
        if self.current_step + 1 < total_steps {
            self.current_step += 1;
            self.timer = 0.0;
        } else {
            self.close();
        }
    }

    pub fn prev(&mut self) {
        if self.current_step > 0 {
            self.current_step -= 1;
            self.timer = 0.0;
        }
    }
}

pub fn render_tour_overlay(
    ctx: &egui::Context,
    state: &mut TourState,
    active_tab: &mut AppTab,
    lang: Language,
) {
    if !state.is_active {
        return;
    }

    let t = lang.t();
    let total_steps = 7;

    // Advance autoplay timer
    if state.auto_play {
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        state.timer += dt;
        if state.timer >= 6.5 {
            state.next(total_steps);
        }
        ctx.request_repaint();
    }

    let (step_tab, pointer_icon, title, desc) = match state.current_step {
        0 => (AppTab::Table, "👉 [ Périmètre ]", t.tour_s1_title, t.tour_s1_desc),
        1 => (AppTab::Table, "👉 [ 🎯 Détecter mon PC ]", t.tour_s2_title, t.tour_s2_desc),
        2 => (AppTab::Table, "👉 [ 🖥️ Matériel / VRAM ]", t.tour_s3_title, t.tour_s3_desc),
        3 => (AppTab::Table, "👇 [ ☑ Modèles Racines (Groupé) ]", t.tour_s4_title, t.tour_s4_desc),
        4 => (AppTab::Table, "👇 [ 💰 Fiche d'Inspection & Prix ]", t.tour_s5_title, t.tour_s5_desc),
        5 => (AppTab::Graph, "👉 [ 📈 Courbe de Pareto & Électricité ]", t.tour_s6_title, t.tour_s6_desc),
        _ => (AppTab::Simulator, "👉 [ 🛠️ Simulateur Matériel & NAS ]", t.tour_s7_title, t.tour_s7_desc),
    };

    // Synchronize active tab automatically with the step being presented
    if *active_tab != step_tab {
        *active_tab = step_tab;
    }

    let screen_rect = ctx.screen_rect();
    let overlay_pos = egui::pos2(screen_rect.center().x, screen_rect.max.y - 145.0);

    egui::Area::new(egui::Id::new("tour_interactive_overlay"))
        .fixed_pos(overlay_pos)
        .pivot(egui::Align2::CENTER_CENTER)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgb(14, 18, 28))
                .stroke(egui::Stroke::new(2.0, ThemeTokens::GOLD_5090))
                .inner_margin(14.0)
                .rounding(10.0)
                .show(ui, |ui| {
                    ui.set_max_width(740.0);
                    ui.vertical(|ui| {
                        // Header with step badge and target indicator
                        ui.horizontal(|ui| {
                            let step_badge = format!("🎬 {} {} / {} :", t.tour_step_prefix, state.current_step + 1, total_steps);
                            ui.label(egui::RichText::new(step_badge).strong().color(ThemeTokens::GOLD_5090).size(13.0));
                            ui.label(egui::RichText::new(title).strong().color(ThemeTokens::TEXT_PRIMARY).size(14.0));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(egui::RichText::new(t.tour_close).strong().size(11.0)).clicked() {
                                    state.close();
                                }
                                ui.label(egui::RichText::new(pointer_icon).strong().color(ThemeTokens::CYAN_EDGE).size(12.0));
                            });
                        });

                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(desc).color(ThemeTokens::TEXT_SECONDARY).size(12.5));

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
                                egui::RichText::new(format!("{} ({:.0}s)", t.tour_pause, 6.5 - state.timer)).color(ThemeTokens::GOLD_5090)
                            } else {
                                egui::RichText::new(t.tour_autoplay).color(ThemeTokens::SUCCESS)
                            };
                            if ui.button(auto_btn).clicked() {
                                state.auto_play = !state.auto_play;
                                state.timer = 0.0;
                            }

                            // Step dots / progress indicator
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                for i in (0..total_steps).rev() {
                                    let is_cur = i == state.current_step;
                                    let dot = if is_cur { "●" } else { "○" };
                                    let color = if is_cur { ThemeTokens::GOLD_5090 } else { ThemeTokens::TEXT_MUTED };
                                    if ui.button(egui::RichText::new(dot).color(color).size(13.0)).clicked() {
                                        state.jump_to(i);
                                    }
                                }
                            });
                        });
                    });
                });
        });
}
