use eframe::egui;
use crate::presentation::i18n::sub_translations_tools::ToolCardData;
use crate::presentation::i18n::{Language, TOOLS_LIST};
use crate::presentation::theme::ThemeTokens;

pub fn render_local_ai_tools_guide(ui: &mut egui::Ui, lang: Language) {
    let t = lang.tools();

    // 1. Hero Banner
    render_hero_banner(ui, t);
    ui.add_space(14.0);

    // 2. Step-by-step 4-step quickstart roadmap
    render_roadmap(ui, t);
    ui.add_space(16.0);

    // 3. Grid-Off & Smartphone Special Section
    render_off_grid_section(ui, t);
    ui.add_space(16.0);

    // 4. Tools Catalog Grid (LM Studio, Ollama, PocketPal AI, Jan, etc.)
    render_tools_catalog(ui, t);
    ui.add_space(16.0);
}

fn render_hero_banner(ui: &mut egui::Ui, t: &'static crate::presentation::i18n::LocalToolsTranslations) {
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(18, 24, 38))
        .stroke(egui::Stroke::new(1.5, ThemeTokens::CYAN_EDGE))
        .inner_margin(14.0)
        .rounding(8.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("[Local AI]").strong().color(ThemeTokens::CYAN_EDGE).size(14.0));
                    ui.heading(egui::RichText::new(t.hero_title).color(ThemeTokens::TEXT_PRIMARY).strong().size(15.0));
                });
                ui.add_space(4.0);
                ui.label(egui::RichText::new(t.hero_subtitle).color(ThemeTokens::TEXT_SECONDARY).size(12.0));
            });
        });
}

fn render_roadmap(ui: &mut egui::Ui, t: &'static crate::presentation::i18n::LocalToolsTranslations) {
    ui.label(egui::RichText::new(t.section_steps).size(14.5).color(ThemeTokens::GOLD_5090).strong());
    ui.add_space(6.0);

    let steps = [
        (t.step1_title, t.step1_desc, ThemeTokens::CYAN_EDGE),
        (t.step2_title, t.step2_desc, ThemeTokens::GOLD_5090),
        (t.step3_title, t.step3_desc, ThemeTokens::SUCCESS),
        (t.step4_title, t.step4_desc, ThemeTokens::PRIMARY),
    ];

    let w = (ui.available_width() - 24.0) / 2.0;

    ui.horizontal(|ui| {
        render_step_box(ui, w, steps[0].0, steps[0].1, steps[0].2);
        ui.add_space(8.0);
        render_step_box(ui, w, steps[1].0, steps[1].1, steps[1].2);
    });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        render_step_box(ui, w, steps[2].0, steps[2].1, steps[2].2);
        ui.add_space(8.0);
        render_step_box(ui, w, steps[3].0, steps[3].1, steps[3].2);
    });
}

fn render_step_box(ui: &mut egui::Ui, width: f32, title: &str, desc: &str, accent: egui::Color32) {
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(16, 22, 32))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 48, 68)))
        .inner_margin(10.0)
        .rounding(6.0)
        .show(ui, |ui| {
            ui.set_width(width);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).strong().color(accent).size(12.5));
                ui.add_space(3.0);
                ui.label(egui::RichText::new(desc).color(ThemeTokens::TEXT_SECONDARY).size(11.0));
            });
        });
}

fn render_off_grid_section(ui: &mut egui::Ui, t: &'static crate::presentation::i18n::LocalToolsTranslations) {
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(20, 26, 42))
        .stroke(egui::Stroke::new(1.2, egui::Color32::from_rgb(255, 180, 50)))
        .inner_margin(12.0)
        .rounding(8.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("[Mobile]").strong().color(ThemeTokens::GOLD_5090).size(12.0));
                    ui.label(egui::RichText::new(t.off_grid_title).strong().color(ThemeTokens::GOLD_5090).size(13.5));
                });
                ui.add_space(4.0);
                ui.label(egui::RichText::new(t.off_grid_desc).color(ThemeTokens::TEXT_SECONDARY).size(11.5));
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(t.off_grid_p1).color(ThemeTokens::SUCCESS).size(11.0));
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new(t.off_grid_p2).color(ThemeTokens::CYAN_EDGE).size(11.0));
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new(t.off_grid_p3).color(ThemeTokens::GOLD_5090).size(11.0));
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(t.off_grid_models_title).strong().color(ThemeTokens::TEXT_PRIMARY).size(11.5));
                    ui.label(egui::RichText::new(t.off_grid_models_desc).color(ThemeTokens::CYAN_EDGE).size(11.0));
                });
            });
        });
}

fn render_tools_catalog(ui: &mut egui::Ui, t: &'static crate::presentation::i18n::LocalToolsTranslations) {
    ui.label(egui::RichText::new(t.section_tools).size(14.5).color(ThemeTokens::CYAN_EDGE).strong());
    ui.add_space(6.0);

    let card_w = (ui.available_width() - 16.0) / 2.0;

    for chunk in TOOLS_LIST.chunks(2) {
        ui.horizontal(|ui| {
            render_tool_card(ui, card_w, &chunk[0], t);
            if chunk.len() > 1 {
                ui.add_space(12.0);
                render_tool_card(ui, card_w, &chunk[1], t);
            }
        });
        ui.add_space(10.0);
    }
}

fn render_tool_card(ui: &mut egui::Ui, width: f32, tool: &ToolCardData, t: &'static crate::presentation::i18n::LocalToolsTranslations) {
    let accent = egui::Color32::from_rgb(tool.color_rgb.0, tool.color_rgb.1, tool.color_rgb.2);

    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(15, 20, 30))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 48, 70)))
        .inner_margin(12.0)
        .rounding(8.0)
        .show(ui, |ui| {
            ui.set_width(width);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(tool.name).strong().size(15.0).color(accent));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(tool.difficulty).size(10.5).color(ThemeTokens::TEXT_MUTED));
                    });
                });

                ui.label(egui::RichText::new(tool.category).size(11.0).color(ThemeTokens::CYAN_EDGE).strong());
                ui.label(egui::RichText::new(tool.tagline).italics().size(11.0).color(ThemeTokens::TEXT_SECONDARY));
                ui.add_space(4.0);

                ui.label(egui::RichText::new(tool.description).size(11.5).color(ThemeTokens::TEXT_PRIMARY));
                ui.add_space(6.0);

                ui.label(egui::RichText::new(format!("Plateformes : {}", tool.platforms)).size(10.5).color(ThemeTokens::TEXT_MUTED));
                ui.add_space(4.0);

                // Command box with Copy button
                let copy_key = egui::Id::new(format!("copy_tool_{}", tool.name));
                let last_copy = ui.ctx().memory_mut(|m| m.data.get_temp::<f64>(copy_key)).unwrap_or(0.0);
                let is_recent = ui.input(|i| i.time) - last_copy < 2.5;

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(tool.command).monospace().size(10.5).color(egui::Color32::WHITE));
                    if ui.button(egui::RichText::new(if is_recent { t.btn_copied } else { t.btn_copy_cmd }).size(10.5).color(if is_recent { ThemeTokens::SUCCESS } else { ThemeTokens::CYAN_EDGE })).clicked() {
                        ui.ctx().copy_text(tool.command.to_string());
                        ui.ctx().memory_mut(|m| m.data.insert_temp(copy_key, ui.input(|i| i.time)));
                    }
                });

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button(egui::RichText::new(t.btn_open_site).strong().color(ThemeTokens::GOLD_5090).size(11.0)).clicked() {
                        let _ = open::that(tool.url);
                    }
                });
            });
        });
}
