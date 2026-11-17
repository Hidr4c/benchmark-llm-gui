use std::collections::HashSet;
use eframe::egui;
use crate::domain::hardware_detector::DetectedHardware;
use crate::domain::model::ModelBenchmark;
use crate::domain::simulator::{GpuTier, HardwareRigConfig, RigSimulationResult, SimTargetMode};
use crate::presentation::components::filter_bar::FilterState;
use crate::presentation::components::simulator_config_bar::render_simulator_config_bar;
use crate::presentation::components::simulator_optimizer_panel::render_simulator_optimizer_panel;
use crate::presentation::components::simulator_table_view::render_simulator_compatible_table;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_simulator_view(
    ui: &mut egui::Ui,
    config: &mut HardwareRigConfig,
    detected_hw: Option<&DetectedHardware>,
    all_models: &[ModelBenchmark],
    filters: &FilterState,
    selected_model_id: &mut Option<String>,
    lang: Language,
) {
    let t = lang.t();
    let is_fr = lang == Language::French;

    ui.vertical(|ui| {
        let sim_hdr_rect = ui.available_rect_before_wrap();
        ui.ctx().memory_mut(|m| {
            let area = egui::Rect::from_min_size(sim_hdr_rect.min, egui::vec2(sim_hdr_rect.width(), 46.0));
            m.data.insert_temp(egui::Id::new("tour_target_sim_area"), area);
        });

        ui.horizontal_wrapped(|ui| {
            ui.heading(egui::RichText::new(t.sim_title).strong().color(ThemeTokens::TEXT_PRIMARY));
            if let Some(hw) = detected_hw {
                ui.separator();
                let btn = egui::Button::new(egui::RichText::new(t.sim_load_detected).strong().color(ThemeTokens::CYAN_EDGE));
                if ui.add(btn).on_hover_text(format!("{} :\n{}", if is_fr { "Configuration détectée" } else { "Detected hardware" }, hw.summary())).clicked() {
                    apply_detected_to_config(config, hw);
                }
            }
        });
        ui.label(egui::RichText::new(t.sim_subtitle).color(ThemeTokens::TEXT_MUTED).size(12.0));
        ui.add_space(6.0);

        let pre_sim = config.simulate();
        let compat_count = all_models.iter()
            .filter(|m| !m.is_cloud && m.vram_rec_gb <= pre_sim.total_ai_capacity_gb)
            .filter(|m| config.max_storage_gb.map_or(true, |max_s| m.size_gb <= max_s))
            .count();

        render_simulator_config_bar(ui, config, compat_count, t, lang);
        let sim = config.simulate();

        ui.add_space(8.0);
        render_kpi_strip(ui, &sim, is_fr);
        ui.add_space(8.0);
        render_recommended_cards(ui, config, &sim, all_models, filters, selected_model_id, is_fr);
        ui.add_space(8.0);
        render_simulator_compatible_table(ui, config, &sim, all_models, &filters.sources, selected_model_id, lang);
        render_simulator_optimizer_panel(ui, config, &sim, all_models, &filters.sources, selected_model_id, lang);
    });
}

fn apply_detected_to_config(config: &mut HardwareRigConfig, hw: &DetectedHardware) {
    config.platform = hw.matched_platform;
    config.gpu = GpuTier::from_platform(hw.matched_platform);
    config.ram_gb = hw.total_ram_gb.max(8.0);
    config.mode = if hw.matched_platform.is_embedded() { SimTargetMode::HomeServerNas } else { SimTargetMode::WorkstationPc };
}

fn render_kpi_strip(ui: &mut egui::Ui, sim: &RigSimulationResult, is_fr: bool) {
    let unit = if is_fr { "Go" } else { "GB" };
    let speed_color = if sim.projected_speed_tps >= 35.0 { ThemeTokens::SUCCESS } else if sim.projected_speed_tps >= 15.0 { ThemeTokens::GOLD_5090 } else { ThemeTokens::DANGER };
    let speed_lvl = if is_fr { if sim.projected_speed_tps >= 50.0 { "Très Fluide" } else if sim.projected_speed_tps >= 20.0 { "Interactif" } else { "Lent (Offload)" } } else { if sim.projected_speed_tps >= 50.0 { "Very Fast" } else if sim.projected_speed_tps >= 20.0 { "Interactive" } else { "Slow (Offload)" } };
    let oom_color = if sim.stability_percent >= 80.0 { ThemeTokens::SUCCESS } else if sim.stability_percent >= 50.0 { ThemeTokens::GOLD_5090 } else { ThemeTokens::DANGER };
    let oom_lbl = if is_fr { if sim.stability_percent >= 80.0 { "Stable & Fluide" } else if sim.stability_percent >= 50.0 { "Marge Restreinte" } else { "Risque OOM !" } } else { if sim.stability_percent >= 80.0 { "Stable & Smooth" } else if sim.stability_percent >= 50.0 { "Tight Margin" } else { "OOM Risk!" } };

    let cards = [
        (if is_fr { "[VRAM] MEMOIRE ALLOUEE" } else { "[VRAM] ALLOCATED MEMORY" }, format!("{:.1} {unit}", sim.usable_vram_gb), format!("RAM: {:.1} {unit} | Total IA: {:.1} {unit}", sim.usable_ram_gb, sim.total_ai_capacity_gb), ThemeTokens::TEXT_PRIMARY),
        (if is_fr { "[~] DEBIT INFERENCE" } else { "[~] INFERENCE SPEED" }, format!("{:.0} tok/s", sim.projected_speed_tps), format!("{speed_lvl} | {:.0} {unit}/s", sim.effective_bandwidth_gbps), speed_color),
        (if is_fr { "STABILITE / RISQUE OOM" } else { "STABILITY / OOM RISK" }, format!("{:.0}%", sim.stability_percent), format!("{oom_lbl} ({:.1} {unit})", sim.ram_headroom_gb.max(0.0)), oom_color),
        (if is_fr { "ECONOMIE VS CLOUD" } else { "SAVINGS VS CLOUD" }, format!("-{:.1}%", sim.savings_percent), if is_fr { format!("{:.3} € vs {:.2} € Cloud", sim.cost_per_100_tasks_local_eur, sim.cost_per_100_tasks_cloud_eur) } else { format!("${:.3} vs ${:.2} Cloud", sim.cost_per_100_tasks_local_eur, sim.cost_per_100_tasks_cloud_eur) }, ThemeTokens::SUCCESS),
    ];

    let cols = if ui.available_width() < 960.0 { 2 } else { 4 };
    for chunk in cards.chunks(cols) {
        ui.columns(chunk.len(), |ui_cols| {
            for (i, (title, val, sub, color)) in chunk.iter().enumerate() {
                egui::Frame::group(ui_cols[i].style()).fill(ThemeTokens::BG_CARD).stroke(egui::Stroke::new(1.0, ThemeTokens::TEXT_MUTED)).inner_margin(10.0).show(&mut ui_cols[i], |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(egui::RichText::new(*title).size(10.5).color(ThemeTokens::TEXT_MUTED).strong());
                    ui.label(egui::RichText::new(val).size(20.0).strong().color(*color));
                    ui.label(egui::RichText::new(sub).size(11.0).color(ThemeTokens::TEXT_SECONDARY));
                });
            }
        });
        ui.add_space(4.0);
    }
}

fn render_recommended_cards(
    ui: &mut egui::Ui,
    config: &HardwareRigConfig,
    sim: &RigSimulationResult,
    all_models: &[ModelBenchmark],
    filters: &FilterState,
    selected_model_id: &mut Option<String>,
    is_fr: bool,
) {
    let unit = if is_fr { "Go" } else { "GB" };
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(if is_fr { "Top 4 Recommandés (Cliquer pour inspecter)" } else { "Top 4 Picks (Click card to inspect)" }).strong().color(ThemeTokens::GOLD_5090).size(13.5));
        let cap_txt = if let Some(max_s) = config.max_storage_gb {
            format!("(<= {:.1} {unit} VRAM, <= {:.1} {unit})", sim.total_ai_capacity_gb, max_s)
        } else {
            format!("(<= {:.1} {unit} Total)", sim.total_ai_capacity_gb)
        };
        ui.label(egui::RichText::new(cap_txt).size(11.5).color(ThemeTokens::TEXT_MUTED));
    });
    ui.add_space(4.0);

    let mut rec_models: Vec<&ModelBenchmark> = all_models.iter()
        .filter(|m| !m.is_cloud && m.vram_rec_gb <= sim.total_ai_capacity_gb)
        .filter(|m| config.max_storage_gb.map_or(true, |max_s| m.size_gb <= max_s))
        .collect();
    rec_models.sort_by(|a, b| {
        b.score_for_sources(&filters.sources).unwrap_or(0.0).total_cmp(&a.score_for_sources(&filters.sources).unwrap_or(0.0))
    });

    let mut seen = HashSet::new();
    let mut top4 = Vec::new();
    for m in rec_models {
        if seen.insert(m.root_model_name()) {
            top4.push(m);
            if top4.len() >= 4 { break; }
        }
    }
    if top4.is_empty() { return; }

    let cols = if ui.available_width() < 680.0 { 1 } else if ui.available_width() < 1000.0 { 2 } else { top4.len() };
    for chunk in top4.chunks(cols) {
        ui.columns(chunk.len(), |ui_cols| {
            for (i, m) in chunk.iter().enumerate() {
                let sc = m.score_for_sources(&filters.sources).unwrap_or(0.0);
                let is_sel = selected_model_id.as_deref() == Some(&m.id);
                let stroke = if is_sel { ThemeTokens::GOLD_5090 } else { ThemeTokens::CYAN_EDGE };
                let resp = egui::Frame::group(ui_cols[i].style())
                    .fill(ThemeTokens::BG_CARD)
                    .stroke(egui::Stroke::new(if is_sel { 1.8 } else { 1.0 }, stroke))
                    .inner_margin(9.0)
                    .show(&mut ui_cols[i], |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.set_min_height(62.0);
                        ui.horizontal(|ui| {
                            let mark = if is_sel { "[✓] " } else { "" };
                            ui.label(egui::RichText::new(format!("{mark}{}", m.name)).strong().size(12.5).color(ThemeTokens::TEXT_PRIMARY));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(m.modality.tag_short()).size(9.5).color(ThemeTokens::CYAN_EDGE));
                            });
                        });
                        ui.add_space(3.0);
                        let tps_txt = m.tokens_per_sec.map_or_else(|| "-".into(), |v| format!("{v:.0} t/s"));
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("{sc:.1} pts")).strong().size(12.0).color(ThemeTokens::GOLD_5090));
                            ui.label(egui::RichText::new("|").color(ThemeTokens::TEXT_MUTED));
                            ui.label(egui::RichText::new(tps_txt).size(11.5).color(if m.tokens_per_sec.is_some() { ThemeTokens::SUCCESS } else { ThemeTokens::TEXT_MUTED }));
                        });
                        ui.label(egui::RichText::new(format!("VRAM: {:.1} {unit} | {}: {:.1} {unit}", m.vram_rec_gb, if is_fr { "Disque" } else { "Disk" }, m.size_gb)).size(11.0).color(ThemeTokens::TEXT_SECONDARY));
                    }).response;
                if resp.interact(egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    *selected_model_id = Some(m.id.clone());
                }
            }
        });
        ui.add_space(4.0);
    }
}
