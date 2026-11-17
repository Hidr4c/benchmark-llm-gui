use crate::domain::modality::ModelModality;
use crate::domain::model::ModelBenchmark;
use crate::presentation::components::filter_bar::FilterState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphMetric {
    CrossScore,
    CodingScore,
    AgenticScore,
    GeneralScore,
    SizeGb,
    VramRecGb,
    SpeedTps,
    Efficiency,
    PriceUsd,
}

impl GraphMetric {
    pub fn label(&self) -> &'static str {
        self.label_with_state(false)
    }

    pub fn label_with_state(&self, include_electricity: bool) -> &'static str {
        match self {
            GraphMetric::CrossScore => "Cross Score Average (pts)",
            GraphMetric::CodingScore => "Code & Dev Score (pts)",
            GraphMetric::AgenticScore => "Agents & Tools Score (pts)",
            GraphMetric::GeneralScore => "General Academic Score (pts)",
            GraphMetric::SizeGb => "Download Size (GB)",
            GraphMetric::VramRecGb => "Required VRAM (GB)",
            GraphMetric::SpeedTps => "Generation Speed (tok/s)",
            GraphMetric::Efficiency => "Efficiency Ratio (pts/GB)",
            GraphMetric::PriceUsd => {
                if include_electricity {
                    "Coût Global ($/1M | API & Elec)"
                } else {
                    "Tarif Cloud ($/1M tokens)"
                }
            }
        }
    }

    pub fn value_of(&self, m: &ModelBenchmark, filters: &FilterState, include_electricity: bool) -> f64 {
        match self {
            GraphMetric::CrossScore => m.score_for_sources(&filters.sources).unwrap_or(0.0) as f64,
            GraphMetric::CodingScore => m.scores.coding.unwrap_or(0.0) as f64,
            GraphMetric::AgenticScore => m.scores.agentic.unwrap_or(0.0) as f64,
            GraphMetric::GeneralScore => m.scores.open_llm.or(m.scores.average).unwrap_or(0.0) as f64,
            GraphMetric::SizeGb => m.size_gb as f64,
            GraphMetric::VramRecGb => m.vram_rec_gb as f64,
            GraphMetric::SpeedTps => m.tokens_per_sec.unwrap_or(0.0) as f64,
            GraphMetric::Efficiency => m.efficiency_ratio_with_sources(&filters.sources) as f64,
            GraphMetric::PriceUsd => {
                if m.is_cloud {
                    cloud_api_price_usd(m).unwrap_or(-1.0)
                } else if include_electricity {
                    m.estimated_electricity_cost_usd_per_1m() as f64
                } else {
                    0.0
                }
            }
        }
    }
}

/// Computes blended (3:1 input/output) or available Cloud API pricing in $/1M tokens.
/// Returns `None` when a Cloud model has no public pricing data so it is never plotted as "$0.00".
pub fn cloud_api_price_usd(m: &ModelBenchmark) -> Option<f64> {
    if !m.is_cloud {
        return None;
    }
    let inp = m.pricing_input_usd.filter(|&v| v > 0.0).map(|v| v as f64);
    let out = m.pricing_output_usd.filter(|&v| v > 0.0).map(|v| v as f64);
    match (inp, out) {
        (Some(i), Some(o)) => Some(i * 0.75 + o * 0.25),
        (Some(i), None) => Some(i),
        (None, Some(o)) => Some(o),
        (None, None) => None,
    }
}

pub struct GraphViewState {
    pub x_metric: GraphMetric,
    pub y_metric: GraphMetric,
    pub show_pareto_curve: bool,
    pub show_node_labels: bool,
    pub show_champion_labels: bool,
    pub reset_requested: bool,
    pub vram_budget_gb: f32,
    pub include_electricity_cost: bool,
    pub log_price_x: bool,
    pub search_query: String,
    pub last_criteria_key: Option<(GraphMetric, GraphMetric, bool, bool, usize, usize)>,
}

impl Default for GraphViewState {
    fn default() -> Self {
        Self {
            x_metric: GraphMetric::VramRecGb,
            y_metric: GraphMetric::CrossScore,
            show_pareto_curve: true,
            show_node_labels: true,
            show_champion_labels: true,
            reset_requested: false,
            vram_budget_gb: 48.0,
            include_electricity_cost: true,
            log_price_x: true,
            search_query: String::new(),
            last_criteria_key: None,
        }
    }
}

pub struct PlottedPoint {
    pub index: usize,
    pub x: f64,
    pub y: f64,
    pub name: String,
    pub family: String,
    pub vram_gb: f32,
    pub size_gb: f32,
    pub score: f32,
    pub speed: Option<f32>,
    pub speed_hw: Option<String>,
    pub year: Option<u16>,
    pub modality: ModelModality,
}

pub fn compute_pareto_frontier(plotted: &[PlottedPoint]) -> (Vec<usize>, Vec<[f64; 2]>) {
    let mut pareto_indices = Vec::new();
    let mut coords = Vec::new();
    if plotted.len() < 2 {
        return (pareto_indices, coords);
    }

    let mut sorted: Vec<usize> = (0..plotted.len()).collect();
    sorted.sort_by(|&a, &b| {
        plotted[a].x.total_cmp(&plotted[b].x)
            .then_with(|| plotted[b].y.total_cmp(&plotted[a].y))
    });

    let mut max_y = f64::NEG_INFINITY;
    for idx in sorted {
        if plotted[idx].y > max_y {
            max_y = plotted[idx].y;
            pareto_indices.push(idx);
            coords.push([plotted[idx].x, plotted[idx].y]);
        }
    }
    (pareto_indices, coords)
}

pub fn render_inspection_card(
    ui: &mut eframe::egui::Ui,
    m: &ModelBenchmark,
    plotted: &[PlottedPoint],
    all_models: &[ModelBenchmark],
    filters: &FilterState,
    selected_model_id: &mut Option<String>,
    t: &crate::presentation::i18n::Translations,
    is_log_price: bool,
) {
    ui.add_space(4.0);
    eframe::egui::Frame::group(ui.style())
        .fill(crate::presentation::theme::ThemeTokens::BG_CARD)
        .stroke(eframe::egui::Stroke::new(1.5, crate::presentation::theme::ThemeTokens::GOLD_5090))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(eframe::egui::RichText::new(format!("[Cible] {}", m.name)).color(crate::presentation::theme::ThemeTokens::TEXT_PRIMARY).strong());
                ui.label(eframe::egui::RichText::new(format!("({}, {})", m.family, m.release_year_label())).color(crate::presentation::theme::ThemeTokens::CYAN_EDGE).strong());
                ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                    if ui.button(t.close_inspection).clicked() { *selected_model_id = None; }
                    if let Some(tag) = &m.ollama_tag {
                        if ui.button(t.copy_ollama).on_hover_text(format!("ollama run {tag}")).clicked() {
                            ui.output_mut(|o| o.copied_text = format!("ollama run {tag}"));
                        }
                    }
                    if ui.button(t.view_online).clicked() { let _ = open::that(&m.hf_url); }
                });
            });
            ui.horizontal(|ui| {
                let sc = m.score_for_sources(&filters.sources).unwrap_or(0.0);
                ui.label(eframe::egui::RichText::new(format!("* {} : {:.1} pts", t.col_score, sc)).strong().color(crate::presentation::theme::ThemeTokens::GOLD_5090));
                ui.separator();
                if m.is_cloud {
                    let price_str = match (m.pricing_input_usd.filter(|&v| v > 0.0), m.pricing_output_usd.filter(|&v| v > 0.0)) {
                        (Some(i), Some(o)) => format!("In: ${i:.2}/1M | Out: ${o:.2}/1M"),
                        (Some(i), None) => format!("In: ${i:.2}/1M"),
                        (None, Some(o)) => format!("Out: ${o:.2}/1M"),
                        (None, None) => "Tarif API: N/A".into(),
                    };
                    ui.label(eframe::egui::RichText::new(format!(" {price_str} ({})", m.provider.as_deref().unwrap_or("API"))).color(crate::presentation::theme::ThemeTokens::GOLD_5090));
                } else {
                    let elec = m.estimated_electricity_cost_usd_per_1m();
                    ui.label(eframe::egui::RichText::new(format!(" 0$ API | [~] Elec: ~${:.2}/1M", elec)).color(crate::presentation::theme::ThemeTokens::SUCCESS));
                    ui.separator();
                    ui.label(eframe::egui::RichText::new(format!("[VRAM] VRAM: {:.1} GB", m.vram_rec_gb)).color(crate::presentation::theme::ThemeTokens::TEXT_SECONDARY));
                }
                ui.separator();
                if let Some(tps) = m.tokens_per_sec {
                    ui.label(eframe::egui::RichText::new(format!("[~] {:.0} tok/s ({})", tps, m.tps_hardware.as_deref().unwrap_or("GPU"))).color(crate::presentation::theme::ThemeTokens::SUCCESS));
                }
                ui.separator();
                ui.label(eframe::egui::RichText::new(m.embedded_recommendation()).color(crate::presentation::theme::ThemeTokens::CYAN_EDGE));
            });

            // Co-located models switcher if multiple models share this coordinate
            if let Some(target_p) = plotted.iter().find(|p| all_models[p.index].id == m.id) {
                let to_x = crate::presentation::components::graph_bounds::to_plot_x;
                let target_px = to_x(target_p.x, is_log_price);
                let dx_tol = if is_log_price { 0.09 } else { 0.35 };
                let co_located: Vec<&PlottedPoint> = plotted.iter()
                    .filter(|p| (to_x(p.x, is_log_price) - target_px).abs() < dx_tol && (p.y - target_p.y).abs() < 0.45)
                    .collect();
                if co_located.len() > 1 {
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(eframe::egui::RichText::new(format!(" {} co-localisés :", co_located.len())).strong().color(crate::presentation::theme::ThemeTokens::CYAN_EDGE));
                        for cp in co_located {
                            let is_cur = all_models[cp.index].id == m.id;
                            if ui.selectable_label(is_cur, format!("{} ({:.1} pts)", cp.name, cp.score)).clicked() {
                                *selected_model_id = Some(all_models[cp.index].id.clone());
                            }
                        }
                    });
                }
            }
        });
}

pub use crate::presentation::components::graph_bounds::{apply_optimal_data_bounds, clamp_plot_bounds, compute_data_bounds};

pub fn render_champion_bar(
    ui: &mut eframe::egui::Ui,
    state: &mut GraphViewState,
    plotted: &[PlottedPoint],
    all_models: &[ModelBenchmark],
    filters: &FilterState,
    selected_model_id: &mut Option<String>,
    t: &crate::presentation::i18n::Translations,
) {
    ui.add_space(3.0);
    ui.horizontal(|ui| {
        if filters.scope == crate::presentation::filtering::ModelScope::CloudOnly || state.x_metric == GraphMetric::PriceUsd {
            if let Some(b) = plotted.iter().max_by(|a, b| a.y.total_cmp(&b.y)) {
                let unit_lbl = if state.x_metric == GraphMetric::PriceUsd {
                    format!("${:.2}/1M", b.x)
                } else {
                    format!("{:.1}", b.x)
                };
                let champ_lbl = format!(" Best Score : {} ({:.1} pts, {unit_lbl})", b.name, b.y);
                if ui.button(eframe::egui::RichText::new(champ_lbl).strong().color(crate::presentation::theme::ThemeTokens::GOLD_5090)).clicked() {
                    *selected_model_id = Some(all_models[b.index].id.clone());
                }
            }
        } else {
            ui.label(eframe::egui::RichText::new(t.vram_budget_label).strong().color(crate::presentation::theme::ThemeTokens::CYAN_EDGE));
            ui.add(eframe::egui::Slider::new(&mut state.vram_budget_gb, 0.5..=48.0).suffix(" GB").logarithmic(true));
            if let Some(b) = plotted.iter().filter(|p| p.vram_gb <= state.vram_budget_gb).max_by(|a, b| a.y.total_cmp(&b.y)) {
                ui.separator();
                let champ_lbl = format!("{} {:.1} GB : {} ({:.1} pts, {:.1}G)", t.vram_best_prefix, state.vram_budget_gb, b.name, b.y, b.vram_gb);
                if ui.button(eframe::egui::RichText::new(champ_lbl).strong().color(crate::presentation::theme::ThemeTokens::GOLD_5090)).clicked() {
                    *selected_model_id = Some(all_models[b.index].id.clone());
                }
            }
        }
    });
}

pub fn render_pareto_chips(
    ui: &mut eframe::egui::Ui,
    state: &GraphViewState,
    plotted: &[PlottedPoint],
    all_models: &[ModelBenchmark],
    pareto_indices: &[usize],
    selected_model_id: &mut Option<String>,
    t: &crate::presentation::i18n::Translations,
) {
    if pareto_indices.is_empty() { return; }
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        ui.label(eframe::egui::RichText::new(t.pareto_champions).strong().color(crate::presentation::theme::ThemeTokens::GOLD_5090));
        for &pi in pareto_indices {
            let p = &plotted[pi];
            let chip_label = match state.x_metric {
                GraphMetric::PriceUsd => format!("{} (${:.2})", p.name, p.x),
                GraphMetric::SpeedTps => format!("{} ({:.0} t/s)", p.name, p.x),
                _ => format!("{} ({:.1} GB)", p.name, p.vram_gb),
            };
            let is_sel = selected_model_id.as_ref() == Some(&all_models[p.index].id);
            if ui.selectable_label(is_sel, chip_label).on_hover_text(format!("Score: {:.1} pts", p.score)).clicked() {
                *selected_model_id = Some(all_models[p.index].id.clone());
            }
        }
    });
}
