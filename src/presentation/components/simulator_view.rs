use std::collections::HashSet;
use eframe::egui;
use crate::domain::hardware_detector::DetectedHardware;
use crate::domain::model::ModelBenchmark;
use crate::domain::simulator::{GpuTier, HardwareRigConfig, RigSimulationResult, SimTargetMode};
use crate::presentation::components::filter_bar::FilterState;
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
    let sim = config.simulate();

    ui.vertical(|ui| {
        let sim_hdr_rect = ui.available_rect_before_wrap();
        ui.ctx().memory_mut(|m| {
            let area = egui::Rect::from_min_size(
                sim_hdr_rect.min,
                egui::vec2(sim_hdr_rect.width(), 46.0),
            );
            m.data.insert_temp(egui::Id::new("tour_target_sim_area"), area);
        });

        // --- 1. Header & Quick Actions ---
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new(t.sim_title).strong().color(ThemeTokens::TEXT_PRIMARY));
            ui.separator();
            if let Some(hw) = detected_hw {
                let btn = egui::Button::new(
                    egui::RichText::new(t.sim_load_detected).strong().color(ThemeTokens::CYAN_EDGE)
                );
                if ui.add(btn)
                    .on_hover_text(format!("Configuration détectée :\n{}", hw.summary()))
                    .clicked()
                {
                    apply_detected_to_config(config, hw);
                }
            }
        });
        ui.label(egui::RichText::new(t.sim_subtitle).color(ThemeTokens::TEXT_MUTED));
        ui.add_space(8.0);

        // --- 2. Configuration Panel with Dropdown List ---
        egui::Frame::group(ui.style())
            .fill(ThemeTokens::BG_CARD)
            .stroke(egui::Stroke::new(1.0, ThemeTokens::TEXT_MUTED))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new(t.sim_scenario_label).strong().color(ThemeTokens::CYAN_EDGE));
                    
                    let scenario_lbl = match config.mode {
                        SimTargetMode::WorkstationPc => t.sim_scenario_pc,
                        SimTargetMode::HomeServerNas => t.sim_scenario_nas,
                    };
                    egui::ComboBox::from_id_salt("sim_scenario_dropdown")
                        .selected_text(egui::RichText::new(scenario_lbl).strong())
                        .width(340.0)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(config.mode == SimTargetMode::WorkstationPc, t.sim_scenario_pc).clicked() {
                                config.mode = SimTargetMode::WorkstationPc;
                            }
                            if ui.selectable_label(config.mode == SimTargetMode::HomeServerNas, t.sim_scenario_nas).clicked() {
                                config.mode = SimTargetMode::HomeServerNas;
                            }
                        });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                match config.mode {
                    SimTargetMode::WorkstationPc => {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(egui::RichText::new(t.sim_gpu_label).strong());
                            let gpu_resp = egui::ComboBox::from_id_salt("sim_gpu_combo")
                                .selected_text(config.gpu.label())
                                .width(300.0)
                                .show_ui(ui, |ui| {
                                    for g in GpuTier::all() {
                                        if ui.selectable_label(config.gpu == g, g.label()).clicked() { config.gpu = g; }
                                    }
                                }).response;
                            ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_sim_gpu"), gpu_resp.rect));

                            ui.separator();
                            ui.label(egui::RichText::new(t.sim_ram_label).strong());
                            ui.add(egui::Slider::new(&mut config.ram_gb, 8.0..=128.0).suffix(" Go").logarithmic(true));
                        });
                    }
                    SimTargetMode::HomeServerNas => {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(egui::RichText::new(t.sim_ram_label).strong());
                            ui.add(egui::Slider::new(&mut config.ram_gb, 4.0..=64.0).suffix(" Go Total").logarithmic(true));

                            ui.separator();
                            ui.label(egui::RichText::new(t.sim_nas_web_ram).strong());
                            ui.add(egui::Slider::new(&mut config.nas_web_ram_gb, 1.0..=16.0).suffix(" Go Réservés"));
                        });
                    }
                }
            });

        ui.add_space(10.0);

        // --- 3. Visual 4-Card KPI Dashboard (Responsive 2x2 on narrow windows) ---
        let kpi_rect = ui.available_rect_before_wrap();
        ui.ctx().memory_mut(|m| {
            let area = egui::Rect::from_min_size(
                kpi_rect.min,
                egui::vec2(kpi_rect.width(), 70.0),
            );
            m.data.insert_temp(egui::Id::new("tour_result_sim_kpi"), area);
        });

        let speed_color = if sim.projected_speed_tps >= 35.0 { ThemeTokens::SUCCESS } else if sim.projected_speed_tps >= 15.0 { ThemeTokens::GOLD_5090 } else { ThemeTokens::DANGER };
        let speed_level = if sim.projected_speed_tps >= 50.0 { "Très Fluide" } else if sim.projected_speed_tps >= 20.0 { "Interactif" } else { "Lent (Offload)" };
        let oom_color = if sim.stability_percent >= 80.0 { ThemeTokens::SUCCESS } else if sim.stability_percent >= 50.0 { ThemeTokens::GOLD_5090 } else { ThemeTokens::DANGER };
        let oom_verdict = if sim.stability_percent >= 80.0 { "Stable & Fluide" } else if sim.stability_percent >= 50.0 { "Marge Restreinte" } else { "Risque OOM !" };

        let avail_w = ui.available_width();
        if avail_w < 780.0 {
            ui.columns(2, |cols| {
                render_card(&mut cols[0], "[VRAM] MEMOIRE ALLOUEE", &format!("{:.1} Go", sim.usable_vram_gb), &format!("RAM : {:.1} Go | IA Total : {:.1} Go", sim.usable_ram_gb, sim.total_ai_capacity_gb), ThemeTokens::TEXT_PRIMARY);
                render_card(&mut cols[1], "[~] DEBIT INFERENCE", &format!("{:.0} tok/s", sim.projected_speed_tps), &format!("{} | Bandwidth : {:.0} Go/s", speed_level, sim.effective_bandwidth_gbps), speed_color);
            });
            ui.add_space(6.0);
            ui.columns(2, |cols| {
                render_card(&mut cols[0], " STABILITE / RISQUE OOM", &format!("{:.0}%", sim.stability_percent), &format!("{} (Marge: {:.1} Go)", oom_verdict, sim.ram_headroom_gb), oom_color);
                render_card(&mut cols[1], " ECONOMIE VS CLOUD", &format!("-{:.1}%", sim.savings_percent), &format!("{:.3} € local vs {:.2} € Cloud (100 tâches)", sim.cost_per_100_tasks_local_eur, sim.cost_per_100_tasks_cloud_eur), ThemeTokens::SUCCESS);
            });
        } else {
            ui.columns(4, |cols| {
                render_card(&mut cols[0], "[VRAM] MEMOIRE ALLOUEE", &format!("{:.1} Go", sim.usable_vram_gb), &format!("RAM : {:.1} Go | IA Total : {:.1} Go", sim.usable_ram_gb, sim.total_ai_capacity_gb), ThemeTokens::TEXT_PRIMARY);
                render_card(&mut cols[1], "[~] DEBIT INFERENCE", &format!("{:.0} tok/s", sim.projected_speed_tps), &format!("{} | Bandwidth : {:.0} Go/s", speed_level, sim.effective_bandwidth_gbps), speed_color);
                render_card(&mut cols[2], " STABILITE / RISQUE OOM", &format!("{:.0}%", sim.stability_percent), &format!("{} (Marge: {:.1} Go)", oom_verdict, sim.ram_headroom_gb), oom_color);
                render_card(&mut cols[3], " ECONOMIE VS CLOUD", &format!("-{:.1}%", sim.savings_percent), &format!("{:.3} € local vs {:.2} € Cloud (100 tâches)", sim.cost_per_100_tasks_local_eur, sim.cost_per_100_tasks_cloud_eur), ThemeTokens::SUCCESS);
            });
        }

        ui.add_space(12.0);

        // --- 4. Recommended Models (Responsive Grid) ---
        render_recommended_models_grid(ui, &sim, all_models, filters, selected_model_id, t);
    });
}

fn apply_detected_to_config(config: &mut HardwareRigConfig, hw: &DetectedHardware) {
    config.mode = SimTargetMode::WorkstationPc;
    config.ram_gb = hw.total_ram_gb.max(8.0);
    let vram = hw.gpu_vram_gb.unwrap_or(0.0);
    config.gpu = if vram >= 28.0 {
        GpuTier::Rtx5090_32G
    } else if vram >= 20.0 {
        GpuTier::Rtx4090_24G
    } else if vram >= 14.0 {
        GpuTier::Rtx4080_16G
    } else if vram >= 10.0 {
        GpuTier::Rtx4070_12G
    } else if vram >= 7.0 {
        GpuTier::Rtx4060_8G
    } else {
        GpuTier::None
    };
}

fn render_card(ui: &mut egui::Ui, title: &str, big_val: &str, sub: &str, val_color: egui::Color32) {
    egui::Frame::group(ui.style())
        .fill(ThemeTokens::BG_CARD)
        .stroke(egui::Stroke::new(1.0, ThemeTokens::TEXT_MUTED))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new(title).size(11.0).color(ThemeTokens::TEXT_MUTED).strong());
            ui.add_space(3.0);
            ui.label(egui::RichText::new(big_val).size(20.0).strong().color(val_color));
            ui.add_space(3.0);
            ui.label(egui::RichText::new(sub).size(11.0).color(ThemeTokens::TEXT_SECONDARY));
        });
}

fn render_recommended_models_grid(
    ui: &mut egui::Ui,
    sim: &RigSimulationResult,
    all_models: &[ModelBenchmark],
    filters: &FilterState,
    selected_model_id: &mut Option<String>,
    t: &crate::presentation::i18n::Translations,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(t.sim_rec_models_title).strong().color(ThemeTokens::GOLD_5090).size(14.0));
        ui.label(egui::RichText::new(format!("(Modèles compatibles <= {:.1} Go)", sim.total_ai_capacity_gb)).color(ThemeTokens::TEXT_MUTED));
    });

    let mut rec_models: Vec<&ModelBenchmark> = all_models.iter()
        .filter(|m| !m.is_cloud && m.vram_rec_gb <= sim.total_ai_capacity_gb)
        .collect();

    rec_models.sort_by(|a, b| {
        let sc_a = a.score_for_sources(&filters.sources).unwrap_or(0.0);
        let sc_b = b.score_for_sources(&filters.sources).unwrap_or(0.0);
        sc_b.partial_cmp(&sc_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Deduplicate by clean model name
    let mut seen = HashSet::new();
    let mut unique_models = Vec::new();
    for m in rec_models {
        if seen.insert(m.name.clone()) {
            unique_models.push(m);
            if unique_models.len() >= 4 { break; }
        }
    }

    let avail_w = ui.available_width();
    let num_cols = if avail_w < 600.0 { 1 } else if avail_w < 900.0 { 2 } else { unique_models.len().min(4).max(1) };

    ui.add_space(6.0);
    for chunk in unique_models.chunks(num_cols) {
        ui.columns(chunk.len(), |cols| {
            for (i, m) in chunk.iter().enumerate() {
                let col = &mut cols[i];
                let sc = m.score_for_sources(&filters.sources).unwrap_or(0.0);
                let is_sel = selected_model_id.as_deref() == Some(&m.id);
                let stroke_color = if is_sel { ThemeTokens::GOLD_5090 } else { ThemeTokens::CYAN_EDGE };

                egui::Frame::group(col.style())
                    .fill(ThemeTokens::BG_CARD)
                    .stroke(egui::Stroke::new(1.0, stroke_color))
                    .inner_margin(8.0)
                    .show(col, |ui| {
                        ui.label(egui::RichText::new(&m.name).strong().size(13.0).color(ThemeTokens::TEXT_PRIMARY));
                        ui.add_space(2.0);
                        ui.label(egui::RichText::new(format!(" Score : {:.1} pts", sc)).strong().color(ThemeTokens::GOLD_5090));
                        ui.label(egui::RichText::new(format!("[VRAM] VRAM : {:.1} Go", m.vram_rec_gb)).color(ThemeTokens::CYAN_EDGE));
                        if let Some(tps) = m.tokens_per_sec {
                            ui.label(egui::RichText::new(format!(" Débit : {:.0} tok/s", tps)).color(ThemeTokens::SUCCESS));
                        }
                        ui.add_space(4.0);
                        if ui.button(egui::RichText::new(" Inspecter").strong()).clicked() {
                            *selected_model_id = Some(m.id.clone());
                        }
                    });
            }
        });
        ui.add_space(6.0);
    }
}
