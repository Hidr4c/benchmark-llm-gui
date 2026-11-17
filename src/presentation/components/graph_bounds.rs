use egui_plot::{GridInput, GridMark, PlotBounds, PlotUi};
use crate::presentation::components::graph_types::{GraphMetric, PlottedPoint};

#[inline]
pub fn to_plot_x(raw_x: f64, is_log_price: bool) -> f64 {
    if is_log_price {
        raw_x.max(0.005).log10()
    } else {
        raw_x
    }
}

pub fn format_log_price_tick(log_val: f64) -> String {
    let usd = 10f64.powf(log_val);
    if usd >= 10.0 {
        format!("${usd:.0}")
    } else if usd >= 1.0 {
        format!("${usd:.1}")
    } else if usd >= 0.1 {
        format!("${usd:.2}")
    } else {
        format!("${usd:.3}")
    }
}

pub fn log_price_grid_marks(input: GridInput) -> Vec<GridMark> {
    let (min_x, max_x) = input.bounds;
    let steps: [(f64, f64); 15] = [
        (0.005, 0.3), (0.01, 1.0), (0.02, 0.3), (0.05, 0.3),
        (0.1, 1.0), (0.2, 0.3), (0.5, 0.3),
        (1.0, 1.0), (2.0, 0.3), (5.0, 0.3),
        (10.0, 1.0), (20.0, 0.3), (50.0, 0.3),
        (100.0, 1.0), (200.0, 0.3),
    ];
    steps.iter().filter_map(|&(usd, step_size)| {
        let v = usd.log10();
        if v >= min_x - 0.1 && v <= max_x + 0.1 {
            Some(GridMark { value: v, step_size })
        } else {
            None
        }
    }).collect()
}

pub fn compute_data_bounds(plotted: &[PlottedPoint]) -> Option<([f64; 2], [f64; 2])> {
    compute_plot_data_bounds(plotted, false)
}

pub fn compute_plot_data_bounds(plotted: &[PlottedPoint], is_log_price: bool) -> Option<([f64; 2], [f64; 2])> {
    if plotted.is_empty() {
        return None;
    }
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for p in plotted {
        let px = to_plot_x(p.x, is_log_price);
        if px.is_finite() && p.y.is_finite() {
            min_x = min_x.min(px);
            max_x = max_x.max(px);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }
    }

    if min_x.is_finite() && max_x.is_finite() && min_y.is_finite() && max_y.is_finite() {
        Some(([min_x, min_y], [max_x, max_y]))
    } else {
        None
    }
}

pub fn apply_optimal_data_bounds(
    plot_ui: &mut PlotUi,
    x_metric: GraphMetric,
    _y_metric: GraphMetric,
    plotted: &[PlottedPoint],
    is_log_price: bool,
) {
    if let Some(([min_x, min_y], [max_x, max_y])) = compute_plot_data_bounds(plotted, is_log_price) {
        let min_dx = if is_log_price { 0.4 } else if x_metric == GraphMetric::PriceUsd { 0.05 } else { 1.0 };
        let dx = (max_x - min_x).max(min_dx);
        let dy = (max_y - min_y).max(2.0);

        let target_min_x = if is_log_price {
            min_x - dx * 0.08
        } else if x_metric == GraphMetric::PriceUsd {
            (min_x - dx * 0.06).max(0.0)
        } else {
            min_x - dx * 0.08
        };
        let target_max_x = max_x + dx * 0.12;

        let target_min_y = (min_y - dy * 0.08).max(-2.0);
        let target_max_y = max_y + dy * 0.10;

        plot_ui.set_plot_bounds(PlotBounds::from_min_max(
            [target_min_x, target_min_y],
            [target_max_x, target_max_y],
        ));
    } else {
        plot_ui.set_auto_bounds([true, true].into());
    }
}

pub fn clamp_plot_bounds(
    plot_ui: &mut PlotUi,
    x_metric: GraphMetric,
    _y_metric: GraphMetric,
    plotted: &[PlottedPoint],
    is_log_price: bool,
) {
    let bounds = plot_ui.plot_bounds();
    if !bounds.is_finite() {
        return;
    }

    let (min, max) = (bounds.min(), bounds.max());
    let (data_min, data_max) = compute_plot_data_bounds(plotted, is_log_price)
        .unwrap_or(([0.0, 0.0], [50.0, 100.0]));

    let dx = (data_max[0] - data_min[0]).max(if is_log_price { 0.4 } else if x_metric == GraphMetric::PriceUsd { 0.05 } else { 1.0 });
    let dy = (data_max[1] - data_min[1]).max(2.0);

    let min_w = if is_log_price { 0.08 } else if x_metric == GraphMetric::PriceUsd { 0.005 } else { 0.2 };
    let min_h = 0.5;

    let max_w = (dx * 4.5).max(if is_log_price { 6.0 } else if x_metric == GraphMetric::PriceUsd { 3.0 } else { 80.0 });
    let max_h = (dy * 4.5).max(100.0);

    let w = bounds.width().clamp(min_w, max_w);
    let h = bounds.height().clamp(min_h, max_h);

    let limit_min_x = if !is_log_price && x_metric == GraphMetric::PriceUsd { -0.1 } else { data_min[0] - dx * 0.8 };
    let limit_max_x = data_max[0] + dx * 1.2;
    let limit_min_y = (data_min[1] - dy * 1.2).max(-5.0);
    let limit_max_y = data_max[1] + dy * 1.5;

    let mut cx = if (min[0] + max[0]).is_finite() { (min[0] + max[0]) * 0.5 } else { (limit_min_x + limit_max_x) * 0.5 };
    let mut cy = if (min[1] + max[1]).is_finite() { (min[1] + max[1]) * 0.5 } else { (limit_min_y + limit_max_y) * 0.5 };

    let clamp_min_x = limit_min_x + w * 0.5;
    let clamp_max_x = limit_max_x - w * 0.5;
    cx = if clamp_min_x <= clamp_max_x { cx.clamp(clamp_min_x, clamp_max_x) } else { (limit_min_x + limit_max_x) * 0.5 };

    let clamp_min_y = limit_min_y + h * 0.5;
    let clamp_max_y = limit_max_y - h * 0.5;
    cy = if clamp_min_y <= clamp_max_y { cy.clamp(clamp_min_y, clamp_max_y) } else { (limit_min_y + limit_max_y) * 0.5 };

    let new_min = [cx - w * 0.5, cy - h * 0.5];
    let new_max = [cx + w * 0.5, cy + h * 0.5];

    if (bounds.width() - w).abs() > 0.0001
        || (bounds.height() - h).abs() > 0.0001
        || (bounds.min()[0] - new_min[0]).abs() > 0.001
        || (bounds.min()[1] - new_min[1]).abs() > 0.001
    {
        plot_ui.set_plot_bounds(PlotBounds::from_min_max(new_min, new_max));
    }
}
