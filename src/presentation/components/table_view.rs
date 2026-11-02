use eframe::egui;
use egui_extras::{Column, TableBuilder};
use crate::domain::model::{ModelBenchmark, SortField, SortOrder};
use crate::domain::table_preferences::TableColumnId;
use crate::presentation::components::filter_bar::FilterState;
use crate::presentation::components::table_header_cell::render_header_cell;
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
    let items = build_display_items(all_models, filtered_indices, filters);
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    let visible_cols: Vec<TableColumnId> = filters.table_prefs.column_order.iter()
        .copied()
        .filter(|c| filters.table_prefs.is_visible(*c))
        .collect();

    egui::ScrollArea::horizontal()
        .id_salt("table_scroll_area_horizontal")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let any_raw = visible_cols.iter().any(|c| c.is_raw_source());
            let any_adv = visible_cols.iter().any(|c| c.is_advanced());
            let min_w = if any_raw && any_adv {
                1450.0
            } else if any_raw || any_adv {
                1220.0
            } else {
                950.0
            };
            ui.set_min_width(min_w);

            let mut table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center));

            for &col in &visible_cols {
                table = table.column(column_spec(col));
            }

            table.header(50.0, |mut header| {
                for &col in &visible_cols {
                    header.col(|ui| {
                        let (label, sf, def_order) = column_header_info(col, lang);
                        let resp = render_header_cell(ui, col, label, sf, def_order, filters, lang);
                        if col == TableColumnId::Score {
                            resp.on_hover_text(lang.confidence().col_tooltip);
                        }
                    });
                }
            })
            .body(|mut body| {
                if items.is_empty() {
                    body.row(50.0, |mut row| {
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(t.no_matching_models).color(ThemeTokens::TEXT_MUTED).size(13.0));
                                if ui.button(t.reset_filters).clicked() {
                                    filters.reset();
                                }
                            });
                        });
                    });
                } else {
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
                }
            });
        });

    crate::presentation::components::compare_floating_dock::render_floating_compare_dock(
        ui.ctx(), filters, all_models, lang,
    );

    crate::presentation::components::column_chooser_modal::render_column_chooser_modal(
        ui.ctx(),
        &mut filters.column_chooser_open,
        &mut filters.table_prefs,
        lang,
    );
}

fn column_spec(col: TableColumnId) -> Column {
    match col {
        TableColumnId::Rank => Column::exact(64.0).clip(true),
        TableColumnId::Model => Column::initial(220.0).at_least(160.0).clip(true),
        TableColumnId::Year => Column::exact(68.0).clip(true),
        TableColumnId::TargetHardware => Column::initial(135.0).at_least(105.0).clip(true),
        TableColumnId::Size => Column::initial(80.0).at_least(65.0).clip(true),
        TableColumnId::Efficiency => Column::initial(95.0).at_least(80.0).clip(true),
        TableColumnId::Score => Column::initial(120.0).at_least(100.0).clip(true),
        TableColumnId::RealWorld => Column::initial(125.0).at_least(100.0).clip(true),
        TableColumnId::Speed => Column::initial(110.0).at_least(90.0).clip(true),
        TableColumnId::Popularity => Column::initial(90.0).at_least(75.0).clip(true),
        TableColumnId::MmluPro
        | TableColumnId::SweBench
        | TableColumnId::Agentic
        | TableColumnId::OpenLlm => Column::initial(80.0).at_least(65.0).clip(true),
        TableColumnId::CloudPrice => Column::initial(115.0).at_least(90.0).clip(true),
    }
}

fn column_header_info(
    col: TableColumnId,
    lang: Language,
) -> (&'static str, Option<SortField>, SortOrder) {
    let t = lang.t();
    match col {
        TableColumnId::Rank => (t.col_rank, None, SortOrder::Ascending),
        TableColumnId::Model => (t.col_model, Some(SortField::Name), SortOrder::Ascending),
        TableColumnId::Year => (t.col_year, Some(SortField::Year), SortOrder::Descending),
        TableColumnId::TargetHardware => (t.col_hardware, Some(SortField::Memory), SortOrder::Ascending),
        TableColumnId::Size => (t.col_size, Some(SortField::Size), SortOrder::Ascending),
        TableColumnId::Efficiency => (t.col_efficiency, Some(SortField::Efficiency), SortOrder::Descending),
        TableColumnId::Score => (t.col_score, Some(SortField::Intelligence), SortOrder::Descending),
        TableColumnId::RealWorld => (t.col_real_world, Some(SortField::RealWorld), SortOrder::Descending),
        TableColumnId::Speed => (t.col_speed, Some(SortField::Speed), SortOrder::Descending),
        TableColumnId::Popularity => (t.col_popularity, Some(SortField::Downloads), SortOrder::Descending),
        TableColumnId::MmluPro => (t.col_mmlu_pro, None, SortOrder::Descending),
        TableColumnId::SweBench => (t.col_swe_bench, Some(SortField::Coding), SortOrder::Descending),
        TableColumnId::Agentic => (t.col_agentic, Some(SortField::Agentic), SortOrder::Descending),
        TableColumnId::OpenLlm => (t.col_open_llm, Some(SortField::Intelligence), SortOrder::Descending),
        TableColumnId::CloudPrice => (t.col_cloud_price, Some(SortField::Price), SortOrder::Ascending),
    }
}
