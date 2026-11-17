use eframe::egui;
use egui_plot::{Line, Plot, PlotPoint, PlotPoints, Points, Text};
use crate::domain::model::ModelBenchmark;
use crate::presentation::components::filter_bar::FilterState;
use crate::presentation::components::graph_bounds::{
    apply_optimal_data_bounds, clamp_plot_bounds, format_log_price_tick, log_price_grid_marks, to_plot_x,
};
use crate::presentation::components::graph_clustering::{
    detect_clicked_model_id, format_cluster_tooltip, LookupItem,
};
pub use crate::presentation::components::graph_types::{
    compute_pareto_frontier, render_champion_bar, render_inspection_card,
    render_pareto_chips, GraphMetric, GraphViewState, PlottedPoint,
};
use crate::presentation::components::model_search_picker::render_graph_search_picker;
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
    let is_fr = lang == Language::French;
    if filters.scope == ModelScope::CloudOnly && (state.x_metric == GraphMetric::VramRecGb || state.x_metric == GraphMetric::SizeGb) {
        state.x_metric = GraphMetric::PriceUsd;
    } else if filters.scope == ModelScope::LocalOnly && state.x_metric == GraphMetric::PriceUsd && !state.include_electricity_cost {
        state.x_metric = GraphMetric::VramRecGb;
    }

    let eff_include_elec = state.include_electricity_cost && filters.scope != ModelScope::CloudOnly;
    let is_log_price = state.x_metric == GraphMetric::PriceUsd
        && state.log_price_x
        && (filters.scope == ModelScope::CloudOnly || eff_include_elec);

    let criteria_key = (
        state.x_metric, state.y_metric, eff_include_elec, is_log_price,
        filters.scope as usize, filtered_indices.len(),
    );
    if state.last_criteria_key != Some(criteria_key) {
        state.reset_requested = true;
        state.last_criteria_key = Some(criteria_key);
    }

    ui.vertical(|ui| {
        let preset_row_rect = ui.available_rect_before_wrap();
        ui.ctx().memory_mut(|m| {
            m.data.insert_temp(
                egui::Id::new("tour_target_graph_area"),
                egui::Rect::from_min_size(preset_row_rect.min, egui::vec2(preset_row_rect.width(), 42.0)),
            );
        });

        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(t.graph_preset_label).strong());
            let current_preset_lbl = match (state.x_metric, state.y_metric) {
                (GraphMetric::VramRecGb, GraphMetric::CrossScore) => t.preset_pareto,
                (GraphMetric::PriceUsd, GraphMetric::CrossScore) => t.preset_cloud_price,
                (GraphMetric::SpeedTps, GraphMetric::CrossScore) => t.preset_speed_intel,
                (GraphMetric::SizeGb, GraphMetric::Efficiency) => t.preset_embedded_efficiency,
                (GraphMetric::VramRecGb, GraphMetric::CodingScore) => t.preset_code_vram,
                (GraphMetric::VramRecGb, GraphMetric::AgenticScore) => t.preset_agent_vram,
                _ => " Custom",
            };
            egui::ComboBox::from_id_salt("graph_preset_combo").selected_text(current_preset_lbl).width(185.0).show_ui(ui, |ui| {
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
            egui::ComboBox::from_id_salt("graph_x_combo").selected_text(state.x_metric.label_with_state(eff_include_elec)).width(155.0).show_ui(ui, |ui| {
                for m in [GraphMetric::VramRecGb, GraphMetric::PriceUsd, GraphMetric::SizeGb, GraphMetric::SpeedTps, GraphMetric::Efficiency] {
                    if ui.selectable_label(state.x_metric == m, m.label_with_state(eff_include_elec)).clicked() {
                        state.x_metric = m;
                    }
                }
            });

            ui.label(t.axis_y);
            egui::ComboBox::from_id_salt("graph_y_combo").selected_text(state.y_metric.label()).width(140.0).show_ui(ui, |ui| {
                for m in [GraphMetric::CrossScore, GraphMetric::CodingScore, GraphMetric::AgenticScore, GraphMetric::GeneralScore, GraphMetric::SpeedTps, GraphMetric::Efficiency] {
                    if ui.selectable_label(state.y_metric == m, m.label()).clicked() { state.y_metric = m; }
                }
            });

            ui.checkbox(&mut state.show_pareto_curve, t.pareto_curve);
            ui.checkbox(&mut state.show_node_labels, t.graph_show_labels).on_hover_text(t.graph_show_labels_tooltip);
            if state.x_metric == GraphMetric::PriceUsd {
                ui.checkbox(&mut state.log_price_x, if is_fr { "Echelle Log ($)" } else { "Log Scale ($)" });
                if filters.scope != ModelScope::CloudOnly {
                    ui.checkbox(&mut state.include_electricity_cost, t.graph_include_electricity).on_hover_text(t.graph_include_electricity_tooltip);
                }
            }
            ui.separator();
            let recenter_resp = ui.add(egui::Button::new(egui::RichText::new(t.recenter_btn).strong().color(ThemeTokens::CYAN_EDGE))).on_hover_text(t.recenter_tooltip);
            ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_recenter_btn"), recenter_resp.rect));
            if recenter_resp.clicked() {
                state.reset_requested = true;
                ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_evt_recenter_clicked"), true));
            }
        });

        let mut plotted: Vec<PlottedPoint> = Vec::new();
        let mut excluded_no_price = 0usize;
        for &idx in filtered_indices {
            let m = &all_models[idx];
            let x = state.x_metric.value_of(m, filters, eff_include_elec);
            if state.x_metric == GraphMetric::PriceUsd && m.is_cloud && x <= 0.0 {
                excluded_no_price += 1;
                continue;
            }
            let y = state.y_metric.value_of(m, filters, eff_include_elec);
            let valid_x = if state.x_metric == GraphMetric::PriceUsd && !is_log_price { x >= 0.0 } else { x > 0.0 };
            if valid_x && y > 0.0 {
                plotted.push(PlottedPoint {
                    index: idx, x, y,
                    name: m.name.clone(), family: m.family.clone(),
                    vram_gb: m.vram_rec_gb, size_gb: m.size_gb,
                    score: m.score_for_sources(&filters.sources).unwrap_or(0.0),
                    speed: m.tokens_per_sec, speed_hw: m.tps_hardware.clone(),
                    year: m.release_year, modality: m.modality,
                });
            }
        }

        ui.add_space(3.0);
        ui.horizontal_wrapped(|ui| {
            render_graph_search_picker(ui, &mut state.search_query, all_models, selected_model_id, lang);
            if excluded_no_price > 0 && !plotted.is_empty() {
                ui.separator();
                let msg = if is_fr {
                    format!("[i] {} tarifés | {} sans prix API masqués", plotted.len(), excluded_no_price)
                } else {
                    format!("[i] {} priced plotted | {} unpriced API models hidden", plotted.len(), excluded_no_price)
                };
                ui.label(egui::RichText::new(msg).size(11.0).color(ThemeTokens::TEXT_MUTED));
            }
        });
        ui.add_space(3.0);

        if plotted.is_empty() && excluded_no_price > 0 {
            egui::Frame::group(ui.style()).fill(ThemeTokens::BG_CARD).inner_margin(18.0).show(ui, |ui| {
                let note = if is_fr {
                    format!("{excluded_no_price} modèles Cloud sélectionnés n'ont pas de tarif token $/1M publié (ex: APIs Vidéo/Image). Basculez l'Axe X sur le Débit pour comparer leurs performances.")
                } else {
                    format!("All {excluded_no_price} selected Cloud models lack public $/1M token pricing (e.g. Image/Video APIs). Switch X-Axis to Speed to compare them.")
                };
                ui.label(egui::RichText::new(note).color(ThemeTokens::GOLD_5090));
                ui.add_space(8.0);
                if ui.button(egui::RichText::new(if is_fr { "Basculer Axe X sur Débit (tok/s)" } else { "Switch X-Axis to Speed (tok/s)" }).strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                    state.x_metric = GraphMetric::SpeedTps;
                    state.show_pareto_curve = true;
                }
            });
            return;
        }

        let (pareto_indices, pareto_raw_coords) = if state.show_pareto_curve { compute_pareto_frontier(&plotted) } else { (Vec::new(), Vec::new()) };
        let pareto_plot_coords: Vec<[f64; 2]> = pareto_raw_coords.iter().map(|&[x, y]| [to_plot_x(x, is_log_price), y]).collect();

        let x_label = if is_log_price {
            format!("{} [Log]", state.x_metric.label_with_state(eff_include_elec))
        } else {
            state.x_metric.label_with_state(eff_include_elec).to_string()
        };
        let y_label = state.y_metric.label();
        let is_price_metric = state.x_metric == GraphMetric::PriceUsd;

        let lookup_data: Vec<LookupItem> = plotted.iter().map(|p| {
            (to_plot_x(p.x, is_log_price), p.x, p.y, p.name.clone(), p.family.clone(), p.vram_gb, p.score, p.year, p.modality.tag_short(), all_models[p.index].is_cloud)
        }).collect();

        let mut plot = Plot::new("benchmark_plot")
            .x_axis_label(x_label)
            .y_axis_label(y_label)
            .show_grid(true)
            .allow_double_click_reset(true)
            .clamp_grid(true)
            .legend(egui_plot::Legend::default())
            .label_formatter(move |series_name, val| {
                format_cluster_tooltip(val, series_name, &lookup_data, is_price_metric, is_log_price)
            });

        if is_log_price {
            plot = plot
                .x_axis_formatter(|mark, _| format_log_price_tick(mark.value))
                .x_grid_spacer(log_price_grid_marks);
        }

        let sel_id_clone = selected_model_id.clone();
        let vram_lim = state.vram_budget_gb;

        let plot_resp = plot.show(ui, |plot_ui| {
            if state.reset_requested {
                apply_optimal_data_bounds(plot_ui, state.x_metric, state.y_metric, &plotted, is_log_price);
                state.reset_requested = false;
            } else {
                clamp_plot_bounds(plot_ui, state.x_metric, state.y_metric, &plotted, is_log_price);
            }

            let mut in_budget_coords = Vec::new();
            let mut out_budget_coords = Vec::new();
            for p in &plotted {
                let px = to_plot_x(p.x, is_log_price);
                if all_models[p.index].is_cloud || p.vram_gb <= vram_lim {
                    in_budget_coords.push([px, p.y]);
                } else {
                    out_budget_coords.push([px, p.y]);
                }
            }

            if !in_budget_coords.is_empty() {
                plot_ui.points(Points::new(PlotPoints::new(in_budget_coords)).radius(4.5).color(ThemeTokens::CYAN_EDGE).name("Compatible Models"));
            }
            if !out_budget_coords.is_empty() {
                plot_ui.points(Points::new(PlotPoints::new(out_budget_coords)).radius(3.0).color(egui::Color32::from_rgb(60, 75, 90)).name("Over VRAM Budget"));
            }

            if state.show_pareto_curve && pareto_plot_coords.len() >= 2 {
                plot_ui.line(Line::new(PlotPoints::new(pareto_plot_coords)).color(ThemeTokens::GOLD_5090).width(2.5).name(t.pareto_curve));
                let champion_coords: Vec<[f64; 2]> = pareto_indices.iter().map(|&i| [to_plot_x(plotted[i].x, is_log_price), plotted[i].y]).collect();
                plot_ui.points(Points::new(PlotPoints::new(champion_coords)).radius(7.0).color(ThemeTokens::GOLD_5090).name(t.pareto_champions));
            }

            let bounds = plot_ui.plot_bounds();
            let ref_span = if is_log_price { 3.2 } else { 28.0 };
            let zoom_ratio = (ref_span / bounds.width().max(0.15)).clamp(1.0, 3.2) as f32;
            let dynamic_font_size = (11.0 * zoom_ratio).min(24.0);

            if state.show_node_labels {
                crate::presentation::components::graph_labels::render_graph_node_labels(
                    plot_ui, &plotted, all_models, &pareto_indices, sel_id_clone.as_deref(), vram_lim, dynamic_font_size, is_log_price,
                );
            }

            if let Some(sel_id) = sel_id_clone.as_deref() {
                if let Some(p) = plotted.iter().find(|p| all_models[p.index].id == sel_id) {
                    let px = to_plot_x(p.x, is_log_price);
                    plot_ui.points(Points::new(PlotPoints::new(vec![[px, p.y]])).radius(11.0).color(ThemeTokens::GOLD_5090).name("Cible"));
                    if !state.show_node_labels {
                        let txt = egui::RichText::new(format!("[Cible] {}", p.name)).size(dynamic_font_size).color(ThemeTokens::GOLD_5090).strong();
                        plot_ui.text(Text::new(PlotPoint::new(px, p.y), txt).highlight(true).anchor(egui::Align2::LEFT_BOTTOM));
                    }
                }
            }
        });

        ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_result_graph_plot"), plot_resp.response.rect));

        if plot_resp.response.clicked() {
            if let Some(next_id) = detect_clicked_model_id(&plot_resp, &plotted, all_models, selected_model_id.as_deref(), is_log_price) {
                *selected_model_id = Some(next_id);
            }
        }

        render_champion_bar(ui, state, &plotted, all_models, filters, selected_model_id, t);
        if let Some(sel_id) = selected_model_id.as_deref() {
            if let Some(m) = all_models.iter().find(|m| &m.id == sel_id) {
                render_inspection_card(ui, m, &plotted, all_models, filters, selected_model_id, t, is_log_price);
            }
        }
        render_pareto_chips(ui, state, &plotted, all_models, &pareto_indices, selected_model_id, t);
    });
}
