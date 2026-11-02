use eframe::egui;
use egui_extras::TableRow;
use crate::domain::consistency::{ConsistencyIssue, ModelConsistencyValidator};
use crate::domain::model::ModelBenchmark;
use crate::domain::table_preferences::TableColumnId;
use crate::presentation::components::filter_bar::FilterState;
use crate::presentation::components::table_items::{
    format_downloads, hardware_badge_text, render_economic_cell, render_score_badge_cell,
    render_vs_button, DisplayItem,
};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_table_row(
    row: &mut TableRow,
    item: &DisplayItem,
    model: &ModelBenchmark,
    is_first_row: bool,
    is_selected: bool,
    filters: &mut FilterState,
    selected_model_id: &mut Option<String>,
    lang: Language,
) {
    row.set_selected(is_selected);
    let columns = filters.table_prefs.column_order.clone();
    for col in columns {
        if col.is_advanced() && !filters.show_advanced_columns {
            continue;
        }
        if col.is_raw_source() && !filters.show_raw_sources {
            continue;
        }
        render_cell_by_column(
            row, col, item, model, is_first_row, is_selected, filters, selected_model_id, lang,
        );
    }
}

fn render_cell_by_column(
    row: &mut TableRow,
    col: TableColumnId,
    item: &DisplayItem,
    model: &ModelBenchmark,
    is_first_row: bool,
    is_selected: bool,
    filters: &mut FilterState,
    selected_model_id: &mut Option<String>,
    lang: Language,
) {
    let t = lang.t();
    match col {
        TableColumnId::Rank => {
            row.col(|ui| {
                if is_first_row {
                    let row_rect = egui::Rect::from_min_max(
                        ui.min_rect().min,
                        egui::pos2(ui.ctx().screen_rect().right() - 24.0, ui.min_rect().min.y + 70.0),
                    );
                    ui.ctx().memory_mut(|m| {
                        m.data.insert_temp(egui::Id::new("tour_target_table_row"), row_rect);
                        m.data.insert_temp(egui::Id::new("tour_result_table_rows"), row_rect);
                    });
                }
                ui.horizontal(|ui| {
                    render_vs_button(ui, &model.id, filters, lang);
                    let r_color = if item.is_child { ThemeTokens::TEXT_SECONDARY } else { ThemeTokens::TEXT_PRIMARY };
                    ui.label(egui::RichText::new(&item.rank_str).color(r_color).size(11.0));
                });
            });
        }
        TableColumnId::Model => {
            row.col(|ui| {
                ui.horizontal(|ui| {
                    if !item.is_child && item.variant_count > 1 {
                        let arrow = if item.is_expanded { "v" } else { ">" };
                        let tag = format!("{arrow} [{} var.]", item.variant_count);
                        let tag_resp = ui.small_button(egui::RichText::new(tag).size(10.0).color(ThemeTokens::PRIMARY));
                        if is_first_row {
                            ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_result_grouped"), tag_resp.rect));
                        }
                        if tag_resp.clicked() {
                            if item.is_expanded { filters.expanded_roots.remove(&item.root_name); }
                            else { filters.expanded_roots.insert(item.root_name.clone()); }
                        }
                    }

                    let display_name = if item.is_child {
                        format!("  |-- {}", model.name)
                    } else if filters.group_by_root && !item.is_expanded {
                        item.root_name.clone()
                    } else {
                        model.name.clone()
                    };

                    let name_resp = ui.selectable_label(is_selected, egui::RichText::new(display_name).strong());
                    if is_first_row {
                        ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_model_name"), name_resp.rect));
                    }
                    if name_resp.clicked() {
                        *selected_model_id = Some(model.id.clone());
                    }

                    let issues = ModelConsistencyValidator::validate(model);
                    if let Some(first_issue) = issues.first() {
                        let (title, desc) = match first_issue {
                            ConsistencyIssue::ScoreDivergence => (t.anomaly_divergence_title, t.anomaly_divergence_desc),
                            ConsistencyIssue::ExtremeThroughputForHardware => (t.anomaly_speed_title, t.anomaly_speed_desc),
                            ConsistencyIssue::UncalibratedHeavyModel => (t.anomaly_memory_title, t.anomaly_memory_desc),
                            ConsistencyIssue::HighVarianceAcrossSources => (t.anomaly_variance_title, t.anomaly_variance_desc),
                            ConsistencyIssue::TemporalSizeAnomaly => (t.anomaly_temporal_title, t.anomaly_temporal_desc),
                        };
                        ui.label(egui::RichText::new("[!]").color(ThemeTokens::WARNING)).on_hover_text(format!("{title}: {desc}"));
                    }

                    let mod_rgb = model.modality.badge_color_rgb();
                    ui.label(egui::RichText::new(model.modality.tag_short()).size(9.0).color(egui::Color32::from_rgb(mod_rgb.0, mod_rgb.1, mod_rgb.2)))
                        .on_hover_text(model.modality.label());

                    if model.is_cloud {
                        ui.label(egui::RichText::new("CLOUD").size(9.0).color(egui::Color32::from_rgb(175, 110, 245)));
                    } else if model.is_embedded_friendly() {
                        ui.label(egui::RichText::new("EMBED").size(9.0).color(ThemeTokens::CYAN_EDGE));
                    }
                });
            });
        }
        TableColumnId::Year => {
            row.col(|ui| {
                let c = match model.release_year { Some(2026) => ThemeTokens::CYAN_EDGE, Some(2025) => egui::Color32::from_rgb(100, 180, 255), _ => ThemeTokens::TEXT_MUTED };
                ui.label(egui::RichText::new(model.release_year.map(|y| y.to_string()).unwrap_or_else(|| "-".into())).color(c).strong());
            });
        }
        TableColumnId::TargetHardware => {
            row.col(|ui| {
                let (r, g, b) = model.hardware_color_rgb();
                ui.label(egui::RichText::new(hardware_badge_text(model)).color(egui::Color32::from_rgb(r, g, b)).strong()).on_hover_text(model.embedded_recommendation());
            });
        }
        TableColumnId::Size => {
            row.col(|ui| {
                if model.is_cloud { ui.label(egui::RichText::new("0 Go (API)").color(ThemeTokens::TEXT_MUTED)); }
                else if model.size_gb < 1.0 { ui.label(egui::RichText::new(format!("{:.0} Mo", model.size_mb())).color(ThemeTokens::CYAN_EDGE).strong()); }
                else { ui.label(format!("{:.1} Go", model.size_gb)); }
            });
        }
        TableColumnId::Efficiency => {
            row.col(|ui| {
                if model.is_cloud { ui.label(egui::RichText::new("-").color(ThemeTokens::TEXT_MUTED)); }
                else {
                    let eff = model.efficiency_ratio_with_sources(&filters.sources);
                    let eff_c = if eff >= 40.0 { ThemeTokens::CYAN_EDGE } else if eff >= 15.0 { ThemeTokens::SUCCESS } else { ThemeTokens::TEXT_SECONDARY };
                    ui.label(egui::RichText::new(format!("{:.1} pts/Go", eff)).color(eff_c).strong());
                }
            });
        }
        TableColumnId::Score => {
            row.col(|ui| {
                if let Some(breakdown) = model.score_breakdown(&filters.sources) {
                    let score = breakdown.final_score;
                    let progress = (score / 100.0).clamp(0.0, 1.0);
                    let (cr, cg, cb) = breakdown.confidence.color_rgb();
                    let conf_color = egui::Color32::from_rgb(cr, cg, cb);
                    let conf = lang.confidence();
                    let conf_tip = match breakdown.confidence {
                        crate::domain::scoring::ScoreConfidence::High => conf.ok_tooltip,
                        crate::domain::scoring::ScoreConfidence::Medium => conf.med_tooltip,
                        crate::domain::scoring::ScoreConfidence::Low => conf.low_tooltip,
                    };
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("{:.1}", score)).strong().color(ThemeTokens::TEXT_PRIMARY));
                        ui.add(egui::ProgressBar::new(progress).desired_width(28.0).fill(ThemeTokens::PRIMARY));
                        ui.label(egui::RichText::new(breakdown.confidence.badge()).color(conf_color).strong().size(11.0))
                            .on_hover_text(conf_tip);
                    }).response.on_hover_text(format!(
                        "{}\n\n{} {:.1} pts\n{}",
                        conf_tip, t.score_variance_label, breakdown.variance, breakdown.formula
                    ));
                } else {
                    ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED));
                }
            });
        }
        TableColumnId::RealWorld => {
            row.col(|ui| {
                render_economic_cell(ui, model);
            });
        }
        TableColumnId::Speed => {
            row.col(|ui| {
                if let Some(tps) = model.tokens_per_sec {
                    let tps_color = if tps >= 100.0 { ThemeTokens::CYAN_EDGE } else if tps >= 45.0 { ThemeTokens::SUCCESS } else { ThemeTokens::WARNING };
                    let hw = model.tps_hardware.as_deref().unwrap_or("Std GPU");
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(format!("{:.0} tok/s", tps)).color(tps_color).strong());
                        ui.label(egui::RichText::new(hw).size(9.0).color(ThemeTokens::TEXT_MUTED));
                    }).response.on_hover_text(format!("{tps:.0} tok/s ({hw})"));
                } else {
                    ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED));
                }
            });
        }
        TableColumnId::Popularity => {
            row.col(|ui| {
                ui.label(format_downloads(model.downloads))
                    .on_hover_text(format!("{} downloads | {} likes", model.downloads, model.likes));
            });
        }
        TableColumnId::MmluPro => {
            row.col(|ui| { render_score_badge_cell(ui, model.scores.mmlu_pro); });
        }
        TableColumnId::SweBench => {
            row.col(|ui| { render_score_badge_cell(ui, model.scores.coding); });
        }
        TableColumnId::Agentic => {
            row.col(|ui| { render_score_badge_cell(ui, model.scores.agentic); });
        }
        TableColumnId::OpenLlm => {
            row.col(|ui| { render_score_badge_cell(ui, model.scores.open_llm); });
        }
        TableColumnId::CloudPrice => {
            row.col(|ui| {
                if model.is_cloud {
                    if let Some(in_p) = model.pricing_input_usd {
                        let out_p = model.pricing_output_usd.unwrap_or(0.0);
                        ui.label(egui::RichText::new(format!("${in_p:.2} / ${out_p:.2}")).color(ThemeTokens::GOLD_5090))
                            .on_hover_text(format!("Prompt: ${in_p:.2} / 1M | Completion: ${out_p:.2} / 1M"));
                    } else {
                        ui.label("Cloud API");
                    }
                } else {
                    ui.label(egui::RichText::new(t.price_free_local).color(ThemeTokens::SUCCESS));
                }
            });
        }
    }
}
