use eframe::egui;
use crate::domain::hardware::HardwarePlatform;
use crate::presentation::filtering::{FilterState, ModelScope};
use crate::presentation::theme::ThemeTokens;

pub fn render_contextual_hardware_advice(ui: &mut egui::Ui, filters: &FilterState) {
    if filters.scope != ModelScope::CloudOnly {
        let advice = match filters.hardware {
            HardwarePlatform::Gpu32Gb => Some("💡 GPU 32 Go VRAM : Bande passante maximale pour modèles 32B/35B haute précision."),
            HardwarePlatform::Gpu24Gb => Some("💡 GPU 24 Go VRAM : Idéal pour modèles 32B Q4 ou 14B Q8 haute précision."),
            HardwarePlatform::Gpu16Gb => Some("💡 GPU 16 Go VRAM : Excellent équilibre pour 14B Q4 ou 8B haute vitesse."),
            HardwarePlatform::Gpu8Gb => Some("💡 GPU 8 Go VRAM : Recommandé pour modèles 7B-8B (Llama 3, Qwen 2.5, Mistral 7B)."),
            HardwarePlatform::RaspberryPi4 => Some("💡 Raspberry Pi 4 (2-4 Go) : Modèles compacts (< 2 Go comme SmolLM2 ou Qwen 1.5B)."),
            HardwarePlatform::RaspberryPi5 => Some("💡 Raspberry Pi 5 / RK3588 (8 Go) : Adapté aux modèles jusqu'à 3B-4B avec bonne fluidité."),
            HardwarePlatform::MicroIoT => Some("💡 Micro-IoT (< 1 Go RAM) : Modèles ultra-compacts (< 500 Mo) à haut ratio d'efficacité."),
            _ => None,
        };
        if let Some(msg) = advice {
            ui.add_space(2.0);
            ui.label(egui::RichText::new(msg).color(ThemeTokens::TEXT_SECONDARY).size(12.0).italics());
        }
    }
}

pub fn render_advanced_panel(ui: &mut egui::Ui, filters: &mut FilterState, on_export: &mut bool) {
    if filters.show_advanced {
        ui.add_space(4.0);
        egui::Frame::group(ui.style()).fill(ThemeTokens::BG_CARD).inner_margin(6.0).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Architecture :");
                let families = ["All", "Mistral", "Qwen", "Llama", "DeepSeek", "SmolLM", "Gemma", "Phi"];
                egui::ComboBox::from_id_salt("family_arch_combo")
                    .selected_text(&filters.selected_family)
                    .width(130.0)
                    .show_ui(ui, |ui| {
                        for fam in families {
                            if ui.selectable_label(filters.selected_family == fam, fam).clicked() {
                                filters.selected_family = fam.to_string();
                            }
                        }
                    });

                ui.separator();
                ui.label("Taille Max :");
                ui.add(egui::Slider::new(&mut filters.max_size_gb, 0.5..=65.0).step_by(1.0).suffix(" Go"));

                ui.separator();
                ui.label("Score Min :");
                ui.add(egui::Slider::new(&mut filters.min_intelligence, 0.0..=100.0).step_by(5.0).suffix(" pts"));

                ui.separator();
                if ui.button("📥 Exporter CSV").clicked() { *on_export = true; }
            });
        });
    }
}
