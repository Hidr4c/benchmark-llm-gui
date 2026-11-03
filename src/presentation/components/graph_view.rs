use eframe::egui;
use egui_plot::{Line, Plot, PlotPoint, PlotPoints, Points, Text};
use crate::domain::model::ModelBenchmark;
use crate::presentation::components::filter_bar::FilterState;
use crate::presentation::components::graph_clustering::{
    detect_clicked_model_id, format_cluster_tooltip, LookupItem,
};
pub use crate::presentation::components::graph_types::{
    clamp_plot_bounds, compute_pareto_frontier, render_champion_bar, render_inspection_card,
    render_pareto_chips, GraphMetric, GraphViewState, PlottedPoint,
};
use crate::presentation::filtering::ModelScope;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_graph_view(
    ui: &mut egui::Ui,
    state: &mut GraphViewState,
    all_models: &[ModelBenchmark],
    filtered_indices: &[usize],
    filters: &FilterState,
    selected_model_id: &mut Option<String>,
    lang: Language,
) {
    let t = lang.t();
    if filters.scope == ModelScope::CloudOnly && (state.x_metric == GraphMetric::VramRecGb || state.x_metric == GraphMetric::SizeGb) {
        state.x_metric = GraphMetric::PriceUsd;
    } else if filters.scope == ModelScope::LocalOnly && state.x_metric == GraphMetric::PriceUsd && !state.include_electricity_cost {
        state.x_metric = GraphMetric::VramRecGb;
    }

    ui.vertical(|ui| {
        // --- 1. Preset & Metric Configuration Dropdowns ---
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(t.graph_preset_label).strong());
            let current_preset_lbl = match (state.x_metric, state.y_metric) {
                (GraphMetric::VramRecGb, GraphMetric::CrossScore) => t.preset_pareto,
                (GraphMetric::PriceUsd, GraphMetric::CrossScore) => t.preset_cloud_price,
                (GraphMetric::SpeedTps, GraphMetric::CrossScore) => t.preset_speed_intel,
                (GraphMetric::SizeGb, GraphMetric::Efficiency) => t.preset_embedded_efficiency,
                (GraphMetric::VramRecGb, GraphMetric::CodingScore) => t.preset_code_vram,
                (GraphMetric::VramRecGb, GraphMetric::AgenticScore) => t.preset_agent_vram,
                _ => "🎯 Custom",
            };
            egui::ComboBox::from_id_salt("graph_preset_combo")
                .selected_text(current_preset_lbl)
                .width(185.0)
                .show_ui(ui, |ui| {
                    let presets = [
                        (GraphMetric::VramRecGb, GraphMetric::CrossScore, true, t.preset_pareto),
                        (GraphMetric::PriceUsd, GraphMetric::CrossScore, true, t.preset_cloud_price),
                        (GraphMetric::SpeedTps, GraphMetric::CrossScore, false, t.preset_speed_intel),
                        (GraphMetric::SizeGb, GraphMetric::Efficiency, true, t.preset_embedded_efficiency),
                        (GraphMetric::VramRecGb, GraphMetric::CodingScore, true, t.preset_code_vram),
                        (GraphMetric::VramRecGb, GraphMetric::AgenticScore, true, t.preset_agent_vram),
                    ];
                    for (x, y, pareto, label) in presets {
                        if ui.selectable_label(state.x_metric == x && state.y_metric == y, label).clicked() {
                            state.x_metric = x; state.y_metric = y; state.show_pareto_curve = pareto;
                        }
                    }
                });

            ui.separator();

            ui.label(t.axis_x);
            egui::ComboBox::from_id_salt("graph_x_combo")
                .selected_text(state.x_metric.label_with_state(state.include_electricity_cost))
                .width(145.0)
                .show_ui(ui, |ui| {
                    let metrics = [
                        GraphMetric::VramRecGb,
                        GraphMetric::PriceUsd,
                        GraphMetric::SizeGb,
                        GraphMetric::SpeedTps,
                        GraphMetric::Efficiency,
                    ];
                    for m in metrics {
                        let lbl = m.label_with_state(state.include_electricity_cost);
                        if ui.selectable_label(state.x_metric == m, lbl).clicked() { state.x_metric = m; }
                    }
                });

            ui.label(t.axis_y);
            egui::ComboBox::from_id_salt("graph_y_combo")
                .selected_text(state.y_metric.label())
                .width(135.0)
                .show_ui(ui, |ui| {
                    let metrics = [
                        GraphMetric::CrossScore, GraphMetric::CodingScore, GraphMetric::AgenticScore,
                        GraphMetric::GeneralScore, GraphMetric::SpeedTps, GraphMetric::Efficiency,
                    ];
                    for m in metrics {
                        if ui.selectable_label(state.y_metric == m, m.label()).clicked() { state.y_metric = m; }
                    }
                });

            ui.checkbox(&mut state.show_pareto_curve, t.pareto_curve);

            if state.x_metric == GraphMetric::PriceUsd {
                ui.checkbox(&mut state.include_electricity_cost, t.graph_include_electricity)
                    .on_hover_text(t.graph_include_electricity_tooltip);
            }

            ui.separator();

            if ui.button(egui::RichText::new(t.recenter_btn).strong().color(ThemeTokens::CYAN_EDGE))
                .on_hover_text(t.recenter_tooltip)
                .clicked()
            {
                state.reset_requested = true;
            }
        });

        ui.add_space(4.0);

        // --- 2. Prepare plotted points and Pareto frontier ---
        let mut plotted: Vec<PlottedPoint> = Vec::new();
        for &idx in filtered_indices {
            let m = &all_models[idx];
            let x = state.x_metric.value_of(m, filters, state.include_electricity_cost);
            let y = state.y_metric.value_of(m, filters, state.include_electricity_cost);
            let valid_x = if state.x_metric == GraphMetric::PriceUsd { x >= 0.0 } else { x > 0.0 };
            if valid_x && y > 0.0 {
                plotted.push(PlottedPoint {
                    index: idx, x, y,
                    name: m.name.clone(), family: m.family.clone(),
                    vram_gb: m.vram_rec_gb, size_gb: m.size_gb,
                    score: m.score_for_sources(&filters.sources).unwrap_or(0.0),
                    speed: m.tokens_per_sec, speed_hw: m.tps_hardware.clone(),
                    year: m.release_year,
                    modality: m.modality,
                });
            }
        }

        let (pareto_indices, pareto_frontier_coords) = if state.show_pareto_curve {
            compute_pareto_frontier(&plotted)
        } else {
            (Vec::new(), Vec::new())
        };

        // --- 3. Interactive Plot with Light Tooltip & Click-to-Select ---
        let x_label = state.x_metric.label_with_state(state.include_electricity_cost);
        let y_label = state.y_metric.label();
        let is_price_metric = state.x_metric == GraphMetric::PriceUsd;

        let lookup_data: Vec<LookupItem> = plotted
            .iter()
            .map(|p| (p.x, p.y, p.name.clone(), p.family.clone(), p.vram_gb, p.score, p.year, p.modality.tag_short(), all_models[p.index].is_cloud))
            .collect();

        let plot = Plot::new("benchmark_plot")
            .x_axis_label(x_label)
            .y_axis_label(y_label)
            .show_grid(true)
            .allow_double_click_reset(true)
            .clamp_grid(true)
            .legend(egui_plot::Legend::default())
            .label_formatter(move |series_name, val| {
                format_cluster_tooltip(val, series_name, &lookup_data, is_price_metric)
            });

        let sel_id_clone = selected_model_id.clone();
        let vram_lim = state.vram_budget_gb;

        let plot_resp = plot.show(ui, |plot_ui| {
            if state.reset_requested {
                plot_ui.set_auto_bounds([true, true].into());
                state.reset_requested = false;
            } else {
                clamp_plot_bounds(plot_ui);
            }

            // Scatter points (highlighted if cloud or within VRAM budget, dimmed if exceeding)
            let mut in_budget_coords = Vec::new();
            let mut out_budget_coords = Vec::new();
            for p in &plotted {
                if all_models[p.index].is_cloud || p.vram_gb <= vram_lim {
                    in_budget_coords.push([p.x, p.y]);
                } else {
                    out_budget_coords.push([p.x, p.y]);
                }
            }

            if !in_budget_coords.is_empty() {
                plot_ui.points(Points::new(PlotPoints::new(in_budget_coords)).radius(4.5).color(ThemeTokens::CYAN_EDGE).name("Compatible Models"));
            }
            if !out_budget_coords.is_empty() {
                plot_ui.points(Points::new(PlotPoints::new(out_budget_coords)).radius(3.0).color(egui::Color32::from_rgb(60, 75, 90)).name("Over VRAM Budget"));
            }

            // Pareto frontier line & champion markers
            if state.show_pareto_curve && pareto_frontier_coords.len() >= 2 {
                plot_ui.line(Line::new(PlotPoints::new(pareto_frontier_coords)).color(ThemeTokens::GOLD_5090).width(2.5).name(t.pareto_curve));
                let champion_coords: Vec<[f64; 2]> = pareto_indices.iter().map(|&i| [plotted[i].x, plotted[i].y]).collect();
                plot_ui.points(Points::new(PlotPoints::new(champion_coords)).radius(7.0).color(ThemeTokens::GOLD_5090).name(t.pareto_champions));
            }

            // Dynamic text scaling based on zoom level
            let bounds = plot_ui.plot_bounds();
            let zoom_ratio = (28.0 / bounds.width().max(0.3)).clamp(1.0, 3.2) as f32;
            let dynamic_font_size = (11.0 * zoom_ratio).min(24.0);

            // Highlight selected model with golden target halo and dynamic text
            if let Some(sel_id) = sel_id_clone.as_deref() {
                if let Some(p) = plotted.iter().find(|p| all_models[p.index].id == sel_id) {
                    plot_ui.points(Points::new(PlotPoints::new(vec![[p.x, p.y]])).radius(11.0).color(ThemeTokens::GOLD_5090).name("Cible"));
                    let txt = egui::RichText::new(format!("🎯 {}", p.name))
                        .size(dynamic_font_size)
                        .color(ThemeTokens::GOLD_5090)
                        .strong();
                    plot_ui.text(Text::new(PlotPoint::new(p.x, p.y), txt).highlight(true).anchor(egui::Align2::LEFT_BOTTOM));
                }
            }


        });

        // Click detection on plot canvas to select nearest node or cycle through co-located cluster
        if plot_resp.response.clicked() {
            if let Some(next_id) = detect_clicked_model_id(&plot_resp, &plotted, all_models, selected_model_id.as_deref()) {
                *selected_model_id = Some(next_id);
            }
        }

        // --- 4. Interactive Budget Slider & Live Pareto Champion ---
        render_champion_bar(ui, state, &plotted, all_models, filters, selected_model_id, t);

        // --- 5. Selected Model Inspection Card (Docked under plot) ---
        if let Some(sel_id) = selected_model_id.as_deref() {
            if let Some(m) = all_models.iter().find(|m| &m.id == sel_id) {
                render_inspection_card(ui, m, &plotted, all_models, filters, selected_model_id, t);
            }
        }

        // --- 6. Pareto Champions Clickable Filter Chips ---
        render_pareto_chips(ui, state, &plotted, all_models, &pareto_indices, selected_model_id, t);
    });
}
