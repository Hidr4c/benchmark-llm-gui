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
) {
    let arrow = if filters.sort_field == field {
        match filters.sort_order {
            SortOrder::Ascending => " ▲",
            SortOrder::Descending => " ▼",
        }
    } else {
        ""
    };
    if ui.button(format!("{label}{arrow}")).clicked() {
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
    let groups = FilterService::group_into_roots(models, indices);
    let mut items = Vec::new();

    for (r, g) in groups.iter().enumerate() {
        let is_expanded = !filters.group_by_root || filters.expanded_roots.contains(&g.root_name);
        let has_multiple_variants = g.variant_indices.len() > 1;

        if !filters.group_by_root {
            // Flat / Ungrouped View: Every model is shown with stable hierarchical ranking
            // E.g. #11.1, #11.2 for multi-variants, and #12 for standalone models.
            for (v_pos, &v_idx) in g.variant_indices.iter().enumerate() {
                let rank_str = if has_multiple_variants {
                    format!("#{}.{}", r + 1, v_pos + 1)
                } else if r < 3 {
                    match r {
                        0 => "🥇 1".into(),
                        1 => "🥈 2".into(),
                        _ => "🥉 3".into(),
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
        } else {
            // Grouped View: Champion row shown, expandable to show child variants
            let root_rank_str = if r < 3 {
                match r {
                    0 => "🥇 1".into(),
                    1 => "🥈 2".into(),
                    _ => "🥉 3".into(),
                }
            } else {
                format!("#{}", r + 1)
            };

            items.push(DisplayItem {
                model_idx: g.champion_idx,
                is_root: true,
                root_name: g.root_name.clone(),
                variant_count: g.variant_indices.len(),
                is_expanded,
                is_child: false,
                rank_str: if is_expanded && has_multiple_variants {
                    format!("#{}.1", r + 1)
                } else {
                    root_rank_str
                },
            });

            if is_expanded && has_multiple_variants {
                for (v_pos, &v_idx) in g.variant_indices.iter().enumerate() {
                    if v_idx != g.champion_idx {
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
    }

    items
}

pub fn format_downloads(count: u64) -> String {
    if count >= 1_000_000 { format!("{:.1}M", count as f64 / 1_000_000.0) }
    else if count >= 1_000 { format!("{:.0}k", count as f64 / 1_000.0) }
    else if count > 0 { format!("{count}") }
    else { "—".to_string() }
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
