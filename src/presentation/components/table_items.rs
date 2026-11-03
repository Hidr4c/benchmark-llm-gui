use eframe::egui;
use crate::domain::model::{ModelBenchmark, SortField, SortOrder};
use crate::presentation::components::filter_bar::FilterState;
use crate::presentation::filtering::FilterService;

pub fn header_sort_col(
    ui: &mut egui::Ui,
    label: &str,
    field: SortField,
    default_order: SortOrder,
    filters: &mut FilterState,
) -> egui::Response {
    let arrow = if filters.sort_field == field {
        match filters.sort_order {
            SortOrder::Ascending => " ^",
            SortOrder::Descending => " v",
        }
    } else {
        ""
    };
    let resp = ui.button(format!("{label}{arrow}"));
    if resp.clicked() {
        if filters.sort_field == field {
            filters.sort_order = match filters.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
        } else {
            filters.sort_field = field;
            filters.sort_order = default_order;
        }
    }
    resp
}

pub struct DisplayItem {
    pub model_idx: usize,
    pub is_root: bool,
    pub root_name: String,
    pub variant_count: usize,
    pub is_expanded: bool,
    pub is_child: bool,
    pub rank_str: String,
}

pub fn build_display_items(models: &[ModelBenchmark], indices: &[usize], filters: &FilterState) -> Vec<DisplayItem> {
    let mut total_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for m in models {
        *total_counts.entry(m.root_model_name()).or_insert(0) += 1;
    }

    let groups = FilterService::group_into_roots(models, indices);
    let mut items = Vec::new();
    let has_search = !filters.search_text.trim().is_empty();

    for (r, g) in groups.iter().enumerate() {
        let total_in_catalog = total_counts.get(&g.root_name).copied().unwrap_or(g.variant_indices.len());
        let is_category = total_in_catalog > 1;
        let is_expanded = if has_search {
            !filters.expanded_roots.contains(&format!("collapsed:{}", g.root_name))
        } else {
            !filters.group_by_root || filters.expanded_roots.contains(&g.root_name)
        };

        if !filters.group_by_root {
            for (v_pos, &v_idx) in g.variant_indices.iter().enumerate() {
                let rank_str = if is_category {
                    format!("#{}.{}", r + 1, v_pos + 1)
                } else if r < 3 {
                    match r {
                        0 => "#1".into(),
                        1 => "#2".into(),
                        _ => "#3".into(),
                    }
                } else {
                    format!("#{}", r + 1)
                };

                items.push(DisplayItem {
                    model_idx: v_idx,
                    is_root: v_pos == 0,
                    root_name: g.root_name.clone(),
                    variant_count: 1,
                    is_expanded: false,
                    is_child: v_pos > 0,
                    rank_str,
                });
            }
        } else if !is_category {
            items.push(DisplayItem {
                model_idx: g.champion_idx,
                is_root: true,
                root_name: g.root_name.clone(),
                variant_count: 1,
                is_expanded: false,
                is_child: false,
                rank_str: format!("#{}", r + 1),
            });
        } else {
            let root_rank_str = format!("#{}", r + 1);
            items.push(DisplayItem {
                model_idx: g.champion_idx,
                is_root: true,
                root_name: g.root_name.clone(),
                variant_count: g.variant_indices.len(),
                is_expanded,
                is_child: false,
                rank_str: root_rank_str,
            });

            if is_expanded {
                for (v_pos, &v_idx) in g.variant_indices.iter().enumerate() {
                    items.push(DisplayItem {
                        model_idx: v_idx,
                        is_root: false,
                        root_name: g.root_name.clone(),
                        variant_count: 1,
                        is_expanded: false,
                        is_child: true,
                        rank_str: format!("#{}.{}", r + 1, v_pos + 1),
                    });
                }
            }
        }
    }

    items
}

pub fn format_downloads(count: u64) -> String {
    if count >= 1_000_000 { format!("{:.1}M", count as f64 / 1_000_000.0) }
    else if count >= 1_000 { format!("{:.0}k", count as f64 / 1_000.0) }
    else if count > 0 { format!("{count}") }
    else { "-".to_string() }
}

pub fn hardware_badge_text(model: &ModelBenchmark) -> &'static str {
    if model.is_cloud { "Cloud API" }
    else if model.size_gb <= 0.45 { "Micro-IoT (<1G)" }
    else if model.size_gb <= 1.5 { "Pi 4 (2-4G)" }
    else if model.size_gb <= 3.2 { "Pi 5 (8G)" }
    else if model.size_gb <= 5.5 { "Jetson (8G)" }
    else if model.vram_rec_gb <= 8.5 { "GPU 8 Go" }
    else if model.vram_rec_gb <= 12.5 { "GPU 12 Go" }
    else if model.vram_rec_gb <= 16.5 { "GPU 16 Go" }
    else if model.vram_rec_gb <= 24.5 { "GPU 24 Go" }
    else if model.vram_rec_gb <= 32.5 { "GPU 32 Go" }
    else { "Station 48G+" }
}

use crate::presentation::theme::ThemeTokens;

pub fn render_economic_cell(ui: &mut egui::Ui, model: &ModelBenchmark) {
    if let Some(econ) = &model.economic {
        if let Some(comp) = econ.composite_score() {
            let comp_color = if comp >= 60.0 { ThemeTokens::GOLD_5090 } else if comp >= 35.0 { ThemeTokens::CYAN_EDGE } else { ThemeTokens::SUCCESS };
            let info = format!(
                "RLI: {} | SWE-Clar: {} | SWE-Lancer: {} | WebArena: {} | MLE: {}",
                econ.rli_automation_rate.map(|v| format!("{v:.1}%")).unwrap_or_else(|| "-".into()),
                econ.swe_clar_impl.map(|v| format!("{v:.1}%")).unwrap_or_else(|| "-".into()),
                econ.swe_lancer_score.map(|v| format!("{v:.1}%")).unwrap_or_else(|| "-".into()),
                econ.web_arena_success.map(|v| format!("{v:.1}%")).unwrap_or_else(|| "-".into()),
                econ.mle_bench_score.map(|v| format!("{v:.1}%")).unwrap_or_else(|| "-".into()),
            );
            ui.label(egui::RichText::new(format!("{comp:.1} %")).strong().color(comp_color))
                .on_hover_text(info);
            return;
        }
    }
    if let Some(rw) = model.scores.real_world {
        ui.label(egui::RichText::new(format!("{rw:.1} %")).color(ThemeTokens::CYAN_EDGE));
    } else {
        ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED));
    }
}

pub fn render_vs_button(ui: &mut egui::Ui, model_id: &str, filters: &mut FilterState, lang: crate::presentation::i18n::Language) {
    let comp_pos = filters.compared_model_ids.iter().position(|id| id == model_id);
    let (comp_color, comp_fill) = match comp_pos {
        Some(0) => (egui::Color32::WHITE, ThemeTokens::CYAN_EDGE),
        Some(1) => (egui::Color32::BLACK, ThemeTokens::GOLD_5090),
        _ => (ThemeTokens::TEXT_MUTED, egui::Color32::from_rgb(20, 26, 38)),
    };
    let vs_btn = egui::Button::new(
        egui::RichText::new("[VS]").color(comp_color).size(10.0).strong()
    )
    .fill(comp_fill)
    .rounding(3.0);
    if ui.add(vs_btn).on_hover_text(lang.dock().tooltip_vs).clicked() {
        filters.toggle_compare(model_id);
    }
}

pub fn render_score_badge_cell(ui: &mut egui::Ui, score: Option<f32>) {
    if let Some(sc) = score {
        let col = if sc >= 75.0 {
            ThemeTokens::GOLD_5090
        } else if sc >= 50.0 {
            ThemeTokens::SUCCESS
        } else if sc >= 30.0 {
            ThemeTokens::PRIMARY
        } else {
            ThemeTokens::TEXT_SECONDARY
        };
        ui.label(egui::RichText::new(format!("{sc:.1}%")).strong().color(col));
    } else {
        ui.label(egui::RichText::new("N/A").color(ThemeTokens::TEXT_MUTED));
    }
}
