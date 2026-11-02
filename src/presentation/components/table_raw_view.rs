use eframe::egui;
use egui_extras::{Column, TableBuilder};
use crate::domain::model::{ModelBenchmark, SortField, SortOrder};
use crate::presentation::components::filter_bar::FilterState;
use crate::presentation::components::table_items::{build_display_items, header_sort_col};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_raw_benchmark_table(
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
            ui.set_min_width(1150.0);
            TableBuilder::new(ui)
                .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(64.0))                                      // # Rank
        .column(Column::initial(220.0).at_least(160.0).clip(true))        // Model Name & Grouping Toggle
        .column(Column::exact(68.0))                                      // Year
        .column(Column::initial(95.0).at_least(80.0))                     // Open LLM 2
        .column(Column::initial(100.0).at_least(85.0))                    // SWE-Bench (Code)
        .column(Column::initial(100.0).at_least(85.0))                    // Agents (Tools)
        .column(Column::initial(90.0).at_least(75.0))                     // MMLU-PRO
        .column(Column::initial(90.0).at_least(75.0))                     // MATH Lvl 5
        .column(Column::initial(80.0).at_least(70.0))                     // GPQA
        .column(Column::initial(85.0).at_least(75.0))                     // IFEval
        .column(Column::initial(115.0).at_least(95.0))                    // Real-World / Eco
        .header(28.0, |mut header| {
            header.col(|ui| { ui.strong(t.col_rank); });
            header.col(|ui| {
                ui.horizontal(|ui| {
                    header_sort_col(ui, t.col_model, SortField::Name, SortOrder::Ascending, filters);
                    let folder_icon = if filters.group_by_root { "📁" } else { "📂" };
                    let folder_resp = ui.small_button(folder_icon).on_hover_text(t.group_tooltip);
                    ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_grouped"), folder_resp.rect));
                    if folder_resp.clicked() {
                        filters.group_by_root = !filters.group_by_root;
                    }
                });
            });
            header.col(|ui| { header_sort_col(ui, t.col_year, SortField::Year, SortOrder::Descending, filters); });
            header.col(|ui| { header_sort_col(ui, t.col_open_llm, SortField::Intelligence, SortOrder::Descending, filters); });
            header.col(|ui| { header_sort_col(ui, t.col_swe_bench, SortField::Coding, SortOrder::Descending, filters); });
            header.col(|ui| { header_sort_col(ui, t.col_agentic, SortField::Agentic, SortOrder::Descending, filters); });
            header.col(|ui| { ui.strong(t.col_mmlu_pro); });
            header.col(|ui| { ui.strong(t.col_math); });
            header.col(|ui| { ui.strong(t.col_gpqa); });
            header.col(|ui| { ui.strong(t.col_ifeval); });
            header.col(|ui| { header_sort_col(ui, t.col_real_world, SortField::RealWorld, SortOrder::Descending, filters); });
        })
        .body(|body| {
            body.rows(text_height + 14.0, items.len(), |mut row| {
                let item = &items[row.index()];
                let model = &all_models[item.model_idx];
                let is_selected = selected_model_id.as_deref() == Some(&model.id);
                row.set_selected(is_selected);

                let is_first_row = row.index() == 0;

                // Col 1: Rank (Stable hierarchical rank)
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
                    if item.is_child { ui.label(egui::RichText::new(&item.rank_str).color(ThemeTokens::TEXT_SECONDARY).size(11.5)); }
                    else { ui.label(egui::RichText::new(&item.rank_str).strong().color(ThemeTokens::TEXT_PRIMARY)); }
                });

                // Col 2: Model Name
                row.col(|ui| {
                    ui.horizontal(|ui| {
                        if !item.is_child && item.variant_count > 1 {
                            let arrow = if item.is_expanded { "▼" } else { "▶" };
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

                        let display_name = if item.is_child { format!("  └─ {}", model.name) }
                        else if filters.group_by_root { item.root_name.clone() }
                        else { model.name.clone() };

                        let name_resp = ui.selectable_label(is_selected, egui::RichText::new(display_name).strong());
                        if is_first_row {
                            ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_model_name"), name_resp.rect));
                        }
                        if name_resp.clicked() {
                            *selected_model_id = Some(model.id.clone());
                        }

                        let mod_rgb = model.modality.badge_color_rgb();
                        ui.label(egui::RichText::new(model.modality.tag_short()).size(9.0).color(egui::Color32::from_rgb(mod_rgb.0, mod_rgb.1, mod_rgb.2)))
                            .on_hover_text(model.modality.label());
                    });
                });

                // Col 3: Year
                row.col(|ui| {
                    match model.release_year {
                        Some(2026) => { ui.label(egui::RichText::new("✨ 2026").color(ThemeTokens::CYAN_EDGE).strong()); }
                        Some(2025) => { ui.label(egui::RichText::new("🔹 2025").color(egui::Color32::from_rgb(100, 180, 255)).strong()); }
                        Some(y) => { ui.label(egui::RichText::new(format!("{y}")).color(ThemeTokens::TEXT_MUTED)); }
                        None => { ui.label(egui::RichText::new("—").color(ThemeTokens::TEXT_MUTED)); }
                    }
                });

                // Helper for rendering score cells
                fn render_score_cell(ui: &mut egui::Ui, score: Option<f32>) {
                    if let Some(sc) = score {
                        let col = if sc >= 75.0 { ThemeTokens::GOLD_5090 }
                        else if sc >= 50.0 { ThemeTokens::SUCCESS }
                        else if sc >= 30.0 { ThemeTokens::PRIMARY }
                        else { ThemeTokens::TEXT_SECONDARY };
                        ui.label(egui::RichText::new(format!("{sc:.1}%")).strong().color(col));
                    } else {
                        ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED));
                    }
                }

                // Col 4: Open LLM 2
                row.col(|ui| { render_score_cell(ui, model.scores.open_llm); });

                // Col 5: SWE-Bench (Code)
                row.col(|ui| { render_score_cell(ui, model.scores.coding); });

                // Col 6: Agents & Tools
                row.col(|ui| { render_score_cell(ui, model.scores.agentic); });

                // Col 7: MMLU-PRO
                row.col(|ui| { render_score_cell(ui, model.scores.mmlu_pro); });

                // Col 8: MATH Lvl 5
                row.col(|ui| { render_score_cell(ui, model.scores.math_lvl5); });

                // Col 9: GPQA
                row.col(|ui| { render_score_cell(ui, model.scores.gpqa); });

                // Col 10: IFEval
                row.col(|ui| { render_score_cell(ui, model.scores.ifeval); });

                // Col 11: Real-World / Economic
                row.col(|ui| {
                    let rw = model.economic.as_ref().and_then(|e| e.composite_score()).or(model.scores.real_world);
                    render_score_cell(ui, rw);
                });
            });
        });
    });
}
