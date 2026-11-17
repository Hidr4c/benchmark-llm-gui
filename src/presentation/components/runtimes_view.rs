use std::collections::BTreeMap;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use crate::domain::runtime::{LocalRuntimeProfile, RuntimeHardwareFilter};
use crate::domain::table_filter::ColumnFilter;
use crate::presentation::components::table_filter_header::{
    matches_indexed_filters, render_active_filters_chip, render_generic_header_cell, FilterHeaderCol,
};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_runtimes_view(
    ui: &mut egui::Ui,
    runtimes: &[LocalRuntimeProfile],
    is_syncing: bool,
    on_sync: &mut bool,
    lang: Language,
) {
    let eco = lang.eco();
    let is_fr = lang == Language::French;
    let hw_key = egui::Id::new("runtime_hw_filter");
    let search_key = egui::Id::new("runtime_search_query");
    let sort_key = egui::Id::new("runtime_sort_col_v2");
    let asc_key = egui::Id::new("runtime_sort_asc");
    let filter_key = egui::Id::new("runtime_col_filters");
    let sel_key = egui::Id::new("runtime_selected_name");

    let mut hw_idx = ui.ctx().memory_mut(|m| m.data.get_temp::<usize>(hw_key)).unwrap_or(0);
    let mut search = ui.ctx().memory_mut(|m| m.data.get_temp::<String>(search_key)).unwrap_or_default();
    let mut sort_col = ui.ctx().memory_mut(|m| m.data.get_temp::<usize>(sort_key)).unwrap_or(3);
    let mut sort_asc = ui.ctx().memory_mut(|m| m.data.get_temp::<bool>(asc_key)).unwrap_or(false);
    let mut col_filters = ui.ctx().memory_mut(|m| m.data.get_temp::<BTreeMap<usize, ColumnFilter>>(filter_key)).unwrap_or_default();
    let mut sel_name = ui.ctx().memory_mut(|m| m.data.get_temp::<Option<String>>(sel_key)).unwrap_or(None);

    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        ui.heading(egui::RichText::new(eco.run_title).strong().color(ThemeTokens::TEXT_PRIMARY));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !is_syncing && ui.button(egui::RichText::new(eco.harn_btn_sync).strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                *on_sync = true;
            }
        });
    });
    ui.label(egui::RichText::new(eco.run_subtitle).color(ThemeTokens::TEXT_SECONDARY).size(12.0));
    ui.add_space(8.0);

    if runtimes.is_empty() {
        egui::Frame::group(ui.style()).fill(ThemeTokens::BG_CARD).inner_margin(16.0).show(ui, |ui| {
            ui.label(egui::RichText::new(eco.run_no_data).color(ThemeTokens::GOLD_5090));
            ui.add_space(8.0);
            if !is_syncing && ui.button(egui::RichText::new(eco.harn_btn_sync).strong()).clicked() {
                *on_sync = true;
            }
        });
        return;
    }

    let filters = RuntimeHardwareFilter::all();
    ui.horizontal_wrapped(|ui| {
        for (i, f) in filters.iter().enumerate() {
            if ui.selectable_label(hw_idx == i, f.label(is_fr)).clicked() { hw_idx = i; }
        }
        ui.separator();
        ui.add(egui::TextEdit::singleline(&mut search).hint_text(if is_fr { "Rechercher (Ollama, Off Grid, MLX...)" } else { "Search runtime..." }).desired_width(200.0));
        if !search.is_empty() && ui.small_button("X").clicked() { search.clear(); }
        render_active_filters_chip(ui, &mut col_filters, lang);
    });
    ui.add_space(6.0);

    let active_hw = filters.get(hw_idx).copied().unwrap_or(RuntimeHardwareFilter::All);
    let q = search.trim().to_lowercase();
    let mut rows: Vec<&LocalRuntimeProfile> = runtimes.iter().filter(|rt| {
        rt.matches_hardware(active_hw)
            && (q.is_empty()
                || rt.name.to_lowercase().contains(&q)
                || rt.category.to_lowercase().contains(&q)
                || rt.description.to_lowercase().contains(&q)
                || rt.backends.iter().any(|b| b.to_lowercase().contains(&q)))
    }).collect();

    rows.sort_by(|a, b| {
        let ord = match sort_col {
            1 => cmp_str(&a.name.to_lowercase(), &b.name.to_lowercase(), sort_asc),
            2 => cmp_str(&a.category.to_lowercase(), &b.category.to_lowercase(), sort_asc),
            4 => cmp_opt_f32(a.measured_avg_tps, b.measured_avg_tps, sort_asc),
            5 => if sort_asc { a.stars.cmp(&b.stars) } else { b.stars.cmp(&a.stars) },
            6 => cmp_str(offgrid_badge(a), offgrid_badge(b), sort_asc),
            7 => cmp_str(&a.backends.join(" ").to_lowercase(), &b.backends.join(" ").to_lowercase(), sort_asc),
            _ => if sort_asc { a.rating_score().total_cmp(&b.rating_score()) } else { b.rating_score().total_cmp(&a.rating_score()) },
        };
        ord.then_with(|| b.stars.cmp(&a.stars)).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    let rank_filtered = col_filters.get(&0).is_some_and(|f| f.active && !f.pattern.trim().is_empty());
    let filtered_rows: Vec<(usize, &LocalRuntimeProfile)> = rows.into_iter().enumerate()
        .filter(|(idx, rt)| matches_indexed_filters(&col_filters, |c| extract_runtime_col(c, rt, idx + 1)))
        .collect();

    let avail_h = (ui.available_height() - if sel_name.is_some() { 150.0 } else { 10.0 }).max(220.0);
    let headers = [
        FilterHeaderCol::filter_only(0, "#"),
        FilterHeaderCol::sortable(1, if is_fr { "Logiciel / Moteur" } else { "Runtime / Engine" }, true),
        FilterHeaderCol::sortable(2, if is_fr { "Catégorie" } else { "Category" }, true),
        FilterHeaderCol::sortable(3, "Score /100", false),
        FilterHeaderCol::sortable(4, if is_fr { "Débit (tok/s)" } else { "Speed (tok/s)" }, false),
        FilterHeaderCol::sortable(5, "Stars", false),
        FilterHeaderCol::sortable(6, "Off-Grid / Edge", false),
        FilterHeaderCol::sortable(7, "Backends & OS", true),
        FilterHeaderCol::static_col(8, "Action"),
    ];

    egui::ScrollArea::horizontal().id_salt("runtimes_top_table_scroll").show(ui, |ui| {
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .max_scroll_height(avail_h)
            .column(Column::exact(64.0).clip(true))
            .column(Column::initial(205.0).at_least(150.0).clip(true))
            .column(Column::initial(145.0).at_least(100.0).clip(true))
            .column(Column::initial(118.0).at_least(90.0).clip(true))
            .column(Column::initial(112.0).at_least(88.0).clip(true))
            .column(Column::initial(94.0).at_least(75.0).clip(true))
            .column(Column::initial(120.0).at_least(95.0).clip(true))
            .column(Column::initial(200.0).at_least(130.0).clip(true))
            .column(Column::exact(135.0).clip(true))
            .header(50.0, |mut h| {
                for col in headers {
                    h.col(|ui| {
                        render_generic_header_cell(
                            ui, "runtimes_hdr", col, 3, false,
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
                                    hw_idx = 0;
                                }
                            });
                        });
                    });
                } else {
                    body.rows(28.0, filtered_rows.len(), |mut row| {
                        let i = row.index();
                        let (orig_idx, rt) = filtered_rows[i];
                        let disp_rank = if rank_filtered { orig_idx + 1 } else { i + 1 };
                        let score = rt.rating_score();
                        let is_sel = sel_name.as_deref() == Some(&rt.name);
                        row.set_selected(is_sel);

                        row.col(|ui| {
                            let col = if disp_rank == 1 { ThemeTokens::GOLD_5090 } else if disp_rank <= 3 { ThemeTokens::CYAN_EDGE } else { ThemeTokens::TEXT_MUTED };
                            ui.label(egui::RichText::new(format!("#{disp_rank}")).strong().color(col));
                        });
                        row.col(|ui| {
                            if ui.selectable_label(is_sel, egui::RichText::new(&rt.name).strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                                sel_name = if is_sel { None } else { Some(rt.name.clone()) };
                            }
                        });
                        row.col(|ui| { ui.label(egui::RichText::new(&rt.category).size(11.0).color(ThemeTokens::TEXT_SECONDARY)); });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("{score:.1}")).strong().color(ThemeTokens::SUCCESS));
                                ui.add(egui::ProgressBar::new((score / 100.0).clamp(0.0, 1.0)).desired_width(45.0).fill(ThemeTokens::PRIMARY));
                            });
                        });
                        row.col(|ui| {
                            if let Some(tps) = rt.measured_avg_tps {
                                ui.label(egui::RichText::new(format!("{tps:.1} t/s")).strong().color(ThemeTokens::CYAN_EDGE))
                                    .on_hover_text(format!("{} runs ({})", rt.measured_runs, rt.sample_hardware));
                            } else {
                                ui.label(egui::RichText::new("-").color(ThemeTokens::TEXT_MUTED));
                            }
                        });
                        row.col(|ui| {
                            if rt.stars > 0 { ui.label(egui::RichText::new(format!("{:.1}k", rt.stars as f32 / 1000.0)).color(ThemeTokens::GOLD_5090)); }
                            else { ui.label("-"); }
                        });
                        row.col(|ui| {
                            let (txt, col) = if rt.supports_off_grid_mobile {
                                ("[OK] Off-Grid", ThemeTokens::GOLD_5090)
                            } else if rt.supports_cpu_edge {
                                ("Desktop / Edge", ThemeTokens::SUCCESS)
                            } else {
                                ("Desktop / Server", ThemeTokens::TEXT_MUTED)
                            };
                            ui.label(egui::RichText::new(txt).size(11.0).color(col));
                        });
                        row.col(|ui| {
                            ui.label(egui::RichText::new(rt.backends.join(", ")).size(11.0).color(ThemeTokens::TEXT_SECONDARY))
                                .on_hover_text(format!("OS: {}\n{}", rt.platforms.join(" • "), rt.description));
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                if ui.small_button(if is_fr { "Copier" } else { "Copy" }).clicked() { ui.ctx().copy_text(rt.install_cmd.clone()); }
                                if ui.small_button("GitHub").clicked() { let _ = open::that(&rt.official_url); }
                            });
                        });
                    });
                }
            });
    });

    if let Some(ref active) = sel_name {
        if let Some(rt) = runtimes.iter().find(|r| r.name == *active) {
            render_runtime_detail(ui, rt, &mut sel_name, is_fr);
        }
    }

    ui.ctx().memory_mut(|m| {
        m.data.insert_temp(hw_key, hw_idx);
        m.data.insert_temp(search_key, search);
        m.data.insert_temp(sort_key, sort_col);
        m.data.insert_temp(asc_key, sort_asc);
        m.data.insert_temp(filter_key, col_filters);
        m.data.insert_temp(sel_key, sel_name);
    });
}

fn offgrid_badge(rt: &LocalRuntimeProfile) -> &'static str {
    if rt.supports_off_grid_mobile { "[OK] Off-Grid Mobile Smartphone" }
    else if rt.supports_cpu_edge { "Desktop / Edge" }
    else { "Desktop / Server" }
}

fn cmp_str(a: &str, b: &str, asc: bool) -> std::cmp::Ordering {
    let c = a.cmp(b);
    if asc { c } else { c.reverse() }
}

fn cmp_opt_f32(a: Option<f32>, b: Option<f32>, asc: bool) -> std::cmp::Ordering {
    match (a, b) {
        (Some(x), Some(y)) => if asc { x.total_cmp(&y) } else { y.total_cmp(&x) },
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn extract_runtime_col(col_idx: usize, rt: &LocalRuntimeProfile, orig_rank: usize) -> String {
    match col_idx {
        0 => format!("#{orig_rank}"),
        1 => format!("{} {} {}", rt.name, rt.description, rt.license),
        2 => rt.category.clone(),
        3 => format!("{:.1}", rt.rating_score()),
        4 => rt.measured_avg_tps.map_or_else(|| "-".into(), |v| format!("{v:.1}")),
        5 => rt.stars.to_string(),
        6 => offgrid_badge(rt).into(),
        7 => format!("{} {}", rt.backends.join(" "), rt.platforms.join(" ")),
        _ => String::new(),
    }
}

fn render_runtime_detail(ui: &mut egui::Ui, rt: &LocalRuntimeProfile, sel_name: &mut Option<String>, is_fr: bool) {
    ui.add_space(6.0);
    egui::Frame::group(ui.style()).fill(ThemeTokens::BG_CARD).stroke(egui::Stroke::new(1.0, ThemeTokens::CYAN_EDGE)).inner_margin(8.0).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.strong(egui::RichText::new(&rt.name).color(ThemeTokens::GOLD_5090));
            ui.label(egui::RichText::new(format!("({}) | OS: {}", rt.license, rt.platforms.join(", "))).size(11.5).color(ThemeTokens::TEXT_SECONDARY));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("X").clicked() { *sel_name = None; }
            });
        });
        ui.label(egui::RichText::new(&rt.description).size(11.5));
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(&rt.install_cmd).monospace().size(11.0).color(ThemeTokens::CYAN_EDGE));
            if ui.small_button(if is_fr { "Copier la commande" } else { "Copy command" }).clicked() {
                ui.ctx().copy_text(rt.install_cmd.clone());
            }
        });
    });
}
