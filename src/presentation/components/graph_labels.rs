use eframe::egui;
use egui_plot::{PlotPoint, Text};
use crate::domain::model::ModelBenchmark;
use crate::presentation::components::graph_types::PlottedPoint;
use crate::presentation::theme::ThemeTokens;

/// Renders intelligent, collision-free text labels on graph nodes.
/// Prioritizes the target model, Pareto frontier champions, and compatible models.
pub fn render_graph_node_labels(
    plot_ui: &mut egui_plot::PlotUi,
    plotted: &[PlottedPoint],
    all_models: &[ModelBenchmark],
    pareto_indices: &[usize],
    selected_model_id: Option<&str>,
    vram_budget_gb: f32,
    dynamic_font_size: f32,
) {
    let bounds = plot_ui.plot_bounds();
    let plot_w = bounds.width().max(0.1);
    let plot_h = bounds.height().max(0.1);

    // Dynamic collision threshold: adapts automatically to current zoom level
    let min_dx = plot_w * 0.045;
    let min_dy = plot_h * 0.038;

    let mut placed_coords: Vec<[f64; 2]> = Vec::new();

    // 1. Priority indices: Selected Target first, then Pareto champions, then others
    let mut candidate_indices: Vec<usize> = (0..plotted.len()).collect();
    candidate_indices.sort_by(|&a, &b| {
        let is_target_a = selected_model_id == Some(&all_models[plotted[a].index].id);
        let is_target_b = selected_model_id == Some(&all_models[plotted[b].index].id);
        if is_target_a != is_target_b {
            return is_target_b.cmp(&is_target_a);
        }
        let is_champ_a = pareto_indices.contains(&a);
        let is_champ_b = pareto_indices.contains(&b);
        if is_champ_a != is_champ_b {
            return is_champ_b.cmp(&is_champ_a);
        }
        let in_budget_a = all_models[plotted[a].index].is_cloud || plotted[a].vram_gb <= vram_budget_gb;
        let in_budget_b = all_models[plotted[b].index].is_cloud || plotted[b].vram_gb <= vram_budget_gb;
        if in_budget_a != in_budget_b {
            return in_budget_b.cmp(&in_budget_a);
        }
        plotted[b].score.partial_cmp(&plotted[a].score).unwrap_or(std::cmp::Ordering::Equal)
    });

    let font_size = (dynamic_font_size * 0.88).clamp(9.0, 15.0);

    for &i in &candidate_indices {
        let p = &plotted[i];
        let is_target = selected_model_id == Some(&all_models[p.index].id);
        let is_champ = pareto_indices.contains(&i);
        let in_budget = all_models[p.index].is_cloud || p.vram_gb <= vram_budget_gb;

        // Skip over-budget models unless they are target or champ and zoomed in
        if !in_budget && !is_target && !is_champ {
            continue;
        }

        // Collision detection against already placed labels
        let collides = placed_coords.iter().any(|&[ox, oy]| {
            (p.x - ox).abs() < min_dx && (p.y - oy).abs() < min_dy
        });

        // Always allow the selected target; discard other collisions to prevent text clutter
        if collides && !is_target {
            continue;
        }

        placed_coords.push([p.x, p.y]);

        let short_name = truncate_name(&p.name, 18);
        let (label_text, text_color, is_bold) = if is_target {
            (format!("[Cible] {}", short_name), ThemeTokens::CYAN_EDGE, true)
        } else if is_champ {
            (format!("★ {}", short_name), egui::Color32::WHITE, true)
        } else if in_budget {
            (short_name, egui::Color32::from_rgb(180, 220, 250), false)
        } else {
            (short_name, egui::Color32::from_rgb(140, 155, 175), false)
        };

        let mut rich = egui::RichText::new(label_text).size(font_size).color(text_color);
        if is_bold {
            rich = rich.strong();
        }

        plot_ui.text(
            Text::new(PlotPoint::new(p.x, p.y), rich)
                .anchor(egui::Align2::LEFT_BOTTOM),
        );
    }
}

fn truncate_name(name: &str, max_len: usize) -> String {
    let clean = name.trim();
    if clean.chars().count() <= max_len {
        clean.to_string()
    } else {
        let truncated: String = clean.chars().take(max_len.saturating_sub(1)).collect();
        format!("{}…", truncated)
    }
}
