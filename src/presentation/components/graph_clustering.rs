use egui_plot::{PlotPoint, PlotResponse};
use crate::domain::model::ModelBenchmark;
use crate::presentation::components::graph_bounds::to_plot_x;
use crate::presentation::components::graph_types::PlottedPoint;

pub type LookupItem = (f64, f64, f64, String, String, f32, f32, Option<u16>, &'static str, bool);

pub fn format_cluster_tooltip(
    val: &PlotPoint,
    _series_name: &str,
    lookup_data: &[LookupItem],
    is_price_metric: bool,
    is_log_price: bool,
) -> String {
    let scale_y = if is_log_price { 16.0 } else { 1.0 };
    let mut closest = Vec::new();
    let mut min_dist = f64::INFINITY;
    for item in lookup_data {
        let d = ((item.0 - val.x).powi(2) + ((item.2 - val.y) / scale_y).powi(2)).sqrt();
        if d < min_dist { min_dist = d; }
    }

    let max_reach = if is_log_price { 0.45 } else { 4.0 };
    let cluster_eps = if is_log_price { 0.08 } else { 0.35 };
    if min_dist < max_reach {
        for item in lookup_data {
            let d = ((item.0 - val.x).powi(2) + ((item.2 - val.y) / scale_y).powi(2)).sqrt();
            if (d - min_dist).abs() < 0.18 || d < cluster_eps { closest.push(item); }
        }
    }

    if closest.is_empty() {
        return String::new();
    }

    let (_, raw_x, _, mname, _, mvram, mscore, _, _, is_cloud) = closest[0];
    let suffix = if closest.len() > 1 {
        format!(" (+{})", closest.len() - 1)
    } else {
        String::new()
    };

    if is_price_metric {
        let tag = if *is_cloud { "API mix" } else { "Elec" };
        format!("{mname}{suffix} | ${raw_x:.2}/1M ({tag}) | {mscore:.1} pts")
    } else {
        format!("{mname}{suffix} | {mvram:.1} Go VRAM | {mscore:.1} pts")
    }
}

pub fn detect_clicked_model_id(
    plot_resp: &PlotResponse<()>,
    plotted: &[PlottedPoint],
    all_models: &[ModelBenchmark],
    current_selected_id: Option<&str>,
    is_log_price: bool,
) -> Option<String> {
    let mouse_pos = plot_resp.response.hover_pos()?;
    let mut cluster_model_ids: Vec<String> = Vec::new();

    for p in plotted {
        let px = to_plot_x(p.x, is_log_price);
        let screen_pos = plot_resp.transform.position_from_point(&PlotPoint::new(px, p.y));
        let dist = (screen_pos.x - mouse_pos.x).hypot(screen_pos.y - mouse_pos.y);
        if dist < 24.0 {
            cluster_model_ids.push(all_models[p.index].id.clone());
        }
    }

    if cluster_model_ids.is_empty() {
        return None;
    }

    if let Some(cur_id) = current_selected_id {
        if let Some(pos) = cluster_model_ids.iter().position(|id| id == cur_id) {
            return Some(cluster_model_ids[(pos + 1) % cluster_model_ids.len()].clone());
        }
    }
    Some(cluster_model_ids[0].clone())
}
