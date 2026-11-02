use eframe::egui;
use egui_extras::{Column, TableBuilder};
use crate::domain::consistency::{ConsistencyIssue, ModelConsistencyValidator};
use crate::domain::model::{ModelBenchmark, SortField, SortOrder};
use crate::presentation::components::filter_bar::FilterState;
pub use crate::presentation::components::table_items::{build_display_items, format_downloads, hardware_badge_text, header_sort_col, DisplayItem};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_table(
    ui: &mut egui::Ui,
    all_models: &[ModelBenchmark],
    filtered_indices: &[usize],
    filters: &mut FilterState,
    selected_model_id: &mut Option<String>,
    lang: Language,
) {
    let t = lang.t();
    if filtered_indices.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(egui::RichText::new(t.no_matching_models).color(ThemeTokens::TEXT_SECONDARY).size(16.0));
            if ui.button(t.reset_filters).clicked() { filters.reset(); }
        });
        return;
    }

    if filters.column_mode == crate::presentation::filtering::TableColumnMode::RawBenchmarkSources {
        crate::presentation::components::table_raw_view::render_raw_benchmark_table(
            ui, all_models, filtered_indices, filters, selected_model_id, lang
        );
        return;
    }

    let items = build_display_items(all_models, filtered_indices, filters);
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(64.0))                                      // # Rank
        .column(Column::initial(220.0).at_least(160.0).clip(true))        // Model Name & Grouping Toggle
        .column(Column::exact(68.0))                                      // Release Year / Date
        .column(Column::initial(135.0).at_least(105.0))                   // Target Hardware
        .column(Column::initial(80.0).at_least(65.0))                     // Size
        .column(Column::initial(95.0).at_least(80.0))                     // Efficiency (Ratio)
        .column(Column::initial(120.0).at_least(100.0))                   // Calibrated Score
        .column(Column::initial(125.0).at_least(100.0))                   // Real-World Economic
        .column(Column::initial(110.0).at_least(90.0))                    // Speed
        .column(Column::initial(90.0).at_least(75.0))                     // Popularity
        .column(Column::initial(115.0).at_least(90.0))                    // Cloud Pricing
        .header(28.0, |mut header| {
            header.col(|ui| {
                ui.strong(t.col_rank);
            });
            header.col(|ui| {
                ui.horizontal(|ui| {
                    header_sort_col(ui, t.col_model, SortField::Name, SortOrder::Ascending, filters);
                    let folder_icon = if filters.group_by_root { "📁" } else { "📂" };
                    let folder_resp = ui.small_button(folder_icon)
                        .on_hover_text(t.group_tooltip);
                    ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_grouped"), folder_resp.rect));
                    if folder_resp.clicked() {
                        filters.group_by_root = !filters.group_by_root;
                    }
                });
            });
            header.col(|ui| { header_sort_col(ui, t.col_year, SortField::Year, SortOrder::Descending, filters); });
            header.col(|ui| { header_sort_col(ui, t.col_hardware, SortField::Memory, SortOrder::Ascending, filters); });
            header.col(|ui| { header_sort_col(ui, t.col_size, SortField::Size, SortOrder::Ascending, filters); });
            header.col(|ui| { header_sort_col(ui, t.col_efficiency, SortField::Efficiency, SortOrder::Descending, filters); });
            header.col(|ui| { header_sort_col(ui, t.col_score, SortField::Intelligence, SortOrder::Descending, filters); });
            header.col(|ui| { header_sort_col(ui, t.col_real_world, SortField::RealWorld, SortOrder::Descending, filters); });
            header.col(|ui| { header_sort_col(ui, t.col_speed, SortField::Speed, SortOrder::Descending, filters); });
            header.col(|ui| { header_sort_col(ui, t.col_popularity, SortField::Downloads, SortOrder::Descending, filters); });
            header.col(|ui| { header_sort_col(ui, t.col_cloud_price, SortField::Price, SortOrder::Ascending, filters); });
        })
        .body(|body| {
            body.rows(text_height + 14.0, items.len(), |mut row| {
                let item = &items[row.index()];
                let model = &all_models[item.model_idx];
                let is_selected = selected_model_id.as_deref() == Some(&model.id);
                row.set_selected(is_selected);

                let is_first_row = row.index() == 0;

                // Col 1: Rank (Stable hierarchical rank, e.g. #11.1, #11.2, #12)
                row.col(|ui| {
                    if is_first_row {
                        let row_rect = egui::Rect::from_min_max(
                            ui.min_rect().min,
                            egui::pos2(ui.ctx().screen_rect().right() - 24.0, ui.min_rect().min.y + 30.0),
                        );
                        ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_table_row"), row_rect));
                    }
                    if item.is_child {
                        ui.label(egui::RichText::new(&item.rank_str).color(ThemeTokens::TEXT_SECONDARY).size(11.5));
                    } else {
                        ui.label(egui::RichText::new(&item.rank_str).strong().color(ThemeTokens::TEXT_PRIMARY));
                    }
                });

                // Col 2: Model Name & Foldout Arrow
                row.col(|ui| {
                    ui.horizontal(|ui| {
                        if !item.is_child && item.variant_count > 1 {
                            let arrow = if item.is_expanded { "▼" } else { "▶" };
                            let tag = format!("{arrow} [{} var.]", item.variant_count);
                            if ui.small_button(egui::RichText::new(tag).size(10.0).color(ThemeTokens::PRIMARY)).clicked() {
                                if item.is_expanded { filters.expanded_roots.remove(&item.root_name); }
                                else { filters.expanded_roots.insert(item.root_name.clone()); }
                            }
                        }

                        let display_name = if item.is_child {
                            format!("  └─ {}", model.name)
                        } else if filters.group_by_root && !item.is_expanded {
                            item.root_name.clone()
                        } else {
                            model.name.clone()
                        };

                        if ui.selectable_label(is_selected, egui::RichText::new(display_name).strong()).clicked() {
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
                            ui.label(egui::RichText::new("⚠️").color(ThemeTokens::WARNING))
                                .on_hover_text(format!("{title}: {desc}"));
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

                // Col 3: Release Year
                row.col(|ui| {
                    match model.release_year {
                        Some(2026) => { ui.label(egui::RichText::new("2026").color(ThemeTokens::CYAN_EDGE).strong()); }
                        Some(2025) => { ui.label(egui::RichText::new("2025").color(egui::Color32::from_rgb(100, 180, 255)).strong()); }
                        Some(y) => { ui.label(egui::RichText::new(format!("{y}")).color(ThemeTokens::TEXT_MUTED)); }
                        None => { ui.label(egui::RichText::new("—").color(ThemeTokens::TEXT_MUTED)); }
                    }
                });

                // Col 4: Target Hardware
                row.col(|ui| {
                    let (r, g, b) = model.hardware_color_rgb();
                    let badge_color = egui::Color32::from_rgb(r, g, b);
                    ui.label(egui::RichText::new(hardware_badge_text(model)).color(badge_color).strong())
                        .on_hover_text(model.embedded_recommendation());
                });

                // Col 5: Size
                row.col(|ui| {
                    if model.is_cloud {
                        ui.label(egui::RichText::new("0 Go (API)").color(ThemeTokens::TEXT_MUTED));
                    } else if model.size_gb < 1.0 {
                        ui.label(egui::RichText::new(format!("{:.0} Mo", model.size_mb())).color(ThemeTokens::CYAN_EDGE).strong());
                    } else {
                        ui.label(format!("{:.1} Go", model.size_gb));
                    }
                });

                // Col 6: Efficiency
                row.col(|ui| {
                    if model.is_cloud {
                        ui.label(egui::RichText::new("—").color(ThemeTokens::TEXT_MUTED));
                    } else {
                        let eff = model.efficiency_ratio_with_sources(&filters.sources);
                        let eff_color = if eff >= 40.0 { ThemeTokens::CYAN_EDGE } else if eff >= 15.0 { ThemeTokens::SUCCESS } else { ThemeTokens::TEXT_SECONDARY };
                        ui.label(egui::RichText::new(format!("{:.1} ★", eff)).color(eff_color).strong());
                    }
                });

                // Col 7: Calibrated Score
                row.col(|ui| {
                    if let Some(breakdown) = model.score_breakdown(&filters.sources) {
                        let score = breakdown.final_score;
                        let progress = (score / 100.0).clamp(0.0, 1.0);
                        let bar_color = if score >= 75.0 { ThemeTokens::GOLD_5090 } else if score >= 50.0 { ThemeTokens::SUCCESS } else if score >= 30.0 { ThemeTokens::PRIMARY } else { ThemeTokens::WARNING };
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("{:.1}", score)).strong().color(bar_color));
                            ui.add(egui::ProgressBar::new(progress).desired_width(32.0).fill(bar_color));
                            ui.label(egui::RichText::new(breakdown.confidence.badge()).size(10.0));
                        }).response.on_hover_text(format!(
                            "{:?} | {} {:.1} pts\n{}",
                            breakdown.confidence, t.score_variance_label, breakdown.variance, breakdown.formula
                        ));
                    } else {
                        ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED));
                    }
                });

                // Col 8: Real-World / Economic Value
                row.col(|ui| {
                    if let Some(econ) = &model.economic {
                        if let Some(comp) = econ.composite_score() {
                            let comp_color = if comp >= 60.0 { ThemeTokens::GOLD_5090 } else if comp >= 35.0 { ThemeTokens::CYAN_EDGE } else { ThemeTokens::SUCCESS };
                            let info = format!(
                                "RLI: {} | SWE-Clar: {} | SWE-Lancer: {} | WebArena: {} | MLE: {}",
                                econ.rli_automation_rate.map(|v| format!("{v:.1}%")).unwrap_or_else(|| "—".into()),
                                econ.swe_clar_impl.map(|v| format!("{v:.1}%")).unwrap_or_else(|| "—".into()),
                                econ.swe_lancer_score.map(|v| format!("{v:.1}%")).unwrap_or_else(|| "—".into()),
                                econ.web_arena_success.map(|v| format!("{v:.1}%")).unwrap_or_else(|| "—".into()),
                                econ.mle_bench_score.map(|v| format!("{v:.1}%")).unwrap_or_else(|| "—".into()),
                            );
                            ui.label(egui::RichText::new(format!("💼 {comp:.1} %")).strong().color(comp_color))
                                .on_hover_text(info);
                        } else {
                            ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED));
                        }
                    } else if let Some(rw) = model.scores.real_world {
                        ui.label(egui::RichText::new(format!("💼 {rw:.1} %")).color(ThemeTokens::CYAN_EDGE));
                    } else {
                        ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED));
                    }
                });

                // Col 9: Speed with Hardware Reference
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

                // Col 10: Popularity (Downloads & Likes)
                row.col(|ui| {
                    ui.label(format_downloads(model.downloads))
                        .on_hover_text(format!("{} downloads | {} likes", model.downloads, model.likes));
                });

                // Col 11: Cloud Pricing
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
            });
        });
}
