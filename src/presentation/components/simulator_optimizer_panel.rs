use eframe::egui;
use crate::domain::model::{ModelBenchmark, SourceSelection};
use crate::domain::runtime_optimizer::compute_optimal_runtime_plan;
use crate::domain::simulator::{HardwareRigConfig, RigSimulationResult};
use crate::domain::simulator_ranker::{rank_compatible_models, SimTableSort};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_simulator_optimizer_panel(
    ui: &mut egui::Ui,
    config: &HardwareRigConfig,
    sim: &RigSimulationResult,
    all_models: &[ModelBenchmark],
    sources: &SourceSelection,
    selected_model_id: &mut Option<String>,
    lang: Language,
) {
    let eco = lang.eco();
    let is_fr = lang == Language::French;
    let unit_gb = if is_fr { "Go" } else { "GB" };
    let local_models: Vec<&ModelBenchmark> = all_models.iter().filter(|m| !m.is_cloud).collect();
    if local_models.is_empty() {
        return;
    }

    let opt_id_key = egui::Id::new("sim_optimizer_model_id");
    let copied_key = egui::Id::new("sim_optimizer_copied_tag");
    let mut target_id = selected_model_id.clone()
        .filter(|id| local_models.iter().any(|m| &m.id == id))
        .or_else(|| ui.ctx().memory_mut(|m| m.data.get_temp::<String>(opt_id_key)))
        .filter(|id| local_models.iter().any(|m| &m.id == id))
        .unwrap_or_else(|| {
            rank_compatible_models(all_models, config, sim, sources, SimTableSort::Score, true)
                .first()
                .map(|e| e.id.clone())
                .unwrap_or_else(|| local_models[0].id.clone())
        });

    let Some(model) = local_models.iter().find(|m| m.id == target_id).copied() else {
        return;
    };
    let plan = compute_optimal_runtime_plan(model, config, sim);
    let copied_tag = ui.ctx().memory_mut(|m| m.data.get_temp::<String>(copied_key)).unwrap_or_default();

    ui.add_space(10.0);
    egui::Frame::group(ui.style())
        .fill(ThemeTokens::BG_CARD)
        .stroke(egui::Stroke::new(1.0, ThemeTokens::CYAN_EDGE))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(eco.sim_opt_title).strong().size(14.0).color(ThemeTokens::CYAN_EDGE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut choices = rank_compatible_models(all_models, config, sim, sources, SimTableSort::Score, true);
                    choices.truncate(40);
                    egui::ComboBox::from_id_salt("sim_opt_model_combo")
                        .selected_text(format!("{} ({:.1} {unit_gb})", model.name, model.size_gb))
                        .width(260.0)
                        .show_ui(ui, |ui| {
                            for ch in &choices {
                                if ui.selectable_label(ch.id == target_id, format!("{} | {:.1} pts | {:.1} {unit_gb}", ch.name, ch.score, ch.size_gb)).clicked() {
                                    target_id = ch.id.clone();
                                    *selected_model_id = Some(ch.id.clone());
                                }
                            }
                        });
                    ui.label(egui::RichText::new(eco.sim_opt_model_lbl).color(ThemeTokens::TEXT_MUTED).size(11.5));
                });
            });
            ui.add_space(6.0);

            let cards = [
                (
                    eco.sim_opt_quant_lbl,
                    plan.recommended_quant.to_string(),
                    format!("{}: {:.1} {unit_gb}", if is_fr { "Poids" } else { "Weights" }, plan.weight_vram_gb),
                    ThemeTokens::GOLD_5090,
                ),
                (
                    eco.sim_opt_kv_lbl,
                    plan.kv_cache.cli_flag().to_uppercase(),
                    format!("Flash Attn: {}", if plan.flash_attention { "ON (-fa 1)" } else { "OFF" }),
                    ThemeTokens::CYAN_EDGE,
                ),
                (
                    eco.sim_opt_ctx_lbl,
                    format!("{}k ({})", plan.recommended_ctx / 1024, plan.recommended_ctx),
                    format!("KV VRAM: {:.2} {unit_gb}", plan.kv_vram_gb),
                    ThemeTokens::SUCCESS,
                ),
                (
                    eco.sim_opt_gpu_lbl,
                    format!("-ngl {} ({}%)", plan.recommended_ngl, plan.gpu_offload_pct),
                    format!("Batch: -b {} | Total: {:.1} {unit_gb}", plan.batch_size, plan.total_vram_gb),
                    ThemeTokens::TEXT_PRIMARY,
                ),
            ];
            let cols = if ui.available_width() < 860.0 { 2 } else { 4 };
            for chunk in cards.chunks(cols) {
                ui.columns(chunk.len(), |ui_cols| {
                    for (i, (title, val, sub, color)) in chunk.iter().enumerate() {
                        egui::Frame::group(ui_cols[i].style()).fill(ThemeTokens::BG_PANEL).inner_margin(8.0).show(&mut ui_cols[i], |ui| {
                            ui.label(egui::RichText::new(*title).size(10.0).strong().color(ThemeTokens::TEXT_MUTED));
                            ui.label(egui::RichText::new(val).size(16.0).strong().color(*color));
                            ui.label(egui::RichText::new(sub).size(10.5).color(ThemeTokens::TEXT_SECONDARY));
                        });
                    }
                });
                ui.add_space(4.0);
            }

            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(eco.sim_opt_kv_compare).size(11.5).strong().color(ThemeTokens::TEXT_MUTED));
                for opt in &plan.context_options {
                    let active = opt.kv_type == plan.kv_cache;
                    let clr = if active { ThemeTokens::SUCCESS } else { ThemeTokens::TEXT_SECONDARY };
                    let mark = if active { "[OK] " } else { "" };
                    let txt = format!("{mark}{}: {}k ctx ({:.2} {unit_gb})", opt.kv_type.cli_flag().to_uppercase(), opt.max_safe_ctx / 1024, opt.kv_vram_gb);
                    ui.label(egui::RichText::new(txt).size(11.5).strong().color(clr));
                    ui.separator();
                }
            });
            ui.add_space(4.0);

            for (tag, label, cmd) in [("llama", "llama-server:", &plan.llama_cmd), ("ollama", "Ollama:", &plan.ollama_cmd)] {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(label).size(11.0).strong().color(ThemeTokens::TEXT_MUTED));
                    ui.label(egui::RichText::new(cmd).monospace().size(11.0).color(ThemeTokens::TEXT_PRIMARY));
                    let btn_lbl = if copied_tag == tag { eco.sim_opt_copied } else { eco.sim_opt_copy };
                    if ui.small_button(btn_lbl).clicked() {
                        ui.ctx().copy_text(cmd.clone());
                        ui.ctx().memory_mut(|m| m.data.insert_temp(copied_key, tag.to_string()));
                    }
                });
            }
        });

    ui.ctx().memory_mut(|m| m.data.insert_temp(opt_id_key, target_id));
}
