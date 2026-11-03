use eframe::egui;
use crate::domain::model::ModelBenchmark;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_roi_simulation_card(ui: &mut egui::Ui, model: &ModelBenchmark, lang: Language) {
    let roi = lang.roi();
    let vol_id = egui::Id::new("detail_roi_monthly_volume");
    let mut volume = ui.ctx().memory_mut(|m| m.data.get_temp::<f32>(vol_id)).unwrap_or(10_000.0);

    ui.add_space(4.0);
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(16, 22, 34))
        .stroke(egui::Stroke::new(1.0, ThemeTokens::BG_PANEL))
        .inner_margin(8.0)
        .rounding(6.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new(roi.title).strong().size(12.0).color(ThemeTokens::GOLD_5090));
            ui.add_space(2.0);

            // Slider: Monthly Volume (1,000 to 100,000 requests/month)
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(roi.volume_label).size(11.0).color(ThemeTokens::TEXT_MUTED));
                let resp = ui.add(
                    egui::Slider::new(&mut volume, 1_000.0..=100_000.0)
                        .step_by(1_000.0)
                        .suffix(roi.volume_suffix)
                        .logarithmic(true)
                );
                if resp.changed() {
                    ui.ctx().memory_mut(|m| m.data.insert_temp(vol_id, volume));
                }
            });

            ui.add_space(3.0);

            let prompt_toks = 800.0;
            let compl_toks = 250.0;

            if model.is_cloud {
                let in_p = model.pricing_input_usd.unwrap_or(2.50);
                let out_p = model.pricing_output_usd.unwrap_or(10.00);
                let cloud_monthly_usd = volume * (prompt_toks * in_p + compl_toks * out_p) / 1_000_000.0;
                let cloud_monthly_eur = cloud_monthly_usd * 0.92;

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("Facture Cloud Estimée : ~{:.2} € / mois (~${:.2})", cloud_monthly_eur, cloud_monthly_usd)).strong().color(ThemeTokens::CYAN_EDGE));
                });
            } else {
                let tps = model.tokens_per_sec.unwrap_or(45.0).max(1.0);
                let sec_per_req = compl_toks / tps;
                let active_hours = (volume * sec_per_req) / 3600.0;
                let watts = match model.size_gb {
                    s if s <= 2.5 => 15.0,  // Pi / SBC
                    s if s <= 8.0 => 45.0,  // Jetson / Mini-PC
                    s if s <= 16.0 => 160.0, // RTX 4060/4070
                    s if s <= 24.0 => 320.0, // RTX 4090 / 3090
                    _ => 450.0,             // Workstation / RTX 5090
                };
                let monthly_kwh = (active_hours * watts) / 1000.0;
                let local_cost_eur = monthly_kwh * 0.25;

                // Typical cloud equivalent: $0.60 prompt / $2.00 completion per 1M tokens
                let equiv_cloud_usd = volume * (prompt_toks * 0.60 + compl_toks * 2.00) / 1_000_000.0;
                let equiv_cloud_eur = equiv_cloud_usd * 0.92;
                let net_savings = equiv_cloud_eur - local_cost_eur;

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("{} {:.2} €/mois", roi.cost_local, local_cost_eur)).color(ThemeTokens::TEXT_PRIMARY).size(11.0));
                    ui.label(egui::RichText::new("|").color(ThemeTokens::TEXT_MUTED));
                    ui.label(egui::RichText::new(format!("{} {:.2} €/mois", roi.cost_cloud, equiv_cloud_eur)).color(ThemeTokens::TEXT_SECONDARY).size(11.0));
                });

                ui.add_space(2.0);
                if net_savings > 0.0 {
                    ui.label(egui::RichText::new(format!("{} +{:.2} €/mois !", roi.roi_gain, net_savings)).strong().color(ThemeTokens::SUCCESS).size(11.5));
                } else {
                    ui.label(egui::RichText::new(roi.roi_cheaper_cloud).color(ThemeTokens::GOLD_5090).size(11.0));
                }
            }

            ui.add_space(1.0);
            ui.label(egui::RichText::new(roi.assumptions).size(9.5).color(ThemeTokens::TEXT_MUTED));
        });
}
