use eframe::egui;
use egui_extras::{Column, TableBuilder};
use crate::domain::model::{ModelBenchmark, SortField, SortOrder};
use crate::presentation::components::filter_bar::FilterState;
pub use crate::presentation::components::table_items::{
    build_display_items, hardware_badge_text, header_sort_col, render_vs_button, DisplayItem,
};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_simple_table(
    ui: &mut egui::Ui,
    all_models: &[ModelBenchmark],
    filtered_indices: &[usize],
    filters: &mut FilterState,
    selected_model_id: &mut Option<String>,
    lang: Language,
) {
    let t = lang.t();
    let items = build_display_items(all_models, filtered_indices, filters);
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(850.0);
            TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(64.0))                               // 1. # Rank & VS
                .column(Column::initial(240.0).at_least(180.0).clip(true)) // 2. Model Name & Group
                .column(Column::initial(140.0).at_least(110.0))            // 3. Target Hardware
                .column(Column::initial(90.0).at_least(75.0))              // 4. Size (RAM / VRAM)
                .column(Column::initial(130.0).at_least(110.0))            // 5. Score Calibré
                .column(Column::initial(110.0).at_least(90.0))             // 6. Vitesse (tok/s)
                .column(Column::initial(115.0).at_least(90.0))             // 7. Tarif / Éco
                .header(28.0, |mut header| {
                    header.col(|ui| { ui.strong(t.col_rank); });
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            header_sort_col(ui, t.col_model, SortField::Name, SortOrder::Ascending, filters);
                            let folder_icon = if filters.group_by_root { "[+]" } else { "[-]" };
                            if ui.small_button(folder_icon).on_hover_text(t.group_tooltip).clicked() {
                                filters.group_by_root = !filters.group_by_root;
                            }
                        });
                    });
                    header.col(|ui| { header_sort_col(ui, t.col_hardware, SortField::Memory, SortOrder::Ascending, filters); });
                    header.col(|ui| { header_sort_col(ui, t.col_size, SortField::Size, SortOrder::Ascending, filters); });
                    header.col(|ui| {
                        let resp = header_sort_col(ui, t.col_score, SortField::Intelligence, SortOrder::Descending, filters);
                        resp.on_hover_text(lang.confidence().col_tooltip);
                    });
                    header.col(|ui| { header_sort_col(ui, t.col_speed, SortField::Speed, SortOrder::Descending, filters); });
                    header.col(|ui| { header_sort_col(ui, t.col_cloud_price, SortField::Price, SortOrder::Ascending, filters); });
                })
                .body(|body| {
                    body.rows(text_height + 14.0, items.len(), |mut row| {
                        let item = &items[row.index()];
                        let model = &all_models[item.model_idx];
                        let is_selected = selected_model_id.as_deref() == Some(&model.id);
                        row.set_selected(is_selected);

                        // 1. # Rank & Comparison Button
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                render_vs_button(ui, &model.id, filters, lang);
                                let r_color = if item.is_child { ThemeTokens::TEXT_SECONDARY } else { ThemeTokens::TEXT_PRIMARY };
                                ui.label(egui::RichText::new(&item.rank_str).color(r_color).size(11.0));
                            });
                        });

                        // 2. Model Name & Group
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                if !item.is_child && (item.variant_count > 1 || item.is_expanded) {
                                    let arrow = if item.is_expanded { "v" } else { ">" };
                                    let tag = format!("{arrow} [{} var.]", item.variant_count);
                                    if ui.small_button(egui::RichText::new(tag).size(10.0).color(ThemeTokens::PRIMARY)).clicked() {
                                        let has_search = !filters.search_text.trim().is_empty();
                                        if has_search {
                                            let key = format!("collapsed:{}", item.root_name);
                                            if item.is_expanded { filters.expanded_roots.insert(key); }
                                            else { filters.expanded_roots.remove(&key); }
                                        } else if item.is_expanded {
                                            filters.expanded_roots.remove(&item.root_name);
                                        } else {
                                            filters.expanded_roots.insert(item.root_name.clone());
                                        }
                                    }
                                }
                                let display_name = if item.is_child {
                                    format!("  |-- {}", model.name)
                                } else if filters.group_by_root && (item.variant_count > 1 || item.is_expanded) {
                                    item.root_name.clone()
                                } else {
                                    model.name.clone()
                                };
                                let resp = ui.selectable_label(
                                    is_selected,
                                    egui::RichText::new(display_name).strong().color(ThemeTokens::TEXT_PRIMARY),
                                );
                                if resp.clicked() {
                                    *selected_model_id = Some(model.id.clone());
                                }
                                let mod_rgb = model.modality.badge_color_rgb();
                                ui.label(egui::RichText::new(model.modality.tag_short()).size(9.0).color(egui::Color32::from_rgb(mod_rgb.0, mod_rgb.1, mod_rgb.2)))
                                    .on_hover_text(model.modality.label());
                            });
                        });

                        // 3. Target Hardware
                        row.col(|ui| {
                            let (r, g, b) = model.hardware_color_rgb();
                            ui.label(egui::RichText::new(hardware_badge_text(model)).color(egui::Color32::from_rgb(r, g, b)).strong()).on_hover_text(model.embedded_recommendation());
                        });

                        // 4. Size (RAM / VRAM requirement)
                        row.col(|ui| {
                            if model.is_cloud { ui.label(egui::RichText::new("0 Go (API)").color(ThemeTokens::TEXT_MUTED)); }
                            else if model.size_gb < 1.0 { ui.label(egui::RichText::new(format!("{:.0} Mo", model.size_mb())).color(ThemeTokens::CYAN_EDGE).strong()); }
                            else { ui.label(format!("{:.1} Go", model.size_gb)); }
                        });

                        // 5. Score Calibré
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

                        // 6. Vitesse (tok/s)
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

                        // 7. Tarif / Éco
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
        });
}
