use egui_plot::{PlotPoint, PlotResponse};
use crate::domain::model::ModelBenchmark;
use crate::presentation::components::graph_types::PlottedPoint;

pub type LookupItem = (f64, f64, String, String, f32, f32, Option<u16>, &'static str, bool);

pub fn format_cluster_tooltip(val: &PlotPoint, _series_name: &str, lookup_data: &[LookupItem], is_price_metric: bool) -> String {
    let mut closest = Vec::new();
    let mut min_dist = f64::INFINITY;
    for item in lookup_data {
        let d = ((item.0 - val.x).powi(2) + (item.1 - val.y).powi(2)).sqrt();
        if d < min_dist { min_dist = d; }
    }

    if min_dist < 4.0 {
        for item in lookup_data {
            let d = ((item.0 - val.x).powi(2) + (item.1 - val.y).powi(2)).sqrt();
            if (d - min_dist).abs() < 0.25 || d < 0.35 { closest.push(item); }
        }
    }

    if closest.len() == 1 {
        let (px, _, mname, _, mvram, mscore, _, _, is_cloud) = closest[0];
        if is_price_metric {
            let tag = if *is_cloud { "API" } else { "Elec" };
            format!("{mname} | ${px:.2}/1M ({tag}) | {mscore:.1} pts")
        } else {
            format!("{mname} | {mvram:.1} Go VRAM | {mscore:.1} pts")
        }
    } else if closest.len() > 1 {
        let (px, _, mname, _, mvram, mscore, _, _, is_cloud) = closest[0];
        let extra = closest.len() - 1;
        if is_price_metric {
            let tag = if *is_cloud { "API" } else { "Elec" };
            format!("{mname} (+{extra} autres) | ${px:.2}/1M ({tag}) | {mscore:.1} pts")
        } else {
            format!("{mname} (+{extra} autres) | {mvram:.1} Go | {mscore:.1} pts")
        }
    } else {
        String::new()
    }
}


pub fn detect_clicked_model_id(
    plot_resp: &PlotResponse<()>,
    plotted: &[PlottedPoint],
    all_models: &[ModelBenchmark],
    current_selected_id: Option<&str>,
) -> Option<String> {
    let mouse_pos = plot_resp.response.hover_pos()?;
    let mut cluster_model_ids: Vec<String> = Vec::new();

    for p in plotted {
        let screen_pos = plot_resp.transform.position_from_point(&PlotPoint::new(p.x, p.y));
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
