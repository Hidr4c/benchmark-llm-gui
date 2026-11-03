use eframe::egui;
use crate::presentation::components::tour_overlay::TourState;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_tutorial_view(
    ui: &mut egui::Ui,
    tour_state: &mut TourState,
    lang: Language,
) {
    let t = lang.t();

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(6.0);

        // 1. Hero Banner: Interactive Video / Walkthrough Launcher
        egui::Frame::group(ui.style())
            .fill(egui::Color32::from_rgb(18, 24, 38))
            .stroke(egui::Stroke::new(1.5, ThemeTokens::GOLD_5090))
            .inner_margin(14.0)
            .rounding(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🎬").size(32.0));
                    ui.vertical(|ui| {
                        ui.heading(egui::RichText::new(t.tut_hero_title).color(ThemeTokens::TEXT_PRIMARY).strong().size(15.0));
                        ui.label(egui::RichText::new(t.tut_hero_desc).color(ThemeTokens::TEXT_SECONDARY).size(12.5));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new("▶ Mode Vidéo (Auto-Play)").size(12.5).strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                            tour_state.start_autoplay();
                        }
                        if ui.button(egui::RichText::new(t.tut_btn_launch_tour).size(12.5).strong().color(ThemeTokens::GOLD_5090)).clicked() {
                            tour_state.start();
                        }
                    });
                });
            });

        ui.add_space(12.0);
        ui.label(egui::RichText::new("📖 Guides Pratiques & Cas d'Usage Clés").size(15.0).color(ThemeTokens::CYAN_EDGE).strong());
        ui.add_space(4.0);

        // 2. Interactive Guide Cards (2x2 Grid)
        let card_w = (ui.available_width() - 16.0) * 0.5;

        ui.horizontal(|ui| {
            render_card(ui, card_w, "🎯", t.tut_card1_title, t.tut_card1_desc, "👉 Voir le Tableau Matériel", || tour_state.jump_to(2));
            ui.add_space(12.0);
            render_card(ui, card_w, "⚡", t.tut_card2_title, t.tut_card2_desc, "👉 Voir le Graphique Pareto", || tour_state.jump_to(5));
        });

        ui.add_space(12.0);

        ui.horizontal(|ui| {
            render_card(ui, card_w, "🛡️", t.tut_card3_title, t.tut_card3_desc, "👉 Ouvrir le Simulateur NAS", || tour_state.jump_to(6));
            ui.add_space(12.0);
            render_card(ui, card_w, "🔬", t.tut_card4_title, t.tut_card4_desc, "👉 Inspecter les Benchmarks", || tour_state.jump_to(4));
        });
    });
}

fn render_card<F: FnOnce()>(
    ui: &mut egui::Ui,
    width: f32,
    icon: &'static str,
    title: &str,
    desc: &str,
    btn_text: &str,
    action: F,
) {
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(16, 21, 32))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 65, 95)))
        .inner_margin(12.0)
        .rounding(6.0)
        .show(ui, |ui| {
            ui.set_width(width);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(icon).size(18.0));
                    ui.label(egui::RichText::new(title).strong().color(ThemeTokens::TEXT_PRIMARY).size(13.5));
                });
                ui.add_space(4.0);
                ui.label(egui::RichText::new(desc).color(ThemeTokens::TEXT_SECONDARY).size(12.0));
                ui.add_space(8.0);
                if ui.button(egui::RichText::new(btn_text).color(ThemeTokens::CYAN_EDGE).size(11.5)).clicked() {
                    action();
                }
            });
        });
}
