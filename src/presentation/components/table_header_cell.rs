use eframe::egui;
use crate::domain::model::{SortField, SortOrder};
use crate::domain::table_preferences::{ColumnFilter, FilterOperator, TableColumnId};
use crate::presentation::components::filter_bar::FilterState;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_header_cell(
    ui: &mut egui::Ui,
    col_id: TableColumnId,
    label: &str,
    sort_field: Option<SortField>,
    default_order: SortOrder,
    filters: &mut FilterState,
    lang: Language,
) -> egui::Response {
    let t = lang.t();
    let is_sorted = sort_field == Some(filters.sort_field);
    let arrow = if is_sorted {
        match filters.sort_order {
            SortOrder::Ascending => " ⬆",
            SortOrder::Descending => " ⬇",
        }
    } else {
        ""
    };

    let has_filter = filters.table_prefs.has_active_filter(col_id);
    let filter_badge = if has_filter { " 🔍" } else { "" };
    let text = format!("{label}{arrow}{filter_badge}");

    let mut text_widget = egui::RichText::new(text);
    if has_filter {
        text_widget = text_widget.color(ThemeTokens::CYAN_EDGE).strong();
    } else if is_sorted {
        text_widget = text_widget.color(ThemeTokens::GOLD_5090).strong();
    }

    let resp = ui.button(text_widget);

    let hover_text = if has_filter {
        if let Some(cf) = filters.table_prefs.column_filters.get(&col_id) {
            format!("{} : \"{}\" (Clic droit pour éditer)", operator_label(cf.operator, lang), cf.pattern)
        } else {
            "Clic droit pour filtrer / réordonner".into()
        }
    } else {
        "Clic gauche pour trier | Clic droit pour filtrer & ordonner".into()
    };
    let resp = resp.on_hover_text(hover_text);

    // Left click: Toggle sort
    if resp.clicked() {
        if let Some(sf) = sort_field {
            if filters.sort_field == sf {
                filters.sort_order = match filters.sort_order {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::Ascending,
                };
            } else {
                filters.sort_field = sf;
                filters.sort_order = default_order;
            }
            filters.table_prefs.sort_field = filters.sort_field;
            filters.table_prefs.sort_order = filters.sort_order;
        }
    }

    // Right click: Context menu for Filter & Column Operations
    resp.context_menu(|ui| {
        ui.set_min_width(220.0);

        // --- SECTION 1: SORTING ---
        if let Some(sf) = sort_field {
            ui.label(egui::RichText::new("TRI DE COLONNE").size(10.5).color(ThemeTokens::TEXT_MUTED));
            if ui.button(t.table_menu.menu_sort_asc).clicked() {
                filters.sort_field = sf;
                filters.sort_order = SortOrder::Ascending;
                filters.table_prefs.sort_field = sf;
                filters.table_prefs.sort_order = SortOrder::Ascending;
                ui.close_menu();
            }
            if ui.button(t.table_menu.menu_sort_desc).clicked() {
                filters.sort_field = sf;
                filters.sort_order = SortOrder::Descending;
                filters.table_prefs.sort_field = sf;
                filters.table_prefs.sort_order = SortOrder::Descending;
                ui.close_menu();
            }
            if ui.button(t.table_menu.menu_reset_sort).clicked() {
                filters.sort_field = SortField::Intelligence;
                filters.sort_order = SortOrder::Descending;
                filters.table_prefs.sort_field = SortField::Intelligence;
                filters.table_prefs.sort_order = SortOrder::Descending;
                ui.close_menu();
            }
            ui.separator();
        }

        // --- SECTION 2: FILTERING ---
        ui.label(egui::RichText::new(t.table_menu.menu_filter_title).strong().color(ThemeTokens::CYAN_EDGE).size(11.5));
        let cf = filters.table_prefs.column_filters.entry(col_id).or_insert_with(ColumnFilter::default);

        ui.horizontal(|ui| {
            ui.label(t.table_menu.menu_operator_label);
            egui::ComboBox::from_id_salt(format!("op_combo_{:?}", col_id))
                .selected_text(operator_label(cf.operator, lang))
                .show_ui(ui, |ui| {
                    for &op in FilterOperator::all() {
                        ui.selectable_value(&mut cf.operator, op, operator_label(op, lang));
                    }
                });
        });

        ui.add_space(3.0);
        let text_edit = ui.text_edit_singleline(&mut cf.pattern);
        if text_edit.changed() {
            cf.active = !cf.pattern.trim().is_empty();
        }

        ui.horizontal(|ui| {
            if ui.button(egui::RichText::new(t.table_menu.menu_apply_filter).strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                cf.active = !cf.pattern.trim().is_empty();
                ui.close_menu();
            }
            if cf.active && ui.button(t.table_menu.menu_clear_filter).clicked() {
                cf.pattern.clear();
                cf.active = false;
                ui.close_menu();
            }
        });

        ui.separator();

        // --- SECTION 3: REORDERING ---
        ui.label(egui::RichText::new("ORDRE DE LA COLONNE").size(10.5).color(ThemeTokens::TEXT_MUTED));
        let col_pos = filters.table_prefs.column_order.iter().position(|&c| c == col_id).unwrap_or(0);
        let max_pos = filters.table_prefs.column_order.len().saturating_sub(1);

        ui.horizontal(|ui| {
            let left_btn = ui.add_enabled(col_pos > 0, egui::Button::new(t.table_menu.menu_move_left));
            if left_btn.clicked() {
                filters.table_prefs.move_column(col_id, -1);
                ui.close_menu();
            }

            let right_btn = ui.add_enabled(col_pos < max_pos, egui::Button::new(t.table_menu.menu_move_right));
            if right_btn.clicked() {
                filters.table_prefs.move_column(col_id, 1);
                ui.close_menu();
            }
        });

        if ui.button(t.table_menu.menu_reset_columns).clicked() {
            filters.table_prefs.reset_order();
            ui.close_menu();
        }

        // --- SECTION 4: GLOBAL RESET IF ANY ACTIVE ---
        if filters.table_prefs.active_filter_count() > 0 {
            ui.separator();
            if ui.button(egui::RichText::new(t.table_menu.menu_clear_all_filters).color(ThemeTokens::DANGER)).clicked() {
                filters.table_prefs.reset_filters();
                ui.close_menu();
            }
        }
    });

    resp
}

fn operator_label(op: FilterOperator, lang: Language) -> &'static str {
    let t = lang.t();
    match op {
        FilterOperator::Contains => t.table_menu.op_contains,
        FilterOperator::Exact => t.table_menu.op_exact,
        FilterOperator::NotContains => t.table_menu.op_not_contains,
        FilterOperator::MatchRegex => t.table_menu.op_match_regex,
        FilterOperator::NotMatchRegex => t.table_menu.op_not_match_regex,
    }
}
