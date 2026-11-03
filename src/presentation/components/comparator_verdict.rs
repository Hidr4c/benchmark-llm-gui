use eframe::egui;
use crate::domain::comparator::ModelHardwareMetrics;
use crate::domain::simulator::HardwareRigConfig;
use crate::presentation::theme::ThemeTokens;

/// Renders the hardware diagnostic specification grid directly inside the model card
pub fn render_model_hardware_specs_grid(
    ui: &mut egui::Ui,
    m: &ModelHardwareMetrics,
    rig: &HardwareRigConfig,
    color: egui::Color32,
) {
    let total_vram = (rig.gpu.vram_gb() - 0.6).max(1.0);
    let pct = (m.vram_usage_gb / total_vram).clamp(0.0, 1.0);

    let (badge_title, _) = m.role.badge_text(true);
    ui.label(egui::RichText::new(badge_title).size(10.5).color(color).strong());

    ui.separator();

    egui::Grid::new(format!("verdict_grid_{}", m.model_name))
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("Stabilité Machine :").color(ThemeTokens::TEXT_MUTED).size(11.0));
            let (stab_text, stab_color) = if m.fits_pure_vram && pct <= 0.70 {
                ("100% Stable (Confort)", ThemeTokens::SUCCESS)
            } else if m.fits_pure_vram {
                ("90% Stable (Optimisé)", ThemeTokens::SUCCESS)
            } else {
                ("25% Instable (Offload RAM)", ThemeTokens::DANGER)
            };
            ui.label(egui::RichText::new(stab_text).color(stab_color).size(11.0).strong());
            ui.end_row();

            ui.label(egui::RichText::new("Débit Réel GPU :").color(ThemeTokens::TEXT_MUTED).size(11.0));
            ui.label(
                egui::RichText::new(format!("{:.0} tok/s (~{:.1}s / rép.)", m.estimated_tps, m.sec_per_200_words))
                    .color(ThemeTokens::CYAN_EDGE)
                    .size(11.0)
                    .strong(),
            );
            ui.end_row();

            ui.label(egui::RichText::new("Niveau d'Intelligence :").color(ThemeTokens::TEXT_MUTED).size(11.0));
            ui.label(
                egui::RichText::new(format!("{:.1} / 100", m.intelligence_score))
                    .color(ThemeTokens::GOLD_5090)
                    .size(11.0)
                    .strong(),
            );
            ui.end_row();

            ui.label(egui::RichText::new("Rapport Efficience :").color(ThemeTokens::TEXT_MUTED).size(11.0));
            ui.label(
                egui::RichText::new(format!("{:.0} pts / Go", m.lightness_ratio))
                    .color(egui::Color32::WHITE)
                    .size(11.0)
                    .strong(),
            );
            ui.end_row();

            ui.label(egui::RichText::new("Énergie & Conso :").color(ThemeTokens::TEXT_MUTED).size(11.0));
            ui.label(
                egui::RichText::new(format!("{:.0} W | {:.3} €/100k", m.power_watts, m.cost_per_100k_tokens_eur))
                    .size(10.5),
            );
            ui.end_row();
        });
}

/// Renders the unified Machine Fitness Recommendation Box across both models
pub fn render_machine_fitness_box(
    ui: &mut egui::Ui,
    m1: &ModelHardwareMetrics,
    m2: &ModelHardwareMetrics,
    rig: &HardwareRigConfig,
) {
    let (fast, deep) = if m1.estimated_tps >= m2.estimated_tps { (m1, m2) } else { (m2, m1) };
    let speedup = (fast.estimated_tps / deep.estimated_tps.max(1.0)).clamp(1.0, 50.0);
    let intel_gain = (deep.intelligence_score - fast.intelligence_score).max(0.0);

    let (most_stable, least_stable) = if m1.vram_headroom_gb >= m2.vram_headroom_gb {
        (m1, m2)
    } else {
        (m2, m1)
    };

    let most_efficient = if m1.lightness_ratio >= m2.lightness_ratio { m1 } else { m2 };

    egui::Frame::group(ui.style())
        .fill(ThemeTokens::BG_PANEL)
        .stroke(egui::Stroke::new(1.0, ThemeTokens::PRIMARY))
        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            // Row 1: Stability Verdict (wrapped so it never overflows)
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("* Stabilité Machine :").strong().color(ThemeTokens::SUCCESS).size(11.5));
                let stab_text = if least_stable.fits_pure_vram {
                    format!(
                        "Les 2 modèles s'exécutent à 100% en VRAM. {} est le plus stable (marge de {:.1} Go libres, 0 risque de saturation).",
                        most_stable.model_name, most_stable.vram_headroom_gb
                    )
                } else {
                    format!(
                        "{} est 100% stable en VRAM. Attention : {} déborde sur la RAM système (vitesse dégradée).",
                        most_stable.model_name, least_stable.model_name
                    )
                };
                ui.label(egui::RichText::new(stab_text).size(11.0).strong());
            });

            ui.add_space(2.5);

            // Row 2: Efficiency & Speed Verdict (wrapped)
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("[~] Débit & Efficacité :").strong().color(ThemeTokens::CYAN_EDGE).size(11.5));
                ui.label(
                    egui::RichText::new(format!(
                        "{} est {:.1}x plus rapide ({:.0} vs {:.0} tok/s). Modèle au meilleur rendement matériel : {} ({:.0} pts/Go).",
                        fast.model_name, speedup, fast.estimated_tps, deep.estimated_tps, most_efficient.model_name, most_efficient.lightness_ratio
                    ))
                    .size(11.0)
                    .strong(),
                );
            });

            ui.add_space(2.5);

            // Row 3: Usage Recommendation for this hardware (wrapped)
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Recommandation :").strong().color(ThemeTokens::GOLD_5090).size(11.5));
                let reco_text = format!(
                    "Pour un usage quotidien fluide sur votre {} : privilégiez {}. Pour du code complexe ou de l'analyse pointue : basculez sur {} (+{:.1} pts de précision).",
                    rig.gpu.label(), fast.model_name, deep.model_name, intel_gain
                );
                ui.label(egui::RichText::new(reco_text).color(ThemeTokens::GOLD_5090).size(11.0).strong());
            });
        });
}

pub fn render_comparator_verdict(
    ui: &mut egui::Ui,
    metrics: &[ModelHardwareMetrics],
    rig: &HardwareRigConfig,
) {
    if metrics.len() >= 2 {
        render_machine_fitness_box(ui, &metrics[0], &metrics[1], rig);
    }
}
