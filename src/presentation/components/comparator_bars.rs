use eframe::egui;
use crate::domain::comparator::ModelHardwareMetrics;
use crate::domain::model::ModelBenchmark;
use crate::domain::simulator::HardwareRigConfig;
use crate::presentation::theme::ThemeTokens;

pub fn render_thematic_dual_bars(
    ui: &mut egui::Ui,
    m1_opt: Option<(&ModelBenchmark, &ModelHardwareMetrics)>,
    m2_opt: Option<(&ModelBenchmark, &ModelHardwareMetrics)>,
    rig: &HardwareRigConfig,
) {
    let (m1, met1) = match m1_opt {
        Some(pair) => pair,
        None => return,
    };
    let (m2, met2) = match m2_opt {
        Some(pair) => pair,
        None => return,
    };

    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("COMPARAISON THEMATIQUE DUEL FACE-A-FACE (BARRES SUPERPOSEES)")
                    .strong()
                    .size(13.0)
                    .color(ThemeTokens::PRIMARY),
            );
            ui.label(
                egui::RichText::new(format!("(Matériel : {})", rig.gpu.label()))
                    .size(11.0)
                    .color(ThemeTokens::TEXT_MUTED),
            );
        });

        ui.add_space(4.0);

        // Pillar 1: Intelligence Globale
        let int1 = met1.intelligence_score;
        let int2 = met2.intelligence_score;
        let int_delta = int1 - int2;
        let int_winner_text = if int_delta.abs() < 0.5 {
            "Egalité parfaite sur l'intelligence globale".to_string()
        } else if int_delta > 0.0 {
            format!("[Gagnant : #1] +{:.1} pts (+{:.0}%) d'intelligence académique et de raisonnement", int_delta, (int_delta / int2.max(1.0)) * 100.0)
        } else {
            format!("[Gagnant : #2] +{:.1} pts (+{:.0}%) d'intelligence académique et de raisonnement", -int_delta, (-int_delta / int1.max(1.0)) * 100.0)
        };
        render_theme_card(
            ui,
            "1. Intelligence Globale & Raisonnement (Score Calibré 0 - 100)",
            &met1.model_name, int1, 100.0, format!("{:.1} / 100", int1),
            &met2.model_name, int2, 100.0, format!("{:.1} / 100", int2),
            &int_winner_text,
            int_delta > 0.5, int_delta < -0.5,
        );

        ui.add_space(3.0);

        // Pillar 2: Vitesse Réelle sur ce GPU
        let tps1 = met1.estimated_tps;
        let tps2 = met2.estimated_tps;
        let speed_ratio = if tps1 >= tps2 { tps1 / tps2.max(1.0) } else { tps2 / tps1.max(1.0) };
        let speed_winner_text = if (tps1 - tps2).abs() < 5.0 {
            "Débit quasi-identique".to_string()
        } else if tps1 > tps2 {
            format!("[Gagnant : #1] {:.1}x plus rapide ({:.0} vs {:.0} tok/s) - Réponse en {:.1}s vs {:.1}s", speed_ratio, tps1, tps2, met1.sec_per_200_words, met2.sec_per_200_words)
        } else {
            format!("[Gagnant : #2] {:.1}x plus rapide ({:.0} vs {:.0} tok/s) - Réponse en {:.1}s vs {:.1}s", speed_ratio, tps2, tps1, met2.sec_per_200_words, met1.sec_per_200_words)
        };
        let max_tps = tps1.max(tps2).max(100.0);
        render_theme_card(
            ui,
            "2. Vitesse Réelle Projetée (Tokens par seconde)",
            &met1.model_name, tps1, max_tps, format!("{:.0} tok/s (~{:.1}s / 200 mots)", tps1, met1.sec_per_200_words),
            &met2.model_name, tps2, max_tps, format!("{:.0} tok/s (~{:.1}s / 200 mots)", tps2, met2.sec_per_200_words),
            &speed_winner_text,
            tps1 > tps2 + 5.0, tps2 > tps1 + 5.0,
        );

        ui.add_space(3.0);

        // Pillar 3: Empreinte Mémoire & Stabilité VRAM
        let total_vram = (rig.gpu.vram_gb() - 0.6).max(1.0);
        let vram1_text = if met1.is_cloud {
            "0.0 Go (Cloud API - Zéro VRAM locale)".into()
        } else {
            format!("{:.1} Go ({:.0}%) - {}", met1.vram_usage_gb, (met1.vram_usage_gb / total_vram) * 100.0, if met1.fits_pure_vram { "100% VRAM OK" } else { "Déborde sur RAM" })
        };
        let vram2_text = if met2.is_cloud {
            "0.0 Go (Cloud API - Zéro VRAM locale)".into()
        } else {
            format!("{:.1} Go ({:.0}%) - {}", met2.vram_usage_gb, (met2.vram_usage_gb / total_vram) * 100.0, if met2.fits_pure_vram { "100% VRAM OK" } else { "Déborde sur RAM" })
        };
        let vram_winner_text = if met1.vram_usage_gb < met2.vram_usage_gb {
            format!("[Modèle le plus sobre : #1] Laisse +{:.1} Go de mémoire libre face à #2", met2.vram_usage_gb - met1.vram_usage_gb)
        } else if met2.vram_usage_gb < met1.vram_usage_gb {
            format!("[Modèle le plus sobre : #2] Laisse +{:.1} Go de mémoire libre face à #1", met1.vram_usage_gb - met2.vram_usage_gb)
        } else {
            "Empreinte mémoire équivalente".to_string()
        };
        render_theme_card(
            ui,
            &format!("3. Stabilité & Empreinte Mémoire (Capacité Matériel : {:.1} Go)", rig.gpu.vram_gb()),
            &met1.model_name, met1.vram_usage_gb, total_vram, vram1_text,
            &met2.model_name, met2.vram_usage_gb, total_vram, vram2_text,
            &vram_winner_text,
            met1.vram_usage_gb < met2.vram_usage_gb, met2.vram_usage_gb < met1.vram_usage_gb,
        );

        ui.add_space(3.0);

        // Pillar 4: Code & Développement Logiciel
        let code1 = m1.scores.coding.unwrap_or(met1.intelligence_score);
        let code2 = m2.scores.coding.unwrap_or(met2.intelligence_score);
        let code_delta = code1 - code2;
        let code_winner_text = if code_delta.abs() < 0.5 {
            "Performances en code équivalentes".to_string()
        } else if code_delta > 0.0 {
            format!("[Gagnant : #1] +{:.1} pts sur les tâches de programmation et tests SWE-Bench", code_delta)
        } else {
            format!("[Gagnant : #2] +{:.1} pts sur les tâches de programmation et tests SWE-Bench", -code_delta)
        };
        render_theme_card(
            ui,
            "4. Code & Développement Logiciel (SWE-Bench / HumanEval)",
            &met1.model_name, code1, 100.0, format!("{:.1} / 100", code1),
            &met2.model_name, code2, 100.0, format!("{:.1} / 100", code2),
            &code_winner_text,
            code_delta > 0.5, code_delta < -0.5,
        );

        ui.add_space(3.0);

        // Pillar 5: Rendement Efficience
        let eff1 = met1.lightness_ratio;
        let eff2 = met2.lightness_ratio;
        let eff_ratio = if eff1 >= eff2 { eff1 / eff2.max(0.1) } else { eff2 / eff1.max(0.1) };
        let eff_winner_text = if (eff1 - eff2).abs() < 0.5 {
            "Rendement équivalent".to_string()
        } else if eff1 > eff2 {
            format!("[Gagnant : #1] {:.1}x plus sobre par Go de mémoire (Maximise l'intelligence utile)", eff_ratio)
        } else {
            format!("[Gagnant : #2] {:.1}x plus sobre par Go de mémoire (Maximise l'intelligence utile)", eff_ratio)
        };
        let max_eff = eff1.max(eff2).max(20.0);
        render_theme_card(
            ui,
            "5. Rendement Efficience (Points d'intelligence par Go de taille)",
            &met1.model_name, eff1, max_eff, format!("{:.1} pts/Go ({:.1} Go)", eff1, m1.size_gb),
            &met2.model_name, eff2, max_eff, format!("{:.1} pts/Go ({:.1} Go)", eff2, m2.size_gb),
            &eff_winner_text,
            eff1 > eff2 + 0.5, eff2 > eff1 + 0.5,
        );
    });
}

fn render_theme_card(
    ui: &mut egui::Ui,
    title: &str,
    name1: &str, val1: f32, max1: f32, text1: String,
    name2: &str, val2: f32, max2: f32, text2: String,
    winner_text: &str,
    w1: bool, w2: bool,
) {
    egui::Frame::group(ui.style())
        .fill(ThemeTokens::BG_CARD)
        .stroke(egui::Stroke::new(1.0, ThemeTokens::BG_PANEL))
        .inner_margin(egui::Margin::symmetric(10.0, 7.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(title).strong().size(11.5).color(ThemeTokens::TEXT_PRIMARY));
            ui.add_space(4.0);

            let common_max = max1.max(max2).max(0.1);
            let pct1 = (val1 / common_max).clamp(0.0, 1.0);
            let pct2 = (val2 / common_max).clamp(0.0, 1.0);

            // Strict fixed column allocation guarantees 100% horizontal alignment
            let col0_w = 250.0;
            let col1_w = 160.0;
            let bar_w = (ui.available_width() - col0_w - col1_w - 30.0).max(140.0);

            egui::Grid::new(title)
                .num_columns(3)
                .spacing(egui::vec2(10.0, 5.0))
                .show(ui, |ui| {
                    // Row 1: Model #1 (Cyan)
                    ui.allocate_ui_with_layout(egui::vec2(col0_w, 20.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("[#1]").color(ThemeTokens::CYAN_EDGE).strong().size(11.5));
                        ui.label(egui::RichText::new(name1).strong().size(11.5));
                        if w1 {
                            ui.label(egui::RichText::new("[MEILLEUR]").color(ThemeTokens::SUCCESS).strong().size(10.0));
                        }
                    });

                    ui.allocate_ui_with_layout(egui::vec2(col1_w, 20.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(&text1).size(11.0).color(ThemeTokens::CYAN_EDGE).strong());
                    });

                    ui.add(
                        egui::ProgressBar::new(pct1)
                            .desired_width(bar_w)
                            .fill(ThemeTokens::CYAN_EDGE)
                            .animate(false)
                    );
                    ui.end_row();

                    // Row 2: Model #2 (Gold)
                    ui.allocate_ui_with_layout(egui::vec2(col0_w, 20.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("[#2]").color(ThemeTokens::GOLD_5090).strong().size(11.5));
                        ui.label(egui::RichText::new(name2).strong().size(11.5));
                        if w2 {
                            ui.label(egui::RichText::new("[MEILLEUR]").color(ThemeTokens::SUCCESS).strong().size(10.0));
                        }
                    });

                    ui.allocate_ui_with_layout(egui::vec2(col1_w, 20.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(&text2).size(11.0).color(ThemeTokens::GOLD_5090).strong());
                    });

                    ui.add(
                        egui::ProgressBar::new(pct2)
                            .desired_width(bar_w)
                            .fill(ThemeTokens::GOLD_5090)
                            .animate(false)
                    );
                    ui.end_row();
                });

            ui.add_space(3.0);
            ui.label(egui::RichText::new(winner_text).color(ThemeTokens::TEXT_MUTED).size(10.5));
        });
}
