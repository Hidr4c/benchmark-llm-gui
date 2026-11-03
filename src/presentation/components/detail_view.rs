use eframe::egui;
use crate::domain::consistency::{ConsistencyIssue, ModelConsistencyValidator};
use crate::domain::model::ModelBenchmark;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_detail_panel(
    ui: &mut egui::Ui,
    model: &ModelBenchmark,
    selected_model_id: &mut Option<String>,
    lang: Language,
) {
    let t = lang.t();
    egui::Frame::group(ui.style())
        .fill(ThemeTokens::BG_CARD)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // Top Header
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new(&model.name).color(ThemeTokens::TEXT_PRIMARY).strong());
                    let (mr, mg, mb) = model.modality.badge_color_rgb();
                    ui.label(egui::RichText::new(model.modality.label()).color(egui::Color32::from_rgb(mr, mg, mb)).strong());
                    if model.specialty != crate::domain::specialty::DomainSpecialty::General {
                        let (sr, sg, sb) = model.specialty.badge_color_rgb();
                        let spec_lbl = if lang == Language::French { model.specialty.label_fr() } else { model.specialty.label_en() };
                        ui.label(egui::RichText::new(format!("[{}] {}", model.specialty.badge_text(), spec_lbl)).color(egui::Color32::from_rgb(sr, sg, sb)).strong());
                    }
                    let year_badge = format!(" {}", model.release_year_label());
                    let year_color = if model.is_very_recent() { ThemeTokens::CYAN_EDGE } else { ThemeTokens::TEXT_MUTED };
                    ui.label(egui::RichText::new(year_badge).color(year_color).strong());

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(t.close_inspection).clicked() {
                            *selected_model_id = None;
                        }
                    });
                });

                ui.label(egui::RichText::new(&model.id).color(ThemeTokens::TEXT_MUTED).monospace());
                ui.add_space(4.0);

                let issues = ModelConsistencyValidator::validate(model);
                for issue in &issues {
                    let (title, desc) = match issue {
                        ConsistencyIssue::ScoreDivergence => (t.anomaly_divergence_title, t.anomaly_divergence_desc),
                        ConsistencyIssue::ExtremeThroughputForHardware => (t.anomaly_speed_title, t.anomaly_speed_desc),
                        ConsistencyIssue::UncalibratedHeavyModel => (t.anomaly_memory_title, t.anomaly_memory_desc),
                        ConsistencyIssue::HighVarianceAcrossSources => (t.anomaly_variance_title, t.anomaly_variance_desc),
                        ConsistencyIssue::TemporalSizeAnomaly => (t.anomaly_temporal_title, t.anomaly_temporal_desc),
                    };
                    egui::Frame::group(ui.style())
                        .fill(egui::Color32::from_rgb(50, 35, 15))
                        .stroke(egui::Stroke::new(1.0, ThemeTokens::WARNING))
                        .inner_margin(6.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("[!]").color(ThemeTokens::WARNING));
                                ui.label(egui::RichText::new(title).strong().color(ThemeTokens::WARNING));
                                ui.label(egui::RichText::new(desc).size(11.5).color(ThemeTokens::TEXT_SECONDARY));
                            });
                        });
                    ui.add_space(2.0);
                }

                // Hardware & Embedded Diagnostic Card
                let (r, g, b) = model.hardware_color_rgb();
                let border_color = egui::Color32::from_rgb(r, g, b);

                egui::Frame::group(ui.style())
                    .stroke(egui::Stroke::new(1.5, border_color))
                    .fill(ThemeTokens::BG_PANEL)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(t.diagnostic_title).strong());
                            ui.label(egui::RichText::new(model.embedded_recommendation()).color(ThemeTokens::CYAN_EDGE).strong());
                        });

                        ui.add_space(2.0);
                        let hw = model.tps_hardware.as_deref().unwrap_or("Standard");
                        let tps_str = model.tokens_per_sec.map(|v| format!("{v:.0} tok/s")).unwrap_or_else(|| "N/A".into());
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!(
                                "{} : {:.1} GB ({:.0} MB)  |  {} : {:.1} GB  |  {} : {} ({})  |  {} : {:.1} *",
                                t.col_size, model.size_gb, model.size_mb(),
                                t.col_hardware, model.vram_rec_gb,
                                t.col_speed, tps_str, hw,
                                t.col_efficiency, model.efficiency_ratio()
                            )).size(12.0).color(ThemeTokens::TEXT_SECONDARY));
                        });
                    });

                ui.add_space(6.0);

                // Pricing & Operational Cost Card
                egui::Frame::group(ui.style())
                    .fill(egui::Color32::from_rgb(20, 26, 38))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 80, 130)))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(t.pricing_section_title).strong().color(ThemeTokens::GOLD_5090));
                            if model.is_cloud {
                                let prov = model.provider.as_deref().unwrap_or("Cloud API");
                                ui.label(egui::RichText::new(format!(" {prov}")).color(ThemeTokens::CYAN_EDGE).strong());
                            } else {
                                ui.label(egui::RichText::new(t.pricing_local_free).color(ThemeTokens::SUCCESS).strong());
                            }
                        });
                        ui.add_space(2.0);

                        crate::presentation::components::detail_roi::render_roi_simulation_card(ui, model, lang);
                    });

                ui.add_space(8.0);

                // Benchmark Scores Grid (Calibrated to 0-100 base)
                ui.label(egui::RichText::new(t.scores_detail_title).strong());

                if let Some(breakdown) = model.score_breakdown(&crate::domain::model::SourceSelection::default()) {
                    let (cr, cg, cb) = breakdown.confidence.color_rgb();
                    let conf_color = egui::Color32::from_rgb(cr, cg, cb);
                    let conf = lang.confidence();
                    let (conf_lbl, conf_tip) = match breakdown.confidence {
                        crate::domain::scoring::ScoreConfidence::High => (conf.ok_label, conf.ok_tooltip),
                        crate::domain::scoring::ScoreConfidence::Medium => (conf.med_label, conf.med_tooltip),
                        crate::domain::scoring::ScoreConfidence::Low => (conf.low_label, conf.low_tooltip),
                    };

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(t.score_confidence_level).strong());
                        ui.label(egui::RichText::new(format!("{} {}", breakdown.confidence.badge(), conf_lbl)).color(conf_color).strong())
                            .on_hover_text(conf_tip);
                        ui.label(egui::RichText::new(format!("| {} {:.1} pts", t.score_variance_label, breakdown.variance)).color(ThemeTokens::TEXT_SECONDARY));
                    });
                    ui.label(egui::RichText::new(format!("{} {}", t.score_calibrated_formula, breakdown.formula)).size(11.5).color(ThemeTokens::TEXT_MUTED).monospace());
                    ui.add_space(2.0);
                }

                ui.label(egui::RichText::new(t.raw_score_notice).size(11.0).color(ThemeTokens::TEXT_MUTED));
                egui::Grid::new("scores_grid").num_columns(4).spacing([18.0, 5.0]).striped(true).show(ui, |ui| {
                    render_score_cell(ui, t.bench_general, model.scores.open_llm.or(model.scores.average), "open_llm");
                    render_score_cell(ui, t.bench_coding, model.scores.coding, "coding");
                    ui.end_row();

                    render_score_cell(ui, t.bench_agentic, model.scores.agentic, "agentic");
                    render_score_cell(ui, "MMLU-PRO :", model.scores.mmlu_pro, "mmlu");
                    ui.end_row();

                    render_score_cell(ui, "IFEval :", model.scores.ifeval, "open_llm");
                    render_score_cell(ui, "BBH :", model.scores.bbh, "open_llm");
                    ui.end_row();

                    render_score_cell(ui, "MATH Lvl 5 :", model.scores.math_lvl5, "math");
                    render_score_cell(ui, "GPQA :", model.scores.gpqa, "gpqa");
                    ui.end_row();
                });

                // Real-World & Economic Value Benchmarks
                if let Some(econ) = &model.economic {
                    ui.add_space(4.0);
                    egui::Frame::group(ui.style())
                        .fill(egui::Color32::from_rgb(25, 30, 42))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 100, 160)))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(t.economic_section_title).strong().color(ThemeTokens::CYAN_EDGE));
                                if let Some(comp) = econ.composite_score() {
                                    ui.label(egui::RichText::new(format!("* Composite: {comp:.1} / 100")).strong().color(ThemeTokens::GOLD_5090));
                                }
                            });
                            ui.add_space(3.0);

                            egui::Grid::new("econ_grid").num_columns(4).spacing([16.0, 4.0]).show(ui, |ui| {
                                ui.label(t.rli_label);
                                if let Some(rli) = econ.rli_automation_rate {
                                    ui.label(egui::RichText::new(format!("{rli:.1} %")).color(ThemeTokens::SUCCESS).strong())
                                        .on_hover_text("Automation Rate: % of freelance tasks completed to client acceptance standard.");
                                } else { ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED)); }

                                ui.label(t.swe_lancer_label);
                                if let Some(swe) = econ.swe_lancer_score {
                                    let dl = econ.swe_lancer_dollars.map(|d| format!(" (${d:.0})")).unwrap_or_default();
                                    ui.label(egui::RichText::new(format!("{swe:.1} %{dl}")).color(ThemeTokens::SUCCESS).strong())
                                        .on_hover_text("SWE-Lancer benchmark performance % and total dollars earned.");
                                } else { ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED)); }
                                ui.end_row();

                                ui.label(t.swe_clar_label);
                                if let Some(imp) = econ.swe_clar_impl {
                                    let clr_str = econ.swe_clar_clarif.map(|c| format!(" | {}: {:.1}%", t.clarification_label, c)).unwrap_or_default();
                                    ui.label(egui::RichText::new(format!("{:.1}%{}", imp, clr_str)).color(ThemeTokens::CYAN_EDGE).strong())
                                        .on_hover_text("SWE-Clar: Implementation and requirement clarification scores.");
                                } else { ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED)); }

                                ui.label(t.osworld_label);
                                if let Some(os) = econ.os_world_success {
                                    ui.label(egui::RichText::new(format!("{os:.1} %")).color(ThemeTokens::SUCCESS).strong())
                                        .on_hover_text("OSWorld open computer operating system task success rate.");
                                } else { ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED)); }
                                ui.end_row();

                                ui.label(t.webarena_label);
                                if let Some(web) = econ.web_arena_success {
                                    ui.label(egui::RichText::new(format!("{web:.1} %")).color(ThemeTokens::SUCCESS).strong())
                                        .on_hover_text("WebArena autonomous web browsing and navigation success rate.");
                                } else { ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED)); }

                                ui.label(t.mle_bench_label);
                                if let Some(mle) = econ.mle_bench_score {
                                    let med = econ.mle_bench_medals.as_deref().unwrap_or("");
                                    ui.label(egui::RichText::new(format!("{mle:.1} %")).color(ThemeTokens::GOLD_5090).strong())
                                        .on_hover_text(format!("MLE-bench: 75 Kaggle competitions ({med})"));
                                } else { ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED)); }
                                ui.end_row();
                            });
                        });
                }

                ui.add_space(8.0);

                // Quantization Matrix
                let p = model.param_count_b.unwrap_or(model.size_gb / 0.58);
                let q4 = (p * 0.58).max(0.15);
                let q5 = (p * 0.70).max(0.20);
                let q8 = (p * 1.05).max(0.30);

                ui.label(egui::RichText::new(t.quant_title).strong());
                egui::Grid::new("quant_grid").num_columns(4).spacing([14.0, 4.0]).show(ui, |ui| {
                    ui.label(egui::RichText::new("Quant").strong());
                    ui.label(egui::RichText::new(t.col_size).strong());
                    ui.label(egui::RichText::new(t.col_hardware).strong());
                    ui.label(egui::RichText::new("Tier").strong());
                    ui.end_row();

                    for (name, val, extra, tier) in [
                        ("Q4_K_M", q4, 0.8, t.quant_recommended),
                        ("Q5_K_M", q5, 0.8, t.quant_enhanced),
                        ("Q8_0", q8, 1.2, t.quant_fp16),
                    ] {
                        ui.label(name);
                        ui.label(format!("{:.1} GB", val));
                        ui.label(format!("{:.1} GB", val + extra));
                        ui.label(tier);
                        ui.end_row();
                    }
                });

                ui.add_space(8.0);

                // Actions, Direct Execution & Online View
                crate::presentation::components::detail_runner::render_runner_card(ui, model, lang);
                ui.add_space(4.0);
                if ui.button(t.view_online).clicked() {
                    let _ = open::that(&model.hf_url);
                }
            });
        });
}

fn render_score_cell(ui: &mut egui::Ui, title: &str, score: Option<f32>, domain: &str) {
    ui.label(title);
    if let Some(raw) = score {
        let norm = ModelBenchmark::calibrate_score(raw, domain);
        let color = if norm >= 70.0 { ThemeTokens::GOLD_5090 } else if norm >= 45.0 { ThemeTokens::SUCCESS } else { ThemeTokens::PRIMARY };
        ui.label(egui::RichText::new(format!("{:.1} / 100", norm)).color(color).strong())
            .on_hover_text(format!("Raw: {:.1} % (Calibrated: {:.1})", raw, norm));
    } else {
        ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED));
    }
}
