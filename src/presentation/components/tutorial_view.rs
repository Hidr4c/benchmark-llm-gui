use eframe::egui;
use crate::presentation::components::top_bar::AppTab;
use crate::presentation::components::tour_overlay::TourState;
use crate::presentation::filtering::FilterState;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_tutorial_view(
    ui: &mut egui::Ui,
    tour_state: &mut TourState,
    active_tab: &mut AppTab,
    filters: &mut FilterState,
    lang: Language,
) {
    let t_tools = lang.tools();
    let subtab_key = egui::Id::new("tutorial_selected_subtab");
    let mut selected_subtab = ui.ctx().memory_mut(|m| m.data.get_temp::<usize>(subtab_key)).unwrap_or(0);

    // Segmented header switcher
    ui.horizontal(|ui| {
        let is_tab0 = selected_subtab == 0;
        let btn0 = if is_tab0 {
            ui.button(egui::RichText::new(t_tools.tab_overview).strong().color(ThemeTokens::CYAN_EDGE).size(13.0))
        } else {
            ui.button(egui::RichText::new(t_tools.tab_overview).color(ThemeTokens::TEXT_MUTED).size(13.0))
        };
        if btn0.clicked() {
            selected_subtab = 0;
            ui.ctx().memory_mut(|m| m.data.insert_temp(subtab_key, 0));
        }

        ui.add_space(8.0);

        let is_tab1 = selected_subtab == 1;
        let btn1 = if is_tab1 {
            ui.button(egui::RichText::new(t_tools.tab_install_guide).strong().color(ThemeTokens::GOLD_5090).size(13.0))
        } else {
            ui.button(egui::RichText::new(t_tools.tab_install_guide).color(ThemeTokens::TEXT_MUTED).size(13.0))
        };
        if btn1.clicked() {
            selected_subtab = 1;
            ui.ctx().memory_mut(|m| m.data.insert_temp(subtab_key, 1));
        }
    });

    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        if selected_subtab == 1 {
            crate::presentation::components::local_ai_tools_view::render_local_ai_tools_guide(ui, lang);
        } else {
            // 1. Hero Banner: Interactive Tour / Auto-Play Launcher
            render_hero_banner(ui, tour_state, lang);

            ui.add_space(14.0);

            // 2. Guide Section: Table & Columns Architecture
            render_columns_guide(ui, active_tab, lang);

            ui.add_space(14.0);

            // 3. Guide Section: Advanced Filters & Confidence Levels
            render_filters_guide(ui, active_tab, filters, lang);

            ui.add_space(14.0);

            // 4. Practical Guides & Key Use Cases
            render_practical_guides(ui, tour_state, lang);

            ui.add_space(16.0);
        }
    });
}

fn render_hero_banner(ui: &mut egui::Ui, tour_state: &mut TourState, lang: Language) {
    let t = lang.t();
    let avail_w = ui.available_width();

    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(18, 24, 38))
        .stroke(egui::Stroke::new(1.5, ThemeTokens::GOLD_5090))
        .inner_margin(14.0)
        .rounding(8.0)
        .show(ui, |ui| {
            if avail_w < 880.0 {
                ui.vertical(|ui| {
                    ui.heading(egui::RichText::new(t.tut_hero_title).color(ThemeTokens::TEXT_PRIMARY).strong().size(15.0));
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        if ui.button(egui::RichText::new(t.tut_btn_launch_tour).size(12.0).strong().color(ThemeTokens::GOLD_5090)).clicked() {
                            tour_state.start();
                        }
                        if ui.button(egui::RichText::new("> Mode Video (Auto-Play)").size(12.0).strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                            tour_state.start_autoplay();
                        }
                    });
                });
            } else {
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new(t.tut_hero_title).color(ThemeTokens::TEXT_PRIMARY).strong().size(15.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new("> Mode Video (Auto-Play)").size(12.0).strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                            tour_state.start_autoplay();
                        }
                        if ui.button(egui::RichText::new(t.tut_btn_launch_tour).size(12.0).strong().color(ThemeTokens::GOLD_5090)).clicked() {
                            tour_state.start();
                        }
                    });
                });
            }

            ui.add_space(4.0);
            ui.label(egui::RichText::new(t.tut_hero_desc).color(ThemeTokens::TEXT_SECONDARY).size(12.0));
        });
}

fn render_columns_guide(ui: &mut egui::Ui, active_tab: &mut AppTab, lang: Language) {
    let t = lang.t();
    let guide = t.tut_guide;
    let avail_w = ui.available_width();

    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(15, 20, 30))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 60, 90)))
        .inner_margin(12.0)
        .rounding(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!(" {}", guide.table_guide_title)).size(14.5).color(ThemeTokens::CYAN_EDGE).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new(format!("{} >", guide.btn_try_cols)).color(ThemeTokens::CYAN_EDGE).strong().size(11.5)).clicked() {
                        *active_tab = AppTab::Table;
                    }
                });
            });
            ui.add_space(2.0);
            ui.label(egui::RichText::new(guide.table_guide_desc).color(ThemeTokens::TEXT_MUTED).size(11.5));
            ui.add_space(8.0);

            if avail_w < 780.0 {
                render_subcard(ui, avail_w - 32.0, guide.col_essential_badge, ThemeTokens::CYAN_EDGE, guide.col_essential_desc);
                ui.add_space(6.0);
                render_subcard(ui, avail_w - 32.0, guide.col_adv_badge, ThemeTokens::GOLD_5090, guide.col_adv_desc);
                ui.add_space(6.0);
                render_subcard(ui, avail_w - 32.0, guide.col_raw_badge, ThemeTokens::SUCCESS, guide.col_raw_desc);
            } else {
                let col_w = ((avail_w - 40.0) / 3.0 - 20.0).max(160.0);
                ui.horizontal(|ui| {
                    render_subcard(ui, col_w, guide.col_essential_badge, ThemeTokens::CYAN_EDGE, guide.col_essential_desc);
                    ui.add_space(8.0);
                    render_subcard(ui, col_w, guide.col_adv_badge, ThemeTokens::GOLD_5090, guide.col_adv_desc);
                    ui.add_space(8.0);
                    render_subcard(ui, col_w, guide.col_raw_badge, ThemeTokens::SUCCESS, guide.col_raw_desc);
                });
            }
        });
}

fn render_filters_guide(ui: &mut egui::Ui, active_tab: &mut AppTab, filters: &mut FilterState, lang: Language) {
    let t = lang.t();
    let guide = t.tut_guide;
    let avail_w = ui.available_width();

    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(15, 20, 30))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 60, 90)))
        .inner_margin(12.0)
        .rounding(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!(" {}", guide.filters_guide_title)).size(14.5).color(ThemeTokens::GOLD_5090).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new(format!("{} >", guide.btn_try_filters)).color(ThemeTokens::GOLD_5090).strong().size(11.5)).clicked() {
                        *active_tab = AppTab::Table;
                        filters.show_advanced = true;
                    }
                });
            });
            ui.add_space(2.0);
            ui.label(egui::RichText::new(guide.filters_guide_desc).color(ThemeTokens::TEXT_MUTED).size(11.5));
            ui.add_space(8.0);

            if avail_w < 780.0 {
                render_subcard(ui, avail_w - 32.0, guide.filter_ram_title, ThemeTokens::CYAN_EDGE, guide.filter_ram_desc);
                ui.add_space(6.0);
                render_subcard(ui, avail_w - 32.0, guide.filter_rel_title, ThemeTokens::SUCCESS, guide.filter_rel_desc);
                ui.add_space(6.0);
                render_subcard(ui, avail_w - 32.0, guide.filter_license_title, ThemeTokens::GOLD_5090, guide.filter_license_desc);
            } else {
                let col_w = ((avail_w - 40.0) / 3.0 - 20.0).max(160.0);
                ui.horizontal(|ui| {
                    render_subcard(ui, col_w, guide.filter_ram_title, ThemeTokens::CYAN_EDGE, guide.filter_ram_desc);
                    ui.add_space(8.0);
                    render_subcard(ui, col_w, guide.filter_rel_title, ThemeTokens::SUCCESS, guide.filter_rel_desc);
                    ui.add_space(8.0);
                    render_subcard(ui, col_w, guide.filter_license_title, ThemeTokens::GOLD_5090, guide.filter_license_desc);
                });
            }
        });
}

fn render_practical_guides(ui: &mut egui::Ui, tour_state: &mut TourState, lang: Language) {
    let t = lang.t();
    let avail_w = ui.available_width();
    ui.label(egui::RichText::new(" Guides Pratiques & Cas d'Usage Cles").size(14.5).color(ThemeTokens::CYAN_EDGE).strong());
    ui.add_space(4.0);

    if avail_w < 780.0 {
        let card_w = (avail_w - 20.0).max(200.0);
        render_card(ui, card_w, "", t.tut_card1_title, t.tut_card1_desc, " Voir le Classement des Modeles", || tour_state.jump_to(2));
        ui.add_space(8.0);
        render_card(ui, card_w, "[~]", t.tut_card2_title, t.tut_card2_desc, " Voir le Graphique Pareto", || tour_state.jump_to(6));
        ui.add_space(8.0);
        render_card(ui, card_w, "", t.tut_card3_title, t.tut_card3_desc, " Ouvrir le Simulateur NAS", || tour_state.jump_to(8));
        ui.add_space(8.0);
        render_card(ui, card_w, "", t.tut_card4_title, t.tut_card4_desc, " Inspecter les Benchmarks", || tour_state.jump_to(5));
    } else {
        let card_w = ((avail_w - 36.0) * 0.5 - 24.0).max(180.0);
        ui.horizontal(|ui| {
            render_card(ui, card_w, "", t.tut_card1_title, t.tut_card1_desc, " Voir le Classement des Modeles", || tour_state.jump_to(2));
            ui.add_space(12.0);
            render_card(ui, card_w, "[~]", t.tut_card2_title, t.tut_card2_desc, " Voir le Graphique Pareto", || tour_state.jump_to(6));
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            render_card(ui, card_w, "", t.tut_card3_title, t.tut_card3_desc, " Ouvrir le Simulateur NAS", || tour_state.jump_to(8));
            ui.add_space(12.0);
            render_card(ui, card_w, "", t.tut_card4_title, t.tut_card4_desc, " Inspecter les Benchmarks", || tour_state.jump_to(5));
        });
    }
}

fn render_subcard(ui: &mut egui::Ui, width: f32, title: &str, accent: egui::Color32, desc: &str) {
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(18, 24, 36))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 48, 70)))
        .inner_margin(10.0)
        .rounding(6.0)
        .show(ui, |ui| {
            ui.set_width(width);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).strong().color(accent).size(12.5));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(desc).color(ThemeTokens::TEXT_SECONDARY).size(11.5));
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
                    if !icon.is_empty() {
                        ui.label(egui::RichText::new(icon).size(14.0));
                    }
                    ui.label(egui::RichText::new(title).strong().color(ThemeTokens::TEXT_PRIMARY).size(13.0));
                });
                ui.add_space(4.0);
                ui.label(egui::RichText::new(desc).color(ThemeTokens::TEXT_SECONDARY).size(11.5));
                ui.add_space(8.0);
                if ui.button(egui::RichText::new(btn_text).color(ThemeTokens::CYAN_EDGE).size(11.5)).clicked() {
                    action();
                }
            });
        });
}
