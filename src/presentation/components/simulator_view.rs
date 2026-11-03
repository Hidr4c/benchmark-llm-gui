use std::collections::HashSet;
use eframe::egui;
use crate::domain::hardware::HardwarePlatform;
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
            let area = egui::Rect::from_min_size(sim_hdr_rect.min, egui::vec2(sim_hdr_rect.width(), 46.0));
            m.data.insert_temp(egui::Id::new("tour_target_sim_area"), area);
        });

        // --- 1. Header & Quick Actions ---
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new(t.sim_title).strong().color(ThemeTokens::TEXT_PRIMARY));
            ui.separator();
            if let Some(hw) = detected_hw {
                let btn = egui::Button::new(egui::RichText::new(t.sim_load_detected).strong().color(ThemeTokens::CYAN_EDGE));
                if ui.add(btn).on_hover_text(format!("Configuration détectée :\n{}", hw.summary())).clicked() {
                    apply_detected_to_config(config, hw);
                }
            }
        });
        ui.label(egui::RichText::new(t.sim_subtitle).color(ThemeTokens::TEXT_MUTED));
        ui.add_space(8.0);

        // --- 2. Unified Hardware & Storage Configuration Panel ---
        egui::Frame::group(ui.style())
            .fill(ThemeTokens::BG_CARD)
            .stroke(egui::Stroke::new(1.0, ThemeTokens::TEXT_MUTED))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    // 1. Hardware Target Selector (Identical 3 categories)
                    ui.label(egui::RichText::new("Matériel Cible :").strong().color(ThemeTokens::CYAN_EDGE));
                    egui::ComboBox::from_id_salt("sim_hw_platform_dropdown")
                        .selected_text(egui::RichText::new(config.platform.label()).strong().color(ThemeTokens::GOLD_5090))
                        .width(280.0)
                        .show_ui(ui, |ui| {
                            ui.label(egui::RichText::new("--- Smartphones & Mobiles ---").color(ThemeTokens::TEXT_MUTED).size(10.5));
                            let mobiles = [HardwarePlatform::SmartphoneMid, HardwarePlatform::SmartphoneHigh, HardwarePlatform::SmartphonePro];
                            for hw in mobiles {
                                if ui.selectable_label(config.platform == hw, hw.label()).clicked() {
                                    config.platform = hw;
                                    config.gpu = GpuTier::from_platform(hw);
                                    config.mode = SimTargetMode::WorkstationPc;
                                    config.ram_gb = hw.max_usable_memory_gb().unwrap_or(4.0) * 1.5;
                                }
                            }
                            ui.separator();
                            ui.label(egui::RichText::new("--- Cartes Graphiques PC ---").color(ThemeTokens::TEXT_MUTED).size(10.5));
                            let gpus = [
                                HardwarePlatform::Gpu8Gb, HardwarePlatform::Gpu12Gb, HardwarePlatform::Gpu16Gb,
                                HardwarePlatform::Gpu24Gb, HardwarePlatform::Gpu32Gb, HardwarePlatform::Workstation,
                            ];
                            for hw in gpus {
                                if ui.selectable_label(config.platform == hw, hw.label()).clicked() {
                                    config.platform = hw;
                                    config.gpu = GpuTier::from_platform(hw);
                                    config.mode = SimTargetMode::WorkstationPc;
                                }
                            }
                            ui.separator();
                            ui.label(egui::RichText::new("--- Systèmes Embarqués & Serveur / NAS ---").color(ThemeTokens::TEXT_MUTED).size(10.5));
                            let edges = [
                                HardwarePlatform::MicroIoT, HardwarePlatform::RaspberryPi4,
                                HardwarePlatform::RaspberryPi5, HardwarePlatform::JetsonOrinNano, HardwarePlatform::MiniPcCpu,
                            ];
                            for hw in edges {
                                if ui.selectable_label(config.platform == hw, hw.label()).clicked() {
                                    config.platform = hw;
                                    config.gpu = GpuTier::from_platform(hw);
                                    config.mode = SimTargetMode::HomeServerNas;
                                }
                            }
                        });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // 2. Storage / Bundle Size Limit Selector
                    ui.label(egui::RichText::new("Taille Disque / Bundle Max :").strong().color(ThemeTokens::TEXT_PRIMARY));
                    let storage_lbl = match config.max_storage_gb {
                        None => "Illimité (Tous modèles)",
                        Some(s) if s <= 0.6 => "500 Mo (Micro App / IoT)",
                        Some(s) if s <= 1.2 => "1.0 Go (App Mobile / Chatbot)",
                        Some(s) if s <= 2.2 => "2.0 Go (App Mobile Pro)",
                        Some(s) if s <= 4.2 => "4.0 Go (Standard PC / NAS)",
                        Some(s) if s <= 8.2 => "8.0 Go (Modèles 7B-8B)",
                        Some(s) if s <= 16.2 => "16.0 Go (Modèles 14B-30B)",
                        Some(_) => "32.0 Go (Gros Modèles)",
                    };
                    egui::ComboBox::from_id_salt("sim_storage_dropdown")
                        .selected_text(egui::RichText::new(storage_lbl).strong())
                        .width(220.0)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(config.max_storage_gb.is_none(), "Illimité (Tous modèles)").clicked() { config.max_storage_gb = None; }
                            if ui.selectable_label(config.max_storage_gb == Some(0.5), "500 Mo (Micro App / IoT)").clicked() { config.max_storage_gb = Some(0.5); }
                            if ui.selectable_label(config.max_storage_gb == Some(1.0), "1.0 Go (App Mobile / Chatbot)").clicked() { config.max_storage_gb = Some(1.0); }
                            if ui.selectable_label(config.max_storage_gb == Some(2.0), "2.0 Go (App Mobile Pro)").clicked() { config.max_storage_gb = Some(2.0); }
                            if ui.selectable_label(config.max_storage_gb == Some(4.0), "4.0 Go (Standard PC / NAS)").clicked() { config.max_storage_gb = Some(4.0); }
                            if ui.selectable_label(config.max_storage_gb == Some(8.0), "8.0 Go (Modèles 7B-8B)").clicked() { config.max_storage_gb = Some(8.0); }
                            if ui.selectable_label(config.max_storage_gb == Some(16.0), "16.0 Go (Modèles 14B-30B)").clicked() { config.max_storage_gb = Some(16.0); }
                            if ui.selectable_label(config.max_storage_gb == Some(32.0), "32.0 Go (Gros Modèles)").clicked() { config.max_storage_gb = Some(32.0); }
                        });
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // Contextual Memory Controls
                if config.platform.is_embedded() && !matches!(config.platform, HardwarePlatform::MiniPcCpu) {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Profil Mobile / Embarqué : Inférence NPU/CPU | Mémoire système réservée OS : 1.5 Go").color(ThemeTokens::TEXT_MUTED).size(11.5));
                    });
                } else if config.mode == SimTargetMode::HomeServerNas {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new(t.sim_ram_label).strong());
                        ui.add(egui::Slider::new(&mut config.ram_gb, 4.0..=64.0).suffix(" Go Total").logarithmic(true));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(t.sim_nas_web_ram).strong());
                        ui.add(egui::Slider::new(&mut config.nas_web_ram_gb, 1.0..=16.0).suffix(" Go Réservés"));
                    });
                } else {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new(t.sim_ram_label).strong());
                        ui.add(egui::Slider::new(&mut config.ram_gb, 8.0..=128.0).suffix(" Go").logarithmic(true));
                    });
                }
            });

        ui.add_space(10.0);

        // --- 3. Visual 4-Card KPI Dashboard (Responsive 2x2 on narrow windows) ---
        let speed_color = if sim.projected_speed_tps >= 35.0 { ThemeTokens::SUCCESS } else if sim.projected_speed_tps >= 15.0 { ThemeTokens::GOLD_5090 } else { ThemeTokens::DANGER };
        let speed_level = if sim.projected_speed_tps >= 50.0 { "Très Fluide" } else if sim.projected_speed_tps >= 20.0 { "Interactif" } else { "Lent (Offload)" };
        let oom_color = if sim.stability_percent >= 80.0 { ThemeTokens::SUCCESS } else if sim.stability_percent >= 50.0 { ThemeTokens::GOLD_5090 } else { ThemeTokens::DANGER };
        let oom_verdict = if sim.stability_percent >= 80.0 { "Stable & Fluide" } else if sim.stability_percent >= 50.0 { "Marge Restreinte" } else { "Risque OOM !" };

        let avail_w = ui.available_width();
        if avail_w < 960.0 {
            ui.columns(2, |cols| {
                render_card(&mut cols[0], "[VRAM] MEMOIRE ALLOUEE", &format!("{:.1} Go", sim.usable_vram_gb), &format!("RAM : {:.1} Go | IA Total : {:.1} Go", sim.usable_ram_gb, sim.total_ai_capacity_gb), ThemeTokens::TEXT_PRIMARY);
                render_card(&mut cols[1], "[~] DEBIT INFERENCE", &format!("{:.0} tok/s", sim.projected_speed_tps), &format!("{} | Bandwidth : {:.0} Go/s", speed_level, sim.effective_bandwidth_gbps), speed_color);
            });
            ui.add_space(6.0);
            ui.columns(2, |cols| {
                render_card(&mut cols[0], " STABILITE / RISQUE OOM", &format!("{:.0}%", sim.stability_percent), &format!("{} (Marge: {:.1} Go)", oom_verdict, sim.ram_headroom_gb), oom_color);
                render_card(&mut cols[1], " ECONOMIE VS CLOUD", &format!("-{:.1}%", sim.savings_percent), &format!("{:.3} € local vs {:.2} € Cloud", sim.cost_per_100_tasks_local_eur, sim.cost_per_100_tasks_cloud_eur), ThemeTokens::SUCCESS);
            });
        } else {
            ui.columns(4, |cols| {
                render_card(&mut cols[0], "[VRAM] MEMOIRE ALLOUEE", &format!("{:.1} Go", sim.usable_vram_gb), &format!("RAM : {:.1} Go | IA Total : {:.1} Go", sim.usable_ram_gb, sim.total_ai_capacity_gb), ThemeTokens::TEXT_PRIMARY);
                render_card(&mut cols[1], "[~] DEBIT INFERENCE", &format!("{:.0} tok/s", sim.projected_speed_tps), &format!("{} | Bandwidth : {:.0} Go/s", speed_level, sim.effective_bandwidth_gbps), speed_color);
                render_card(&mut cols[2], " STABILITE / RISQUE OOM", &format!("{:.0}%", sim.stability_percent), &format!("{} (Marge: {:.1} Go)", oom_verdict, sim.ram_headroom_gb), oom_color);
                render_card(&mut cols[3], " ECONOMIE VS CLOUD", &format!("-{:.1}%", sim.savings_percent), &format!("{:.3} € local vs {:.2} € Cloud", sim.cost_per_100_tasks_local_eur, sim.cost_per_100_tasks_cloud_eur), ThemeTokens::SUCCESS);
            });
        }

        ui.add_space(12.0);

        // --- 4. Recommended Models (Responsive Grid) ---
        render_recommended_models_grid(ui, config, &sim, all_models, filters, selected_model_id, t);
    });
}

fn apply_detected_to_config(config: &mut HardwareRigConfig, hw: &DetectedHardware) {
    config.platform = hw.matched_platform;
    config.gpu = GpuTier::from_platform(hw.matched_platform);
    config.ram_gb = hw.total_ram_gb.max(8.0);
    config.mode = if hw.matched_platform.is_embedded() { SimTargetMode::HomeServerNas } else { SimTargetMode::WorkstationPc };
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
    config: &HardwareRigConfig,
    sim: &RigSimulationResult,
    all_models: &[ModelBenchmark],
    filters: &FilterState,
    selected_model_id: &mut Option<String>,
    t: &crate::presentation::i18n::Translations,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(t.sim_rec_models_title).strong().color(ThemeTokens::GOLD_5090).size(14.0));
        let cap_txt = if let Some(max_s) = config.max_storage_gb {
            format!("(VRAM <= {:.1} Go, Disque <= {:.1} Go)", sim.total_ai_capacity_gb, max_s)
        } else {
            format!("(VRAM <= {:.1} Go)", sim.total_ai_capacity_gb)
        };
        ui.label(egui::RichText::new(cap_txt).color(ThemeTokens::TEXT_MUTED));
    });

    let mut rec_models: Vec<&ModelBenchmark> = all_models.iter()
        .filter(|m| !m.is_cloud && m.vram_rec_gb <= sim.total_ai_capacity_gb)
        .filter(|m| config.max_storage_gb.map_or(true, |max_s| m.size_gb <= max_s))
        .collect();

    rec_models.sort_by(|a, b| {
        let sc_a = a.score_for_sources(&filters.sources).unwrap_or(0.0);
        let sc_b = b.score_for_sources(&filters.sources).unwrap_or(0.0);
        sc_b.partial_cmp(&sc_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut seen = HashSet::new();
    let mut unique_models = Vec::new();
    for m in rec_models {
        if seen.insert(m.name.clone()) {
            unique_models.push(m);
            if unique_models.len() >= 4 { break; }
        }
    }

    let avail_w = ui.available_width();
    let num_cols = if avail_w < 680.0 { 1 } else if avail_w < 1000.0 { 2 } else { unique_models.len().min(4).max(1) };

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
                        ui.label(egui::RichText::new(format!(" Disque : {:.1} Go", m.size_gb)).color(ThemeTokens::TEXT_MUTED));
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
