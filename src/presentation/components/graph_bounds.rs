use egui_plot::{PlotBounds, PlotUi};
use crate::presentation::components::graph_types::{GraphMetric, PlottedPoint};

pub fn compute_data_bounds(plotted: &[PlottedPoint]) -> Option<([f64; 2], [f64; 2])> {
    if plotted.is_empty() {
        return None;
    }
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for p in plotted {
        if p.x.is_finite() && p.y.is_finite() {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
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
) {
    if let Some(([min_x, min_y], [max_x, max_y])) = compute_data_bounds(plotted) {
        let min_dx = if x_metric == GraphMetric::PriceUsd { 0.05 } else { 1.0 };
        let dx = (max_x - min_x).max(min_dx);
        let dy = (max_y - min_y).max(2.0);

        // Apply comfortable 8% - 12% padding around the scatter cloud
        let target_min_x = if x_metric == GraphMetric::PriceUsd {
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
) {
    let bounds = plot_ui.plot_bounds();
    if !bounds.is_finite() {
        return;
    }

    let (min, max) = (bounds.min(), bounds.max());
    let (data_min, data_max) = match compute_data_bounds(plotted) {
        Some((dmin, dmax)) => (dmin, dmax),
        None => ([0.0, 0.0], [50.0, 100.0]),
    };

    let dx = (data_max[0] - data_min[0]).max(if x_metric == GraphMetric::PriceUsd { 0.05 } else { 1.0 });
    let dy = (data_max[1] - data_min[1]).max(2.0);

    // Dynamic min/max zoom window sizes depending on metric scale
    let min_w = if x_metric == GraphMetric::PriceUsd { 0.005 } else { 0.2 };
    let min_h = 0.5;

    let max_w = (dx * 4.5).max(if x_metric == GraphMetric::PriceUsd { 3.0 } else { 80.0 });
    let max_h = (dy * 4.5).max(100.0);

    let w = bounds.width().clamp(min_w, max_w);
    let h = bounds.height().clamp(min_h, max_h);

    // Pan bounds: prevent getting lost in deep negative void while allowing exploration
    let limit_min_x = if x_metric == GraphMetric::PriceUsd { -0.1 } else { data_min[0] - dx * 1.2 };
    let limit_max_x = data_max[0] + dx * 1.5;
    let limit_min_y = (data_min[1] - dy * 1.2).max(-5.0);
    let limit_max_y = data_max[1] + dy * 1.5;

    let mut cx = (min[0] + max[0]) * 0.5;
    let mut cy = (min[1] + max[1]) * 0.5;

    cx = cx.clamp(limit_min_x + w * 0.5, limit_max_x - w * 0.5);
    cy = cy.clamp(limit_min_y + h * 0.5, limit_max_y - h * 0.5);

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
