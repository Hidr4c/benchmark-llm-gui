use eframe::egui;
use egui_extras::{Column, TableBuilder};
use crate::domain::model::{ModelBenchmark, SortField, SortOrder};
use crate::presentation::components::filter_bar::FilterState;
pub use crate::presentation::components::table_items::{
    build_display_items, format_downloads, hardware_badge_text, header_sort_col,
    render_economic_cell, render_score_badge_cell, render_vs_button, DisplayItem,
};
use crate::presentation::components::table_row::render_table_row;
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

    let items = build_display_items(all_models, filtered_indices, filters);
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let min_w = if filters.show_raw_sources && filters.show_advanced_columns {
                1450.0
            } else if filters.show_raw_sources || filters.show_advanced_columns {
                1220.0
            } else {
                950.0
            };
            ui.set_min_width(min_w);

            let mut table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(64.0))                                      // Col 1: # Rank
                .column(Column::initial(220.0).at_least(160.0).clip(true));       // Col 2: Model Name

            if filters.show_advanced_columns {
                table = table.column(Column::exact(68.0));                         // Year
            }

            table = table
                .column(Column::initial(135.0).at_least(105.0))                   // Target Hardware
                .column(Column::initial(80.0).at_least(65.0));                    // Size

            if filters.show_advanced_columns {
                table = table.column(Column::initial(95.0).at_least(80.0));       // Efficiency
            }

            table = table.column(Column::initial(120.0).at_least(100.0));        // Calibrated Score

            if filters.show_advanced_columns {
                table = table.column(Column::initial(125.0).at_least(100.0));     // Real-World Economic
            }

            table = table.column(Column::initial(110.0).at_least(90.0));          // Speed

            if filters.show_advanced_columns {
                table = table.column(Column::initial(90.0).at_least(75.0));       // Popularity
            }

            if filters.show_raw_sources {
                table = table
                    .column(Column::initial(80.0).at_least(65.0))                 // MMLU-PRO
                    .column(Column::initial(80.0).at_least(65.0))                 // SWE-Bench
                    .column(Column::initial(80.0).at_least(65.0))                 // Agentic
                    .column(Column::initial(80.0).at_least(65.0));                // OpenLLM
            }

            table = table.column(Column::initial(115.0).at_least(90.0));          // Cloud Pricing

            table.header(28.0, |mut header| {
                header.col(|ui| { ui.strong(t.col_rank); });
                header.col(|ui| {
                    ui.horizontal(|ui| {
                        header_sort_col(ui, t.col_model, SortField::Name, SortOrder::Ascending, filters);
                        let folder_icon = if filters.group_by_root { "[+]" } else { "[-]" };
                        let folder_resp = ui.small_button(folder_icon).on_hover_text(t.group_tooltip);
                        ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_grouped"), folder_resp.rect));
                        if folder_resp.clicked() {
                            filters.group_by_root = !filters.group_by_root;
                        }
                    });
                });
                if filters.show_advanced_columns {
                    header.col(|ui| { header_sort_col(ui, t.col_year, SortField::Year, SortOrder::Descending, filters); });
                }
                header.col(|ui| { header_sort_col(ui, t.col_hardware, SortField::Memory, SortOrder::Ascending, filters); });
                header.col(|ui| { header_sort_col(ui, t.col_size, SortField::Size, SortOrder::Ascending, filters); });
                if filters.show_advanced_columns {
                    header.col(|ui| { header_sort_col(ui, t.col_efficiency, SortField::Efficiency, SortOrder::Descending, filters); });
                }
                header.col(|ui| {
                    let resp = header_sort_col(ui, t.col_score, SortField::Intelligence, SortOrder::Descending, filters);
                    resp.on_hover_text(lang.confidence().col_tooltip);
                });
                if filters.show_advanced_columns {
                    header.col(|ui| { header_sort_col(ui, t.col_real_world, SortField::RealWorld, SortOrder::Descending, filters); });
                }
                header.col(|ui| { header_sort_col(ui, t.col_speed, SortField::Speed, SortOrder::Descending, filters); });
                if filters.show_advanced_columns {
                    header.col(|ui| { header_sort_col(ui, t.col_popularity, SortField::Downloads, SortOrder::Descending, filters); });
                }
                if filters.show_raw_sources {
                    header.col(|ui| { ui.strong(t.col_mmlu_pro); });
                    header.col(|ui| { header_sort_col(ui, t.col_swe_bench, SortField::Coding, SortOrder::Descending, filters); });
                    header.col(|ui| { header_sort_col(ui, t.col_agentic, SortField::Agentic, SortOrder::Descending, filters); });
                    header.col(|ui| { header_sort_col(ui, t.col_open_llm, SortField::Intelligence, SortOrder::Descending, filters); });
                }
                header.col(|ui| { header_sort_col(ui, t.col_cloud_price, SortField::Price, SortOrder::Ascending, filters); });
            })
            .body(|body| {
                body.rows(text_height + 14.0, items.len(), |mut row| {
                    let item = &items[row.index()];
                    let model = &all_models[item.model_idx];
                    let is_selected = selected_model_id.as_deref() == Some(&model.id);
                    let is_first_row = row.index() == 0;

                    render_table_row(
                        &mut row,
                        item,
                        model,
                        is_first_row,
                        is_selected,
                        filters,
                        selected_model_id,
                        lang,
                    );
                });
            });
        });

    crate::presentation::components::compare_floating_dock::render_floating_compare_dock(
        ui.ctx(), filters, all_models, lang,
    );
}
