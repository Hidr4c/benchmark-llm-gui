use std::collections::BTreeMap;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use crate::domain::model::{ModelBenchmark, SourceSelection};
use crate::domain::simulator::{HardwareRigConfig, RigSimulationResult};
use crate::domain::simulator_ranker::{
    rank_compatible_models, sort_simulated_entries, SimTableSort, SimulatedModelEntry,
};
use crate::domain::table_filter::ColumnFilter;
use crate::presentation::components::table_filter_header::{
    matches_indexed_filters, render_active_filters_chip, render_generic_header_cell, FilterHeaderCol,
};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

/// Renders the compatible models benchmark table inline inside the Simulator view.
/// Uses TableBuilder row-level click sensing so clicking anywhere on a row reliably inspects the model.
pub fn render_simulator_compatible_table(
    ui: &mut egui::Ui,
    config: &HardwareRigConfig,
    sim: &RigSimulationResult,
    all_models: &[ModelBenchmark],
    sources: &SourceSelection,
    selected_model_id: &mut Option<String>,
    lang: Language,
) {
    let eco = lang.eco();
    let is_fr = lang == Language::French;
    let sort_key = egui::Id::new("sim_inline_sort_col");
    let asc_key = egui::Id::new("sim_inline_sort_asc");
    let filter_key = egui::Id::new("sim_inline_col_filters");
    let dedup_key = egui::Id::new("sim_inline_dedup");
    let mod_key = egui::Id::new("sim_inline_mod_filter");

    let mut sort_col = ui.ctx().memory_mut(|m| m.data.get_temp::<usize>(sort_key)).unwrap_or(3);
    let mut sort_asc = ui.ctx().memory_mut(|m| m.data.get_temp::<bool>(asc_key)).unwrap_or(false);
    let mut col_filters = ui.ctx().memory_mut(|m| m.data.get_temp::<BTreeMap<usize, ColumnFilter>>(filter_key)).unwrap_or_default();
    let mut dedup = ui.ctx().memory_mut(|m| m.data.get_temp::<bool>(dedup_key)).unwrap_or(true);
    let mut mod_idx = ui.ctx().memory_mut(|m| m.data.get_temp::<usize>(mod_key)).unwrap_or(0);

    let dedup_sort = match sort_col {
        4 => SimTableSort::Coding,
        5 => SimTableSort::Agentic,
        7 => SimTableSort::Speed,
        6 => SimTableSort::Vram,
        99 => SimTableSort::Efficiency,
        _ => SimTableSort::Score,
    };

    let mut rows = rank_compatible_models(all_models, config, sim, sources, dedup_sort, dedup);
    sort_simulated_entries(&mut rows, sort_col, sort_asc, is_fr);
    let total_compatible = rows.len();

    let mod_tags = ["ALL", "TEXT", "VISION", "IMAGE", "VIDEO", "3D", "AUDIO"];
    if mod_idx > 0 && mod_idx < mod_tags.len() {
        let target = mod_tags[mod_idx];
        rows.retain(|r| r.modality_tag == target);
    }

    let rank_filtered = col_filters.get(&0).is_some_and(|f| f.active && !f.pattern.trim().is_empty());
    let filtered_rows: Vec<(usize, &SimulatedModelEntry)> = rows.iter().enumerate()
        .filter(|(idx, r)| matches_indexed_filters(&col_filters, |c| extract_sim_col(c, r, idx + 1, is_fr)))
        .collect();

    egui::Frame::group(ui.style())
        .fill(ThemeTokens::BG_CARD)
        .stroke(egui::Stroke::new(1.0, ThemeTokens::TEXT_MUTED))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(eco.sim_modal_title).strong().size(13.5).color(ThemeTokens::GOLD_5090));
                ui.label(
                    egui::RichText::new(format!("({} / {})", filtered_rows.len(), total_compatible))
                        .size(11.5)
                        .color(ThemeTokens::TEXT_MUTED),
                );
                ui.separator();
                let hint = if is_fr { "Cliquer sur une ligne pour inspecter en bas" } else { "Click any row to inspect below" };
                ui.label(egui::RichText::new(hint).size(11.0).color(ThemeTokens::TEXT_SECONDARY));
                render_active_filters_chip(ui, &mut col_filters, lang);
            });
            ui.add_space(4.0);

            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(eco.sim_modal_sort).size(11.5).strong().color(ThemeTokens::TEXT_MUTED));
                let quick_sorts = if is_fr {
                    [(3, false, "Score"), (4, false, "Code"), (5, false, "Agent"), (7, false, "Débit"), (6, true, "VRAM"), (99, false, "Efficience")]
                } else {
                    [(3, false, "Score"), (4, false, "Code"), (5, false, "Agent"), (7, false, "Speed"), (6, true, "VRAM"), (99, false, "Efficiency")]
                };
                for (col_id, def_asc, lbl) in quick_sorts {
                    if ui.selectable_label(sort_col == col_id, lbl).clicked() {
                        if sort_col == col_id { sort_asc = !sort_asc; }
                        else { sort_col = col_id; sort_asc = def_asc; }
                    }
                }
                ui.separator();
                ui.checkbox(&mut dedup, eco.sim_modal_dedup);
                ui.separator();
                for (i, tag) in mod_tags.iter().enumerate() {
                    if ui.selectable_label(mod_idx == i, *tag).clicked() { mod_idx = i; }
                }
            });
            ui.add_space(6.0);

            if rows.is_empty() && mod_idx == 0 {
                ui.add_space(10.0);
                ui.label(egui::RichText::new(eco.sim_modal_empty).color(ThemeTokens::WARNING));
                return;
            }

            let unit_gb = if is_fr { "Go" } else { "GB" };
            let click_tip = if is_fr { "Cliquer pour inspecter & calibrer ce modèle" } else { "Click to inspect & calibrate this model" };
            let headers = [
                FilterHeaderCol::filter_only(0, "#"),
                FilterHeaderCol::sortable(1, "Model", true),
                FilterHeaderCol::sortable(2, "Type", true),
                FilterHeaderCol::sortable(3, "Score", false),
                FilterHeaderCol::sortable(4, "Code", false),
                FilterHeaderCol::sortable(5, "Agent", false),
                FilterHeaderCol::sortable(6, "VRAM", true),
                FilterHeaderCol::sortable(7, eco.sim_col_est_tps, false),
                FilterHeaderCol::sortable(8, eco.sim_col_fit, true),
            ];

            egui::ScrollArea::horizontal().id_salt("sim_inline_table_scroll").show(ui, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .max_scroll_height(280.0)
                    .column(Column::exact(64.0).clip(true))
                    .column(Column::initial(250.0).at_least(165.0).clip(true))
                    .column(Column::initial(92.0).at_least(74.0).clip(true))
                    .column(Column::initial(86.0).at_least(68.0).clip(true))
                    .column(Column::initial(82.0).at_least(66.0).clip(true))
                    .column(Column::initial(82.0).at_least(66.0).clip(true))
                    .column(Column::initial(88.0).at_least(70.0).clip(true))
                    .column(Column::initial(96.0).at_least(76.0).clip(true))
                    .column(Column::remainder().at_least(125.0))
                    .header(50.0, |mut h| {
                        for col in headers {
                            h.col(|ui| {
                                render_generic_header_cell(
                                    ui, "sim_inline_hdr", col, 3, false,
                                    &mut sort_col, &mut sort_asc, &mut col_filters, lang,
                                );
                            });
                        }
                    })
                    .body(|mut body| {
                        if filtered_rows.is_empty() {
                            body.row(44.0, |mut row| {
                                row.col(|_| {});
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(lang.t().no_matching_models).color(ThemeTokens::TEXT_MUTED));
                                        if ui.small_button(lang.t().reset_filters).clicked() {
                                            col_filters.clear();
                                            mod_idx = 0;
                                        }
                                    });
                                });
                            });
                        } else {
                            body.rows(25.0, filtered_rows.len(), |mut row| {
                                let i = row.index();
                                let (orig_idx, r) = filtered_rows[i];
                                let disp_rank = if rank_filtered { orig_idx + 1 } else { i + 1 };
                                let is_sel = selected_model_id.as_deref() == Some(&r.id);
                                row.set_selected(is_sel);
                                let (cr, cg, cb) = r.fit_status.color_rgb();
                                let rank_col = if disp_rank == 1 { ThemeTokens::GOLD_5090 } else if disp_rank <= 3 { ThemeTokens::CYAN_EDGE } else { ThemeTokens::TEXT_MUTED };
                                let name_col = if is_sel { ThemeTokens::GOLD_5090 } else { ThemeTokens::TEXT_PRIMARY };

                                let mut clicked_row = false;
                                clicked_row |= cell_col(&mut row, click_tip, |ui| { ui.label(egui::RichText::new(format!("#{disp_rank}")).strong().color(rank_col)); });
                                clicked_row |= cell_col(&mut row, click_tip, |ui| { ui.label(egui::RichText::new(&r.name).strong().color(name_col)); });
                                clicked_row |= cell_col(&mut row, click_tip, |ui| { ui.label(egui::RichText::new(r.modality_tag).size(11.0).color(ThemeTokens::TEXT_SECONDARY)); });
                                clicked_row |= cell_col(&mut row, click_tip, |ui| { ui.label(egui::RichText::new(format!("{:.1}", r.score)).strong().color(ThemeTokens::SUCCESS)); });
                                clicked_row |= cell_col(&mut row, click_tip, |ui| { ui.label(r.coding.map_or("-".into(), |v| format!("{v:.0}"))); });
                                clicked_row |= cell_col(&mut row, click_tip, |ui| { ui.label(r.agentic.map_or("-".into(), |v| format!("{v:.0}"))); });
                                clicked_row |= cell_col(&mut row, click_tip, |ui| { ui.label(format!("{:.1} {unit_gb}", r.vram_rec_gb)); });
                                clicked_row |= cell_col(&mut row, click_tip, |ui| { ui.label(format!("{:.0} t/s", r.est_tps)); });
                                clicked_row |= cell_col(&mut row, click_tip, |ui| { ui.label(egui::RichText::new(r.fit_status.label(is_fr)).color(egui::Color32::from_rgb(cr, cg, cb)).size(11.0)); });

                                if clicked_row {
                                    *selected_model_id = Some(r.id.clone());
                                }
                            });
                        }
                    });
            });
        });

    ui.ctx().memory_mut(|m| {
        m.data.insert_temp(sort_key, sort_col);
        m.data.insert_temp(asc_key, sort_asc);
        m.data.insert_temp(filter_key, col_filters);
        m.data.insert_temp(dedup_key, dedup);
        m.data.insert_temp(mod_key, mod_idx);
    });
}

fn cell_col(
    row: &mut egui_extras::TableRow<'_, '_>,
    click_tip: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) -> bool {
    let mut clicked = false;
    row.col(|ui| {
        add_contents(ui);
        let gapless = ui.max_rect().expand2(egui::vec2(ui.spacing().item_spacing.x * 0.5, 0.0));
        let hit = ui.interact(gapless, ui.id().with("cell_hit"), egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(click_tip);
        clicked = hit.clicked();
    });
    clicked
}

fn extract_sim_col(col_idx: usize, r: &SimulatedModelEntry, orig_rank: usize, is_fr: bool) -> String {
    match col_idx {
        0 => format!("#{orig_rank}"),
        1 => format!("{} {}", r.name, r.family),
        2 => r.modality_tag.into(),
        3 => format!("{:.1}", r.score),
        4 => r.coding.map_or_else(|| "-".into(), |v| format!("{v:.1}")),
        5 => r.agentic.map_or_else(|| "-".into(), |v| format!("{v:.1}")),
        6 => format!("{:.1}", r.vram_rec_gb),
        7 => format!("{:.0}", r.est_tps),
        8 => r.fit_status.label(is_fr).into(),
        _ => String::new(),
    }
}
