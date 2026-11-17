use eframe::egui;
use crate::domain::harness::{compare_harnesses_by_model, HarnessSnapshotData};
use crate::presentation::components::harnesses_catalog::render_harness_catalog_tab;
use crate::presentation::components::searchable_combo::{render_searchable_combo, ComboOption};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_harnesses_view(
    ui: &mut egui::Ui,
    data: &HarnessSnapshotData,
    is_syncing: bool,
    on_sync: &mut bool,
    lang: Language,
) {
    let eco = lang.eco();
    let is_fr = lang == Language::French;
    let subtab_key = egui::Id::new("harness_subtab_idx");
    let model_key_id = egui::Id::new("harness_selected_model");
    let model_q_key = egui::Id::new("harness_model_search");
    let harn_filter_id = egui::Id::new("harness_cmp_filter");
    let harn_sel_id = egui::Id::new("harness_cmp_sel");
    let cat_key = egui::Id::new("harness_cat_filter");
    let search_key = egui::Id::new("harness_search_query");

    let mut subtab = ui.ctx().memory_mut(|m| m.data.get_temp::<usize>(subtab_key)).unwrap_or(0);
    let mut sel_model = ui.ctx().memory_mut(|m| m.data.get_temp::<String>(model_key_id)).unwrap_or_default();
    let mut model_q = ui.ctx().memory_mut(|m| m.data.get_temp::<String>(model_q_key)).unwrap_or_default();
    let mut harn_q = ui.ctx().memory_mut(|m| m.data.get_temp::<String>(harn_filter_id)).unwrap_or_default();
    let mut harn_sel = ui.ctx().memory_mut(|m| m.data.get_temp::<Option<String>>(harn_sel_id)).unwrap_or(None);
    let mut cat_idx = ui.ctx().memory_mut(|m| m.data.get_temp::<usize>(cat_key)).unwrap_or(0);
    let mut search = ui.ctx().memory_mut(|m| m.data.get_temp::<String>(search_key)).unwrap_or_default();

    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        ui.heading(egui::RichText::new(eco.harn_title).strong().color(ThemeTokens::TEXT_PRIMARY));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !is_syncing && ui.button(egui::RichText::new(eco.harn_btn_sync).strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                *on_sync = true;
            }
        });
    });
    ui.label(egui::RichText::new(eco.harn_subtitle).color(ThemeTokens::TEXT_SECONDARY).size(12.0));
    render_harness_sources_bar(ui, is_fr);
    ui.add_space(6.0);

    if data.profiles.is_empty() && data.runs.is_empty() {
        egui::Frame::group(ui.style()).fill(ThemeTokens::BG_CARD).inner_margin(16.0).show(ui, |ui| {
            ui.label(egui::RichText::new(eco.harn_no_data).color(ThemeTokens::GOLD_5090));
            ui.add_space(8.0);
            if !is_syncing && ui.button(egui::RichText::new(eco.harn_btn_sync).strong()).clicked() {
                *on_sync = true;
            }
        });
        return;
    }

    ui.horizontal(|ui| {
        if ui.selectable_label(subtab == 0, egui::RichText::new(eco.harn_subtab_compare).strong()).clicked() { subtab = 0; }
        ui.add_space(8.0);
        if ui.selectable_label(subtab == 1, egui::RichText::new(eco.harn_subtab_catalog).strong()).clicked() { subtab = 1; }
    });
    ui.separator();

    if subtab == 0 {
        egui::ScrollArea::vertical().show(ui, |ui| {
            render_same_model_comparator(ui, data, &mut sel_model, &mut model_q, &mut harn_q, &mut harn_sel, is_fr);
        });
    } else {
        render_harness_catalog_tab(ui, data, &mut cat_idx, &mut search, is_fr);
    }

    ui.ctx().memory_mut(|m| {
        m.data.insert_temp(subtab_key, subtab);
        m.data.insert_temp(model_key_id, sel_model);
        m.data.insert_temp(model_q_key, model_q);
        m.data.insert_temp(harn_filter_id, harn_q);
        m.data.insert_temp(harn_sel_id, harn_sel);
        m.data.insert_temp(cat_key, cat_idx);
        m.data.insert_temp(search_key, search);
    });
}

fn render_harness_sources_bar(ui: &mut egui::Ui, is_fr: bool) {
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(if is_fr { "Sources live :" } else { "Live Sources:" }).size(11.0).strong().color(ThemeTokens::TEXT_MUTED));
        let sources = [
            ("SWE-bench", "https://www.swebench.com/"),
            ("Terminal-Bench 2.0", "https://www.tbench.ai/leaderboard/terminal-bench/2.0"),
            ("Terminal-Bench 1.0", "https://www.tbench.ai/leaderboard/terminal-bench/1.0"),
            ("Cline / OpenCode CSV", "https://github.com/cline/benchmark-results"),
            ("KiloBench", "https://kilo.ai/kilobench"),
            ("Aider Polyglot", "https://aider.chat/docs/leaderboards/"),
        ];
        for (lbl, url) in sources {
            if ui.small_button(egui::RichText::new(lbl).size(10.5).color(ThemeTokens::CYAN_EDGE)).clicked() {
                let _ = open::that(url);
            }
        }
    });
}

fn render_same_model_comparator(
    ui: &mut egui::Ui,
    data: &HarnessSnapshotData,
    sel_model: &mut String,
    model_q: &mut String,
    harn_q: &mut String,
    harn_sel: &mut Option<String>,
    is_fr: bool,
) {
    let all_groups = compare_harnesses_by_model(&data.runs, 1);
    if all_groups.is_empty() {
        ui.label(if is_fr { "Aucun run public indexé." } else { "No public runs indexed." });
        return;
    }

    let model_options: Vec<ComboOption> = all_groups.iter().map(|g| ComboOption {
        key: g.model_key.clone(),
        label: format!("{} ({} runs, +{:.1} pts)", g.model_name, g.entries.len(), g.spread_pts),
        search_blob: format!("{} {}", g.model_name, g.model_key).to_lowercase(),
    }).collect();

    let mut harness_names: Vec<String> = data.runs.iter().map(|r| r.harness_name.clone()).collect();
    harness_names.sort_by_key(|s| s.to_lowercase());
    harness_names.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    let harn_options: Vec<ComboOption> = harness_names.iter().map(|h| ComboOption {
        key: h.clone(),
        label: h.clone(),
        search_blob: h.to_lowercase(),
    }).collect();

    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(if is_fr { "Modèle :" } else { "Model:" }).strong().color(ThemeTokens::CYAN_EDGE));
        let mut sel_opt = if sel_model.is_empty() { None } else { Some(sel_model.clone()) };
        if let Some(picked) = render_searchable_combo(
            ui, "harness_model_unified", model_q, &mut sel_opt, &model_options,
            if is_fr { "Taper ou choisir (GLM, Claude, Qwen, Kimi...)" } else { "Type or select model..." },
            250.0, is_fr,
        ) {
            *sel_model = picked;
        } else {
            let typed = model_q.trim().to_lowercase();
            if typed.is_empty() && sel_opt.is_none() {
                sel_model.clear();
            } else if let Some(s) = sel_opt {
                *sel_model = s;
            }
        }

        ui.add_space(10.0);
        ui.label(egui::RichText::new(if is_fr { "Harnais :" } else { "Harness:" }).strong().color(ThemeTokens::GOLD_5090));
        if let Some(picked_h) = render_searchable_combo(
            ui, "harness_filter_unified", harn_q, harn_sel, &harn_options,
            if is_fr { "Filtrer par harnais (OpenCode, Kilo, Cline...)" } else { "Filter harness..." },
            210.0, is_fr,
        ) {
            *harn_q = picked_h;
        }
    });

    let mq = model_q.trim().to_lowercase();
    let hq = harn_q.trim().to_lowercase();
    let mut visible_groups: Vec<_> = all_groups.iter().filter(|g| {
        let model_ok = if !sel_model.is_empty() && g.model_key == *sel_model {
            true
        } else if mq.is_empty() {
            g.entries.len() >= 2 || all_groups.iter().all(|x| x.entries.len() < 2)
        } else {
            g.model_name.to_lowercase().contains(&mq) || g.model_key.contains(&mq)
        };
        let harn_ok = hq.is_empty() || g.entries.iter().any(|e| {
            e.harness_name.to_lowercase().contains(&hq) || e.suite.to_lowercase().contains(&hq)
        });
        model_ok && harn_ok
    }).collect();

    if visible_groups.is_empty() {
        ui.add_space(6.0);
        ui.label(if is_fr { "Aucun modèle / harnais correspondant trouvé." } else { "No matching model or harness found." });
        return;
    }

    let current = visible_groups.iter()
        .find(|g| g.model_key == *sel_model)
        .copied()
        .unwrap_or(visible_groups[0]);
    if sel_model.is_empty() && mq.is_empty() {
        *sel_model = current.model_key.clone();
    }

    if visible_groups.len() > 1 {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(if is_fr { "Modèles suggérés :" } else { "Suggested models:" }).size(11.0).color(ThemeTokens::TEXT_MUTED));
            for g in visible_groups.drain(..visible_groups.len().min(7)) {
                if ui.selectable_label(g.model_key == current.model_key, egui::RichText::new(format!("{} ({}x)", g.model_name, g.entries.len())).size(11.0)).clicked() {
                    *sel_model = g.model_key.clone();
                    *model_q = g.model_name.clone();
                }
            }
        });
    }

    ui.add_space(4.0);
    ui.label(egui::RichText::new(format!(
        "{}: {} | Spread: +{:.1} pts ({:.1}% vs {:.1}%) | {}",
        if is_fr { "Meilleur harnais" } else { "Best Harness" },
        current.entries.first().map(|e| e.harness_name.as_str()).unwrap_or("-"),
        current.spread_pts, current.best_score, current.worst_score, current.model_name
    )).strong().color(ThemeTokens::GOLD_5090));
    ui.add_space(6.0);

    for (rank, run) in current.entries.iter().enumerate() {
        if !hq.is_empty() && !run.harness_name.to_lowercase().contains(&hq) && !run.suite.to_lowercase().contains(&hq) {
            continue;
        }
        let ratio = (run.score_pct / 100.0).clamp(0.0, 1.0);
        egui::Frame::group(ui.style()).fill(ThemeTokens::BG_CARD).inner_margin(10.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("#{} {}", rank + 1, run.harness_name)).strong().size(14.0).color(ThemeTokens::TEXT_PRIMARY));
                ui.label(egui::RichText::new(format!("[{}]", run.suite)).size(11.0).color(ThemeTokens::CYAN_EDGE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(cost) = run.cost_usd {
                        ui.label(egui::RichText::new(format!("${cost:.2}")).color(ThemeTokens::TEXT_MUTED).size(11.5));
                    }
                    ui.label(egui::RichText::new(format!("{:.1}%", run.score_pct)).strong().size(15.0).color(ThemeTokens::SUCCESS));
                });
            });
            let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width().max(120.0), 8.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 4.0, egui::Color32::from_rgb(28, 36, 52));
            let fill = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * ratio, rect.height()));
            ui.painter().rect_filled(fill, 4.0, if rank == 0 { ThemeTokens::GOLD_5090 } else { ThemeTokens::CYAN_EDGE });
        });
        ui.add_space(4.0);
    }
}
