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
    let is_dragging_any_col = egui::DragAndDrop::has_payload_of_type::<TableColumnId>(ui.ctx());

    let (drop_resp, dropped_col) = ui.dnd_drop_zone::<TableColumnId, _>(
        egui::Frame::none(),
        |ui| {
            render_header_cell_content(ui, col_id, label, sort_field, default_order, filters, lang)
        },
    );

    if let Some(source_payload) = dropped_col {
        let source_col = *source_payload;
        if source_col != col_id {
            filters.table_prefs.reorder_column(source_col, col_id);
        }
    }

    if is_dragging_any_col && drop_resp.response.hovered() {
        ui.painter().rect_stroke(
            drop_resp.response.rect,
            3.0,
            egui::Stroke::new(2.0, ThemeTokens::CYAN_EDGE),
        );
    }

    drop_resp.inner
}

fn render_header_cell_content(
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

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);

        // --- Row 1: Column Title & Drag Source (Fixed Height 22.0px) ---
        let title_resp = ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 22.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);

                let dnd_id = egui::Id::new("col_header_dnd").with(col_id);
                let drag_source = ui.dnd_drag_source(dnd_id, col_id, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);
                        let grab_icon = egui::RichText::new("⠿").size(10.0).color(ThemeTokens::TEXT_MUTED);
                        ui.label(grab_icon);

                        let mut title_text = egui::RichText::new(format!("{label}{arrow}")).size(12.0);
                        if has_filter {
                            title_text = title_text.color(ThemeTokens::CYAN_EDGE).strong();
                        } else if is_sorted {
                            title_text = title_text.color(ThemeTokens::GOLD_5090).strong();
                        }
                        ui.label(title_text);
                    });
                });

                let btn_resp = drag_source.response
                    .on_hover_cursor(egui::CursorIcon::Grab)
                    .on_hover_text(t.table_menu.header_tooltip);

                if btn_resp.clicked() {
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

                // If this is the Model column, place group [+] / [-] button directly on the title row
                if col_id == TableColumnId::Model {
                    let folder_icon = if filters.group_by_root { "[+]" } else { "[-]" };
                    let folder_resp = ui.small_button(folder_icon).on_hover_text(t.group_tooltip);
                    ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_grouped"), folder_resp.rect));
                    if folder_resp.clicked() {
                        filters.group_by_root = !filters.group_by_root;
                    }
                }

                btn_resp
            },
        ).inner;

        // --- Row 2: Continuous Inline Filter (Fixed Height 22.0px) ---
        let default_op = col_id.default_operator();
        let cf = filters.table_prefs.column_filters.entry(col_id).or_insert_with(|| ColumnFilter {
            operator: default_op,
            pattern: String::new(),
            active: false,
        });

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 22.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);

                // Operator selector button
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
                }).response.on_hover_text(format!("Règle : {} (Cliquer pour changer)", operator_label(cf.operator, lang)));

                // Input field: always reserve 18px for clear button so column width never oscillates!
                let edit_w = (ui.available_width() - 18.0).max(18.0);
                let text_edit = ui.add_sized(
                    [edit_w, 18.0],
                    egui::TextEdit::singleline(&mut cf.pattern)
                        .hint_text("...")
                        .clip_text(true)
                        .font(egui::TextStyle::Small),
                );

                if text_edit.changed() {
                    cf.active = !cf.pattern.trim().is_empty();
                }

                // Quick clear button
                if !cf.pattern.is_empty() {
                    if ui.small_button("×").on_hover_text("Effacer le filtre").clicked() {
                        cf.pattern.clear();
                        cf.active = false;
                        cf.operator = FilterOperator::Contains;
                    }
                }
            },
        );

        // Context Menu (Right Click on Title/Cell)
        title_resp.context_menu(|ui| {
            ui.set_min_width(220.0);

            // Column Chooser
            if ui.button(egui::RichText::new(format!("⚙ {}", t.table_menu.btn_column_chooser)).strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                filters.column_chooser_open = true;
                ui.close_menu();
            }
            ui.separator();

            // Sorting
            if let Some(sf) = sort_field {
                ui.label(egui::RichText::new(t.table_menu.section_sort).size(10.0).color(ThemeTokens::TEXT_MUTED));
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

            // Column Reordering
            ui.label(egui::RichText::new(t.table_menu.section_order).size(10.0).color(ThemeTokens::TEXT_MUTED));
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

            // Reset filters
            if filters.table_prefs.active_filter_count() > 0 {
                ui.separator();
                if ui.button(egui::RichText::new(t.table_menu.menu_clear_all_filters).color(ThemeTokens::DANGER)).clicked() {
                    filters.table_prefs.reset_filters();
                    ui.close_menu();
                }
            }
        });

        title_resp
    }).inner
}

pub fn operator_label(op: FilterOperator, lang: Language) -> &'static str {
    let t = lang.t();
    match op {
        FilterOperator::Contains => t.table_menu.op_contains,
        FilterOperator::Exact => t.table_menu.op_exact,
        FilterOperator::LessEqual => t.table_menu.op_less_equal,
        FilterOperator::GreaterEqual => t.table_menu.op_greater_equal,
        FilterOperator::NotContains => t.table_menu.op_not_contains,
        FilterOperator::MatchRegex => t.table_menu.op_match_regex,
        FilterOperator::NotMatchRegex => t.table_menu.op_not_match_regex,
    }
}
