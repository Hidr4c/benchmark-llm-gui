use eframe::egui;
use crate::domain::hardware::HardwarePlatform;
use crate::domain::simulator::{GpuTier, HardwareRigConfig, SimTargetMode};
use crate::presentation::i18n::{Language, Translations};
use crate::presentation::theme::ThemeTokens;

/// Renders the live-reactive Hardware Configuration Bar (instant updates with zero Apply button).
pub fn render_simulator_config_bar(
    ui: &mut egui::Ui,
    config: &mut HardwareRigConfig,
    compatible_count: usize,
    t: &Translations,
    lang: Language,
) {
    let is_fr = lang == Language::French;
    egui::Frame::group(ui.style())
        .fill(ThemeTokens::BG_CARD)
        .stroke(egui::Stroke::new(1.0, ThemeTokens::TEXT_MUTED))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(if is_fr { "Matériel :" } else { "Hardware:" }).strong().color(ThemeTokens::CYAN_EDGE));
                render_platform_combo(ui, config, is_fr);
                ui.add_space(6.0);
                ui.label(egui::RichText::new(if is_fr { "Disque Max :" } else { "Max Storage:" }).strong().color(ThemeTokens::TEXT_PRIMARY));
                render_storage_combo(ui, config, is_fr);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let count_lbl = if is_fr {
                        format!("{compatible_count} modèles compatibles")
                    } else {
                        format!("{compatible_count} compatible models")
                    };
                    ui.label(egui::RichText::new(count_lbl).strong().size(11.5).color(ThemeTokens::SUCCESS));
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);
            render_memory_sliders(ui, config, t, is_fr);
        });
}

fn render_platform_combo(ui: &mut egui::Ui, config: &mut HardwareRigConfig, is_fr: bool) {
    egui::ComboBox::from_id_salt("sim_hw_platform_dropdown")
        .selected_text(egui::RichText::new(config.platform.label()).strong().color(ThemeTokens::GOLD_5090))
        .width(245.0)
        .show_ui(ui, |ui| {
            ui.label(egui::RichText::new(if is_fr { "--- Smartphones & Mobiles ---" } else { "--- Smartphones & Mobile ---" }).color(ThemeTokens::TEXT_MUTED).size(10.5));
            for hw in [HardwarePlatform::SmartphoneMid, HardwarePlatform::SmartphoneHigh, HardwarePlatform::SmartphonePro] {
                if ui.selectable_label(config.platform == hw, hw.label()).clicked() {
                    config.platform = hw;
                    config.gpu = GpuTier::from_platform(hw);
                    config.mode = SimTargetMode::WorkstationPc;
                    config.ram_gb = hw.max_usable_memory_gb().unwrap_or(4.0) * 1.5;
                }
            }
            ui.separator();
            ui.label(egui::RichText::new(if is_fr { "--- Cartes Graphiques PC ---" } else { "--- PC GPUs & Workstations ---" }).color(ThemeTokens::TEXT_MUTED).size(10.5));
            for hw in [HardwarePlatform::Gpu8Gb, HardwarePlatform::Gpu12Gb, HardwarePlatform::Gpu16Gb, HardwarePlatform::Gpu24Gb, HardwarePlatform::Gpu32Gb, HardwarePlatform::Workstation] {
                if ui.selectable_label(config.platform == hw, hw.label()).clicked() {
                    config.platform = hw;
                    config.gpu = GpuTier::from_platform(hw);
                    config.mode = SimTargetMode::WorkstationPc;
                }
            }
            ui.separator();
            ui.label(egui::RichText::new(if is_fr { "--- Embarqué & Serveur / NAS ---" } else { "--- Embedded & Server / NAS ---" }).color(ThemeTokens::TEXT_MUTED).size(10.5));
            for hw in [HardwarePlatform::MicroIoT, HardwarePlatform::RaspberryPi4, HardwarePlatform::RaspberryPi5, HardwarePlatform::JetsonOrinNano, HardwarePlatform::MiniPcCpu] {
                if ui.selectable_label(config.platform == hw, hw.label()).clicked() {
                    config.platform = hw;
                    config.gpu = GpuTier::from_platform(hw);
                    config.mode = SimTargetMode::HomeServerNas;
                }
            }
        });
}

fn render_storage_combo(ui: &mut egui::Ui, config: &mut HardwareRigConfig, is_fr: bool) {
    let storage_lbl = match config.max_storage_gb {
        None => if is_fr { "Illimité (Tous modèles)" } else { "Unlimited (All models)" },
        Some(s) if s <= 0.6 => "500 Mo (Micro / IoT)",
        Some(s) if s <= 1.2 => "1.0 Go (Mobile)",
        Some(s) if s <= 2.2 => "2.0 Go (Mobile Pro)",
        Some(s) if s <= 4.2 => "4.0 Go (PC / NAS)",
        Some(s) if s <= 8.2 => "8.0 Go (7B-8B)",
        Some(s) if s <= 16.2 => "16.0 Go (14B-30B)",
        Some(_) => "32.0 Go (Large)",
    };
    egui::ComboBox::from_id_salt("sim_storage_dropdown")
        .selected_text(egui::RichText::new(storage_lbl).strong())
        .width(185.0)
        .show_ui(ui, |ui| {
            let unlim = if is_fr { "Illimité (Tous modèles)" } else { "Unlimited (All models)" };
            for (val, lbl) in [(None, unlim), (Some(0.5), "500 MB (IoT)"), (Some(1.0), "1.0 GB (Mobile)"), (Some(2.0), "2.0 GB (Mobile Pro)"), (Some(4.0), "4.0 GB (PC / NAS)"), (Some(8.0), "8.0 GB (7B-8B)"), (Some(16.0), "16.0 GB (14B-30B)"), (Some(32.0), "32.0 GB (Large)")] {
                if ui.selectable_label(config.max_storage_gb == val, lbl).clicked() { config.max_storage_gb = val; }
            }
        });
}

fn render_memory_sliders(ui: &mut egui::Ui, config: &mut HardwareRigConfig, t: &Translations, is_fr: bool) {
    if config.platform.is_embedded() && !matches!(config.platform, HardwarePlatform::MiniPcCpu) {
        let msg = if is_fr { "Profil Mobile / Embarqué : Inférence NPU/CPU | Mémoire système réservée OS : 1.5 Go" } else { "Mobile / Embedded Profile: NPU/CPU inference | Reserved OS memory: 1.5 GB" };
        ui.label(egui::RichText::new(msg).color(ThemeTokens::TEXT_MUTED).size(11.5));
        return;
    }
    let unit = if is_fr { "Go" } else { "GB" };
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(t.sim_ram_label).strong());
        let max_r = if config.mode == SimTargetMode::HomeServerNas { 64.0 } else { 128.0 };
        let min_r = if config.mode == SimTargetMode::HomeServerNas { 4.0 } else { 8.0 };
        ui.add(egui::Slider::new(&mut config.ram_gb, min_r..=max_r).suffix(format!(" {unit}")).logarithmic(true));
        if config.mode == SimTargetMode::HomeServerNas {
            ui.separator();
            ui.label(egui::RichText::new(t.sim_nas_web_ram).strong());
            ui.add(egui::Slider::new(&mut config.nas_web_ram_gb, 1.0..=16.0).suffix(format!(" {unit}")));
        }
        let preview = config.simulate();
        let os_res = (config.ram_gb - preview.usable_ram_gb).max(0.0);
        ui.label(
            egui::RichText::new(format!("({:.1} {unit} - {:.1} OS = {:.1} {unit} IA)", config.ram_gb, os_res, preview.usable_ram_gb))
                .size(11.0)
                .color(ThemeTokens::TEXT_SECONDARY),
        );
    });
}
