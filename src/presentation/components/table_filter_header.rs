use std::collections::BTreeMap;
use eframe::egui;
use crate::domain::table_filter::{ColumnFilter, FilterOperator};
use crate::presentation::components::table_header_cell::operator_label;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

/// Configuration for a column rendered with sortable + filterable 2-row header controls.
#[derive(Clone, Copy)]
pub struct FilterHeaderCol<'a> {
    pub col_idx: usize,
    pub label: &'a str,
    pub sortable: bool,
    pub default_asc: bool,
    pub filterable: bool,
    pub default_op: FilterOperator,
}

impl<'a> FilterHeaderCol<'a> {
    pub fn sortable(col_idx: usize, label: &'a str, default_asc: bool) -> Self {
        Self {
            col_idx,
            label,
            sortable: true,
            default_asc,
            filterable: true,
            default_op: FilterOperator::Contains,
        }
    }

    pub fn filter_only(col_idx: usize, label: &'a str) -> Self {
        Self {
            col_idx,
            label,
            sortable: false,
            default_asc: true,
            filterable: true,
            default_op: FilterOperator::Contains,
        }
    }

    pub fn static_col(col_idx: usize, label: &'a str) -> Self {
        Self {
            col_idx,
            label,
            sortable: false,
            default_asc: true,
            filterable: false,
            default_op: FilterOperator::Contains,
        }
    }
}

/// Renders the shared Row 2 inline filter controls (`~`, `=`, `<=`, `>=`, `!~`, `.*`, `!.*` + search box + clear).
pub fn render_column_filter_row(
    ui: &mut egui::Ui,
    id_salt: egui::Id,
    cf: &mut ColumnFilter,
    default_op: FilterOperator,
    lang: Language,
) {
    let t = lang.t();
    let op_tooltip = if lang == Language::French {
        format!("Règle : {} (Cliquer pour changer)", operator_label(cf.operator, lang))
    } else {
        format!("Rule: {} (Click to change)", operator_label(cf.operator, lang))
    };

    ui.push_id(id_salt, |ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 22.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);

                let op_btn_text = egui::RichText::new(cf.operator.symbol())
                    .size(11.0)
                    .color(if cf.active { ThemeTokens::CYAN_EDGE } else { ThemeTokens::TEXT_MUTED });

                ui.menu_button(op_btn_text, |ui| {
                    ui.set_min_width(180.0);
                    ui.label(egui::RichText::new(t.table_menu.menu_operator_label).size(10.5).color(ThemeTokens::TEXT_MUTED));
                    ui.separator();
                    for &op in FilterOperator::all() {
                        let label = format!("{:>3}  {}", op.symbol(), operator_label(op, lang));
                        if ui.selectable_value(&mut cf.operator, op, label).clicked() {
                            cf.active = !cf.pattern.trim().is_empty();
                            ui.close_menu();
                        }
                    }
                })
                .response
                .on_hover_text(op_tooltip);

                // Reserve 18px on the right so column layout never shifts when clear button appears
                let edit_w = (ui.available_width() - 18.0).max(18.0);
                let text_edit = ui.add_sized(
                    [edit_w, 18.0],
                    egui::TextEdit::singleline(&mut cf.pattern)
                        .hint_text("...")
                        .clip_text(true)
                        .font(egui::TextStyle::Small)
                        .id_source(id_salt.with("txt")),
                );

                if text_edit.changed() {
                    cf.active = !cf.pattern.trim().is_empty();
                }

                if !cf.pattern.is_empty() && ui.small_button("×").on_hover_text(t.table_menu.menu_clear_filter).clicked() {
                    cf.pattern.clear();
                    cf.active = false;
                    cf.operator = default_op;
                }
            },
        );
    });
}

/// Renders a 2-row (50px) table header cell with clickable sort toggle, right-click menu, and inline column filter.
#[allow(clippy::too_many_arguments)]
pub fn render_generic_header_cell(
    ui: &mut egui::Ui,
    table_id: &str,
    col: FilterHeaderCol<'_>,
    default_sort_col: usize,
    default_sort_asc: bool,
    sort_col: &mut usize,
    sort_asc: &mut bool,
    filters: &mut BTreeMap<usize, ColumnFilter>,
    lang: Language,
) -> egui::Response {
    let t = lang.t();
    let is_sorted = col.sortable && *sort_col == col.col_idx;
    let arrow = if is_sorted {
        if *sort_asc { " ^" } else { " v" }
    } else {
        ""
    };
    let has_filter = col.filterable
        && filters.get(&col.col_idx).is_some_and(|c| c.active && !c.pattern.trim().is_empty());

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);

        let title_resp = ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 22.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let mut title_text = egui::RichText::new(format!("{}{}", col.label, arrow)).size(12.0);
                if has_filter {
                    title_text = title_text.color(ThemeTokens::CYAN_EDGE).strong();
                } else if is_sorted {
                    title_text = title_text.color(ThemeTokens::GOLD_5090).strong();
                } else {
                    title_text = title_text.strong();
                }

                let sense = if col.sortable { egui::Sense::click() } else { egui::Sense::hover() };
                let resp = ui.add(egui::Label::new(title_text).sense(sense));
                if col.sortable {
                    let tip = if lang == Language::French {
                        "Clic gauche : trier | Clic droit : options"
                    } else {
                        "Left click: sort | Right click: options"
                    };
                    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(tip);
                    if resp.clicked() {
                        if *sort_col == col.col_idx {
                            *sort_asc = !*sort_asc;
                        } else {
                            *sort_col = col.col_idx;
                            *sort_asc = col.default_asc;
                        }
                    }
                    resp
                } else {
                    resp
                }
            },
        ).inner;

        if col.filterable {
            let cf = filters.entry(col.col_idx).or_insert_with(|| ColumnFilter {
                operator: col.default_op,
                pattern: String::new(),
                active: false,
            });
            render_column_filter_row(ui, egui::Id::new(table_id).with(col.col_idx), cf, col.default_op, lang);
        } else {
            ui.allocate_space(egui::vec2(ui.available_width().max(1.0), 22.0));
        }

        title_resp.context_menu(|ui| {
            ui.set_min_width(200.0);
            if col.sortable {
                ui.label(egui::RichText::new(t.table_menu.section_sort).size(10.0).color(ThemeTokens::TEXT_MUTED));
                if ui.button(t.table_menu.menu_sort_asc).clicked() {
                    *sort_col = col.col_idx;
                    *sort_asc = true;
                    ui.close_menu();
                }
                if ui.button(t.table_menu.menu_sort_desc).clicked() {
                    *sort_col = col.col_idx;
                    *sort_asc = false;
                    ui.close_menu();
                }
                if ui.button(t.table_menu.menu_reset_sort).clicked() {
                    *sort_col = default_sort_col;
                    *sort_asc = default_sort_asc;
                    ui.close_menu();
                }
            }
            if col.filterable && has_filter {
                if col.sortable { ui.separator(); }
                if ui.button(t.table_menu.menu_clear_filter).clicked() {
                    if let Some(cf) = filters.get_mut(&col.col_idx) {
                        cf.pattern.clear();
                        cf.active = false;
                        cf.operator = col.default_op;
                    }
                    ui.close_menu();
                }
            }
            if active_filter_count(filters) > 0 {
                ui.separator();
                if ui.button(egui::RichText::new(t.table_menu.menu_clear_all_filters).color(ThemeTokens::DANGER)).clicked() {
                    filters.clear();
                    ui.close_menu();
                }
            }
        });

        title_resp
    }).inner
}

/// Counts how many columns currently have a non-empty active filter.
pub fn active_filter_count(filters: &BTreeMap<usize, ColumnFilter>) -> usize {
    filters.values().filter(|f| f.active && !f.pattern.trim().is_empty()).count()
}

/// Evaluates all active column filters lazily, avoiding allocations when no filter is active.
pub fn matches_indexed_filters<F>(filters: &BTreeMap<usize, ColumnFilter>, mut extract_col: F) -> bool
where
    F: FnMut(usize) -> String,
{
    for (&col_idx, cf) in filters {
        if cf.active && !cf.pattern.trim().is_empty() {
            let val = extract_col(col_idx);
            if !cf.matches(&val) {
                return false;
            }
        }
    }
    true
}

/// Renders a compact clear-all badge when one or more column filters are active.
pub fn render_active_filters_chip(
    ui: &mut egui::Ui,
    filters: &mut BTreeMap<usize, ColumnFilter>,
    lang: Language,
) {
    let count = active_filter_count(filters);
    if count == 0 {
        return;
    }
    let lbl = if lang == Language::French {
        format!("Filtres colonnes ({count}) [X]")
    } else {
        format!("Column filters ({count}) [X]")
    };
    if ui.small_button(egui::RichText::new(lbl).color(ThemeTokens::CYAN_EDGE).strong()).clicked() {
        filters.clear();
    }
}
