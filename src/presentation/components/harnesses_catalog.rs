use std::collections::BTreeMap;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use crate::domain::harness::{HarnessCategory, HarnessProfile, HarnessSnapshotData};
use crate::domain::table_filter::ColumnFilter;
use crate::presentation::components::searchable_combo::{render_searchable_combo, ComboOption};
use crate::presentation::components::table_filter_header::{
    matches_indexed_filters, render_active_filters_chip, render_generic_header_cell, FilterHeaderCol,
};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_harness_catalog_tab(
    ui: &mut egui::Ui,
    data: &HarnessSnapshotData,
    cat_idx: &mut usize,
    search: &mut String,
    is_fr: bool,
) {
    let lang = if is_fr { Language::French } else { Language::English };
    let sort_id = egui::Id::new("harn_cat_sort_col");
    let asc_id = egui::Id::new("harn_cat_sort_asc");
    let filter_id = egui::Id::new("harn_cat_col_filters");
    let sel_harn_id = egui::Id::new("harn_cat_selected");

    let mut sort_col = ui.ctx().memory_mut(|m| m.data.get_temp::<usize>(sort_id)).unwrap_or(3);
    let mut sort_asc = ui.ctx().memory_mut(|m| m.data.get_temp::<bool>(asc_id)).unwrap_or(false);
    let mut col_filters = ui.ctx().memory_mut(|m| m.data.get_temp::<BTreeMap<usize, ColumnFilter>>(filter_id)).unwrap_or_default();
    let mut sel_harn = ui.ctx().memory_mut(|m| m.data.get_temp::<Option<String>>(sel_harn_id)).unwrap_or(None);

    ui.horizontal_wrapped(|ui| {
        let cats = [
            if is_fr { "Agents Production (CLI + IDE)" } else { "Production Agents (CLI + IDE)" },
            if is_fr { "Tous (incluant Scaffolds)" } else { "All (incl. Scaffolds)" },
            if is_fr { "CLI Terminal" } else { "CLI Agents" },
            if is_fr { "Extensions IDE" } else { "IDE Extensions" },
            if is_fr { "Scaffolds Recherche" } else { "Research Scaffolds" },
        ];
        for (i, c) in cats.iter().enumerate() {
            if ui.selectable_label(*cat_idx == i, *c).clicked() { *cat_idx = i; }
        }
    });
    ui.add_space(4.0);

    let options: Vec<ComboOption> = data.profiles.iter().map(|p| {
        let score_tag = p.best_score_pct.map(|s| format!("{s:.1}%")).unwrap_or_else(|| "GitHub".into());
        ComboOption {
            key: p.name.clone(),
            label: format!("{} [{}] ({})", p.name, p.category.label(is_fr), score_tag),
            search_blob: format!("{} {} {}", p.name, p.org, p.description).to_lowercase(),
        }
    }).collect();

    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(if is_fr { "Rechercher / Choisir un harnais :" } else { "Search / Pick Harness:" }).strong().color(ThemeTokens::CYAN_EDGE));
        if let Some(picked) = render_searchable_combo(
            ui, "harness_catalog_combo", search, &mut sel_harn, &options,
            if is_fr { "OpenCode, Kilo, Cline, Continue, Claude Code..." } else { "OpenCode, Kilo, Cline, Continue..." },
            280.0, is_fr,
        ) {
            if let Some(prof) = data.profiles.iter().find(|p| p.name == picked) {
                *search = prof.name.clone();
                if matches!(prof.category, HarnessCategory::ResearchScaffold) && *cat_idx == 0 {
                    *cat_idx = 1;
                }
            }
        }
        render_active_filters_chip(ui, &mut col_filters, lang);
    });
    ui.add_space(6.0);

    let q = search.trim().to_lowercase();
    let mut rows: Vec<&HarnessProfile> = data.profiles.iter().filter(|p| {
        let cat_ok = match *cat_idx {
            0 => matches!(p.category, HarnessCategory::CliAgent | HarnessCategory::IdeExtension),
            2 => p.category == HarnessCategory::CliAgent,
            3 => p.category == HarnessCategory::IdeExtension,
            4 => p.category == HarnessCategory::ResearchScaffold,
            _ => true,
        };
        cat_ok && (q.is_empty()
            || p.name.to_lowercase().contains(&q)
            || p.org.to_lowercase().contains(&q)
            || p.description.to_lowercase().contains(&q))
    }).collect();

    rows.sort_by(|a, b| {
        let ord = match sort_col {
            1 => {
                let cmp = a.name.to_lowercase().cmp(&b.name.to_lowercase());
                if sort_asc { cmp } else { cmp.reverse() }
            }
            2 => {
                let cmp = a.category.label(is_fr).cmp(b.category.label(is_fr));
                if sort_asc { cmp } else { cmp.reverse() }
            }
            4 => cmp_opt_score(a.avg_score_pct, b.avg_score_pct, sort_asc),
            5 => {
                let cmp = a.runs_count.cmp(&b.runs_count);
                if sort_asc { cmp } else { cmp.reverse() }
            }
            6 => {
                let cmp = a.stars.cmp(&b.stars);
                if sort_asc { cmp } else { cmp.reverse() }
            }
            _ => cmp_opt_score(a.best_score_pct, b.best_score_pct, sort_asc),
        };
        ord.then_with(|| b.stars.cmp(&a.stars)).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    let rank_filtered = col_filters.get(&0).is_some_and(|f| f.active && !f.pattern.trim().is_empty());
    let filtered_rows: Vec<(usize, &HarnessProfile)> = rows.into_iter().enumerate()
        .filter(|(idx, p)| matches_indexed_filters(&col_filters, |c| extract_harness_col(c, p, idx + 1, is_fr)))
        .collect();

    let avail_h = (ui.available_height() - if sel_harn.is_some() { 185.0 } else { 10.0 }).max(200.0);
    let headers = [
        FilterHeaderCol::filter_only(0, "#"),
        FilterHeaderCol::sortable(1, if is_fr { "Harnais" } else { "Harness" }, true),
        FilterHeaderCol::sortable(2, if is_fr { "Catégorie" } else { "Category" }, true),
        FilterHeaderCol::sortable(3, "Best %", false),
        FilterHeaderCol::sortable(4, if is_fr { "Moy. %" } else { "Avg %" }, false),
        FilterHeaderCol::sortable(5, "Runs", false),
        FilterHeaderCol::sortable(6, "Stars", false),
        FilterHeaderCol::static_col(7, "GitHub"),
    ];

    egui::ScrollArea::horizontal().id_salt("harn_table_scroll").show(ui, |ui| {
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .max_scroll_height(avail_h)
            .column(Column::exact(64.0).clip(true))
            .column(Column::initial(220.0).at_least(150.0).clip(true))
            .column(Column::initial(130.0).at_least(95.0).clip(true))
            .column(Column::initial(95.0).at_least(72.0).clip(true))
            .column(Column::initial(90.0).at_least(70.0).clip(true))
            .column(Column::initial(82.0).at_least(68.0).clip(true))
            .column(Column::initial(94.0).at_least(75.0).clip(true))
            .column(Column::exact(80.0).clip(true))
            .header(50.0, |mut h| {
                for col in headers {
                    h.col(|ui| {
                        render_generic_header_cell(
                            ui, "harn_cat_hdr", col, 3, false,
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
                                    search.clear();
                                    col_filters.clear();
                                }
                            });
                        });
                    });
                } else {
                    body.rows(26.0, filtered_rows.len(), |mut row| {
                        let i = row.index();
                        let (orig_idx, p) = filtered_rows[i];
                        let disp_rank = if rank_filtered { orig_idx + 1 } else { i + 1 };
                        let is_sel = sel_harn.as_deref() == Some(&p.name);
                        row.set_selected(is_sel);
                        row.col(|ui| {
                            let col = if disp_rank == 1 { ThemeTokens::GOLD_5090 } else if disp_rank <= 3 { ThemeTokens::CYAN_EDGE } else { ThemeTokens::TEXT_MUTED };
                            ui.label(egui::RichText::new(format!("#{disp_rank}")).strong().color(col));
                        });
                        row.col(|ui| {
                            if ui.selectable_label(is_sel, egui::RichText::new(&p.name).strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                                sel_harn = if is_sel { None } else { Some(p.name.clone()) };
                            }
                        });
                        row.col(|ui| { ui.label(egui::RichText::new(p.category.label(is_fr)).size(11.0).color(ThemeTokens::TEXT_SECONDARY)); });
                        row.col(|ui| {
                            if let Some(best) = p.best_score_pct {
                                ui.label(egui::RichText::new(format!("{best:.1}%")).strong().color(ThemeTokens::SUCCESS));
                            } else {
                                ui.label(egui::RichText::new("-").color(ThemeTokens::TEXT_MUTED));
                            }
                        });
                        row.col(|ui| {
                            if let Some(avg) = p.avg_score_pct { ui.label(format!("{avg:.1}%")); }
                            else { ui.label(egui::RichText::new("-").color(ThemeTokens::TEXT_MUTED)); }
                        });
                        row.col(|ui| { ui.label(p.runs_count.to_string()); });
                        row.col(|ui| {
                            if p.stars > 0 { ui.label(egui::RichText::new(format!("{:.1}k", p.stars as f32 / 1000.0)).color(ThemeTokens::GOLD_5090)); }
                            else { ui.label("-"); }
                        });
                        row.col(|ui| {
                            if ui.small_button(if is_fr { "Ouvrir" } else { "Open" }).clicked() { let _ = open::that(&p.url); }
                        });
                    });
                }
            });
    });

    if let Some(active_name) = sel_harn.clone() {
        render_harness_runs_detail(ui, data, &active_name, &mut sel_harn, is_fr);
    }

    ui.ctx().memory_mut(|m| {
        m.data.insert_temp(sort_id, sort_col);
        m.data.insert_temp(asc_id, sort_asc);
        m.data.insert_temp(filter_id, col_filters);
        m.data.insert_temp(sel_harn_id, sel_harn);
    });
}

fn cmp_opt_score(a: Option<f32>, b: Option<f32>, asc: bool) -> std::cmp::Ordering {
    match (a, b) {
        (Some(x), Some(y)) => if asc { x.total_cmp(&y) } else { y.total_cmp(&x) },
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn extract_harness_col(col_idx: usize, p: &HarnessProfile, orig_rank: usize, is_fr: bool) -> String {
    match col_idx {
        0 => format!("#{orig_rank}"),
        1 => format!("{} {} {}", p.name, p.org, p.description),
        2 => p.category.label(is_fr).to_string(),
        3 => p.best_score_pct.map_or_else(|| "-".into(), |v| format!("{v:.1}")),
        4 => p.avg_score_pct.map_or_else(|| "-".into(), |v| format!("{v:.1}")),
        5 => p.runs_count.to_string(),
        6 => p.stars.to_string(),
        _ => String::new(),
    }
}

fn render_harness_runs_detail(
    ui: &mut egui::Ui,
    data: &HarnessSnapshotData,
    harness_name: &str,
    sel_harn: &mut Option<String>,
    is_fr: bool,
) {
    let mut runs: Vec<_> = data.runs.iter().filter(|r| r.harness_name.eq_ignore_ascii_case(harness_name)).collect();
    runs.sort_by(|a, b| b.score_pct.total_cmp(&a.score_pct).then_with(|| a.model_name.cmp(&b.model_name)));
    ui.add_space(6.0);
    egui::Frame::group(ui.style()).fill(ThemeTokens::BG_CARD).stroke(egui::Stroke::new(1.0, ThemeTokens::CYAN_EDGE)).inner_margin(8.0).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.strong(egui::RichText::new(format!("{} ({} runs)", harness_name, runs.len())).color(ThemeTokens::GOLD_5090));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("X").clicked() { *sel_harn = None; }
            });
        });
        if runs.is_empty() {
            ui.label(egui::RichText::new(if is_fr { "Aucun run public indexé pour ce harnais (suivi GitHub actif)." } else { "No public benchmark run indexed yet (tracked via GitHub)." }).size(11.5).color(ThemeTokens::TEXT_MUTED));
        } else {
            egui::ScrollArea::vertical().id_salt("harn_detail_runs_scroll").max_height(130.0).show(ui, |ui| {
                for r in runs {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("{:.1}%", r.score_pct)).strong().color(ThemeTokens::SUCCESS));
                        ui.label(egui::RichText::new(&r.model_name).strong());
                        ui.label(egui::RichText::new(format!("[{}]", r.suite)).size(11.0).color(ThemeTokens::TEXT_MUTED));
                    });
                }
            });
        }
    });
}
